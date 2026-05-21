/* eslint-disable no-console */
// One-shot devnet bootstrap. Creates the AGC/xAGC/USDC/BTC mints, the
// settlement-recipient ATAs, runs `initialize_protocol`, configures a BTC
// collateral asset + oracle, and opens a credit facility against it.
//
// Run from repo root:
//   pnpm exec tsx script/devnet-bootstrap.ts
//
// Persists every relevant address to `deployments/devnet.json`. Re-running is
// not safe — the state PDA is one-shot per program.

import anchorPkg from "@coral-xyz/anchor";
const { AnchorProvider, BN, Program, Wallet, setProvider } = anchorPkg;
type Idl = anchorPkg.Idl;
import {
  AuthorityType,
  TOKEN_PROGRAM_ID,
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  setAuthority,
} from "@solana/spl-token";
import {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";
import { fileURLToPath } from "node:url";

import idl from "../solana/target/idl/agc_solana.json" with { type: "json" };
import type { AgcSolana } from "../solana/target/types/agc_solana";

const PROGRAM_ID = new PublicKey("H1n8VTp6pMY5WFfVfi4MNkQ9q5szkMpVWcHQ21JRETXC");
const PRICE_SCALE = new BN("1000000000000000000"); // 1.0e18

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const REPO_ROOT = path.resolve(__dirname, "..");
const OUTPUT_PATH = path.join(REPO_ROOT, "deployments", "devnet.json");

function loadKeypair(filepath: string): Keypair {
  const expanded = filepath.startsWith("~")
    ? path.join(os.homedir(), filepath.slice(1))
    : filepath;
  const secret = JSON.parse(fs.readFileSync(expanded, "utf8")) as number[];
  return Keypair.fromSecretKey(new Uint8Array(secret));
}

function pda(seed: string, programId: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync([Buffer.from(seed)], programId)[0];
}

function pdaWith(seed: string, key: PublicKey, programId: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(seed), key.toBuffer()],
    programId,
  )[0];
}

function policyParams() {
  return {
    normalBandBps: 300,
    stressedBandBps: 700,
    anchorEmaBps: 500,
    maxAnchorCrawlBps: 100,
    minPremiumBps: 100,
    premiumPersistenceRequired: 2,
    minGrossBuyFloorBps: 50,
    minLockedShareBps: 1000,
    targetGrossBuyBps: 500,
    targetNetBuyBps: 250,
    targetLockFlowBps: 100,
    targetBuyGrowthBps: 500,
    targetLockedShareBps: 3000,
    expansionReserveCoverageBps: 3000,
    targetReserveCoverageBps: 8000,
    neutralReserveCoverageBps: 2000,
    defenseReserveCoverageBps: 1500,
    hardDefenseReserveCoverageBps: 800,
    minStableCashCoverageBps: 1200,
    targetStableCashCoverageBps: 2500,
    defenseStableCashCoverageBps: 800,
    minLiquidityDepthCoverageBps: 2000,
    targetLiquidityDepthCoverageBps: 5000,
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
    policyEpochDuration: new BN(3600),
  };
}

