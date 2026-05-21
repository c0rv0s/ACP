// Demo: open a credit line, post BTC collateral, draw AGC.
// Run: pnpm exec tsx script/demo/borrow.ts [collateral-btc] [draw-agc] [line-id]
//
// Uses the deployer as both the credit operator (admin) and the borrower.
// Default: 0.5 BTC collateral, 5,000 AGC draw, line_id=1.

import { PublicKey, SystemProgram } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";

import { BN, balanceOf, fmtAgc, makeContext, pdaWithU64 } from "./_lib.ts";

async function main() {
  const collateralBtc = parseFloat(process.argv[2] ?? "0.5");
  const drawAgc = parseFloat(process.argv[3] ?? "5000");
  const lineIdArg = parseInt(process.argv[4] ?? "1", 10);

  const collateralUnits = Math.floor(collateralBtc * 1e8); // BTC has 8 decimals
  const drawUnits = new BN(Math.floor(drawAgc * 1e9));
  const lineId = new BN(lineIdArg);

  const { admin, addresses, programId, connection, program, pk } = await makeContext();
  const facility = pk("facility");
  const creditLine = pdaWithU64("credit-line", facility, admin.publicKey, lineId, programId);

  console.log(`Opening line_id=${lineIdArg} for borrower ${admin.publicKey.toBase58()}`);
  console.log(`  limit: 100,000 AGC, maturity: 30 days`);
  const maturity = new BN(Math.floor(Date.now() / 1000) + 30 * 86400);
  await program.methods
    .openCreditLine(lineId, {
      creditLimitAgc: new BN(100_000 * 1e9),
      maturityTimestamp: maturity,
    })
    .accountsStrict({
      state: pk("state"),
      authority: admin.publicKey,
      facility,
      borrower: admin.publicKey,
      creditLine,
      systemProgram: SystemProgram.programId,
    })
    .signers([admin])
    .rpc();

  console.log(`Depositing ${collateralBtc} BTC collateral...`);
  await program.methods
    .depositCreditCollateral(new BN(collateralUnits))
    .accountsStrict({
      state: pk("state"),
      facility,
      creditLine,
      borrower: admin.publicKey,
      borrowerCollateral: pk("deployerBtc"),
      collateralVault: pk("collateralVault"),
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .signers([admin])
    .rpc();

  console.log(`Drawing ${drawAgc.toLocaleString()} AGC...`);
  const before = await balanceOf(connection, pk("deployerAgc"));
  const sig = await program.methods
    .drawCreditLine(drawUnits)
    .accountsStrict({
      state: pk("state"),
      facility,
      collateralAsset: pk("collateralAsset"),
      collateralOracle: pk("collateralOracle"),
      creditLine,
      borrower: admin.publicKey,
      agcMint: pk("agcMint"),
      borrowerAgcDestination: pk("deployerAgc"),
      treasuryAgc: pk("treasuryAgc"),
      mintAuthority: pk("mintAuthority"),
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .signers([admin])
    .rpc();

  const after = await balanceOf(connection, pk("deployerAgc"));
  const lineState = await program.account.creditLine.fetch(creditLine);
  console.log(`Borrower AGC delta: +${fmtAgc(after - before)} (origination fee retained)`);
  console.log(`Line principal debt: ${fmtAgc(BigInt(lineState.principalDebtAgc.toString()))} AGC`);
  console.log(`tx: https://explorer.solana.com/tx/${sig}?cluster=${addresses.cluster}`);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
