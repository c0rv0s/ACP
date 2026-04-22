import test from "node:test";
import assert from "node:assert/strict";
import { isHex, keccak256, toHex } from "viem";
import {
  attestProductivePayment,
  FacilitatorRequestError,
  publicFacilitatorConfig,
  type FacilitatorConfig,
} from "./lib.js";

const config: FacilitatorConfig = {
  chainId: 31337,
  routerAddress: "0x0000000000000000000000000000000000000a11",
  facilitatorPrivateKey:
    "0x1111111111111111111111111111111111111111111111111111111111111111",
  partnerPolicies: [
    {
      key: "demo-x402",
      name: "Demo x402 Relay",
      routeTag: "demo-x402",
      qualityScoreBps: 9500,
      ttlSeconds: 300,
    },
  ],
};

test("attestProductivePayment rejects unknown partner keys", async () => {
  await assert.rejects(
    attestProductivePayment(
      config,
      {
        payer: "0x0000000000000000000000000000000000000b0b",
        recipient: "0x0000000000000000000000000000000000000c0c",
        agcAmountIn: 10n ** 18n,
        paymentId:
          "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        partnerKey: "missing-partner",
      },
      1_700_000_000,
    ),
    (error: unknown) =>
      error instanceof FacilitatorRequestError &&
      error.statusCode === 404 &&
      error.message === "Unknown partnerKey.",
  );
});

test("attestProductivePayment uses configured quality route and deadline", async () => {
  const response = await attestProductivePayment(
    config,
    {
      payer: "0x0000000000000000000000000000000000000b0b",
      recipient: "0x0000000000000000000000000000000000000c0c",
      agcAmountIn: 10n ** 18n,
      paymentId:
        "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
      partnerKey: "demo-x402",
      // @ts-expect-error service ignores client-supplied reward controls
      qualityScoreBps: 1,
      // @ts-expect-error service ignores client-supplied route
      routeHash:
        "0xcccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
      // @ts-expect-error service ignores client-supplied expiry
      deadline: 7,
    },
    1_700_000_000,
  );

  assert.equal(response.attestation.qualityScoreBps, 9500);
  assert.equal(response.attestation.deadline, 1_700_000_300);
  assert.equal(response.attestation.routeHash, keccak256(toHex("demo-x402")));
  assert.equal(response.attestation.agcAmountIn, (10n ** 18n).toString());
  assert.ok(isHex(response.signature, { strict: true }));
});

test("publicFacilitatorConfig exposes partner policy metadata", () => {
  const publicConfig = publicFacilitatorConfig(config);
  assert.equal(publicConfig.chainId, 31337);
  assert.equal(publicConfig.routerAddress, config.routerAddress);
  assert.equal(publicConfig.partners.length, 1);
  assert.equal(publicConfig.partners[0]?.qualityScoreBps, 9500);
  assert.equal(publicConfig.partners[0]?.routeHash, keccak256(toHex("demo-x402")));
});
