// Build + submit transactions against the deployed AGC program from the
// dashboard. The connected wallet (window.solana) signs and submits each tx;
// instructions are built via Anchor's `Program.methods.X(...).instruction()`.
//
// Each handler returns the confirmed signature or throws an Error whose
// `message` is suitable for display in the dashboard's tx note.

import { AnchorProvider, BN, Program } from "@coral-xyz/anchor";
import type { Idl, Program as ProgramT, Wallet as WalletShape } from "@coral-xyz/anchor";
import {
  TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountInstruction,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
  TransactionInstruction,
} from "@solana/web3.js";

import idl from "./agc/agc_solana.json";
import type { AgcSolana } from "./agc/agc_solana";
import { solanaAddresses, solanaRpcUrl } from "./contracts";

// Phantom-style provider shape that App.tsx already uses.
interface PhantomLike {
  publicKey?: { toString: () => string; toBuffer?: () => Buffer };
  signAndSendTransaction?: (tx: Transaction) => Promise<{ signature: string }>;
  signTransaction?: (tx: Transaction) => Promise<Transaction>;
}

declare global {
  interface Window {
    // Re-declared more permissively than the App.tsx version; both shapes are
    // compatible with Phantom's actual injected object.
    solana?: PhantomLike & Window["solana"];
  }
}

const AGC_DECIMALS = 9;
const BTC_DECIMALS = 8;
const PROGRAM_ID = new PublicKey(solanaAddresses.programId);
const DEFAULT_LINE_ID = 1n;

function pk(key: keyof typeof solanaAddresses): PublicKey {
  return new PublicKey(solanaAddresses[key]);
}

function pda(seed: string): PublicKey {
  return PublicKey.findProgramAddressSync([Buffer.from(seed)], PROGRAM_ID)[0];
}

function toUnits(human: string, decimals: number): bigint {
  const trimmed = human.trim();
  if (!trimmed) throw new Error("Amount is empty.");
  if (!/^\d+(\.\d+)?$/.test(trimmed)) {
    throw new Error(`Invalid amount: ${human}`);
  }
  const [whole, frac = ""] = trimmed.split(".");
  const padded = (frac + "0".repeat(decimals)).slice(0, decimals);
  const value = BigInt(whole) * 10n ** BigInt(decimals) + BigInt(padded || "0");
  if (value <= 0n) throw new Error("Amount must be positive.");
  return value;
}

function getConnection(): Connection {
  return new Connection(solanaRpcUrl, "confirmed");
}

function getProgram(connection: Connection, walletPubkey: PublicKey): ProgramT<AgcSolana> {
  // Anchor's Provider needs a wallet shape with publicKey + sign methods; we
  // only use it for instruction-building, so the sign methods are no-ops.
  const stubWallet = {
    publicKey: walletPubkey,
    signTransaction: async <T extends Transaction>(t: T) => t,
    signAllTransactions: async <T extends Transaction>(t: T[]) => t,
    payer: Keypair.generate(),
  } as unknown as WalletShape;
  const provider = new AnchorProvider(connection, stubWallet, {
    commitment: "confirmed",
  });
  return new Program(idl as Idl, provider) as unknown as ProgramT<AgcSolana>;
}

async function ensureAta(
  connection: Connection,
  owner: PublicKey,
  mint: PublicKey,
  payer: PublicKey,
): Promise<{ address: PublicKey; createIx: TransactionInstruction | null }> {
  const address = getAssociatedTokenAddressSync(mint, owner);
  const info = await connection.getAccountInfo(address);
  if (info) return { address, createIx: null };
  return {
    address,
    createIx: createAssociatedTokenAccountInstruction(payer, address, owner, mint),
  };
}

