import { execFile } from "node:child_process";
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { promisify } from "node:util";
import {
  createPublicClient,
  createWalletClient,
  encodeDeployData,
  getCreate2Address,
  http,
  keccak256,
  stringToHex,
  toHex,
  zeroHash,
} from "viem";
import { privateKeyToAccount } from "viem/accounts";
import { anvil } from "viem/chains";

const execFileAsync = promisify(execFile);
const root = process.cwd();

const contractRefs = {
  AGCToken: { identifier: "src/AGCToken.sol:AGCToken" },
  AGCHook: { identifier: "src/AGCHook.sol:AGCHook" },
  HookDeployer: { identifier: "HookDeployer", contractsPath: "script" },
  MockUSDC: { identifier: "src/mocks/MockUSDC.sol:MockUSDC" },
  PolicyController: { identifier: "src/PolicyController.sol:PolicyController" },
  PolicyEngine: { identifier: "src/PolicyEngine.sol:PolicyEngine" },
  PoolManager: { identifier: "PoolManager", contractsPath: "lib/v4-core/src" },
  PoolModifyLiquidityTest: { identifier: "PoolModifyLiquidityTest", contractsPath: "lib/v4-core/src/test" },
  SettlementRouter: { identifier: "src/SettlementRouter.sol:SettlementRouter" },
  StabilityVault: { identifier: "src/StabilityVault.sol:StabilityVault" },
  XAGCVault: { identifier: "src/XAGCVault.sol:XAGCVault" },
};

const REQUIRED_HOOK_FLAGS =
  (1n << 11n) |
  (1n << 10n) |
  (1n << 9n) |
  (1n << 8n) |
  (1n << 7n) |
  (1n << 6n) |
  (1n << 2n) |
  1n;

const artifactCache = new Map();

function normalizeKey(result) {
  return {
    currency0: result.currency0 ?? result[0],
    currency1: result.currency1 ?? result[1],
    fee: result.fee ?? result[2],
    tickSpacing: result.tickSpacing ?? result[3],
    hooks: result.hooks ?? result[4],
  };
}

function sqrt(value) {
  if (value < 2n) return value;
  let x0 = value;
  let x1 = (x0 + value / x0) >> 1n;
  while (x1 < x0) {
    x0 = x1;
    x1 = (x0 + value / x0) >> 1n;
  }
  return x0;
}

function floorTickToSpacing(tick, spacing) {
  let compressed = BigInt(tick) / BigInt(spacing);
  if (BigInt(tick) < 0n && BigInt(tick) % BigInt(spacing) !== 0n) {
    compressed -= 1n;
  }
  return Number(compressed * BigInt(spacing));
}

function sqrtPriceX96ForPriceX18(poolKey, agcAddress, priceX18) {
  const agcIsCurrency0 = poolKey.currency0.toLowerCase() === agcAddress.toLowerCase();
  const amount0 = agcIsCurrency0 ? 10n ** 18n : 10n ** 6n;
  const amount1 = agcIsCurrency0 ? 10n ** 6n : 10n ** 18n;
  const ratioX192 = (BigInt(priceX18) * amount0 << 192n) / (amount1 * 10n ** 18n);
  return sqrt(ratioX192);
}

function mintDistributionArgs() {
  return [5000, 2000, 1000, 500, 1500];
}

function policyParamArgs() {
  return [
    300,
    700,
    300,
    10,
    100,
    3,
    100,
    1000,
    500,
    100,
    100,
    200,
    2500,
    2000,
    3000,
    1200,
    1200,
    800,
    150,
    400,
    1800,
    3500,
    150,
    100,
    500,
    2500,
    200,
    1000,
    1000,
    2,
    3600,
  ];
}

function hookFeeArgs() {
  return [1000, 200, 150, 500, 300, 500, 2000, 1500, 86400];
}

