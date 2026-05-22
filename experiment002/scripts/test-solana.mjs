import { spawnSync } from "node:child_process";

import { SOLANA_DIR } from "../src/chain.mjs";

const rustc = spawnSync("rustup", ["which", "rustc", "--toolchain", "stable"], {
  encoding: "utf8",
});

if (rustc.status !== 0) {
  process.stderr.write(rustc.stderr || rustc.stdout);
  process.exit(rustc.status ?? 1);
}

const rustcPath = rustc.stdout.trim();
const result = spawnSync(
  "rustup",
  [
    "run",
    "stable",
    "cargo",
    "--config",
    `build.rustc="${rustcPath}"`,
    "test",
    "--manifest-path",
    "programs/agc_credit_control/Cargo.toml",
    "--lib",
  ],
  {
    cwd: SOLANA_DIR,
    stdio: "inherit",
  },
);

process.exit(result.status ?? 1);
