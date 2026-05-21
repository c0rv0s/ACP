// Demo: drive a full expansion settlement cycle on devnet.
//
//   1. Lowers policy params to enable a short epoch (2s) and relaxed targets so
//      the bootstrap-seeded state can hit the Expansion gates.
//   2. Records two sets of swaps separated by a 3s sleep, settling each.
//   3. Cycle 1 lands as Neutral (buy_growth_bps == 0 on first settle).
//   4. Cycle 2 lands as Expansion → AGC minted into xAGC vault / growth / lp /
//      integrators / treasury per the configured distribution_bps.
//
// Run: pnpm exec tsx script/demo/expansion-cycle.ts
//
// After it finishes, the dashboard tiles should show:
//   - Regime: Expansion
//   - Last settled epoch: 2
//   - AGC supply, xAGC vault assets, treasury all bumped up

import {
  TOKEN_PROGRAM_ID,
  getOrCreateAssociatedTokenAccount,
} from "@solana/spl-token";

import { BN, balanceOf, fmtAgc, makeContext } from "./_lib.ts";

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

function fastParams() {
  return {
    normalBandBps: 300,
    stressedBandBps: 700,
    anchorEmaBps: 500,
    maxAnchorCrawlBps: 100,
    minPremiumBps: 100,
    premiumPersistenceRequired: 1,
    minGrossBuyFloorBps: 1,
    minLockedShareBps: 1,
    targetGrossBuyBps: 5,
    targetNetBuyBps: 5,
    targetLockFlowBps: 1,
    targetBuyGrowthBps: 100,
    targetLockedShareBps: 30,
    expansionReserveCoverageBps: 3000,
    targetReserveCoverageBps: 3010,
    neutralReserveCoverageBps: 2000,
    defenseReserveCoverageBps: 1500,
    hardDefenseReserveCoverageBps: 800,
    minStableCashCoverageBps: 1200,
    targetStableCashCoverageBps: 1300,
    defenseStableCashCoverageBps: 800,
    minLiquidityDepthCoverageBps: 2000,
    targetLiquidityDepthCoverageBps: 2100,
    maxReserveConcentrationBps: 6000,
    maxOracleConfidenceBps: 150,
    maxStaleOracleCount: 0,
    maxExpansionVolatilityBps: 300,
    defenseVolatilityBps: 1000,
    maxExpansionExitPressureBps: 3000,
    defenseExitPressureBps: 7000,
    expansionKappaBps: 1000,
    maxMintPerEpochBps: 100,
    maxMintPerDayBps: 250,
    buybackKappaBps: 5000,
    mildDefenseSpendBps: 500,
    severeDefenseSpendBps: 1500,
    severeStressThresholdBps: 1000,
    recoveryCooldownEpochs: 2,
    policyEpochDuration: new BN(2),
  };
}

