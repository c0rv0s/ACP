import fs from "node:fs";

import {
  ACTION_LOG_PATH,
  DEFAULT_RPC_URL,
  IDL_PATH,
  KEY_DIR,
  PROGRAM_KEYPAIR_PATH,
  PROGRAM_SO_PATH,
  PublicKey,
  SOLANA_DIR,
  SystemProgram,
  TOKEN_PROGRAM_ID,
  airdropIfNeeded,
  anchor,
  approvalModeEnum,
  bn,
  bnUsdc,
  borrowerTypeEnum,
  bytes,
  createProgram,
  createProvider,
  ensureDir,
  findPda,
  getOrCreateKeypair,
  hashBytes,
  keyPaths,
  riskGradeEnum,
  saveKeypair,
  seed16,
  splToken,
  toBuffer,
  verificationStatusEnum,
  web3,
  writeDeployment,
} from "../src/chain.mjs";
import { waitForRpc } from "./lib/process.mjs";

const rpcUrl = process.env.AGC_RPC_URL ?? DEFAULT_RPC_URL;
const connection = await waitForRpc(rpcUrl);
ensureDir(KEY_DIR);

const keys = keyPaths();
const admin = getOrCreateKeypair(keys.admin);

// The local demo intentionally uses one imported wallet for every role so the
// browser can exercise the full workflow through wallet-signed transactions.
saveKeypair(keys.router, admin);
saveKeypair(keys.underwriter, admin);
saveKeypair(keys.revenuePayer, admin);
const poolManager = admin;
const borrowerOperator = admin;
const repaymentPayer = admin;

await airdropIfNeeded(connection, admin.publicKey, 5);

const provider = createProvider(admin, rpcUrl);
anchor.setProvider(provider);
const program = createProgram(provider);
const programId = program.programId;

const usdcMint = await splToken.createMint(connection, admin, admin.publicKey, null, 6);
const createTokenAccount = (owner) => splToken.createAccount(connection, admin, usdcMint, owner, web3.Keypair.generate());
const poolManagerUsdc = await createTokenAccount(poolManager.publicKey);
const borrowerUsdc = await createTokenAccount(borrowerOperator.publicKey);
const repaymentPayerUsdc = await createTokenAccount(repaymentPayer.publicKey);

await splToken.mintTo(connection, admin, usdcMint, poolManagerUsdc, admin, 10_000n * 1_000_000n);
await splToken.mintTo(connection, admin, usdcMint, repaymentPayerUsdc, admin, 2_000n * 1_000_000n);

const applicationSeed = seed16(40);
const lineSeed = seed16(80);
const scoreHash = hashBytes(200);

const marketConfig = findPda([Buffer.from("market")], programId);
const feeVaultAuthority = findPda([Buffer.from("fee-vault-authority")], programId);
const feeVault = findPda([Buffer.from("fee-vault-usdc")], programId);
const borrower = findPda([Buffer.from("borrower"), borrowerOperator.publicKey.toBuffer()], programId);
const liquidityPool = findPda([Buffer.from("liquidity-pool"), poolManager.publicKey.toBuffer()], programId);
const poolAuthority = findPda([Buffer.from("pool-authority"), liquidityPool.toBuffer()], programId);
const poolUsdc = findPda([Buffer.from("pool-usdc"), liquidityPool.toBuffer()], programId);
const creditApplication = findPda(
  [Buffer.from("credit-application"), borrower.toBuffer(), liquidityPool.toBuffer(), toBuffer(applicationSeed)],
  programId,
);
const creditLine = findPda([Buffer.from("credit-line"), creditApplication.toBuffer(), toBuffer(lineSeed)], programId);
const creditAttestation = findPda([Buffer.from("attestation"), borrower.toBuffer(), toBuffer(scoreHash)], programId);

await program.methods
  .initializeMarket({
    riskAdmin: admin.publicKey,
    emergencyAdmin: admin.publicKey,
    platformFeeBps: 10,
    maxTotalCreditUsdc: bnUsdc("5000"),
    maxSingleBorrowerUsdc: bnUsdc("1000"),
  })
  .accounts({
    marketConfig,
    feeVaultAuthority,
    feeVault,
    usdcMint,
    admin: admin.publicKey,
    tokenProgram: TOKEN_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
    rent: web3.SYSVAR_RENT_PUBKEY,
  })
  .rpc();

await program.methods
  .registerBorrower({
    primaryWallet: borrowerOperator.publicKey,
    metadataHash: bytes(32, 1),
    borrowerType: borrowerTypeEnum("Business"),
    verificationStatus: verificationStatusEnum("ZkVerified"),
    trustScore: 858,
  })
  .accounts({ marketConfig, borrower, operator: borrowerOperator.publicKey, systemProgram: SystemProgram.programId })
  .rpc();