async function inspect(contract, field, { viaIr = false } = {}) {
  const key = `${contract}:${field}:${viaIr}`;
  if (artifactCache.has(key)) return artifactCache.get(key);

  const ref = contractRefs[contract] ?? { identifier: contract };
  const args = ["inspect"];
  if (ref.contractsPath) args.push("--contracts", ref.contractsPath);
  args.push(ref.identifier, field, "--json");
  if (viaIr) args.push("--via-ir");

  const { stdout } = await execFileAsync("forge", args, {
    cwd: root,
    maxBuffer: 64 * 1024 * 1024,
  });

  const trimmed = stdout.trim();
  const value = trimmed.startsWith("0x") ? trimmed : JSON.parse(trimmed);
  artifactCache.set(key, value);
  return value;
}

async function deployContract(walletClient, publicClient, contract, args = [], options = {}) {
  const abi = await inspect(contract, "abi", options);
  const bytecode = await inspect(contract, "bytecode", options);
  const hash = await walletClient.deployContract({
    account: walletClient.account,
    abi,
    bytecode,
    args,
  });
  const receipt = await publicClient.waitForTransactionReceipt({ hash });
  if (!receipt.contractAddress) {
    throw new Error(`Deployment failed for ${contract}`);
  }
  return { address: receipt.contractAddress, abi, bytecode };
}

async function writeContract(walletClient, publicClient, { address, abi, functionName, args = [] }) {
  const hash = await walletClient.writeContract({
    account: walletClient.account,
    address,
    abi,
    functionName,
    args,
  });
  return publicClient.waitForTransactionReceipt({ hash });
}

async function findHookAddress(hookDeployerAddress, creationCode) {
  for (let i = 0n; i < 250000n; i += 1n) {
    const salt = toHex(i, { size: 32 });
    const candidate = getCreate2Address({
      from: hookDeployerAddress,
      salt,
      bytecodeHash: keccak256(creationCode),
    });
    if ((BigInt(candidate) & REQUIRED_HOOK_FLAGS) === REQUIRED_HOOK_FLAGS) {
      return { salt, address: candidate };
    }
  }

  throw new Error("Failed to find a valid hook salt.");
}

