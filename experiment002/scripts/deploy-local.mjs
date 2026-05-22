import fs from "node:fs";

import {
  DEFAULT_RPC_URL,
  KEY_DIR,
  PROGRAM_KEYPAIR_PATH,
  PROGRAM_SO_PATH,
  SOLANA_DIR,
  airdropIfNeeded,
  createProvider,
  ensureDir,
  getOrCreateKeypair,
  keyPaths,
} from "../src/chain.mjs";
import { run, waitForRpc } from "./lib/process.mjs";

const args = new Set(process.argv.slice(2));
const skipBuild = args.has("--skip-build");
const rpcUrl = process.env.AGC_RPC_URL ?? DEFAULT_RPC_URL;

ensureDir(KEY_DIR);
const keys = keyPaths();
const admin = getOrCreateKeypair(keys.admin);
const connection = await waitForRpc(rpcUrl);
await airdropIfNeeded(connection, admin.publicKey, 10);

if (!skipBuild) {
  run("anchor", ["build"], { cwd: SOLANA_DIR });
}

if (!fs.existsSync(PROGRAM_SO_PATH)) {
  throw new Error(`Missing program binary at ${PROGRAM_SO_PATH}. Run npm run build:solana first.`);
}
if (!fs.existsSync(PROGRAM_KEYPAIR_PATH)) {
  throw new Error(`Missing program keypair at ${PROGRAM_KEYPAIR_PATH}. Run npm run build:solana first.`);
}

run("solana", [
  "program",
  "deploy",
  "--url",
  rpcUrl,
  "--keypair",
  keys.admin,
  "--program-id",
  PROGRAM_KEYPAIR_PATH,
  PROGRAM_SO_PATH,
]);

const provider = createProvider(admin, rpcUrl);
console.log(`Deployed AGC credit control program to ${provider.connection.rpcEndpoint}`);
