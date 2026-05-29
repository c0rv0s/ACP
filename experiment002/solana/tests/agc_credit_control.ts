import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  createAccount,
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

function bytes(length: number, seed: number): number[] {
  return Array.from({ length }, (_, index) => (seed + index) % 256);
}

function hash(seed: number): number[] {
  return bytes(32, seed);
}

function seed16(seed: number): number[] {
  return bytes(16, seed);
}

function pda(seeds: Buffer[]): PublicKey {
  return PublicKey.findProgramAddressSync(seeds, PROGRAM_ID)[0];
}

describe("agc_credit_control", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider() as anchor.AnchorProvider;
  const program = anchor.workspace.AgcCreditControl as Program<AgcCreditControl>;
  const admin = provider.wallet.publicKey;
  const applicationSeed = Buffer.from(seed16(40));
  const lineSeed = Buffer.from(seed16(80));
  const scoreHash = Buffer.from(hash(200));

  const marketConfig = pda([Buffer.from("market")]);
  const feeVaultAuthority = pda([Buffer.from("fee-vault-authority")]);
  const feeVault = pda([Buffer.from("fee-vault-usdc")]);
  const borrower = pda([Buffer.from("borrower"), admin.toBuffer()]);
  const liquidityPool = pda([Buffer.from("liquidity-pool"), admin.toBuffer()]);
  const poolAuthority = pda([Buffer.from("pool-authority"), liquidityPool.toBuffer()]);
  const poolUsdc = pda([Buffer.from("pool-usdc"), liquidityPool.toBuffer()]);
  const creditApplication = pda([
    Buffer.from("credit-application"),
    borrower.toBuffer(),
    liquidityPool.toBuffer(),
    applicationSeed,
  ]);
  const creditLine = pda([Buffer.from("credit-line"), creditApplication.toBuffer(), lineSeed]);
  const creditAttestation = pda([Buffer.from("attestation"), borrower.toBuffer(), scoreHash]);

  let usdcMint: PublicKey;
  let managerUsdc: PublicKey;
  let borrowerUsdc: PublicKey;
  let repaymentPayerUsdc: PublicKey;

  before(async () => {
    usdcMint = await createMint(provider.connection, provider.wallet.payer, admin, null, 6);
    managerUsdc = await createAccount(provider.connection, provider.wallet.payer, usdcMint, admin, Keypair.generate());
    borrowerUsdc = await createAccount(provider.connection, provider.wallet.payer, usdcMint, admin, Keypair.generate());
    repaymentPayerUsdc = await createAccount(provider.connection, provider.wallet.payer, usdcMint, admin, Keypair.generate());
    await mintTo(provider.connection, provider.wallet.payer, usdcMint, managerUsdc, provider.wallet.payer, 2_500 * USDC);
    await mintTo(provider.connection, provider.wallet.payer, usdcMint, repaymentPayerUsdc, provider.wallet.payer, 500 * USDC);
  });

  it("runs the onchain credit market loop", async () => {
    await program.methods
      .initializeMarket({
        riskAdmin: admin,
        emergencyAdmin: admin,
        platformFeeBps: 10,
        maxTotalCreditUsdc: new anchor.BN(5_000 * USDC),
        maxSingleBorrowerUsdc: new anchor.BN(1_000 * USDC),
      })
      .accounts({
        marketConfig,
        feeVaultAuthority,
        feeVault,
        usdcMint,
        admin,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc();

    await program.methods
      .registerBorrower({
        primaryWallet: admin,
        metadataHash: hash(1),
        borrowerType: { business: {} },
        verificationStatus: { zkVerified: {} },
        trustScore: 858,
      })
      .accounts({ marketConfig, borrower, operator: admin, systemProgram: SystemProgram.programId })
      .rpc();

    await program.methods
      .createLiquidityPool({
        metadataHash: hash(2),
        policyHash: hash(3),
        approvalMode: { agentic: {} },
        maxSingleLineUsdc: new anchor.BN(1_000 * USDC),
        minTrustScore: 720,
        maxAprBps: 1400,
        autoApproveUnderUsdc: new anchor.BN(100 * USDC),
      })
      .accounts({
        marketConfig,
        liquidityPool,
        vaultAuthority: poolAuthority,
        usdcVault: poolUsdc,
        usdcMint,
        manager: admin,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc();

    await program.methods
      .fundLiquidityPool(new anchor.BN(2_500 * USDC))
      .accounts({
        marketConfig,
        liquidityPool,
        managerUsdc,
        poolUsdcVault: poolUsdc,
        manager: admin,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    await program.methods
      .submitCreditApplication({
        applicationSeed: [...applicationSeed],
        metadataHash: hash(4),
        requestedLimitUsdc: new anchor.BN(500 * USDC),
        proposedAprBps: 875,
        collateralizationBps: 12_000,
      })
      .accounts({
        marketConfig,
        borrower,
        liquidityPool,
        creditApplication,
        operator: admin,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    await program.methods
      .approveCreditApplication({
        lineSeed: [...lineSeed],
        metadataHash: hash(5),
        approvedLimitUsdc: new anchor.BN(500 * USDC),
        aprBps: 875,
        tenorSeconds: new anchor.BN(365 * 24 * 60 * 60),
        riskGrade: { a: {} },
      })
      .accounts({
        marketConfig,
        borrower,
        liquidityPool,
        creditApplication,
        creditLine,
        borrowerUsdc,
        authority: admin,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    await program.methods
      .drawCredit(new anchor.BN(75 * USDC))
      .accounts({
        marketConfig,
        borrower,
        liquidityPool,
        creditLine,
        vaultAuthority: poolAuthority,
        poolUsdcVault: poolUsdc,
        borrowerUsdc,
        operator: admin,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    let borrowerToken = await getAccount(provider.connection, borrowerUsdc);
    assert.equal(Number(borrowerToken.amount), 75 * USDC);

    await program.methods
      .repayCredit(new anchor.BN(25 * USDC))
      .accounts({
        marketConfig,
        borrower,
        liquidityPool,
        creditLine,
        payerUsdc: repaymentPayerUsdc,
        poolUsdcVault: poolUsdc,
        feeVault,
        payer: admin,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    const line = await program.account.creditLine.fetch(creditLine);
    assert.equal(line.principalOutstandingUsdc.toNumber(), 50 * USDC);
    assert.equal(line.platformFeesPaidUsdc.toNumber(), 25_000);

    const feeVaultAfter = await getAccount(provider.connection, feeVault);
    assert.equal(Number(feeVaultAfter.amount), 25_000);

    await program.methods
      .updateCreditAttestation({
        scoreHash: [...scoreHash],
        score: 872,
        trustScore: 872,
        riskGrade: { a: {} },
        recommendedLimitUsdc: new anchor.BN(650 * USDC),
        recommendedAprBps: 825,
        pdEstimateBps: 125,
        lgdEstimateBps: 2100,
        confidenceBps: 9400,
        featuresHash: hash(6),
      })
      .accounts({ marketConfig, borrower, creditLine, creditAttestation, authority: admin, systemProgram: SystemProgram.programId })
      .rpc();

    const attestation = await program.account.creditAttestation.fetch(creditAttestation);
    assert.equal(attestation.trustScore, 872);
  });
});