async function buildAndSend(
  connection: Connection,
  walletPubkey: PublicKey,
  instructions: TransactionInstruction[],
): Promise<string> {
  const provider = window.solana;
  if (!provider) throw new Error("No Solana wallet connected.");
  if (!provider.signAndSendTransaction && !provider.signTransaction) {
    throw new Error("Connected wallet cannot sign transactions.");
  }

  const tx = new Transaction();
  instructions.forEach((ix) => tx.add(ix));
  tx.feePayer = walletPubkey;
  const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash();
  tx.recentBlockhash = blockhash;

  let signature: string;
  if (provider.signAndSendTransaction) {
    const result = await provider.signAndSendTransaction(tx);
    signature = result.signature;
  } else {
    const signed = await provider.signTransaction!(tx);
    signature = await connection.sendRawTransaction(signed.serialize());
  }

  await connection.confirmTransaction(
    { signature, blockhash, lastValidBlockHeight },
    "confirmed",
  );
  return signature;
}

// ---- Public handlers ----

export async function depositXagc(walletPubkey: PublicKey, agcAmount: string): Promise<string> {
  const amount = toUnits(agcAmount, AGC_DECIMALS);
  const connection = getConnection();
  const program = getProgram(connection, walletPubkey);
  const userAgc = await ensureAta(connection, walletPubkey, pk("agcMint"), walletPubkey);
  const userXagc = await ensureAta(connection, walletPubkey, pk("xagcMint"), walletPubkey);

  const ix = await program.methods
    .depositXagc(new BN(amount.toString()))
    .accountsStrict({
      state: pk("state"),
      depositor: walletPubkey,
      depositorAgc: userAgc.address,
      xagcVaultAgc: pk("xagcVaultAgc"),
      xagcMint: pk("xagcMint"),
      receiverXagc: userXagc.address,
      mintAuthority: pda("mint-authority"),
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .instruction();

  return buildAndSend(
    connection,
    walletPubkey,
    [userAgc.createIx, userXagc.createIx, ix].filter((x): x is TransactionInstruction =>
      Boolean(x),
    ),
  );
}

export async function redeemXagc(walletPubkey: PublicKey, xagcShares: string): Promise<string> {
  const shares = toUnits(xagcShares, AGC_DECIMALS);
  const connection = getConnection();
  const program = getProgram(connection, walletPubkey);
  const userXagc = await ensureAta(connection, walletPubkey, pk("xagcMint"), walletPubkey);
  const userAgc = await ensureAta(connection, walletPubkey, pk("agcMint"), walletPubkey);

  const ix = await program.methods
    .redeemXagc(new BN(shares.toString()))
    .accountsStrict({
      state: pk("state"),
      owner: walletPubkey,
      ownerXagc: userXagc.address,
      xagcMint: pk("xagcMint"),
      xagcVaultAgc: pk("xagcVaultAgc"),
      treasuryAgc: pk("treasuryAgc"),
      receiverAgc: userAgc.address,
      xagcAuthority: pda("xagc-authority"),
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .instruction();

  return buildAndSend(
    connection,
    walletPubkey,
    [userXagc.createIx, userAgc.createIx, ix].filter(
      (x): x is TransactionInstruction => Boolean(x),
    ),
  );
}

export async function underwriteAgc(walletPubkey: PublicKey, agcAmount: string): Promise<string> {
  const amount = toUnits(agcAmount, AGC_DECIMALS);
  const connection = getConnection();
  const program = getProgram(connection, walletPubkey);
  const userAgc = await ensureAta(connection, walletPubkey, pk("agcMint"), walletPubkey);
  const facility = pk("facility");
  const underwriterPosition = PublicKey.findProgramAddressSync(
    [Buffer.from("underwriter-position"), facility.toBuffer(), walletPubkey.toBuffer()],
    PROGRAM_ID,
  )[0];

  const ix = await program.methods
    .depositUnderwriterAgc(new BN(amount.toString()))
    .accountsStrict({
      state: pk("state"),
      facility,
      underwriter: walletPubkey,
      underwriterAgc: userAgc.address,
      underwriterVaultAgc: pk("underwriterVaultAgc"),
      underwriterPosition,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .instruction();

  return buildAndSend(
    connection,
    walletPubkey,
    [userAgc.createIx, ix].filter((x): x is TransactionInstruction => Boolean(x)),
  );
}

function lineFor(walletPubkey: PublicKey, lineId: bigint = DEFAULT_LINE_ID): PublicKey {
  const facility = pk("facility");
  const lineIdBuf = Buffer.alloc(8);
  lineIdBuf.writeBigUInt64LE(lineId);
  return PublicKey.findProgramAddressSync(
    [Buffer.from("credit-line"), facility.toBuffer(), walletPubkey.toBuffer(), lineIdBuf],
    PROGRAM_ID,
  )[0];
}

export async function depositCollateral(
  walletPubkey: PublicKey,
  btcAmount: string,
): Promise<string> {
  const amount = toUnits(btcAmount, BTC_DECIMALS);
  const connection = getConnection();
  const program = getProgram(connection, walletPubkey);
  const facility = pk("facility");
  const userBtc = await ensureAta(connection, walletPubkey, pk("btcMint"), walletPubkey);
  const creditLine = lineFor(walletPubkey);
  const collateralVault = PublicKey.findProgramAddressSync(
    [Buffer.from("credit-collateral-vault"), facility.toBuffer()],
    PROGRAM_ID,
  )[0];

  const ix = await program.methods
    .depositCreditCollateral(new BN(amount.toString()))
    .accountsStrict({
      state: pk("state"),
      facility,
      creditLine,
      borrower: walletPubkey,
      borrowerCollateral: userBtc.address,
      collateralVault,
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .instruction();

  return buildAndSend(connection, walletPubkey, [ix]);
}

export async function drawCredit(walletPubkey: PublicKey, agcAmount: string): Promise<string> {
  const amount = toUnits(agcAmount, AGC_DECIMALS);
  const connection = getConnection();
  const program = getProgram(connection, walletPubkey);
  const userAgc = await ensureAta(connection, walletPubkey, pk("agcMint"), walletPubkey);
  const creditLine = lineFor(walletPubkey);

  const ix = await program.methods
    .drawCreditLine(new BN(amount.toString()))
    .accountsStrict({
      state: pk("state"),
      facility: pk("facility"),
      collateralAsset: pk("collateralAsset"),
      collateralOracle: pk("collateralOracle"),
      creditLine,
      borrower: walletPubkey,
      agcMint: pk("agcMint"),
      borrowerAgcDestination: userAgc.address,
      treasuryAgc: pk("treasuryAgc"),
      mintAuthority: pda("mint-authority"),
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .instruction();

  return buildAndSend(
    connection,
    walletPubkey,
    [userAgc.createIx, ix].filter((x): x is TransactionInstruction => Boolean(x)),
  );
}

export async function repayCredit(walletPubkey: PublicKey, agcAmount: string): Promise<string> {
  const amount = toUnits(agcAmount, AGC_DECIMALS);
  const connection = getConnection();
  const program = getProgram(connection, walletPubkey);
  const userAgc = await ensureAta(connection, walletPubkey, pk("agcMint"), walletPubkey);
  const creditLine = lineFor(walletPubkey);

  const ix = await program.methods
    .repayCreditLine(new BN(amount.toString()))
    .accountsStrict({
      state: pk("state"),
      facility: pk("facility"),
      creditLine,
      payer: walletPubkey,
      payerAgc: userAgc.address,
      underwriterVaultAgc: pk("underwriterVaultAgc"),
      agcMint: pk("agcMint"),
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .instruction();

  return buildAndSend(connection, walletPubkey, [ix]);
}

export function explorerTxUrl(signature: string, cluster: string): string {
  const params = cluster === "mainnet-beta" ? "" : `?cluster=${cluster}`;
  return `https://explorer.solana.com/tx/${signature}${params}`;
}
