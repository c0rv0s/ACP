// Pre-recording reset. Brings the live devnet protocol back to a clean
// demo-ready state:
//   1. Redeems all of the deployer's xAGC back into AGC.
//   2. Drains the deployer's AGC down to TARGET_AGC so deposit/underwrite
//      operations during the recording produce visible balance changes.
//   3. Settles an epoch with no fresh swaps recorded, which lands on Neutral
//      regime (premium-persistence carries but gross_buy_floor + buy_growth
//      drop to zero, so expansion gate fails).
//
// Run from repo root:
//   pnpm exec tsx script/demo/reset.ts

import { PublicKey, SystemProgram } from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  createTransferInstruction,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";

import { BN, balanceOf, fmtAgc, makeContext, pdaWith } from "./_lib.ts";

const TARGET_AGC = 2_000n * 1_000_000_000n; // 2,000 AGC * 1e9

async function main() {
  const { admin, addresses, programId, connection, program, pk } =
    await makeContext();
  const dash = "─".repeat(64);
  console.log(dash);
  console.log("Pre-recording reset");
  console.log("Wallet:", admin.publicKey.toBase58());
  console.log(dash);

  const userXagcAta = getAssociatedTokenAddressSync(
    pk("xagcMint"),
    admin.publicKey,
  );
  const userAgcAta = pk("deployerAgc");
  const mintAuthority = PublicKey.findProgramAddressSync(
    [Buffer.from("mint-authority")],
    programId,
  )[0];
  const xagcAuthority = PublicKey.findProgramAddressSync(
    [Buffer.from("xagc-authority")],
    programId,
  )[0];

  // ─────────────────────────────────────────────────────────────────────
  // 1. Redeem all xAGC shares
  // ─────────────────────────────────────────────────────────────────────
  const xagcShares = await balanceOf(connection, userXagcAta);
  if (xagcShares > 0n) {
    console.log(`\n[1/3] Redeeming ${fmtAgc(xagcShares)} xAGC shares…`);
    const sig = await program.methods
      .redeemXagc(new BN(xagcShares.toString()))
      .accountsStrict({
        state: pk("state"),
        owner: admin.publicKey,
        ownerXagc: userXagcAta,
        xagcMint: pk("xagcMint"),
        xagcVaultAgc: pk("xagcVaultAgc"),
        treasuryAgc: pk("treasuryAgc"),
        receiverAgc: userAgcAta,
        xagcAuthority,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([admin])
      .rpc();
    console.log(`     tx: ${sig}`);
  } else {
    console.log("\n[1/3] No xAGC to redeem, skipping.");
  }

  // ─────────────────────────────────────────────────────────────────────
  // 2. Drain AGC to TARGET_AGC via transfer to treasury_agc
  // ─────────────────────────────────────────────────────────────────────
  const currentAgc = await balanceOf(connection, userAgcAta);
  if (currentAgc > TARGET_AGC) {
    const drain = currentAgc - TARGET_AGC;
    console.log(
      `\n[2/3] Draining ${fmtAgc(drain)} AGC → treasury (keeping ${fmtAgc(TARGET_AGC)} AGC)…`,
    );
    const drainIx = createTransferInstruction(
      userAgcAta,
      pk("treasuryAgc"),
      admin.publicKey,
      drain,
    );
    const sig = await program.provider.sendAndConfirm!(
      new (await import("@solana/web3.js")).Transaction().add(drainIx),
      [admin],
    );
    console.log(`     tx: ${sig}`);
  } else {
    console.log(
      `\n[2/3] AGC already at or below target (${fmtAgc(currentAgc)} ≤ ${fmtAgc(TARGET_AGC)}), skipping.`,
    );
  }

  // ─────────────────────────────────────────────────────────────────────
  // 3. Settle an empty epoch → regime drops back to Neutral
  // ─────────────────────────────────────────────────────────────────────
  console.log("\n[3/3] Settling empty epoch (regime should drop to Neutral)…");
  const healthyMetrics = {
    depthToTargetSlippageQuoteX18: new BN("250000000000000000000000"),
    stableCashReserveQuoteX18: new BN("200000000000000000000000"),
    riskWeightedReserveQuoteX18: new BN("300000000000000000000000"),
    liquidityDepthQuoteX18: new BN("250000000000000000000000"),
    largestCollateralConcentrationBps: 4500,
    oracleConfidenceBps: 25,
    staleOracleCount: 0,
  };
  // Wait long enough for the epoch window. expansion-cycle.ts shrank
  // policy_epoch_duration to 2s; even if it's been restored to 3600s, the
  // accumulator's started_at is the last-settle timestamp and we just need
  // to be past it. Wait 3s defensively.
  await new Promise((r) => setTimeout(r, 3000));

  try {
    const sig = await program.methods
      .settleEpoch(healthyMetrics)
      .accountsStrict({
        state: pk("state"),
        authority: admin.publicKey,
        keeper: admin.publicKey,
        agcMint: pk("agcMint"),
        xagcMint: pk("xagcMint"),
        xagcVaultAgc: pk("xagcVaultAgc"),
        treasuryAgc: pk("treasuryAgc"),
        treasuryUsdc: pk("treasuryUsdc"),
        growthProgramsAgc: pk("growthAgc"),
        lpAgc: pk("lpAgc"),
        integratorsAgc: pk("integratorsAgc"),
        mintAuthority,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([admin])
      .rpc();
    console.log(`     tx: ${sig}`);
    const after = await program.account.protocolState.fetch(pk("state"));
    console.log(`     regime now: ${JSON.stringify(after.regime)}`);
    console.log(`     last_settled_epoch: ${after.lastSettledEpoch.toString()}`);
  } catch (err) {
    console.log("     settle failed (likely EpochTooSoon — try again in 30s):", String(err).split("\n")[0]);
  }

  console.log("\n" + dash);
  const finalAgc = await balanceOf(connection, userAgcAta);
  const finalXagc = await balanceOf(connection, userXagcAta);
  console.log(`Ready for recording. Deployer holds ${fmtAgc(finalAgc)} AGC, ${fmtAgc(finalXagc)} xAGC.`);
  console.log(dash);
}

main().catch((err) => {
  console.error("\nreset failed:", err);
  process.exit(1);
});
