---
name: agc-agent-protocol
description: Explain how the Agent Credit Protocol currently works, how agents should use AGC, how public settlement differs from facilitator-attested productive settlement, how reward receipts and streams work, and what safety or trust-model caveats apply. Use when users ask for protocol walkthroughs, integration guidance, payment-flow help, reward guidance, or operator-facing explanations of the current implementation versus the broader spec.
---

# AGC Agent Protocol

## Overview

Explain the protocol as it is implemented now, not as the README theory alone describes it. Be explicit about whether you are describing:
- the current contracts in this repo
- the broader protocol spec / roadmap
- an intended future capability that is not live yet

Default to the current implementation in:
- `src/SettlementRouter.sol`
- `src/AGCHook.sol`
- `src/RewardDistributor.sol`
- `src/PolicyController.sol`
- `service/facilitator`
- `README.md`

## Workflow

### 1. Identify the caller's role

Frame the explanation around the user's role:
- agent operator paying a service
- merchant or API provider receiving USDC
- facilitator attesting productive flows
- LP or integrator asking how rewards work
- protocol operator asking about policy, treasury, or trust assumptions

### 2. Explain the two payment lanes clearly

Always distinguish these paths:
- Public settlement:
  - call `settlePayment`
  - router swaps `AGC -> USDC`
  - recipient gets USDC
  - no productive metadata is sent to the hook
  - no reward receipt is created by default
- Facilitator-attested productive settlement:
  - call `settleProductivePayment`
  - requires a trusted facilitator signature
  - signature covers payer, recipient, amount, payment ID, quality score, deadline, and route hash
  - hook sees `ProductivePayment` metadata
  - a productive reward receipt can be created

If the user asks how to get rewards, the answer is: use the facilitator-attested lane, not the public lane.

### 3. Explain rewards in two stages

Describe the reward path in order:
1. productive payment is attested and settled
2. hook creates a receipt
3. user claims the receipt through `RewardDistributor`
4. distributor creates a time-vested AGC stream
5. user later calls `claimStream`

Be explicit that receipt creation and stream claiming are separate actions.

### 4. Explain attestation practically

Do not assume merchants are the attesters.

Default explanation:
- the merchant only needs to receive USDC
- the attester is a trusted facilitator already in the payment path
- examples: payment gateway, x402 relay, sponsor, API marketplace, or agent platform
- if there is no facilitator attestation, settlement still works but rewards do not

### 5. Explain the trust model honestly

When asked about safety, include the current caveats:
- productive rewards depend on trusted facilitators
- policy is deterministic once the hook snapshot and external metrics are provided, but depth/growth inputs are still hybrid offchain inputs in v1
- local governance and owner-controlled safety controls are intended to sit behind a timelock
- the protocol is still being completed toward the broader spec
- public settlement is simpler and lower-trust than reward-eligible productive settlement

### 6. Explain the operator flow when relevant

If the user is asking how to actually use the system end to end, include:
- hold or acquire `AGC`
- use public settlement when you just need `AGC -> USDC` payment routing
- use the facilitator service when you need a productive attestation for rewards
- claim the reward receipt into a stream
- claim vested `AGC` from the stream later

Mention the in-repo facilitator service when useful:
- `POST /attest/productive-payment`
- `GET /health`
- `GET /config/public`

Avoid overstating autonomy or claiming the full spec is already live.

## Response Patterns

### Agent operator

- Explain when to use public settlement versus productive settlement.
- Tell them what the facilitator must sign.
- Tell them they still receive USDC settlement for the merchant either way.
- Tell them rewards are delayed and stream-based, not instant.

### Merchant or service provider

- Emphasize that the merchant can remain a plain USDC recipient.
- If they do not want to participate in attestations, they do not have to.
- The protocol can still settle to them through the public lane.

### Facilitator or integrator

- Explain the EIP-712 signature role.
- Explain that only trusted facilitators can unlock productive rewards.
- Explain that the facilitator is responsible for attesting real supported flows.

### Spec versus implementation

If the user asks what the protocol is supposed to do, give both layers:
- current implementation
- target spec or roadmap

Make the gap explicit instead of blending them together.
