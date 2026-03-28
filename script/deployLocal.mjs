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
  PoolManager: { identifier: "PoolManager", contractsPath: "lib/v4-core/src" },
  PoolModifyLiquidityTest: { identifier: "PoolModifyLiquidityTest", contractsPath: "lib/v4-core/src/test" },
  RewardDistributor: { identifier: "src/RewardDistributor.sol:RewardDistributor" },
  SettlementRouter: { identifier: "src/SettlementRouter.sol:SettlementRouter" },
  StabilityVault: { identifier: "src/StabilityVault.sol:StabilityVault" },
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

function softDollarSqrtPriceX96(poolKey, agcAddress) {
  const agcIsCurrency0 = poolKey.currency0.toLowerCase() === agcAddress.toLowerCase();
  const amount0 = agcIsCurrency0 ? 10n ** 18n : 10n ** 6n;
  const amount1 = agcIsCurrency0 ? 10n ** 6n : 10n ** 18n;
  const ratioX192 = (amount1 << 192n) / amount0;
  return sqrt(ratioX192);
}

function rewardSplitArgs() {
  return [3000, 2000, 2000, 2000, 1000];
}

function policyParamArgs() {
  return [
    200,
    400,
    500,
    10,
    3000,
    1000,
    500,
    200,
    400,
    2000,
    4000,
    5,
    50,
    200,
    4000,
    2500,
    8000,
    4,
    3600,
    86400,
  ];
}

function hookFeeArgs() {
  return [
    1000,
    1500,
    2000,
    400,
    150,
    1000,
    750,
    0,
    100,
    750,
    300,
    2000,
    1500,
    86400,
  ];
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
  const vault = await deployContract(walletClient, publicClient, "StabilityVault", [
    account.address,
    agc.address,
    usdc.address,
  ]);
  const poolManager = await deployContract(walletClient, publicClient, "PoolManager", [account.address]);
  const hookDeployer = await deployContract(walletClient, publicClient, "HookDeployer");

  const agcHookAbi = await inspect("AGCHook", "abi");
  const agcHookBytecode = await inspect("AGCHook", "bytecode");
  const poolConfig = [agc.address, usdc.address, 0x800000, 60, 18, 6];
  const hookCreationCode = encodeDeployData({
    abi: agcHookAbi,
    bytecode: agcHookBytecode,
    args: [account.address, poolManager.address, vault.address, poolConfig, hookFeeArgs()],
  });

  let hookSalt = zeroHash;
  let predictedHookAddress = null;
  for (let i = 0n; i < 250000n; i += 1n) {
    const salt = toHex(i, { size: 32 });
    const candidate = getCreate2Address({
      from: hookDeployer.address,
      salt,
      bytecodeHash: keccak256(hookCreationCode),
    });
    if ((BigInt(candidate) & REQUIRED_HOOK_FLAGS) === REQUIRED_HOOK_FLAGS) {
      hookSalt = salt;
      predictedHookAddress = candidate;
      break;
    }
  }
  if (!predictedHookAddress) {
    throw new Error("Failed to find a valid hook salt.");
  }

  const hookDeployerAbi = await inspect("HookDeployer", "abi");
  await writeContract(walletClient, publicClient, {
    address: hookDeployer.address,
    abi: hookDeployerAbi,
    functionName: "deploy",
    args: [hookCreationCode, hookSalt],
  });
  const hookCode = await publicClient.getBytecode({ address: predictedHookAddress });
  if (!hookCode) {
    throw new Error("AGCHook deployment did not produce bytecode.");
  }
  const hook = { address: predictedHookAddress, abi: agcHookAbi };

  const distributor = await deployContract(walletClient, publicClient, "RewardDistributor", [
    account.address,
    agc.address,
    hook.address,
  ]);
  const router = await deployContract(walletClient, publicClient, "SettlementRouter", [
    account.address,
    agc.address,
    usdc.address,
    poolManager.address,
    hook.address,
    vault.address,
  ]);
  const controller = await deployContract(
    walletClient,
    publicClient,
    "PolicyController",
    [
      account.address,
      [agc.address, hook.address, vault.address, distributor.address, router.address],
      10n ** 18n,
      policyParamArgs(),
      rewardSplitArgs(),
    ],
    { viaIr: true },
  );
  const liquidityHelper = await deployContract(walletClient, publicClient, "PoolModifyLiquidityTest", [
    poolManager.address,
  ]);

  const agcAbi = await inspect("AGCToken", "abi");
  const usdcAbi = await inspect("MockUSDC", "abi");
  const vaultAbi = await inspect("StabilityVault", "abi");
  const distributorAbi = await inspect("RewardDistributor", "abi");
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
    args: [burnerRole, vault.address],
  });
  await writeContract(walletClient, publicClient, {
    address: agc.address,
    abi: agcAbi,
    functionName: "grantRole",
    args: [burnerRole, router.address],
  });

  await writeContract(walletClient, publicClient, {
    address: vault.address,
    abi: vaultAbi,
    functionName: "setPolicyController",
    args: [controller.address],
  });
  await writeContract(walletClient, publicClient, {
    address: vault.address,
    abi: vaultAbi,
    functionName: "setSettlementRouter",
    args: [router.address],
  });
  await writeContract(walletClient, publicClient, {
    address: distributor.address,
    abi: distributorAbi,
    functionName: "setController",
    args: [controller.address],
  });
  await writeContract(walletClient, publicClient, {
    address: hook.address,
    abi: hook.abi,
    functionName: "setController",
    args: [controller.address],
  });
  await writeContract(walletClient, publicClient, {
    address: hook.address,
    abi: hook.abi,
    functionName: "setRewardDistributor",
    args: [distributor.address],
  });
  await writeContract(walletClient, publicClient, {
    address: hook.address,
    abi: hook.abi,
    functionName: "setTrustedRouter",
    args: [router.address, true],
  });
  await writeContract(walletClient, publicClient, {
    address: router.address,
    abi: routerAbi,
    functionName: "setController",
    args: [controller.address],
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
    args: [vault.address, 250_000n * 10n ** 6n],
  });

  const canonicalPoolKeyRaw = await publicClient.readContract({
    address: hook.address,
    abi: hook.abi,
    functionName: "canonicalPoolKey",
  });
  const canonicalPoolKey = normalizeKey(canonicalPoolKeyRaw);
  const sqrtPriceX96 = softDollarSqrtPriceX96(canonicalPoolKey, agc.address);

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
    vault: vault.address,
    rewardDistributor: distributor.address,
    settlementRouter: router.address,
    policyController: controller.address,
    liquidityHelper: liquidityHelper.address,
    sqrtPriceX96: sqrtPriceX96.toString(),
    tickLower,
    tickUpper,
    hookSalt,
    hookAddressPrediction: predictedHookAddress,
    rpcUrl,
    chainId: anvil.id,
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
      `VITE_POLICY_CONTROLLER_ADDRESS=${controller.address}`,
      `VITE_REWARD_DISTRIBUTOR_ADDRESS=${distributor.address}`,
      `VITE_SETTLEMENT_ROUTER_ADDRESS=${router.address}`,
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