async function main() {
  const privateKey = process.env.PRIVATE_KEY;
  if (!privateKey) {
    throw new Error("Set PRIVATE_KEY before running the deployer.");
  }

  const rpcUrl = process.env.RPC_URL ?? "http://127.0.0.1:8545";
  const account = privateKeyToAccount(privateKey);
  const publicClient = createPublicClient({ chain: anvil, transport: http(rpcUrl) });
  const walletClient = createWalletClient({ account, chain: anvil, transport: http(rpcUrl) });

  const agc = await deployContract(walletClient, publicClient, "AGCToken", [account.address]);
  const usdc = await deployContract(walletClient, publicClient, "MockUSDC", [account.address]);
  const treasuryVault = await deployContract(walletClient, publicClient, "StabilityVault", [
    account.address,
    agc.address,
    usdc.address,
  ]);
  const xagcVault = await deployContract(walletClient, publicClient, "XAGCVault", [
    account.address,
    agc.address,
    treasuryVault.address,
    300,
  ]);
  const poolManager = await deployContract(walletClient, publicClient, "PoolManager", [account.address]);
  const hookDeployer = await deployContract(walletClient, publicClient, "HookDeployer");

  const agcHookAbi = await inspect("AGCHook", "abi");
  const agcHookBytecode = await inspect("AGCHook", "bytecode");
  const poolConfig = [agc.address, usdc.address, 0x800000, 60, 18, 6];
  const hookCreationCode = encodeDeployData({
    abi: agcHookAbi,
    bytecode: agcHookBytecode,
    args: [account.address, poolManager.address, treasuryVault.address, poolConfig, hookFeeArgs()],
  });
  const hookDeployment = await findHookAddress(hookDeployer.address, hookCreationCode);

  const hookDeployerAbi = await inspect("HookDeployer", "abi");
  await writeContract(walletClient, publicClient, {
    address: hookDeployer.address,
    abi: hookDeployerAbi,
    functionName: "deploy",
    args: [hookCreationCode, hookDeployment.salt],
  });

  const hookCode = await publicClient.getBytecode({ address: hookDeployment.address });
  if (!hookCode) {
    throw new Error("AGCHook deployment did not produce bytecode.");
  }
  const hook = { address: hookDeployment.address, abi: agcHookAbi };

  const engine = await deployContract(walletClient, publicClient, "PolicyEngine");
  const router = await deployContract(walletClient, publicClient, "SettlementRouter", [
    account.address,
    agc.address,
    usdc.address,
    poolManager.address,
    hook.address,
    treasuryVault.address,
  ]);
  const controller = await deployContract(
    walletClient,
    publicClient,
    "PolicyController",
    [
      account.address,
      [agc.address, hook.address, treasuryVault.address, xagcVault.address, router.address, engine.address],
      5n * 10n ** 17n,
      policyParamArgs(),
      mintDistributionArgs(),
    ],
    { viaIr: true },
  );
  const liquidityHelper = await deployContract(walletClient, publicClient, "PoolModifyLiquidityTest", [
    poolManager.address,
  ]);

  const agcAbi = await inspect("AGCToken", "abi");
  const usdcAbi = await inspect("MockUSDC", "abi");
  const treasuryVaultAbi = await inspect("StabilityVault", "abi");
  const xagcVaultAbi = await inspect("XAGCVault", "abi");
  const routerAbi = await inspect("SettlementRouter", "abi");
  const controllerAbi = await inspect("PolicyController", "abi", { viaIr: true });
  const poolManagerAbi = await inspect("PoolManager", "abi");
  const liquidityHelperAbi = await inspect("PoolModifyLiquidityTest", "abi");

  const minterRole = keccak256(stringToHex("MINTER_ROLE"));
  const burnerRole = keccak256(stringToHex("BURNER_ROLE"));

  await writeContract(walletClient, publicClient, {
    address: agc.address,
    abi: agcAbi,
    functionName: "grantRole",
    args: [minterRole, account.address],
  });
  await writeContract(walletClient, publicClient, {
    address: agc.address,
    abi: agcAbi,
    functionName: "grantRole",
    args: [minterRole, controller.address],
  });
  await writeContract(walletClient, publicClient, {
    address: agc.address,
    abi: agcAbi,
    functionName: "grantRole",
    args: [burnerRole, treasuryVault.address],
  });
  await writeContract(walletClient, publicClient, {
    address: agc.address,
    abi: agcAbi,
    functionName: "grantRole",
    args: [burnerRole, router.address],
  });

  await writeContract(walletClient, publicClient, {
    address: treasuryVault.address,
    abi: treasuryVaultAbi,
    functionName: "setPolicyController",
    args: [controller.address],
  });
  await writeContract(walletClient, publicClient, {
    address: treasuryVault.address,
    abi: treasuryVaultAbi,
    functionName: "setSettlementRouter",
    args: [router.address],
  });
  await writeContract(walletClient, publicClient, {
    address: hook.address,
    abi: hook.abi,
    functionName: "setController",
    args: [controller.address],
  });
  await writeContract(walletClient, publicClient, {
    address: router.address,
    abi: routerAbi,
    functionName: "setController",
    args: [controller.address],
  });
  await writeContract(walletClient, publicClient, {
    address: controller.address,
    abi: controllerAbi,
    functionName: "setKeeper",
    args: [account.address, true],
  });
  await writeContract(walletClient, publicClient, {
    address: controller.address,
    abi: controllerAbi,
    functionName: "setSettlementRecipients",
    args: [[account.address, account.address, account.address]],
  });

  await writeContract(walletClient, publicClient, {
    address: agc.address,
    abi: agcAbi,
    functionName: "mint",
    args: [account.address, 1_000_000n * 10n ** 18n],
  });
  await writeContract(walletClient, publicClient, {
    address: usdc.address,
    abi: usdcAbi,
    functionName: "mint",
    args: [account.address, 1_000_000n * 10n ** 6n],
  });
  await writeContract(walletClient, publicClient, {
    address: usdc.address,
    abi: usdcAbi,
    functionName: "mint",
    args: [treasuryVault.address, 250_000n * 10n ** 6n],
  });

  await writeContract(walletClient, publicClient, {
    address: agc.address,
    abi: agcAbi,
    functionName: "approve",
    args: [xagcVault.address, 2n ** 256n - 1n],
  });
  await writeContract(walletClient, publicClient, {
    address: xagcVault.address,
    abi: xagcVaultAbi,
    functionName: "deposit",
    args: [150_000n * 10n ** 18n, account.address],
  });

  const canonicalPoolKeyRaw = await publicClient.readContract({
    address: hook.address,
    abi: hook.abi,
    functionName: "canonicalPoolKey",
  });
  const canonicalPoolKey = normalizeKey(canonicalPoolKeyRaw);
  const sqrtPriceX96 = sqrtPriceX96ForPriceX18(canonicalPoolKey, agc.address, 5n * 10n ** 17n);

  const simulation = await publicClient.simulateContract({
    account,
    address: poolManager.address,
    abi: poolManagerAbi,
    functionName: "initialize",
    args: [canonicalPoolKey, sqrtPriceX96],
  });
  const initializedTick = Number(simulation.result);
  const initializeHash = await walletClient.writeContract(simulation.request);
  await publicClient.waitForTransactionReceipt({ hash: initializeHash });

  await writeContract(walletClient, publicClient, {
    address: agc.address,
    abi: agcAbi,
    functionName: "approve",
    args: [liquidityHelper.address, 2n ** 256n - 1n],
  });
  await writeContract(walletClient, publicClient, {
    address: usdc.address,
    abi: usdcAbi,
    functionName: "approve",
    args: [liquidityHelper.address, 2n ** 256n - 1n],
  });

  const centerTick = floorTickToSpacing(initializedTick, Number(canonicalPoolKey.tickSpacing));
  const tickLower = centerTick - 600;
  const tickUpper = centerTick + 600;
  await writeContract(walletClient, publicClient, {
    address: liquidityHelper.address,
    abi: liquidityHelperAbi,
    functionName: "modifyLiquidity",
    args: [
      canonicalPoolKey,
      {
        tickLower,
        tickUpper,
        liquidityDelta: 10n ** 18n,
        salt: zeroHash,
      },
      "0x",
    ],
  });

  const deployment = {
    admin: account.address,
    agc: agc.address,
    usdc: usdc.address,
    poolManager: poolManager.address,
    hook: hook.address,
    treasuryVault: treasuryVault.address,
    xagcVault: xagcVault.address,
    policyEngine: engine.address,
    settlementRouter: router.address,
    policyController: controller.address,
    liquidityHelper: liquidityHelper.address,
    hookSalt: hookDeployment.salt,
    hookAddressPrediction: hookDeployment.address,
    sqrtPriceX96: sqrtPriceX96.toString(),
    tickLower,
    tickUpper,
    rpcUrl,
    chainId: anvil.id,
    launchAnchorPriceX18: (5n * 10n ** 17n).toString(),
  };

  await mkdir(path.join(root, "deployments"), { recursive: true });
  await writeFile(
    path.join(root, "deployments", "local.json"),
    `${JSON.stringify(deployment, null, 2)}\n`,
    "utf8",
  );
  await writeFile(
    path.join(root, "web", ".env.local"),
    [
      `VITE_RPC_URL=${rpcUrl}`,
      `VITE_AGC_ADDRESS=${agc.address}`,
      `VITE_USDC_ADDRESS=${usdc.address}`,
      `VITE_HOOK_ADDRESS=${hook.address}`,
      `VITE_POLICY_ENGINE_ADDRESS=${engine.address}`,
      `VITE_POLICY_CONTROLLER_ADDRESS=${controller.address}`,
      `VITE_SETTLEMENT_ROUTER_ADDRESS=${router.address}`,
      `VITE_TREASURY_VAULT_ADDRESS=${treasuryVault.address}`,
      `VITE_XAGC_VAULT_ADDRESS=${xagcVault.address}`,
      "",
    ].join("\n"),
    "utf8",
  );

  console.log(JSON.stringify(deployment, null, 2));
}

main().catch((error) => {
  console.error(error instanceof Error ? error.stack ?? error.message : error);
  process.exitCode = 1;
});
