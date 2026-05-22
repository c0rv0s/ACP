import fs from "node:fs";

import { ACTION_LOG_PATH, DEFAULT_RPC_URL, DEPLOYMENT_PATH, LOCAL_DIR } from "../src/chain.mjs";
import { run, startValidator } from "./lib/process.mjs";

const args = new Set(process.argv.slice(2));
const keepLedger = args.has("--keep-ledger");

if (!keepLedger) {
  fs.rmSync(DEPLOYMENT_PATH, { force: true });
  fs.rmSync(ACTION_LOG_PATH, { force: true });
}

const validator = await startValidator({ background: true, reset: !keepLedger, rpcUrl: DEFAULT_RPC_URL });
if (validator.alreadyRunning) {
  console.log(`Local validator already running at ${DEFAULT_RPC_URL}`);
} else {
  console.log(`Local validator running at ${DEFAULT_RPC_URL} with pid ${validator.pid}`);
}

run("node", ["scripts/deploy-local.mjs"], { cwd: process.cwd() });
run("node", ["scripts/bootstrap-local.mjs"], { cwd: process.cwd() });

console.log("");
console.log("Local AGC sandbox is ready.");
console.log("Run npm run dev-wallet to print the local-only key for Phantom import.");
console.log("Run npm run dev and open http://127.0.0.1:8082");
console.log(`Local files are under ${LOCAL_DIR}`);