await program.methods
  .createLiquidityPool({
    metadataHash: bytes(32, 2),
    policyHash: bytes(32, 3),
    approvalMode: approvalModeEnum("Agentic"),
    maxSingleLineUsdc: bnUsdc("1000"),
    minTrustScore: 720,
    maxAprBps: 1400,
    autoApproveUnderUsdc: bnUsdc("100"),
  })
  .accounts({
    marketConfig,
    liquidityPool,
    vaultAuthority: poolAuthority,
    usdcVault: poolUsdc,
    usdcMint,
    manager: poolManager.publicKey,
    tokenProgram: TOKEN_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
    rent: web3.SYSVAR_RENT_PUBKEY,
  })
  .rpc();

await program.methods
  .fundLiquidityPool(bnUsdc("2500"))
  .accounts({
    marketConfig,
    liquidityPool,
    managerUsdc: poolManagerUsdc,
    poolUsdcVault: poolUsdc,
    manager: poolManager.publicKey,
    tokenProgram: TOKEN_PROGRAM_ID,
  })
  .rpc();

await program.methods
  .submitCreditApplication({
    applicationSeed,
    metadataHash: bytes(32, 4),
    requestedLimitUsdc: bnUsdc("500"),
    proposedAprBps: 875,
    collateralizationBps: 12000,
  })
  .accounts({
    marketConfig,
    borrower,
    liquidityPool,
    creditApplication,
    operator: borrowerOperator.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .rpc();

await program.methods
  .approveCreditApplication({
    lineSeed,
    metadataHash: bytes(32, 5),
    approvedLimitUsdc: bnUsdc("500"),
    aprBps: 875,
    tenorSeconds: bn(365 * 24 * 60 * 60),
    riskGrade: riskGradeEnum("A"),
  })
  .accounts({
    marketConfig,
    borrower,
    liquidityPool,
    creditApplication,
    creditLine,
    borrowerUsdc,
    authority: admin.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .rpc();

await program.methods
  .updateCreditAttestation({
    scoreHash,
    score: 858,
    trustScore: 858,
    riskGrade: riskGradeEnum("A"),
    recommendedLimitUsdc: bnUsdc("500"),
    recommendedAprBps: 875,
    pdEstimateBps: 140,
    lgdEstimateBps: 2200,
    confidenceBps: 9300,
    featuresHash: bytes(32, 6),
  })
  .accounts({
    marketConfig,
    borrower,
    creditLine,
    creditAttestation,
    authority: admin.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .rpc();

const deployment = {
  version: 2,
  generatedAt: new Date().toISOString(),
  rpcUrl,
  programId: programId.toBase58(),
  rootDir: SOLANA_DIR,
  idlPath: IDL_PATH,
  programSoPath: PROGRAM_SO_PATH,
  programKeypairPath: PROGRAM_KEYPAIR_PATH,
  keys,
  publicKeys: {
    admin: admin.publicKey.toBase58(),
    riskAdmin: admin.publicKey.toBase58(),
    poolManager: poolManager.publicKey.toBase58(),
    borrowerOperator: borrowerOperator.publicKey.toBase58(),
    repaymentPayer: repaymentPayer.publicKey.toBase58(),
  },
  seeds: {
    applicationSeed,
    lineSeed,
    scoreHash,
  },
  addresses: {
    marketConfig: marketConfig.toBase58(),
    feeVaultAuthority: feeVaultAuthority.toBase58(),
    feeVault: feeVault.toBase58(),
    borrower: borrower.toBase58(),
    liquidityPool: liquidityPool.toBase58(),
    poolAuthority: poolAuthority.toBase58(),
    poolUsdc: poolUsdc.toBase58(),
    creditApplication: creditApplication.toBase58(),
    creditLine: creditLine.toBase58(),
    creditAttestation: creditAttestation.toBase58(),
    usdcMint: usdcMint.toBase58(),
    tokenAccounts: {
      poolManagerUsdc: poolManagerUsdc.toBase58(),
      borrowerUsdc: borrowerUsdc.toBase58(),
      repaymentPayerUsdc: repaymentPayerUsdc.toBase58(),
    },
  },
};

writeDeployment(deployment);
fs.rmSync(ACTION_LOG_PATH, { force: true });

console.log("Bootstrapped local Agent Credit market");
console.log(`RPC: ${rpcUrl}`);
console.log(`Program: ${programId.toBase58()}`);
console.log(`Credit line: ${creditLine.toBase58()}`);
