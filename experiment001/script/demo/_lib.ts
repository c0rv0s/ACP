// Shared setup for the demo scripts. Loads the deployer keypair, the devnet
// addresses written by `devnet-bootstrap.ts`, and constructs an Anchor program
// instance.

import anchorPkg from "@coral-xyz/anchor";
const { AnchorProvider, Program, Wallet, setProvider } = anchorPkg;
type Idl = anchorPkg.Idl;
type ProgramT<T> = anchorPkg.Program<T>;
type BNT = InstanceType<typeof anchorPkg.BN>;
export const BN = anchorPkg.BN;

import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";
import { fileURLToPath } from "node:url";

import idl from "../../solana/target/idl/agc_solana.json" with { type: "json" };
import type { AgcSolana } from "../../solana/target/types/agc_solana";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const REPO_ROOT = path.resolve(__dirname, "..", "..");

export interface DeploymentAddresses {
  cluster: string;
  programId: string;
  state: string;
  mintAuthority: string;
  treasuryAuthority: string;
  xagcAuthority: string;
  treasuryAgc: string;
  treasuryUsdc: string;
  xagcVaultAgc: string;
  agcMint: string;
  xagcMint: string;
  usdcMint: string;
  btcMint: string;
  btcReserveAccount: string;
  growthAgc: string;
  lpAgc: string;
  integratorsAgc: string;
  collateralAsset: string;
  collateralOracle: string;
  facility: string;
  facilityAuthority: string;
  collateralVault: string;
  underwriterVaultAgc: string;
  deployerAgc: string;
  deployerBtc: string;
}

export function loadAddresses(): DeploymentAddresses {
  const filepath = path.join(REPO_ROOT, "deployments", "devnet.json");
  if (!fs.existsSync(filepath)) {
    throw new Error(
      `Missing ${filepath}. Run \`pnpm exec tsx script/devnet-bootstrap.ts\` first.`,
    );
  }
  return JSON.parse(fs.readFileSync(filepath, "utf8")) as DeploymentAddresses;
}

export function loadKeypair(filepath = "~/.config/solana/id.json"): Keypair {
  const expanded = filepath.startsWith("~")
    ? path.join(os.homedir(), filepath.slice(1))
    : filepath;
  const secret = JSON.parse(fs.readFileSync(expanded, "utf8")) as number[];
  return Keypair.fromSecretKey(new Uint8Array(secret));
}

export interface DemoContext {
  admin: Keypair;
  addresses: DeploymentAddresses;
  programId: PublicKey;
  connection: Connection;
  provider: anchorPkg.AnchorProvider;
  program: ProgramT<AgcSolana>;
  pk: (key: keyof DeploymentAddresses) => PublicKey;
}

export async function makeContext(): Promise<DemoContext> {
  const admin = loadKeypair();
  const addresses = loadAddresses();
  const programId = new PublicKey(addresses.programId);
  const connection = new Connection(
    process.env.SOLANA_RPC_URL ?? "https://api.devnet.solana.com",
    "confirmed",
  );
  const provider = new AnchorProvider(connection, new Wallet(admin), {
    commitment: "confirmed",
  });
  setProvider(provider);
  const program = new Program(idl as Idl, provider) as unknown as ProgramT<AgcSolana>;
  const pk = (key: keyof DeploymentAddresses) => new PublicKey(addresses[key]);
  return { admin, addresses, programId, connection, provider, program, pk };
}

export function pdaWith(
  seed: string,
  key: PublicKey,
  programId: PublicKey,
): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(seed), key.toBuffer()],
    programId,
  )[0];
}

export function pdaWithU64(
  seed: string,
  facility: PublicKey,
  borrower: PublicKey,
  id: BNT,
  programId: PublicKey,
): PublicKey {
  return PublicKey.findProgramAddressSync(
    [
      Buffer.from(seed),
      facility.toBuffer(),
      borrower.toBuffer(),
      id.toArrayLike(Buffer, "le", 8),
    ],
    programId,
  )[0];
}

export async function balanceOf(connection: Connection, pubkey: PublicKey): Promise<bigint> {
  const info = await connection.getAccountInfo(pubkey);
  if (!info) return 0n;
  // SPL token account `amount` is at offset 64, u64 little-endian.
  return info.data.readBigUInt64LE(64);
}

export function fmtAgc(amount: bigint): string {
  const whole = Number(amount) / 1e9;
  return whole.toLocaleString(undefined, { maximumFractionDigits: 2 });
}
