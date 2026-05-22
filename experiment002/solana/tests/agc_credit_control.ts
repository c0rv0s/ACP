import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  createAssociatedTokenAccount,
  createMint,
  getAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import { strict as assert } from "node:assert";

import type { AgcCreditControl } from "../target/types/agc_credit_control";

const USDC = 1_000_000;
const PROGRAM_ID = new PublicKey("6GCCBywkt8WwNoaroggSwnMkd8ggWvLNBzpWnFXWWR6n");
const ZERO = new PublicKey("11111111111111111111111111111111");

function bytes(length: number, seed: number): number[] {
  return Array.from({ length }, (_, index) => (seed + index) % 256);
}

function hash(seed: number): number[] {
  return bytes(32, seed);
}

function seed16(seed: number): number[] {
  return bytes(16, seed);
}

function pubkeyArray(first: PublicKey): PublicKey[] {
  return [first, ...Array.from({ length: 7 }, () => ZERO)];
}

function u16Array(first: number): number[] {
  return [first, ...Array.from({ length: 7 }, () => 0)];
}

function pda(seeds: Buffer[]): PublicKey {
  return PublicKey.findProgramAddressSync(seeds, PROGRAM_ID)[0];
}

describe("agc_credit_control", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider() as anchor.AnchorProvider;
  const program = anchor.workspace.AgcCreditControl as Program<AgcCreditControl>;
  const admin = provider.wallet.publicKey;
  const router = Keypair.generate();
  const underwriter = Keypair.generate();
  const revenuePayer = Keypair.generate();
  const agentWallet = Keypair.generate().publicKey;
  const workflowSeed = Buffer.from(seed16(40));
  const lineSeed = Buffer.from(seed16(80));
  const spendId = Buffer.from(seed16(120));
  const merchantHash = Buffer.from(hash(20));
  const scoreVersionHash = Buffer.from(hash(200));

  const protocolConfig = pda([Buffer.from("protocol")]);
  let usdcMint: PublicKey;
  let borrower: PublicKey;
  let agent: PublicKey;
  let workflow: PublicKey;
  let merchant: PublicKey;
  let policy: PublicKey;
  let underwriterVault: PublicKey;
  let vaultAuthority: PublicKey;
  let vaultUsdc: PublicKey;
  let creditLine: PublicKey;
  let spendAuthorization: PublicKey;
  let scoreAttestation: PublicKey;
  let underwriterUsdc: PublicKey;
  let merchantUsdc: PublicKey;
  let revenuePayerUsdc: PublicKey;
  let borrowerReceivableUsdc: PublicKey;

  before(async () => {
    for (const keypair of [router, underwriter, revenuePayer]) {
      const sig = await provider.connection.requestAirdrop(keypair.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL);
      await provider.connection.confirmTransaction(sig, "confirmed");
    }

    usdcMint = await createMint(provider.connection, provider.wallet.payer, admin, null, 6);
    underwriterUsdc = await createAssociatedTokenAccount(
      provider.connection,
      provider.wallet.payer,
      usdcMint,
      underwriter.publicKey,
    );
    merchantUsdc = await createAssociatedTokenAccount(
      provider.connection,
      provider.wallet.payer,
      usdcMint,
      Keypair.generate().publicKey,
    );
    revenuePayerUsdc = await createAssociatedTokenAccount(
      provider.connection,
      provider.wallet.payer,
      usdcMint,
      revenuePayer.publicKey,
    );
    borrowerReceivableUsdc = await createAssociatedTokenAccount(
      provider.connection,
      provider.wallet.payer,
      usdcMint,
      admin,
    );
    await mintTo(provider.connection, provider.wallet.payer, usdcMint, underwriterUsdc, provider.wallet.payer, 500 * USDC);
    await mintTo(provider.connection, provider.wallet.payer, usdcMint, revenuePayerUsdc, provider.wallet.payer, 50 * USDC);

    borrower = pda([Buffer.from("borrower"), admin.toBuffer()]);
    agent = pda([Buffer.from("agent"), borrower.toBuffer(), agentWallet.toBuffer()]);
    workflow = pda([Buffer.from("workflow"), agent.toBuffer(), workflowSeed]);
    merchant = pda([Buffer.from("merchant"), merchantHash]);
    policy = pda([Buffer.from("policy"), workflow.toBuffer()]);
    underwriterVault = pda([Buffer.from("underwriter-vault"), underwriter.publicKey.toBuffer()]);
    vaultAuthority = pda([Buffer.from("underwriter-vault-authority"), underwriterVault.toBuffer()]);
    vaultUsdc = pda([Buffer.from("underwriter-vault-usdc"), underwriterVault.toBuffer()]);
    creditLine = pda([Buffer.from("credit-line"), workflow.toBuffer(), underwriterVault.toBuffer(), lineSeed]);
    spendAuthorization = pda([Buffer.from("spend-auth"), creditLine.toBuffer(), spendId]);
    scoreAttestation = pda([Buffer.from("score"), creditLine.toBuffer(), scoreVersionHash]);
  });

  it("runs the controlled credit loop", async () => {
    await program.methods
      .initializeProtocol({
        riskAdmin: admin,
        emergencyAdmin: admin,
        paymentRouter: router.publicKey,
        maxTotalLiveCapitalAtRiskUsdc: new anchor.BN(5_000 * USDC),
        maxSingleBorrowerLimitUsdc: new anchor.BN(250 * USDC),
        maxUnsecuredExposurePerBorrowerUsdc: new anchor.BN(100 * USDC),
        maxDailyTotalSpendUsdc: new anchor.BN(1_000 * USDC),
        maxLossBudgetUsdc: new anchor.BN(500 * USDC),
      })
      .accounts({ protocolConfig, usdcMint, admin, systemProgram: SystemProgram.programId })
      .rpc();

    await program.methods
      .registerBorrower({
        primaryWallet: admin,
        metadataHash: hash(1),
        borrowerType: { company: {} },
        verificationStatus: { manual: {} },
      })
      .accounts({ protocolConfig, borrower, operator: admin, systemProgram: SystemProgram.programId })
      .rpc();

    await program.methods
      .registerAgent({
        wallet: agentWallet,
        metadataHash: hash(2),
        framework: { custom: {} },
      })
      .accounts({ protocolConfig, borrower, agent, operator: admin, systemProgram: SystemProgram.programId })
      .rpc();

    await program.methods
      .createWorkflow({
        workflowSeed: [...workflowSeed],
        metadataHash: hash(3),
      })
      .accounts({ protocolConfig, agent, workflow, operator: admin, systemProgram: SystemProgram.programId })
      .rpc();

    await program.methods
      .registerMerchant({
        merchantIdHash: [...merchantHash],
        metadataHash: hash(4),
        category: 101,
        status: { active: {} },
        adapter: { mock: {} },
      })
      .accounts({ protocolConfig, merchant, authority: admin, systemProgram: SystemProgram.programId })
      .rpc();

    await program.methods
      .createSpendPolicy({
        metadataHash: hash(5),
        maxPerTransactionUsdc: new anchor.BN(25 * USDC),
        maxDailySpendUsdc: new anchor.BN(25 * USDC),
        maxWeeklySpendUsdc: new anchor.BN(100 * USDC),
        allowedMerchants: pubkeyArray(merchant),
        allowedMerchantCount: 1,
        blockedMerchants: pubkeyArray(ZERO),
        blockedMerchantCount: 0,
        allowedCategories: u16Array(101),
        allowedCategoryCount: 1,
        humanApprovalThresholdUsdc: new anchor.BN(10 * USDC),
        revenueSweepBps: 3000,
        minAvailableLimitAfterSpendUsdc: new anchor.BN(0),
        cooldownAfterPolicyViolationSeconds: new anchor.BN(3600),
      })
      .accounts({ protocolConfig, borrower, workflow, policy, operator: admin, systemProgram: SystemProgram.programId })
      .rpc();

    await program.methods
      .createUnderwriterVault({
        metadataHash: hash(6),
        maxSingleLineUsdc: new anchor.BN(250 * USDC),
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
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([underwriter])
      .rpc();

    await program.methods
      .fundUnderwriterVault(new anchor.BN(500 * USDC))
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
        lineSeed: [...lineSeed],
        metadataHash: hash(7),
        principalLimitUsdc: new anchor.BN(100 * USDC),
        aprBps: 1800,
        originationFeeBps: 0,
        tenorSeconds: new anchor.BN(7 * 24 * 60 * 60),
        gracePeriodSeconds: new anchor.BN(48 * 60 * 60),
        riskGrade: { b: {} },
        repaymentRule: ZERO,
      })
      .accounts({
        protocolConfig,
        borrower,
        workflow,
        policy,
        underwriterVault,
        creditLine,
        authority: admin,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    await program.methods
      .activateCreditLine()
      .accounts({ protocolConfig, creditLine, authority: admin })
      .rpc();

    await program.methods
      .reserveSpend({
        spendId: [...spendId],
        amountUsdc: new anchor.BN(2.5 * USDC),
        purposeHash: hash(8),
        authorizationTtlSeconds: new anchor.BN(300),
      })
      .accounts({
        protocolConfig,
        borrower,
        creditLine,
        policy,
        merchant,
        spendAuthorization,
        router: router.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([router])
      .rpc();

    await program.methods
      .settleSpend()
      .accounts({
        protocolConfig,
        borrower,
        creditLine,
        spendAuthorization,
        underwriterVault,
        vaultAuthority,
        underwriterUsdcVault: vaultUsdc,
        merchantUsdc,
        router: router.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([router])
      .rpc();

    const merchantAfterSpend = await getAccount(provider.connection, merchantUsdc);
    assert.equal(Number(merchantAfterSpend.amount), 2.5 * USDC);

    await program.methods
      .recordRevenueAndSweep(new anchor.BN(50 * USDC), { manual: {} }, hash(9))
      .accounts({
        protocolConfig,
        borrower,
        creditLine,
        policy,
        underwriterVault,
        revenueSourceUsdc: revenuePayerUsdc,
        underwriterUsdcVault: vaultUsdc,
        borrowerReceivableUsdc,
        revenuePayer: revenuePayer.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([revenuePayer])
      .rpc();

    const line = await program.account.creditLine.fetch(creditLine);
    assert.equal(line.principalOutstandingUsdc.toNumber(), 0);
    assert.deepEqual(line.status, { repaid: {} });

    const borrowerReceivable = await getAccount(provider.connection, borrowerReceivableUsdc);
    assert.equal(Number(borrowerReceivable.amount), 47.5 * USDC);

    await program.methods
      .updateScoreAttestation({
        scoreVersionHash: [...scoreVersionHash],
        score: 765,
        riskGrade: { b: {} },
        recommendedLimitUsdc: new anchor.BN(100 * USDC),
        recommendedAprBps: 1800,
        pdEstimateBps: 900,
        lgdEstimateBps: 4200,
        confidenceBps: 6200,
        featuresHash: hash(10),
      })
      .accounts({ protocolConfig, creditLine, scoreAttestation, authority: admin, systemProgram: SystemProgram.programId })
      .rpc();

    const score = await program.account.scoreAttestation.fetch(scoreAttestation);
    assert.equal(score.score, 765);
  });
});
