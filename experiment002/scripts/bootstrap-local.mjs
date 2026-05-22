import fs from "node:fs";

import {
  ACTION_LOG_PATH,
  DEFAULT_RPC_URL,
  IDL_PATH,
  KEY_DIR,
  LAMPORTS_PER_SOL,
  PROGRAM_KEYPAIR_PATH,
  PROGRAM_SO_PATH,
  PublicKey,
  SOLANA_DIR,
  SystemProgram,
  TOKEN_PROGRAM_ID,
  airdropIfNeeded,
  anchor,
  bn,
  bnUsdc,
  bytes,
  createProgram,
  createProvider,
  ensureDir,
  findPda,
  getOrCreateKeypair,
  hashBytes,
  keyPaths,
  saveKeypair,
  seed16,
  splToken,
  toBuffer,
  web3,
  writeDeployment,
} from "../src/chain.mjs";
import { waitForRpc } from "./lib/process.mjs";

const rpcUrl = process.env.AGC_RPC_URL ?? DEFAULT_RPC_URL;
const connection = await waitForRpc(rpcUrl);
ensureDir(KEY_DIR);

const keys = keyPaths();
const admin = getOrCreateKeypair(keys.admin);
saveKeypair(keys.router, admin);
saveKeypair(keys.underwriter, admin);
saveKeypair(keys.revenuePayer, admin);
const router = admin;
const underwriter = admin;
const revenuePayer = admin;
const agentWallet = getOrCreateKeypair(keys.agentWallet);
const merchantOwner = getOrCreateKeypair(keys.merchantOwner);

await airdropIfNeeded(connection, admin.publicKey, 5);

const provider = createProvider(admin, rpcUrl);
anchor.setProvider(provider);
const program = createProgram(provider);
const programId = program.programId;
const usdcMint = await splToken.createMint(connection, admin, admin.publicKey, null, 6);

const createTokenAccount = (owner) => splToken.createAccount(connection, admin, usdcMint, owner, web3.Keypair.generate());
const underwriterUsdc = await createTokenAccount(underwriter.publicKey);
const revenuePayerUsdc = await createTokenAccount(revenuePayer.publicKey);
const merchantUsdc = await createTokenAccount(merchantOwner.publicKey);
const borrowerReceivableUsdc = await createTokenAccount(admin.publicKey);

await splToken.mintTo(connection, admin, usdcMint, underwriterUsdc, admin, 1_000n * 1_000_000n);
await splToken.mintTo(connection, admin, usdcMint, revenuePayerUsdc, admin, 1_000n * 1_000_000n);

const workflowSeed = seed16(40);
const lineSeed = seed16(80);
const searchMerchantHash = hashBytes(20);
const blockedMerchantHash = hashBytes(60);
const scoreVersionHash = hashBytes(200);

const protocolConfig = findPda([Buffer.from("protocol")], programId);
const borrower = findPda([Buffer.from("borrower"), admin.publicKey.toBuffer()], programId);
const agent = findPda([Buffer.from("agent"), borrower.toBuffer(), agentWallet.publicKey.toBuffer()], programId);
const workflow = findPda([Buffer.from("workflow"), agent.toBuffer(), toBuffer(workflowSeed)], programId);
const searchMerchant = findPda([Buffer.from("merchant"), toBuffer(searchMerchantHash)], programId);
const blockedMerchant = findPda([Buffer.from("merchant"), toBuffer(blockedMerchantHash)], programId);
const policy = findPda([Buffer.from("policy"), workflow.toBuffer()], programId);
const underwriterVault = findPda([Buffer.from("underwriter-vault"), underwriter.publicKey.toBuffer()], programId);
const vaultAuthority = findPda([Buffer.from("underwriter-vault-authority"), underwriterVault.toBuffer()], programId);
const vaultUsdc = findPda([Buffer.from("underwriter-vault-usdc"), underwriterVault.toBuffer()], programId);
const creditLine = findPda(
  [Buffer.from("credit-line"), workflow.toBuffer(), underwriterVault.toBuffer(), toBuffer(lineSeed)],
  programId,
);
const scoreAttestation = findPda(
  [Buffer.from("score"), creditLine.toBuffer(), toBuffer(scoreVersionHash)],
  programId,
);

