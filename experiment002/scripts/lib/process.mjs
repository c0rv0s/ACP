import { spawn, spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

import { DEFAULT_RPC_URL, LOCAL_DIR, ensureDir, web3 } from "../../src/chain.mjs";

export const VALIDATOR_LEDGER_DIR = path.join(LOCAL_DIR, "validator-ledger");
export const VALIDATOR_LOG_PATH = path.join(LOCAL_DIR, "validator.log");
export const VALIDATOR_PID_PATH = path.join(LOCAL_DIR, "validator.pid");
export const TOOLCHAIN_PATH = [
  path.join(process.env.HOME ?? "", ".cargo", "bin"),
  path.join(process.env.HOME ?? "", ".local", "share", "solana", "install", "active_release", "bin"),
  process.env.PATH ?? "",
].join(":");

export function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: options.cwd,
    env: { ...process.env, PATH: TOOLCHAIN_PATH, ...(options.env ?? {}) },
    stdio: "inherit",
  });
  if (result.status !== 0) {
    const detail = result.error?.message ?? result.signal ?? `exit code ${result.status}`;
    throw new Error(`${command} ${args.join(" ")} failed with ${detail}`);
  }
}

export async function waitForRpc(rpcUrl = DEFAULT_RPC_URL, timeoutMs = 30_000) {
  const connection = new web3.Connection(rpcUrl, "confirmed");
  const deadline = Date.now() + timeoutMs;
  let lastError = null;

  while (Date.now() < deadline) {
    try {
      await connection.getVersion();
      return connection;
    } catch (error) {
      lastError = error;
      await new Promise((resolve) => setTimeout(resolve, 500));
    }
  }

  throw new Error(`Timed out waiting for Solana RPC at ${rpcUrl}: ${lastError?.message ?? "unknown error"}`);
}

export async function isRpcUp(rpcUrl = DEFAULT_RPC_URL) {
  try {
    await waitForRpc(rpcUrl, 1_000);
    return true;
  } catch {
    return false;
  }
}

export function readValidatorPid() {
  if (!fs.existsSync(VALIDATOR_PID_PATH)) return null;
  const pid = Number.parseInt(fs.readFileSync(VALIDATOR_PID_PATH, "utf8"), 10);
  return Number.isFinite(pid) ? pid : null;
}

export function stopValidatorFromPidFile() {
  const pid = readValidatorPid();
  if (!pid) return false;
  try {
    process.kill(pid, "SIGTERM");
  } catch {
    // The process may already be gone.
  }
  fs.rmSync(VALIDATOR_PID_PATH, { force: true });
  return true;
}

export async function startValidator({ background = false, reset = false, rpcUrl = DEFAULT_RPC_URL } = {}) {
  ensureDir(LOCAL_DIR);
  if (reset) {
    stopValidatorFromPidFile();
    fs.rmSync(VALIDATOR_LEDGER_DIR, { recursive: true, force: true });
  } else if (await isRpcUp(rpcUrl)) {
    return { alreadyRunning: true, rpcUrl };
  }

  const args = [
    "--ledger",
    VALIDATOR_LEDGER_DIR,
    "--rpc-port",
    "8899",
    "--bind-address",
    "127.0.0.1",
    "--limit-ledger-size",
  ];
  if (reset) args.unshift("--reset");

  if (!background) {
    const child = spawn("solana-test-validator", args, { env: { ...process.env, PATH: TOOLCHAIN_PATH }, stdio: "inherit" });
    child.on("exit", () => {
      fs.rmSync(VALIDATOR_PID_PATH, { force: true });
    });
    fs.writeFileSync(VALIDATOR_PID_PATH, String(child.pid));
    return new Promise((resolve, reject) => {
      child.on("error", reject);
      child.on("exit", (code) => {
        if (code === 0) resolve({ rpcUrl });
        else reject(new Error(`solana-test-validator exited with code ${code}`));
      });
    });
  }

  const log = fs.openSync(VALIDATOR_LOG_PATH, "a");
  const child = spawn("solana-test-validator", args, {
    detached: true,
    env: { ...process.env, PATH: TOOLCHAIN_PATH },
    stdio: ["ignore", log, log],
  });
  child.unref();
  fs.writeFileSync(VALIDATOR_PID_PATH, String(child.pid));
  await waitForRpc(rpcUrl, 30_000);
  return { pid: child.pid, rpcUrl };
}