async function main() {
  const admin = loadKeypair("~/.config/solana/id.json");
  console.log("Deployer / admin:", admin.publicKey.toBase58());

  const connection = new Connection("https://api.devnet.solana.com", "confirmed");
  const provider = new AnchorProvider(
    connection,
    new Wallet(admin),
    { commitment: "confirmed" },
  );
  setProvider(provider);

  const program = new Program(idl as Idl, provider) as unknown as Program<AgcSolana>;

  const balanceBefore = await connection.getBalance(admin.publicKey);
  console.log(`Balance before bootstrap: ${(balanceBefore / 1e9).toFixed(4)} SOL`);

  const mintAuthority = pda("mint-authority", PROGRAM_ID);
  const treasuryAuthority = pda("treasury-authority", PROGRAM_ID);
  const xagcAuthority = pda("xagc-authority", PROGRAM_ID);
  const state = pda("state", PROGRAM_ID);
  const treasuryAgc = pda("treasury-agc", PROGRAM_ID);
  const treasuryUsdc = pda("treasury-usdc", PROGRAM_ID);
  const xagcVaultAgc = pda("xagc-vault-agc", PROGRAM_ID);

  // 1. AGC mint — admin auth temporarily so we can pre-mint launch supply.
  console.log("[1/9] creating AGC mint (admin auth)");
  const agcMint = await createMint(connection, admin, admin.publicKey, null, 9);

  // 2. Pre-mint AGC to the deployer wallet so demos can underwrite + repay.
  console.log("[2/9] minting 10,000,000 AGC to deployer");
  const deployerAgc = (
    await getOrCreateAssociatedTokenAccount(connection, admin, agcMint, admin.publicKey)
  ).address;
  await mintTo(
    connection,
    admin,
    agcMint,
    deployerAgc,
    admin,
    10_000_000n * 1_000_000_000n, // 10M AGC, 9 decimals
  );

  // 3. Hand AGC mint authority to the program PDA so the protocol controls supply.
  console.log("[3/9] transferring AGC mint authority to program PDA");
  await setAuthority(connection, admin, agcMint, admin, AuthorityType.MintTokens, mintAuthority);

  // 4. xAGC mint — PDA auth from inception.
  console.log("[4/9] creating xAGC mint (PDA auth)");
  const xagcMint = await createMint(connection, admin, mintAuthority, null, 9);

  // 5. Mock USDC + BTC mints (devnet has no canonical AGC pool, so we issue
  //    our own stablecoin reference and BTC wrapper for the demo).
  console.log("[5/9] creating mock USDC + BTC mints");
  const usdcMint = await createMint(connection, admin, admin.publicKey, null, 6);
  const btcMint = await createMint(connection, admin, admin.publicKey, null, 8);

  const deployerBtc = (
    await getOrCreateAssociatedTokenAccount(connection, admin, btcMint, admin.publicKey)
  ).address;
  await mintTo(connection, admin, btcMint, deployerBtc, admin, 100_000_000n); // 1 BTC (8 dec)
  const btcReserveAccount = deployerBtc; // configured below as the seize destination

  // 6. Settlement recipient owners + ATAs.
  console.log("[6/9] creating settlement recipient ATAs (growth / lp / integrators)");
  const growthOwner = Keypair.generate();
  const lpOwner = Keypair.generate();
  const integratorsOwner = Keypair.generate();
  const growthAgc = (
    await getOrCreateAssociatedTokenAccount(connection, admin, agcMint, growthOwner.publicKey)
  ).address;
  const lpAgc = (
    await getOrCreateAssociatedTokenAccount(connection, admin, agcMint, lpOwner.publicKey)
  ).address;
  const integratorsAgc = (
    await getOrCreateAssociatedTokenAccount(connection, admin, agcMint, integratorsOwner.publicKey)
  ).address;

  // 7. initialize_protocol — creates the state PDA + treasury_agc/treasury_usdc/xagc_vault_agc.
  console.log("[7/9] calling initialize_protocol");
  await program.methods
    .initializeProtocol({
      initialAnchorPriceX18: PRICE_SCALE,
      policyParams: policyParams(),
      mintDistribution: {
        xagcBps: 3000,
        growthProgramsBps: 2000,
        lpBps: 2000,
        integratorsBps: 1000,
        treasuryBps: 2000,
      },
      settlementRecipients: {
        growthProgramsAgc: growthAgc,
        lpAgc,
        integratorsAgc,
      },
      exitFeeBps: 100,
      growthProgramsEnabled: true,
    })
    .accountsStrict({
      payer: admin.publicKey,
      admin: admin.publicKey,
      state,
      agcMint,
      xagcMint,
      usdcMint,
      treasuryAgc,
      treasuryUsdc,
      xagcVaultAgc,
      mintAuthority,
      treasuryAuthority,
      xagcAuthority,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      rent: SYSVAR_RENT_PUBKEY,
    })
    .signers([admin])
    .rpc();

  // 8. BTC collateral asset + manual oracle.
  console.log("[8/9] configuring BTC collateral asset + oracle price");
  const collateralAsset = pdaWith("collateral-asset", btcMint, PROGRAM_ID);
  const collateralOracle = pdaWith("collateral-oracle", btcMint, PROGRAM_ID);
  await program.methods
    .setCollateralAsset({
      oracleSource: { manual: {} } as never,
      oracleFeed: admin.publicKey,
      pythPriceFeedId: Array(32).fill(0),
      reserveTokenAccount: btcReserveAccount,
      assetClass: { btc: {} } as never,
      reserveWeightBps: 6000,
      collateralFactorBps: 5000,
      liquidationThresholdBps: 6500,
      maxConcentrationBps: 4000,
      maxOracleStalenessSeconds: new BN(86400), // 1d for demo
      maxOracleConfidenceBps: 100,
      enabled: true,
    })
    .accountsStrict({
      state,
      authority: admin.publicKey,
      mint: btcMint,
      collateralAsset,
      systemProgram: SystemProgram.programId,
    })
    .signers([admin])
    .rpc();
  await program.methods
    .setCollateralOraclePrice({
      priceQuoteX18: new BN("100000000000000000000000"), // $100k/BTC * 1e18
      confidenceBps: 25,
    })
    .accountsStrict({
      state,
      authority: admin.publicKey,
      keeper: admin.publicKey,
      mint: btcMint,
      collateralAsset,
      collateralOracle,
      systemProgram: SystemProgram.programId,
    })
    .signers([admin])
    .rpc();

  // 9. Open the BTC credit facility.
  console.log("[9/9] initializing BTC credit facility");
  const facilityId = new BN(1);
  const facilityIdBuf = facilityId.toArrayLike(Buffer, "le", 8);
  const [facility] = PublicKey.findProgramAddressSync(
    [Buffer.from("credit-facility"), facilityIdBuf],
    PROGRAM_ID,
  );
  const facilityAuthority = pdaWith("credit-facility-authority", facility, PROGRAM_ID);
  const collateralVault = pdaWith("credit-collateral-vault", facility, PROGRAM_ID);
  const underwriterVaultAgc = pdaWith("underwriter-vault", facility, PROGRAM_ID);
  await program.methods
    .initializeCreditFacility(facilityId, {
      maxTotalDebtAgc: new BN("1000000000000000"), // 1M AGC
      maxLineDebtAgc: new BN("100000000000000"), // 100k AGC per line
      minCollateralHealthBps: 20000,
      liquidationHealthBps: 14000,
      minUnderwriterReserveBps: 1000,
      interestRateBps: 1200,
      originationFeeBps: 50,
      defaultGraceSeconds: new BN(86400),
      isolated: false,
      enabled: true,
    })
    .accountsStrict({
      state,
      authority: admin.publicKey,
      collateralMint: btcMint,
      collateralAsset,
      facility,
      facilityAuthority,
      agcMint,
      collateralVault,
      underwriterVaultAgc,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      rent: SYSVAR_RENT_PUBKEY,
    })
    .signers([admin])
    .rpc();

  const balanceAfter = await connection.getBalance(admin.publicKey);
  const spent = (balanceBefore - balanceAfter) / 1e9;
  console.log(`Bootstrap complete. Spent ${spent.toFixed(4)} SOL. Balance: ${(balanceAfter / 1e9).toFixed(4)} SOL`);

  const out = {
    cluster: "devnet",
    programId: PROGRAM_ID.toBase58(),
    state: state.toBase58(),
    mintAuthority: mintAuthority.toBase58(),
    treasuryAuthority: treasuryAuthority.toBase58(),
    xagcAuthority: xagcAuthority.toBase58(),
    treasuryAgc: treasuryAgc.toBase58(),
    treasuryUsdc: treasuryUsdc.toBase58(),
    xagcVaultAgc: xagcVaultAgc.toBase58(),
    agcMint: agcMint.toBase58(),
    xagcMint: xagcMint.toBase58(),
    usdcMint: usdcMint.toBase58(),
    btcMint: btcMint.toBase58(),
    btcReserveAccount: btcReserveAccount.toBase58(),
    growthAgc: growthAgc.toBase58(),
    lpAgc: lpAgc.toBase58(),
    integratorsAgc: integratorsAgc.toBase58(),
    collateralAsset: collateralAsset.toBase58(),
    collateralOracle: collateralOracle.toBase58(),
    facility: facility.toBase58(),
    facilityAuthority: facilityAuthority.toBase58(),
    collateralVault: collateralVault.toBase58(),
    underwriterVaultAgc: underwriterVaultAgc.toBase58(),
    deployerAgc: deployerAgc.toBase58(),
    deployerBtc: deployerBtc.toBase58(),
    initialAnchorPriceX18: PRICE_SCALE.toString(),
    btcPriceQuoteX18: "100000000000000000000000",
  };
  fs.mkdirSync(path.dirname(OUTPUT_PATH), { recursive: true });
  fs.writeFileSync(OUTPUT_PATH, JSON.stringify(out, null, 2));
  console.log(`Wrote ${OUTPUT_PATH}`);
  console.log(JSON.stringify(out, null, 2));
}

main().catch((err) => {
  console.error("Bootstrap failed:", err);
  process.exit(1);
});
