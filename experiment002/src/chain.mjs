import { randomBytes } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";

export const ROOT_DIR = path.resolve(fileURLToPath(new URL("..", import.meta.url)));
export const SOLANA_DIR = path.join(ROOT_DIR, "solana");
export const LOCAL_DIR = path.join(ROOT_DIR, ".local");
export const KEY_DIR = path.join(LOCAL_DIR, "keys");
export const DEPLOYMENT_PATH = path.join(LOCAL_DIR, "deployment.json");
export const ACTION_LOG_PATH = path.join(LOCAL_DIR, "actions.json");
export const IDL_PATH = path.join(SOLANA_DIR, "target", "idl", "agc_credit_control.json");
export const PROGRAM_SO_PATH = path.join(SOLANA_DIR, "target", "deploy", "agc_credit_control.so");
export const PROGRAM_KEYPAIR_PATH = path.join(
  SOLANA_DIR,
  "target",
  "deploy",
  "agc_credit_control-keypair.json",
);

const requireFromSolana = createRequire(path.join(SOLANA_DIR, "package.json"));

export const anchor = requireFromSolana("@coral-xyz/anchor");
export const web3 = requireFromSolana("@solana/web3.js");
export const splToken = requireFromSolana("@solana/spl-token");

export const { Keypair, PublicKey, SystemProgram, LAMPORTS_PER_SOL } = web3;
export const TOKEN_PROGRAM_ID = splToken.TOKEN_PROGRAM_ID;
export const USDC_DECIMALS = 6;
export const USDC_BASE_UNITS = 10n ** BigInt(USDC_DECIMALS);
export const DEFAULT_RPC_URL = "http://127.0.0.1:8899";

export function ensureDir(dir) {
  fs.mkdirSync(dir, { recursive: true });
}