const zero = new PublicKey("11111111111111111111111111111111");
const pubkeyArray = (first, second = zero) => [first, second, ...Array.from({ length: 6 }, () => zero)];
const u16Array = (first, second = 0) => [first, second, ...Array.from({ length: 6 }, () => 0)];

await program.methods
  .initializeProtocol({
    riskAdmin: admin.publicKey,
    emergencyAdmin: admin.publicKey,
    paymentRouter: router.publicKey,
    maxTotalLiveCapitalAtRiskUsdc: bnUsdc("5000"),
    maxSingleBorrowerLimitUsdc: bnUsdc("250"),
    maxUnsecuredExposurePerBorrowerUsdc: bnUsdc("100"),
    maxDailyTotalSpendUsdc: bnUsdc("1000"),
    maxLossBudgetUsdc: bnUsdc("500"),
  })
  .accounts({ protocolConfig, usdcMint, admin: admin.publicKey, systemProgram: SystemProgram.programId })
  .rpc();

await program.methods
  .registerBorrower({
    primaryWallet: admin.publicKey,
    metadataHash: bytes(32, 1),
    borrowerType: { company: {} },
    verificationStatus: { manual: {} },
  })
  .accounts({ protocolConfig, borrower, operator: admin.publicKey, systemProgram: SystemProgram.programId })
  .rpc();

await program.methods
  .registerAgent({
    wallet: agentWallet.publicKey,
    metadataHash: bytes(32, 2),
    framework: { custom: {} },
  })
  .accounts({ protocolConfig, borrower, agent, operator: admin.publicKey, systemProgram: SystemProgram.programId })
  .rpc();

await program.methods
  .createWorkflow({
    workflowSeed,
    metadataHash: bytes(32, 3),
  })
  .accounts({ protocolConfig, agent, workflow, operator: admin.publicKey, systemProgram: SystemProgram.programId })
  .rpc();

await program.methods
  .registerMerchant({
    merchantIdHash: searchMerchantHash,
    metadataHash: bytes(32, 4),
    category: 101,
    status: { active: {} },
    adapter: { mock: {} },
  })
  .accounts({ protocolConfig, merchant: searchMerchant, authority: admin.publicKey, systemProgram: SystemProgram.programId })
  .rpc();

await program.methods
  .registerMerchant({
    merchantIdHash: blockedMerchantHash,
    metadataHash: bytes(32, 5),
    category: 999,
    status: { blocked: {} },
    adapter: { mock: {} },
  })
  .accounts({ protocolConfig, merchant: blockedMerchant, authority: admin.publicKey, systemProgram: SystemProgram.programId })
  .rpc();

await program.methods
  .createSpendPolicy({
    metadataHash: bytes(32, 6),
    maxPerTransactionUsdc: bnUsdc("25"),
    maxDailySpendUsdc: bnUsdc("25"),
    maxWeeklySpendUsdc: bnUsdc("100"),
    allowedMerchants: pubkeyArray(searchMerchant),
    allowedMerchantCount: 1,
    blockedMerchants: pubkeyArray(blockedMerchant),
    blockedMerchantCount: 1,
    allowedCategories: u16Array(101),
    allowedCategoryCount: 1,
    humanApprovalThresholdUsdc: bnUsdc("10"),
    revenueSweepBps: 3000,
    minAvailableLimitAfterSpendUsdc: bn(0),
    cooldownAfterPolicyViolationSeconds: bn(3600),
  })
  .accounts({ protocolConfig, borrower, workflow, policy, operator: admin.publicKey, systemProgram: SystemProgram.programId })
  .rpc();

