// Demo: deposit AGC into the BTC facility as an underwriter.
// Run: pnpm exec tsx script/demo/underwrite.ts [amount-agc]
//
// Default deposits 50,000 AGC. The dashboard's "Underwriter vault" tile should
// jump from 0 → the deposited amount within a refresh tick.

import { PublicKey, SystemProgram } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";

import { BN, balanceOf, fmtAgc, makeContext, pdaWith } from "./_lib.ts";

async function main() {
  const amountArg = process.argv[2];
  const amountAgc = amountArg ? parseFloat(amountArg) : 50_000;
  const amount = new BN(Math.floor(amountAgc * 1e9));

  const { admin, addresses, programId, connection, program, pk } = await makeContext();
  const facility = pk("facility");
  const underwriterPosition = PublicKey.findProgramAddressSync(
    [Buffer.from("underwriter-position"), facility.toBuffer(), admin.publicKey.toBuffer()],
    programId,
  )[0];

  const before = await balanceOf(connection, pk("underwriterVaultAgc"));
  console.log(`Underwriter vault before: ${fmtAgc(before)} AGC`);
  console.log(`Depositing: ${amountAgc.toLocaleString()} AGC...`);

  const sig = await program.methods
    .depositUnderwriterAgc(amount)
    .accountsStrict({
      state: pk("state"),
      facility,
      underwriter: admin.publicKey,
      underwriterAgc: pk("deployerAgc"),
      underwriterVaultAgc: pk("underwriterVaultAgc"),
      underwriterPosition,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .signers([admin])
    .rpc();

  const after = await balanceOf(connection, pk("underwriterVaultAgc"));
  console.log(`Underwriter vault after:  ${fmtAgc(after)} AGC`);
  console.log(`tx: https://explorer.solana.com/tx/${sig}?cluster=${addresses.cluster}`);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