async function main() {
  const { admin, addresses, connection, program, pk } = await makeContext();

  console.log("Step 1: lowering policy params for fast demo (2s epoch)...");
  await program.methods
    .setPolicyParams(fastParams())
    .accountsStrict({ state: pk("state"), authority: admin.publicKey })
    .signers([admin])
    .rpc();

  const recordSwap = (usdcAmount: number, priceX18: string, agcToUsdc: boolean) =>
    program.methods
      .recordSwap({
        agcAmount: new BN(0),
        usdcAmount: new BN(usdcAmount),
        priceX18: new BN(priceX18),
        agcToUsdc,
        hookFeeUsdc: new BN(0),
        hookFeeAgc: new BN(0),
      })
      .accountsStrict({
        state: pk("state"),
        authority: admin.publicKey,
        keeper: admin.publicKey,
      })
      .signers([admin])
      .rpc();

  // Sized for the bootstrap-minted ~10M AGC float (credit_outstanding ≈ 1e25).
  // Each metric needs to land above the corresponding bps threshold; we feed
  // 5e25 for everything so coverage is ~5000 bps after the divide.
  const healthy = {
    depthToTargetSlippageQuoteX18: new BN("50000000000000000000000000"),
    stableCashReserveQuoteX18: new BN("50000000000000000000000000"),
    riskWeightedReserveQuoteX18: new BN("50000000000000000000000000"),
    liquidityDepthQuoteX18: new BN("50000000000000000000000000"),
    largestCollateralConcentrationBps: 4500,
    oracleConfidenceBps: 25,
    staleOracleCount: 0,
  };

  const settle = () =>
    program.methods
      .settleEpoch(healthy)
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
        mintAuthority: pk("mintAuthority"),
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([admin])
      .rpc();

  // Cycle 1
  console.log("Step 2: recording cycle 1 swaps (price $1.02, premium ~200 bps)...");
  await recordSwap(5_000_000_000, "1040000000000000000", false); // 5k USDC buy @ $1.04
  await recordSwap(1_000_000_000, "1040000000000000000", true); // 1k USDC sell @ $1.04
  console.log("Step 3: waiting 3s for epoch window...");
  await sleep(3000);
  console.log("Step 4: settling cycle 1 (expected Neutral — buy_growth=0 first epoch)...");
  await settle();
  const state1 = await program.account.protocolState.fetch(pk("state"));
  console.log(`  → regime: ${Object.keys(state1.regime)[0]}, last_settled_epoch: ${state1.lastSettledEpoch}`);

  // Top up xAGC so cycle 2 has positive lock_flow. (After cycle 1 settle, the
  // gross-deposit watermark is rolled forward, so cycle 2 needs new flow.)
  // The amount only needs to be enough that net_deposits * BPS / float_supply
  // produces at least 1 bps — float is ~60k AGC after the reset drained most
  // of the deployer's supply to treasury, so even 10 AGC is plenty.
  console.log("Step 5: depositing 50 AGC to xAGC vault to seed cycle-2 lock_flow...");
  const xagcAta = (
    await getOrCreateAssociatedTokenAccount(
      connection,
      admin,
      pk("xagcMint"),
      admin.publicKey,
    )
  ).address;
  await program.methods
    .depositXagc(new BN(50 * 1e9))
    .accountsStrict({
      state: pk("state"),
      depositor: admin.publicKey,
      depositorAgc: pk("deployerAgc"),
      xagcVaultAgc: pk("xagcVaultAgc"),
      xagcMint: pk("xagcMint"),
      receiverXagc: xagcAta,
      mintAuthority: pk("mintAuthority"),
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .signers([admin])
    .rpc();

  // Cycle 2
  console.log("Step 6: recording cycle 2 swaps (price $1.025, higher gross_buy)...");
  await recordSwap(10_000_000_000, "1060000000000000000", false); // 10k USDC buy @ $1.06
  await recordSwap(1_000_000_000, "1060000000000000000", true); // 1k USDC sell @ $1.06
  console.log("Step 7: waiting 3s for epoch window...");
  await sleep(3000);

  const balancesBefore = {
    xagcVault: await balanceOf(connection, pk("xagcVaultAgc")),
    growth: await balanceOf(connection, pk("growthAgc")),
    lp: await balanceOf(connection, pk("lpAgc")),
    integrators: await balanceOf(connection, pk("integratorsAgc")),
    treasury: await balanceOf(connection, pk("treasuryAgc")),
  };

  console.log("Step 8: settling cycle 2 (expected Expansion)...");
  const sig = await settle();
  const state2 = await program.account.protocolState.fetch(pk("state"));
  const regime = Object.keys(state2.regime)[0];
  console.log(`  → regime: ${regime}, last_settled_epoch: ${state2.lastSettledEpoch}`);

  if (regime !== "expansion") {
    const r = state2.lastEpochResult;
    console.log("⚠ regime did not reach Expansion. Cycle 2 diagnostics:");
    console.log(`   premium_bps=${r.premiumBps} (min 100)`);
    console.log(`   premium_persistence=${r.premiumPersistenceEpochs} (req 1)`);
    console.log(`   gross_buy_floor_bps=${r.grossBuyFloorBps} (min ${1})`);
    console.log(`   net_buy_pressure_bps=${r.netBuyPressureBps} (must be > 0)`);
    console.log(`   buy_growth_bps=${r.buyGrowthBps} (must be > 0)`);
    console.log(`   lock_flow_bps=${r.lockFlowBps} (must be > 0)`);
    console.log(`   locked_share_bps=${r.lockedShareBps} (min 1)`);
    console.log(`   reserve_coverage_bps=${r.reserveCoverageBps} (min 3000)`);
    console.log(`   stable_cash_coverage_bps=${r.stableCashCoverageBps} (min 1200)`);
    console.log(`   liquidity_depth_coverage_bps=${r.liquidityDepthCoverageBps} (min 2000)`);
    console.log(`   exit_pressure_bps=${r.exitPressureBps} (max 3000)`);
    console.log(`   volatility_bps=${r.realizedVolatilityBps} (max 300)`);
    console.log(`   stress_score_bps=${r.stressScoreBps}`);
    return;
  }

  const balancesAfter = {
    xagcVault: await balanceOf(connection, pk("xagcVaultAgc")),
    growth: await balanceOf(connection, pk("growthAgc")),
    lp: await balanceOf(connection, pk("lpAgc")),
    integrators: await balanceOf(connection, pk("integratorsAgc")),
    treasury: await balanceOf(connection, pk("treasuryAgc")),
  };

  console.log("\nMint distribution this epoch:");
  console.log(`  xAGC vault:  +${fmtAgc(balancesAfter.xagcVault - balancesBefore.xagcVault)} AGC`);
  console.log(`  growth:      +${fmtAgc(balancesAfter.growth - balancesBefore.growth)} AGC`);
  console.log(`  lp:          +${fmtAgc(balancesAfter.lp - balancesBefore.lp)} AGC`);
  console.log(`  integrators: +${fmtAgc(balancesAfter.integrators - balancesBefore.integrators)} AGC`);
  console.log(`  treasury:    +${fmtAgc(balancesAfter.treasury - balancesBefore.treasury)} AGC`);
  console.log(`\ntx: https://explorer.solana.com/tx/${sig}?cluster=${addresses.cluster}`);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
