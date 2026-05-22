import assert from "node:assert/strict";
import test from "node:test";

import { bytes, formatUsdc, parseUsdc, seed16 } from "../src/chain.mjs";

test("USDC parser uses six decimal base units", () => {
  assert.equal(parseUsdc("2.50"), 2_500_000n);
  assert.equal(parseUsdc("0.000001"), 1n);
  assert.equal(formatUsdc(2_500_000n), "2.50");
});

test("deterministic local seeds have fixed lengths", () => {
  assert.equal(seed16(40).length, 16);
  assert.equal(bytes(32, 20).length, 32);
});
