// Pre-recording smoke test: runs the five clickable-button actions from the
// dashboard against the live devnet deployment, in the same order and amounts
// the demo script narrates. Prints each tx signature, a one-line state delta,
// and an Explorer link.
//
// Run from repo root:
//   pnpm exec tsx script/demo/full-run.ts
//
// Notes:
// - Uses the deployer keypair (~/.config/solana/id.json) as the wallet. The
//   dashboard during recording is connected with the same keypair imported
//   into Phantom.
// - Skips `open_credit_line` — line 1 was opened by `borrow.ts` during
//   bootstrap and reuses across runs.
// - Skips `settle_epoch` — run `pnpm exec tsx script/demo/expansion-cycle.ts`
//   separately if you also want to rehearse the expansion event.

import { PublicKey, SystemProgram } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, getAssociatedTokenAddressSync } from "@solana/spl-token";

import { BN, balanceOf, fmtAgc, makeContext, pdaWith, pdaWithU64 } from "./_lib.ts";

const LINE_ID = new BN(1);

async function main() {
  const ctx = await makeContext();
  const { admin, addresses, programId, connection, program, pk } = ctx;

  const explorer = (sig: string) =>
    `https://explorer.solana.com/tx/${sig}?cluster=${addresses.cluster}`;

  const facility = pk("facility");
  const collateralAsset = pk("collateralAsset");
  const collateralOracle = pk("collateralOracle");
  const collateralVault = pdaWith("credit-collateral-vault", facility, programId);
  const facilityAuthority = pdaWith("credit-facility-authority", facility, programId);
  const mintAuthority = PublicKey.findProgramAddressSync(
    [Buffer.from("mint-authority")],
    programId,
  )[0];
  const xagcAuthority = PublicKey.findProgramAddressSync(
    [Buffer.from("xagc-authority")],
    programId,
  )[0];
  const underwriterPosition = pdaWith("underwriter-position", facility, programId);
  // The PDA above is wrong — underwriter-position seeds are [SEED, facility, underwriter].
  const underwriterPositionCorrect = PublicKey.findProgramAddressSync(
    [Buffer.from("underwriter-position"), facility.toBuffer(), admin.publicKey.toBuffer()],
    programId,
  )[0];
  const creditLine = pdaWithU64(
    "credit-line",
    facility,
    admin.publicKey,
    LINE_ID,
    programId,
  );

  const userXagcAta = getAssociatedTokenAddressSync(
    pk("xagcMint"),
    admin.publicKey,
  );

  const dashLine = "─".repeat(64);

  console.log(dashLine);
  console.log("AGC dashboard full-run smoke test");
  console.log("Wallet:", admin.publicKey.toBase58());
  console.log("Cluster:", addresses.cluster);
  console.log(dashLine);

  // ──────────────────────────────────────────────────────────────────────
  // 1. depositXagc(100 AGC)
  // ──────────────────────────────────────────────────────────────────────
  {
    console.log("\n[1/5] depositXagc — 100 AGC");
    const before = await balanceOf(connection, pk("xagcVaultAgc"));
    const sig = await program.methods
      .depositXagc(new BN("100000000000")) // 100 * 1e9
      .accountsStrict({
        state: pk("state"),
        depositor: admin.publicKey,
        depositorAgc: pk("deployerAgc"),
        xagcVaultAgc: pk("xagcVaultAgc"),
        xagcMint: pk("xagcMint"),
        receiverXagc: userXagcAta,
        mintAuthority,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([admin])
      .rpc();
    const after = await balanceOf(connection, pk("xagcVaultAgc"));
    console.log(`  xAGC vault: ${fmtAgc(before)} → ${fmtAgc(after)} AGC`);
    console.log(`  ${explorer(sig)}`);
  }

  // ──────────────────────────────────────────────────────────────────────
  // 2. depositUnderwriterAgc(1000 AGC)
  // ──────────────────────────────────────────────────────────────────────
  {
    console.log("\n[2/5] depositUnderwriterAgc — 1,000 AGC");
    const before = await balanceOf(connection, pk("underwriterVaultAgc"));
    const sig = await program.methods
      .depositUnderwriterAgc(new BN("1000000000000")) // 1k AGC
      .accountsStrict({
        state: pk("state"),
        facility,
        underwriter: admin.publicKey,
        underwriterAgc: pk("deployerAgc"),
        underwriterVaultAgc: pk("underwriterVaultAgc"),
        underwriterPosition: underwriterPositionCorrect,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([admin])
      .rpc();
    const after = await balanceOf(connection, pk("underwriterVaultAgc"));
    console.log(`  Underwriter vault: ${fmtAgc(before)} → ${fmtAgc(after)} AGC`);
    console.log(`  ${explorer(sig)}`);
  }

  // ──────────────────────────────────────────────────────────────────────
  // 3. depositCreditCollateral(0.1 BTC) — into pre-opened line_id=1
  // ──────────────────────────────────────────────────────────────────────
  {
    console.log("\n[3/5] depositCreditCollateral — 0.1 BTC");
    const before = await balanceOf(connection, collateralVault);
    const sig = await program.methods
      .depositCreditCollateral(new BN("10000000")) // 0.1 BTC (8 decimals)
      .accountsStrict({
        state: pk("state"),
        facility,
        creditLine,
        borrower: admin.publicKey,
        borrowerCollateral: pk("deployerBtc"),
        collateralVault,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([admin])
      .rpc();
    const after = await balanceOf(connection, collateralVault);
    console.log(`  Collateral vault: ${before} → ${after} satoshi-BTC`);
    console.log(`  ${explorer(sig)}`);
  }

  // ──────────────────────────────────────────────────────────────────────
  // 4. drawCreditLine(200 AGC)
  // ──────────────────────────────────────────────────────────────────────
  {
    console.log("\n[4/5] drawCreditLine — 200 AGC");
    const before = await balanceOf(connection, pk("deployerAgc"));
    const sig = await program.methods
      .drawCreditLine(new BN("200000000000")) // 200 AGC
      .accountsStrict({
        state: pk("state"),
        facility,
        collateralAsset,
        collateralOracle,
        creditLine,
        borrower: admin.publicKey,
        agcMint: pk("agcMint"),
        borrowerAgcDestination: pk("deployerAgc"),
        treasuryAgc: pk("treasuryAgc"),
        mintAuthority,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([admin])
      .rpc();
    const after = await balanceOf(connection, pk("deployerAgc"));
    console.log(`  Deployer AGC: ${fmtAgc(before)} → ${fmtAgc(after)} AGC`);
    console.log(`  ${explorer(sig)}`);
  }

  // ──────────────────────────────────────────────────────────────────────
  // 5. repayCreditLine(100 AGC)
  // ──────────────────────────────────────────────────────────────────────
  {
    console.log("\n[5/5] repayCreditLine — 100 AGC");
    const lineBefore = await program.account.creditLine.fetch(creditLine);
    const sig = await program.methods
      .repayCreditLine(new BN("100000000000")) // 100 AGC
      .accountsStrict({
        state: pk("state"),
        facility,
        creditLine,
        payer: admin.publicKey,
        payerAgc: pk("deployerAgc"),
        underwriterVaultAgc: pk("underwriterVaultAgc"),
        agcMint: pk("agcMint"),
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([admin])
      .rpc();
    const lineAfter = await program.account.creditLine.fetch(creditLine);
    const principalBefore = BigInt(lineBefore.principalDebtAgc.toString());
    const principalAfter = BigInt(lineAfter.principalDebtAgc.toString());
    console.log(
      `  Principal debt: ${fmtAgc(principalBefore)} → ${fmtAgc(principalAfter)} AGC`,
    );
    console.log(`  ${explorer(sig)}`);
  }

  console.log("\n" + dashLine);
  console.log("Smoke test complete. Run `expansion-cycle.ts` for the settle event.");
  console.log(dashLine);
}

main().catch((err) => {
  console.error("\nfull-run failed:", err);
  process.exit(1);
});
