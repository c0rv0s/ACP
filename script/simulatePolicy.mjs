import { readFile } from "node:fs/promises";
import path from "node:path";

const BPS = 10_000n;
const root = process.cwd();

const policyParams = {
  baseBandBps: 200n,
  stressedBandBps: 400n,
  anchorEmaBps: 500n,
  maxAnchorCrawlBps: 10n,
  minProductiveUsageBps: 3000n,
  minCoverageBps: 1000n,
  criticalCoverageBps: 500n,
  maxExpansionVolatilityBps: 200n,
  criticalVolatilityBps: 400n,
  maxExpansionExitBps: 2000n,
  criticalExitBps: 4000n,
  maxMintPerEpochBps: 500n,
  maxMintPerDayBps: 5000n,
  expansionKappaBps: 200n,
  buybackKappaBps: 4000n,
  mildDefenseSpendBps: 2500n,
  severeDefenseSpendBps: 8000n,
};

const initialAnchorPriceX18 = 1_000_000_000_000_000_000n;
const floatSupply = 1_000_000n * 10n ** 18n;
const treasuryUsdc = 250_000n * 10n ** 6n;

function toBigIntSnapshot(snapshot) {
  return {
    ...snapshot,
    productiveVolume: BigInt(snapshot.productiveVolume),
    totalVolume: BigInt(snapshot.totalVolume),
    netExitVolume: BigInt(snapshot.netExitVolume),
    shortTwapPriceX18: BigInt(snapshot.shortTwapPriceX18),
    productiveSettlementPriceX18: BigInt(snapshot.productiveSettlementPriceX18),
    realizedVolatilityBps: BigInt(snapshot.realizedVolatilityBps),
    productiveUsers: BigInt(snapshot.productiveUsers),
    repeatUsers: BigInt(snapshot.repeatUsers),
  };
}

function toBigIntExternalMetrics(metrics) {
  return {
    ...metrics,
    depthTo1Pct: BigInt(metrics.depthTo1Pct),
    depthTo2Pct: BigInt(metrics.depthTo2Pct),
    productiveGrowthBps: BigInt(metrics.productiveGrowthBps),
    lpStabilityBps: BigInt(metrics.lpStabilityBps),
    idleShareBps: BigInt(metrics.idleShareBps),
    buybackMinAgcOut: BigInt(metrics.buybackMinAgcOut),
  };
}

function deriveMetrics(snapshot, externalMetrics) {
  return {
    floatSupply,
    price: snapshot.shortTwapPriceX18,
    productiveUsageBps:
      snapshot.totalVolume === 0n
        ? 0n
        : (snapshot.productiveVolume * BPS) / snapshot.totalVolume,
    coverageBps:
      floatSupply === 0n ? 0n : (externalMetrics.depthTo1Pct * BPS) / floatSupply,
    exitPressureBps:
      snapshot.totalVolume === 0n || snapshot.netExitVolume <= 0n
        ? 0n
        : (snapshot.netExitVolume * BPS) / snapshot.totalVolume,
    repeatUserBps:
      snapshot.productiveUsers === 0n
        ? 0n
        : (snapshot.repeatUsers * BPS) / snapshot.productiveUsers,
    volatilityBps: snapshot.realizedVolatilityBps,
  };
}

function selectRegime(metrics, productiveGrowthBps) {
  const floorPrice =
    (initialAnchorPriceX18 * (BPS - policyParams.baseBandBps)) / BPS;

  const inDefense =
    metrics.price < floorPrice ||
    metrics.volatilityBps >= policyParams.criticalVolatilityBps ||
    metrics.coverageBps < policyParams.criticalCoverageBps ||
    metrics.exitPressureBps >= policyParams.criticalExitBps;

  if (inDefense) return "Defense";

  const canExpand =
    metrics.productiveUsageBps >= policyParams.minProductiveUsageBps &&
    metrics.coverageBps >= policyParams.minCoverageBps &&
    metrics.volatilityBps <= policyParams.maxExpansionVolatilityBps &&
    metrics.exitPressureBps <= policyParams.maxExpansionExitBps &&
    productiveGrowthBps > 0n &&
    metrics.price >= initialAnchorPriceX18;

  return canExpand ? "Expansion" : "Neutral";
}

function updateAnchor(snapshot) {
  const referencePrice =
    snapshot.productiveSettlementPriceX18 > 0n
      ? snapshot.productiveSettlementPriceX18
      : snapshot.shortTwapPriceX18;

  if (referencePrice === 0n) return initialAnchorPriceX18;

  const emaPrice =
    (initialAnchorPriceX18 * (BPS - policyParams.anchorEmaBps) +
      referencePrice * policyParams.anchorEmaBps) /
    BPS;
  const minAnchor =
    (initialAnchorPriceX18 * (BPS - policyParams.maxAnchorCrawlBps)) / BPS;
  const maxAnchor =
    (initialAnchorPriceX18 * (BPS + policyParams.maxAnchorCrawlBps)) / BPS;

  if (emaPrice < minAnchor) return minAnchor;
  if (emaPrice > maxAnchor) return maxAnchor;
  return emaPrice;
}

