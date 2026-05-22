import { DEFAULT_RPC_URL } from "../src/chain.mjs";
import { startValidator } from "./lib/process.mjs";

const args = new Set(process.argv.slice(2));
const background = args.has("--background");
const reset = args.has("--reset");

const result = await startValidator({ background, reset, rpcUrl: DEFAULT_RPC_URL });
if (background) {
  if (result.alreadyRunning) {
    console.log(`Local validator already running at ${DEFAULT_RPC_URL}`);
  } else {
    console.log(`Local validator running at ${DEFAULT_RPC_URL} with pid ${result.pid}`);
  }
}