await program.methods
  .createUnderwriterVault({
    metadataHash: bytes(32, 7),
    maxSingleLineUsdc: bnUsdc("250"),
  })
  .accounts({
    protocolConfig,
    underwriterVault,
    vaultAuthority,
    usdcVault: vaultUsdc,
    usdcMint,
    underwriter: underwriter.publicKey,
    tokenProgram: TOKEN_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
    rent: web3.SYSVAR_RENT_PUBKEY,
  })
  .signers([underwriter])
  .rpc();

await program.methods
  .fundUnderwriterVault(bnUsdc("500"))
  .accounts({
    protocolConfig,
    underwriterVault,
    underwriterUsdc,
    vaultUsdc,
    underwriter: underwriter.publicKey,
    tokenProgram: TOKEN_PROGRAM_ID,
  })
  .signers([underwriter])
  .rpc();

await program.methods
  .approveCreditLine({
    lineSeed,
    metadataHash: bytes(32, 8),
    principalLimitUsdc: bnUsdc("100"),
    aprBps: 1800,
    originationFeeBps: 0,
    tenorSeconds: bn(7 * 24 * 60 * 60),
    gracePeriodSeconds: bn(48 * 60 * 60),
    riskGrade: { b: {} },
    repaymentRule: zero,
  })
  .accounts({
    protocolConfig,
    borrower,
    workflow,
    policy,
    underwriterVault,
    creditLine,
    authority: admin.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .rpc();

await program.methods
  .activateCreditLine()
  .accounts({ protocolConfig, creditLine, authority: admin.publicKey })
  .rpc();

await program.methods
  .updateScoreAttestation({
    scoreVersionHash,
    score: 742,
    riskGrade: { b: {} },
    recommendedLimitUsdc: bnUsdc("100"),
    recommendedAprBps: 1800,
    pdEstimateBps: 900,
    lgdEstimateBps: 4200,
    confidenceBps: 5400,
    featuresHash: bytes(32, 9),
  })
  .accounts({ protocolConfig, creditLine, scoreAttestation, authority: admin.publicKey, systemProgram: SystemProgram.programId })
  .rpc();

const deployment = {
  version: 1,
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
    router: router.publicKey.toBase58(),
    underwriter: underwriter.publicKey.toBase58(),
    revenuePayer: revenuePayer.publicKey.toBase58(),
    agentWallet: agentWallet.publicKey.toBase58(),
    merchantOwner: merchantOwner.publicKey.toBase58(),
  },
  seeds: {
    workflowSeed,
    lineSeed,
    searchMerchantHash,
    blockedMerchantHash,
    scoreVersionHash,
  },
  addresses: {
    protocolConfig: protocolConfig.toBase58(),
    borrower: borrower.toBase58(),
    agent: agent.toBase58(),
    workflow: workflow.toBase58(),
    policy: policy.toBase58(),
    underwriterVault: underwriterVault.toBase58(),
    vaultAuthority: vaultAuthority.toBase58(),
    vaultUsdc: vaultUsdc.toBase58(),
    creditLine: creditLine.toBase58(),
    scoreAttestation: scoreAttestation.toBase58(),
    usdcMint: usdcMint.toBase58(),
    merchants: {
      search: searchMerchant.toBase58(),
      blocked: blockedMerchant.toBase58(),
    },
    tokenAccounts: {
      underwriterUsdc: underwriterUsdc.toBase58(),
      revenuePayerUsdc: revenuePayerUsdc.toBase58(),
      merchantUsdc: merchantUsdc.toBase58(),
      borrowerReceivableUsdc: borrowerReceivableUsdc.toBase58(),
    },
  },
};

writeDeployment(deployment);
fs.rmSync(ACTION_LOG_PATH, { force: true });

console.log("Bootstrapped local AGC credit sandbox");
console.log(`RPC: ${rpcUrl}`);
console.log(`Program: ${programId.toBase58()}`);
console.log(`Credit line: ${creditLine.toBase58()}`);