function mintBudget(metrics, productiveGrowthBps) {
  const usageHeadroom =
    metrics.productiveUsageBps > policyParams.minProductiveUsageBps
      ? metrics.productiveUsageBps - policyParams.minProductiveUsageBps
      : 0n;
  const coverageHeadroom =
    metrics.coverageBps > policyParams.minCoverageBps
      ? metrics.coverageBps - policyParams.minCoverageBps
      : 0n;
  const growthHeadroom = productiveGrowthBps > 0n ? productiveGrowthBps : 0n;

  let healthBps = usageHeadroom;
  if (coverageHeadroom < healthBps) healthBps = coverageHeadroom;
  if (growthHeadroom < healthBps) healthBps = growthHeadroom;
  if (healthBps > BPS) healthBps = BPS;

  let mintRateBps = (policyParams.expansionKappaBps * healthBps) / BPS;
  if (mintRateBps > policyParams.maxMintPerEpochBps) {
    mintRateBps = policyParams.maxMintPerEpochBps;
  }

  return (metrics.floatSupply * mintRateBps) / BPS;
}

function buybackBudget(metrics, anchorPriceX18, bandWidthBps) {
  const floorPrice = (anchorPriceX18 * (BPS - bandWidthBps)) / BPS;
  const priceStressBps =
    metrics.price < floorPrice
      ? ((floorPrice - metrics.price) * BPS) / anchorPriceX18
      : 0n;
  const coverageStressBps =
    metrics.coverageBps < policyParams.criticalCoverageBps
      ? policyParams.criticalCoverageBps - metrics.coverageBps
      : 0n;
  const exitStressBps =
    metrics.exitPressureBps > policyParams.criticalExitBps
      ? metrics.exitPressureBps - policyParams.criticalExitBps
      : 0n;
  const volatilityStressBps =
    metrics.volatilityBps > policyParams.criticalVolatilityBps
      ? metrics.volatilityBps - policyParams.criticalVolatilityBps
      : 0n;

  let stressBps = priceStressBps;
  if (coverageStressBps > stressBps) stressBps = coverageStressBps;
  if (exitStressBps > stressBps) stressBps = exitStressBps;
  if (volatilityStressBps > stressBps) stressBps = volatilityStressBps;
  if (stressBps > BPS) stressBps = BPS;

  const targetSpendRateBps =
    stressBps > policyParams.criticalVolatilityBps
      ? policyParams.severeDefenseSpendBps
      : policyParams.mildDefenseSpendBps;
  const stressSpend =
    (treasuryUsdc *
      policyParams.buybackKappaBps *
      stressBps) /
    BPS /
    BPS;
  const cap = (treasuryUsdc * targetSpendRateBps) / BPS;
  return stressSpend > cap ? cap : stressSpend;
}

function formatBps(value) {
  return `${Number(value) / 100}%`;
}

function formatX18(value) {
  return `${Number(value) / 1e18}`;
}

function formatToken(value, decimals) {
  return `${Number(value) / 10 ** decimals}`;
}

async function main() {
  const file = await readFile(
    path.join(root, "configs/policy/scenarios.json"),
    "utf8",
  );
  const scenarios = JSON.parse(file).scenarios;

  for (const scenario of scenarios) {
    const snapshot = toBigIntSnapshot(scenario.snapshot);
    const externalMetrics = toBigIntExternalMetrics(scenario.externalMetrics);
    const metrics = deriveMetrics(snapshot, externalMetrics);
    const regime = selectRegime(metrics, externalMetrics.productiveGrowthBps);
    const anchorPriceX18 = updateAnchor(snapshot);
    const bandWidthBps =
      regime === "Defense" || regime === "Recovery"
        ? policyParams.stressedBandBps
        : policyParams.baseBandBps;
    const mint =
      regime === "Expansion"
        ? mintBudget(metrics, externalMetrics.productiveGrowthBps)
        : 0n;
    const buyback =
      regime === "Defense"
        ? buybackBudget(metrics, anchorPriceX18, bandWidthBps)
        : 0n;

    console.log(`\n[${scenario.name}]`);
    console.log(`regime: ${regime}`);
    console.log(`anchor: ${formatX18(anchorPriceX18)}`);
    console.log(`price: ${formatX18(metrics.price)}`);
    console.log(`productive usage: ${formatBps(metrics.productiveUsageBps)}`);
    console.log(`coverage: ${formatBps(metrics.coverageBps)}`);
    console.log(`exit pressure: ${formatBps(metrics.exitPressureBps)}`);
    console.log(`volatility: ${formatBps(metrics.volatilityBps)}`);
    console.log(`mint budget: ${formatToken(mint, 18)} AGC`);
    console.log(`buyback budget: ${formatToken(buyback, 6)} USDC`);
    // On-chain: this amount is added to PolicyController.pendingTreasuryBuybackUsdc on
    // settleEpoch; spending is via executePendingTreasuryBuyback (chunked, minAgcOut + sqrtPriceLimitX96).
  }
}

main().catch((error) => {
  console.error(error instanceof Error ? error.stack ?? error.message : error);
  process.exitCode = 1;
});