export function readJson(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

export function writeJson(file, value) {
  ensureDir(path.dirname(file));
  fs.writeFileSync(file, `${JSON.stringify(value, null, 2)}\n`);
}

export function loadIdl() {
  if (!fs.existsSync(IDL_PATH)) {
    throw new Error(`Missing Anchor IDL at ${IDL_PATH}. Run npm run build:solana first.`);
  }
  return readJson(IDL_PATH);
}

export function programIdFromIdl() {
  return new PublicKey(loadIdl().address);
}

export function parseUsdc(value) {
  const raw = String(value ?? "").trim();
  if (!/^\d+(\.\d{1,6})?$/.test(raw)) {
    throw new Error(`Invalid USDC amount: ${value}`);
  }

  const [whole, fractional = ""] = raw.split(".");
  return BigInt(whole) * USDC_BASE_UNITS + BigInt(fractional.padEnd(USDC_DECIMALS, "0"));
}

export function bnUsdc(value) {
  return new anchor.BN(parseUsdc(value).toString());
}

export function bn(value) {
  return new anchor.BN(String(value));
}

export function rawToBigInt(value) {
  if (typeof value === "bigint") return value;
  if (typeof value === "number") return BigInt(value);
  if (typeof value === "string") return BigInt(value);
  if (value && typeof value.toString === "function") return BigInt(value.toString());
  return 0n;
}

export function formatUsdc(value) {
  const raw = rawToBigInt(value);
  const sign = raw < 0n ? "-" : "";
  const absolute = raw < 0n ? -raw : raw;
  const whole = absolute / USDC_BASE_UNITS;
  const fractional = String(absolute % USDC_BASE_UNITS).padStart(USDC_DECIMALS, "0").replace(/0+$/, "");
  return `${sign}${whole.toLocaleString("en-US")}.${(fractional || "00").padEnd(2, "0")}`;
}

export function bytes(length, seed) {
  return Array.from({ length }, (_, index) => (seed + index) % 256);
}

export function randomByteArray(length) {
  return Array.from(randomBytes(length));
}

export function hashBytes(seed) {
  return bytes(32, seed);
}

export function seed16(seed) {
  return bytes(16, seed);
}

export function toBuffer(value) {
  return Buffer.from(value);
}

export function findPda(seeds, programId = programIdFromIdl()) {
  return PublicKey.findProgramAddressSync(seeds, new PublicKey(programId))[0];
}

export function pubkey(value) {
  return value instanceof PublicKey ? value : new PublicKey(value);
}

export function pubkeyString(value) {
  if (!value) return "";
  if (value instanceof PublicKey) return value.toBase58();
  if (typeof value.toBase58 === "function") return value.toBase58();
  return String(value);
}

export function enumName(value) {
  if (!value) return "";
  if (typeof value === "string") return value;
  const [key] = Object.keys(value);
  return key ?? "";
}

export function loadKeypair(file) {
  return Keypair.fromSecretKey(Uint8Array.from(readJson(file)));
}

export function saveKeypair(file, keypair) {
  ensureDir(path.dirname(file));
  fs.writeFileSync(file, `${JSON.stringify(Array.from(keypair.secretKey))}\n`, { mode: 0o600 });
}

export function getOrCreateKeypair(file) {
  if (fs.existsSync(file)) return loadKeypair(file);
  const keypair = Keypair.generate();
  saveKeypair(file, keypair);
  return keypair;
}

export class KeypairWallet {
  constructor(keypair) {
    this.payer = keypair;
    this.publicKey = keypair.publicKey;
  }

  async signTransaction(transaction) {
    transaction.partialSign(this.payer);
    return transaction;
  }

  async signAllTransactions(transactions) {
    for (const transaction of transactions) {
      transaction.partialSign(this.payer);
    }
    return transactions;
  }
}

export function createProvider(adminKeypair, rpcUrl = DEFAULT_RPC_URL) {
  const connection = new web3.Connection(rpcUrl, "confirmed");
  return new anchor.AnchorProvider(connection, new KeypairWallet(adminKeypair), {
    commitment: "confirmed",
    preflightCommitment: "confirmed",
  });
}

export function createProgram(provider, idl = loadIdl()) {
  return new anchor.Program(idl, provider);
}

export function keyPaths() {
  return {
    admin: path.join(KEY_DIR, "admin.json"),
    router: path.join(KEY_DIR, "router.json"),
    underwriter: path.join(KEY_DIR, "underwriter.json"),
    revenuePayer: path.join(KEY_DIR, "revenue-payer.json"),
    agentWallet: path.join(KEY_DIR, "agent-wallet.json"),
    merchantOwner: path.join(KEY_DIR, "merchant-owner.json"),
  };
}

export function deploymentExists() {
  return fs.existsSync(DEPLOYMENT_PATH);
}

export function loadDeployment() {
  if (!deploymentExists()) {
    throw new Error(`Missing local deployment at ${DEPLOYMENT_PATH}. Run npm run setup:local first.`);
  }
  return readJson(DEPLOYMENT_PATH);
}

export function writeDeployment(deployment) {
  writeJson(DEPLOYMENT_PATH, deployment);
}

export function readActionLog() {
  if (!fs.existsSync(ACTION_LOG_PATH)) return [];
  return readJson(ACTION_LOG_PATH);
}

export function appendActionLog(action) {
  const actions = readActionLog();
  actions.unshift({
    id: `act_${String(actions.length + 1).padStart(4, "0")}`,
    createdAt: new Date().toISOString(),
    ...action,
  });
  writeJson(ACTION_LOG_PATH, actions.slice(0, 100));
}

export function sourceEnum(source = "manual") {
  const normalized = source.replace(/_([a-z])/g, (_, char) => char.toUpperCase());
  if (normalized === "wallet") return { wallet: {} };
  if (normalized === "x402") return { x402: {} };
  if (normalized === "stripeAttested") return { stripeAttested: {} };
  if (normalized === "testMerchant") return { testMerchant: {} };
  return { manual: {} };
}

export function riskGradeEnum(grade = "B") {
  const key = String(grade).toLowerCase();
  return { [key]: {} };
}

export function approvalModeEnum(mode = "Agentic") {
  const normalized = String(mode).toLowerCase();
  if (normalized === "manual") return { manual: {} };
  if (normalized === "rules") return { rules: {} };
  return { agentic: {} };
}

export function borrowerTypeEnum(type = "Business") {
  const normalized = String(type).toLowerCase();
  if (normalized === "neobank") return { neobank: {} };
  if (normalized === "protocol") return { protocol: {} };
  return { business: {} };
}

export function verificationStatusEnum(status = "ZkVerified") {
  const normalized = String(status).toLowerCase();
  if (normalized === "unverified") return { unverified: {} };
  if (normalized === "documents") return { documents: {} };
  if (normalized === "manuallyverified" || normalized === "manual") return { manuallyVerified: {} };
  return { zkVerified: {} };
}

export async function airdropIfNeeded(connection, publicKey, minimumSol = 2) {
  const minimumLamports = minimumSol * LAMPORTS_PER_SOL;
  const balance = await connection.getBalance(publicKey, "confirmed");
  if (balance >= minimumLamports / 2) return null;

  const signature = await connection.requestAirdrop(publicKey, minimumLamports);
  await connection.confirmTransaction(signature, "confirmed");
  return signature;
}

export async function getTokenAmount(connection, tokenAccount) {
  try {
    const result = await connection.getTokenAccountBalance(pubkey(tokenAccount), "confirmed");
    return BigInt(result.value.amount);
  } catch {
    return 0n;
  }
}
