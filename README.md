# Agent Credit Protocol

Foundry + Uniswap v4 MVP for the Agent Credit Protocol, with a viem/wagmi dashboard under [`/Users/nate/Desktop/agc/web`](/Users/nate/Desktop/agc/web).

This README now serves two jobs:

1. a reference copy of the protocol spec
2. an implementation note for what the shipped MVP actually changed or narrowed

## Status

The repo implements a conservative v1:

- `AGCToken` with role-gated mint and burn
- canonical `AGC/USDC` Uniswap v4 pool hook with dynamic LP fee overrides, hook fees, productive-flow receipts, oracle-style epoch accounting, and LP anti-JIT penalties
- `PolicyController` for keeper-driven epoch settlement, mint caps, defense buybacks, and reward routing
- `StabilityVault`, `RewardDistributor`, and `SettlementRouter`
- viem local deployment script that mines a valid v4 hook address, initializes the pool, seeds liquidity, and writes frontend env vars

## What Changed In The Implemented MVP

The original prose spec was intentionally broader than the first code release. These are the important differences.

### 1. Policy execution is split between snapshots and keeper commands

The spec describes a rich fully-determined epoch policy engine driven by onchain metrics and formulas. The MVP is narrower:

- [`/Users/nate/Desktop/agc/src/AGCHook.sol`](/Users/nate/Desktop/agc/src/AGCHook.sol) accumulates the fast-path epoch snapshot on every swap and liquidity action.
- [`/Users/nate/Desktop/agc/src/PolicyController.sol`](/Users/nate/Desktop/agc/src/PolicyController.sol) validates a keeper-supplied `EpochCommand` against hard guardrails such as mint caps, band bounds, cooldowns, and defense-only buybacks.
- [`/Users/nate/Desktop/agc/src/PolicyEngine.sol`](/Users/nate/Desktop/agc/src/PolicyEngine.sol) exists as a pure helper for deriving metrics, selecting regimes, updating anchors, and quoting mint/buyback budgets, but it is not yet wired as the sole authority inside `PolicyController`.

That means the policy loop is deterministic enough to simulate and test, but the final regime selection and budget command is still supplied by a trusted keeper in v1.

### 2. The productive-flow trust model is intentionally narrow

The spec leaves room for richer productive-flow attestation. The MVP only trusts explicit router metadata:

- [`/Users/nate/Desktop/agc/src/SettlementRouter.sol`](/Users/nate/Desktop/agc/src/SettlementRouter.sol) is the intended trusted path for productive payment settlement.
- The hook only decodes structured metadata when the calling router is whitelisted.
- There is no permissionless productive-flow attestation in v1.

### 3. Reward streaming is implemented, but inline swap rewards are not

The hook emits reward receipts and the distributor settles them later. This matches the intent of the spec, but the exact flow is narrower:

- productive swaps create non-transferable receipts in the hook
- receipts are later claimed into time-vested AGC streams in [`/Users/nate/Desktop/agc/src/RewardDistributor.sol`](/Users/nate/Desktop/agc/src/RewardDistributor.sol)
- LP and integrator budget streams are controller-scheduled rather than automatically inferred from a general-purpose reputation layer

### 4. USDC decimal handling was normalized in the implementation

The prose spec speaks in economic units, but code needs token-decimal precision. The implemented reward path normalizes around 6-decimal USDC amounts in `RewardDistributor` so productive receipt rewards do not overpay on claim.

### 5. The canonical pool key must be derived from the hook deployment

The protocol summary talks about a dedicated Uniswap v4 hook, but the deployed system has one extra practical requirement:

- the canonical pool key is bound to the actual hook address
- the hook address must satisfy Uniswap v4 hook permission bits
- the local deployment script mines a valid hook deployment and then builds the pool around that address

This is why local deployment lives in [`/Users/nate/Desktop/agc/script/deployLocal.mjs`](/Users/nate/Desktop/agc/script/deployLocal.mjs) instead of a trivial one-shot deployment.

### 6. The buyback path is implemented through the router

Defense buybacks are executed by the controller through the settlement router:

- the vault releases USDC
- the router swaps `USDC -> AGC`
- bought `AGC` is burned

That preserves one canonical swap path instead of maintaining a separate treasury execution module in v1.

### 7. Build and test plumbing is split

The codebase currently needs a split Foundry compile matrix:

- `AGCHook` compiles in the regular pipeline
- `PolicyController`, router-facing tests, and reward/policy suites require `--via-ir`

Use the package scripts rather than plain `forge build` / `forge test` at the repo root.

## Reference Spec

This is the cleaned-up protocol spec adapted to the code that ships in this repository.

## 1. Protocol Summary

Primary use case: agents hold `AGC` as working capital, then convert to `USDC` at the point of payment for x402-style settlement.

This is not a collateral-backed stablecoin.

It is a policy-managed floating credit currency with:

- a soft anchor to `USDC`
- no guaranteed redemption
- no claim on offchain dollars
- no hard 1:1 convertibility promise
- supply expansion and contraction driven by protocol policy
- last-mile settlement via `USDC`

The system is built around a Uniswap v4 pool with a dedicated hook. Uniswap v4 supports pool-specific hooks, before/after swap logic, liquidity hooks, per-swap dynamic fees, and custom accounting. Hook fees are separate from LP fees, and v4 is designed to support custom fee logic, custom oracles, and pool-level behaviors [1][4][5][6].

x402-style commerce makes machine payments legible and frequent. x402 is explicitly designed for automatic stablecoin payments over HTTP, including AI agents paying for APIs and services, which makes `USDC` the natural settlement asset for the last mile [2].

Traditional stablecoins are warehouse receipts for dollars. This protocol instead creates circulating onchain purchasing power for agents.

The core bet is:

1. agents need a transaction balance
2. they do not need that balance to be a legally redeemable dollar
3. they do need low short-horizon volatility and reliable exit liquidity into `USDC`
4. if the protocol can create new supply against real machine-commerce demand, it can expand the agent economy beyond the static stock of existing stablecoins
5. if it can also capture part of that expansion, it becomes a profitable monetary network rather than a passive asset

The purpose is not "be a better USDC." The purpose is:

- to create elastic working capital
- for autonomous buyers
- with `USDC` as settlement substrate
- and a hook-governed market as the monetary policy engine

## 2. Design Goals

### Primary goals

- keep `AGC` legible enough for short-duration holding by agents
- preserve deep, cheap conversion into `USDC`
- expand supply when transactional demand is real and system health is strong
- contract supply when stress rises
- reward productive participation, not passive speculation
- capture protocol revenue through seigniorage, hook fees, and spread

### Non-goals

- no hard redemption at $1
- no offchain reserve attestations
- no collateral vault mint/burn model
- no governance token required for solvency
- no promise that every `AGC` is backed by `USDC`

### Product constraints

- must remain pure algo in the monetary sense
- `USDC` can be used as a market reference and stability buffer, but not as a redeemable backing claim
- the system should be explainable in one sentence:

> Hold AGC for incentives and working capital; convert to USDC only when you actually pay.

## 3. System Architecture

### A. `AGC` token

An ERC-20 fixed-balance token. No global rebases and no wallet-wide balance mutation.

Monetary policy is implemented through:

- controlled minting
- treasury buybacks
- burns
- fee routing
- reward distribution
- optional sinks and locks

### B. `AGC/USDC` Uniswap v4 pool

The canonical liquidity venue for:

- price discovery
- exit liquidity
- policy enforcement
- fee capture

### C. `AGCHook`

The fast-path policy engine. Responsibilities:

- dynamic fee setting
- trade classification
- per-swap metrics collection
- anti-JIT liquidity logic
- hook fee charging
- oracle-style observation updates

### D. `PolicyController`

The slow-path monetary policy engine in the repo. Responsibilities:

- epoch settlement
- regime selection input validation
- issuance budget enforcement
- contraction budget enforcement
- treasury allocation rules
- parameter storage

In the current MVP, the controller validates keeper-supplied epoch commands instead of autonomously deriving the full policy action graph onchain.

### E. `StabilityVault`

Treasury vault that holds:

- `USDC` earned from hook fees
- protocol-owned `AGC`
- reserve budget for buybacks
- incentive budgets

This vault is not redeemable collateral.

### F. `RewardDistributor`

Streams newly issued `AGC` to:

- agent users
- LPs
- x402 facilitators
- integrators
- treasury-adjacent incentive buckets

### G. `SettlementRouter`

Trusted router for payment flows. Responsibilities:

- receive `AGC`
- swap into `USDC`
- execute payment or transfer paths
- attach productive-flow metadata for the hook

Because hooks see the pool manager as `msg.sender`, and the sender passed into hook logic is usually the router rather than the end user, attribution relies on trusted routers that expose the original sender in a standard way [3].

### H. Internal oracle layer

Built from hook observations. It can compute or approximate:

- short TWAP
- settlement-weighted prices
- realized volatility
- net flow imbalance
- productive vs speculative volume mix
- repeat productive user counts

Depth-to-slippage inputs are still expected from keepers or external observation in the MVP rather than inferred purely inside the hook.

## 4. Monetary Frame

### 4.1 The unit

`AGC` is best framed as:

> a floating transaction-credit unit optimized for autonomous commerce

It is not:

- a deposit receipt
- a redeemed dollar
- an IOU on a bank account

It is a network liability accepted because it is useful and liquid.

### 4.2 The anchor

The protocol does not target a strict $1 peg.

Instead it targets a crawling soft anchor against `USDC`.

Definitions:

- `P_t`: current short-horizon TWAP of `AGC/USDC`
- `A_t`: policy anchor
- `B_t`: band half-width
- acceptable band: `[A_t - B_t, A_t + B_t]`

Initial condition:

- `A_0 = 1.00 USDC`

Anchor update target:

`A_t = clamp( EMA_7d(productive settlement execution price), A_(t-1)*(1-eta), A_(t-1)*(1+eta) )`

This gives:

- short-run stability
- long-run flexibility
- no hard redemption fiction

### 4.3 Policy objective

The protocol maximizes a composite objective:

`maximize machine GDP = payment throughput x active agent participation x reserve efficiency x protocol revenue`

Practical proxy objective:

- keep realized volatility low enough for 1-72 hour agent holding periods
- keep `AGC -> USDC` conversion cheap
- expand supply only when productive demand is strong
- accumulate treasury in good times
- spend treasury to defend utility in bad times

## 5. State Variables

The policy engine uses the following variables each epoch.

### Price and volatility

- `P_t`: short TWAP
- `P_mid`: medium TWAP
- `sigma_t`: realized volatility
- `dev_t = (P_t - A_t) / A_t`

### Liquidity and exit health

- `D1_t`: `USDC` depth to 1% slippage
- `D2_t`: `USDC` depth to 2% slippage
- `C_t = D1_t / M_float`
- `X_t`: net exit pressure = net `AGC -> USDC` outflow / total volume

### Usage quality

- `V_t`: payment velocity
- `U_t`: productive usage ratio = productive volume / total volume
- `R_t`: repeat-user ratio = recurring productive wallets / active productive wallets
- `I_t`: idle share = dormant `AGC` / circulating float

### Growth

- `G_t`: growth rate of productive payment flow
- `L_t`: LP stability score
- `S_t`: system stress score

### Supply

- `M_total`
- `M_float`
- `M_locked`
- `M_treasury`

## 6. Regime Framework

The protocol operates in four regimes.

### 6.1 Expansion

Conditions:

- `P_t` within band or slightly above anchor
- `sigma_t` below threshold
- `C_t` above minimum
- `U_t` above minimum
- `X_t` below stress threshold
- `G_t` positive

Effects:

- new issuance allowed
- productive rebates active
- LP incentives active
- treasury seigniorage active

### 6.2 Neutral

Conditions:

- band respected
- mixed but non-stressed usage metrics

Effects:

- no discretionary expansion
- low buybacks
- normal dynamic fees
- rewards only from previously authorized streams

### 6.3 Defense

Triggered when:

- `P_t < A_t - B_t`
- or `sigma_t` spikes
- or `C_t` falls below critical
- or `X_t` exceeds critical threshold

Effects:

- stop issuance
- raise defensive fees on exits and speculation
- deploy `USDC` treasury for buybacks
- widen policy band if needed
- lower or suspend most incentives except direct productive-routing subsidies

### 6.4 Recovery

Activated after defense when:

- price re-enters band
- volatility cools
- coverage improves

Effects:

- no immediate expansion
- limited incentives resume
- treasury rebuild priority
- cooldown before full expansion returns

## 7. Fast-Path Hook Design

### 7.1 Philosophy

The hook should only do what must happen inside the swap path:

- set fees
- classify flow
- record metrics
- enforce LP rules
- emit accounting signals

It should not run heavy monetary-policy computation on every swap.

### 7.2 Hook permissions

The pool-specific hook uses:

- `beforeSwap`
- `afterSwap`
- `beforeAddLiquidity`
- `afterAddLiquidity`
- `beforeRemoveLiquidity`
- `afterRemoveLiquidity`

The MVP does not use AsyncSwap or a custom curve in v1.

### 7.3 `beforeSwap`

Responsibilities:

- identify flow class
- recover original sender when possible
- set dynamic LP fee
- determine hook fee tier
- apply temporary defense surcharge on stressed `AGC -> USDC` exits

Flow classes:

- `productive_payment`
- `inventory_rebalance`
- `speculative_trade`
- `liquidity_management`
- `unknown`

In the shipped code, classification is driven by trusted-router metadata plus a fallback `unknown` class.

Base fee model:

`lpFee = f_base + alpha*sigma_t + beta*imbalance_t + gamma*exitStress_t - delta*productiveDiscount_t`

### 7.4 `afterSwap`

Responsibilities:

- update observations
- update epoch counters
- create reward receipts for productive flow
- update the internal oracle state

The hook maintains buffers for:

- short TWAP
- productive settlement price
- realized vol
- aggregate flow counts

### 7.5 Liquidity hooks

`beforeAddLiquidity` and `afterAddLiquidity`:

- track LP age
- score durable participation

`beforeRemoveLiquidity` and `afterRemoveLiquidity`:

- discourage JIT liquidity
- charge early withdrawal hook fees when position age is below threshold

## 8. Slow-Path Epoch Policy

Intended epoch cadence:

- 15 minutes for metrics snapshot
- 1 hour for policy update
- 24 hours for large supply recalibration

In the MVP, a permissioned keeper or the owner triggers `settleEpoch`.

### 8.1 Step 1: compute health metrics

Compute:

- `dev_t`
- `sigma_t`
- `C_t`
- `U_t`
- `X_t`
- `G_t`
- `R_t`

Expansion score:

`E_t = w1*U_t + w2*G_t + w3*C_t + w4*R_t - w5*sigma_t - w6*X_t - w7*abs(dev_t)`

Stress score:

`S_t = q1*sigma_t + q2*X_t + q3*max(0, C_min - C_t) + q4*max(0, (A_t - B_t) - P_t)`

### 8.2 Step 2: choose regime

- if `S_t >= S_critical` -> Defense
- else if `E_t >= E_expand` and price not weak -> Expansion
- else if recently in defense and cooldown active -> Recovery
- else -> Neutral

### 8.3 Step 3: set budgets

Expansion budget:

`MintBudget_t = M_float * k_e * clamp(E_t - E_expand, 0, cap_e)`

Hard caps:

- max inflation per epoch
- max inflation per day
- no mint while `P_t < A_t`
- no expansion if productive usage is weak

Contraction budget:

`BuybackBudget_t = min(TreasuryUSDC * k_b * S_t, BuybackCap_t)`

In the MVP, the controller enforces caps and Defense-only buybacks while the keeper supplies the concrete budget numbers.

## 9. Issuance Design

### 9.1 Principle

New supply should never be sprayed pro rata to all holders. That rewards passive speculation instead of commerce.

### 9.2 Issuance destinations

Recommended default split:

- 30% productive agent rebates
- 20% LP rewards
- 20% integrator / facilitator incentives
- 20% treasury seigniorage
- 10% stability reserve / insurance bucket

Aggressive growth split:

- 35 / 20 / 15 / 20 / 10

Conservative split:

- 20 / 20 / 15 / 30 / 15

### 9.3 Issuance timing

Do not release instantly. Use streamed emissions:

- agents: vest 24-72 hours
- LPs: stream over 7 days
- integrators: stream over 7-30 days
- treasury: lock 30 days before discretionary use

### 9.4 Productive rebate logic

An agent earns `AGC` for useful actions such as:

- x402 purchase routing
- recurring service consumption
- net-new settlement volume
- bringing high-quality payment throughput

Example:

`AgentReward_i = baseRate * productiveVolume_i * qualityMultiplier_i * regimeMultiplier`

## 10. Contraction Design

### 10.1 Continuous buy-and-burn

Use accumulated `USDC` hook fees and treasury `USDC` to buy back `AGC` and burn it during stress.

### 10.2 Stress exit toll

In defense mode:

- `AGC -> USDC` exits pay elevated hook fees
- proceeds route to the reserve and buyback buffer

### 10.3 No issuance while weak

If `P_t < A_t` or `X_t` is high:

- all growth emissions stop
- only defensive flows remain active

### 10.4 Active-balance preference

Do not demurrage wallets in v1.

Instead:

- active productive balances earn
- idle balances are diluted relative to active users

### 10.5 Sink mechanisms

Future sinks may include:

- discounted API access when paying in `AGC`
- execution priority credits
- premium routing
- facilitator staking
- governance weight if governance exists
- partner app access

## 11. Revenue Model

### 11.1 Revenue streams

- treasury seigniorage during expansion
- hook fees charged separately from LP fees
- defense exit surcharge when liquidity is scarce
- partner routing fees
- treasury inventory management

### 11.2 Treasury spending priorities

1. defense buybacks
2. reserve rebuild
3. productive demand subsidies
4. LP support
5. partner / integrator growth
6. discretionary treasury operations

## 12. Why Agents Hold AGC Instead Of USDC

Agents choose `AGC` because it offers:

### A. Lower effective payment cost

Productive usage can earn rebates.

### B. Monetary upside

`USDC` is inert. `AGC` participates in network growth.

### C. Better working-capital economics

Holding `AGC` can be rational if:

- price variance is tolerable
- expected rewards offset conversion friction
- partner integrations provide usage discounts

### D. Machine-native incentives

Agents that route volume, maintain activity, or deepen network liquidity can earn more.

### E. A share in seigniorage

The network pays growth participants in newly created monetary capacity.

## 13. Settlement Flow

### 13.1 Standard agent purchase flow

1. agent holds `AGC`
2. agent accesses a paid endpoint
3. endpoint demands payment in x402-compatible stablecoin flow
4. agent calls `SettlementRouter`
5. router swaps `AGC -> USDC` through the canonical v4 pool
6. router completes the payment path
7. hook marks volume as productive
8. reward receipt is created
9. reward settles later through the distributor

### 13.2 Working-capital acquisition flow

1. agent or sponsor buys `AGC`
2. holds as transaction float
3. uses only as needed for future payments

### 13.3 LP flow

1. LP deposits into the canonical pool
2. hook tracks LP age and stability
3. durable liquidity earns rewards
4. early mercenary behavior is penalized

## 14. Anti-Abuse Design

### 14.1 Sybil farming

Mitigations:

- trusted router list
- facilitator attestations
- per-epoch wallet reward caps
- recurrence scoring
- counterpart diversity checks
- volume-to-fee anomaly monitoring
- hold-time before release

### 14.2 Wash routing

Mitigations:

- productive flow only through approved routers
- reward receipts tied to unique payment intent IDs
- deny rewards for self-settlement loops
- cross-check destination patterns

### 14.3 Oracle manipulation

Mitigations:

- use medium-horizon references where available
- require non-price signals for expansion
- cap per-epoch mint
- never expand only because price is high

### 14.4 LP griefing / JIT liquidity

Mitigations:

- withdrawal cooldown
- short-horizon removal fee
- LP age weighting
- reward only mature liquidity

### 14.5 Bank-run reflexivity

Mitigations:

- defense regime
- exit toll
- treasury buybacks
- no issuance under stress
- wide enough bands to avoid overreaction

### 14.6 Trusted-router spoofing

Only audited routers should be whitelisted. Router governance is security-critical because the hook relies on router-provided attribution.

## 15. Recommended v1 Parameters

These remain placeholders for simulation rather than final launch constants.

### Band and anchor

- `A_0 = 1.00`
- max anchor crawl = 10 bps/day
- base band = +/- 2.0%
- stressed band = +/- 4.0%

### Epochs

- metrics snapshot: 15 min
- policy epoch: 1 hr
- treasury rebalance: 24 hr

### Fees

- base LP fee: 10 bps
- productive payment LP fee: 3-8 bps
- speculative LP fee: 15-40 bps
- hook fee normal: 0-20 bps
- defense exit hook fee: up to 100-200 bps, capped and temporary

### Issuance

- max mint/day: 0.50% of float in v1
- max mint/epoch: 0.05% of float
- no mint if `P_t < A_t`
- no mint if `U_t < U_min`

### Buybacks

- spend 25-50% of treasury inflows during mild defense
- up to 80% during severe defense, subject to floor reserves

## 16. Governance

### 16.1 Governance philosophy

Governance should remain narrow.

Governance should control:

- thresholds
- reward splits
- trusted router registry
- treasury policies
- kill switches for incentives

Governance should not control:

- arbitrary minting outside policy rules
- selective confiscation
- ad hoc bailouts
- opaque intervention

### 16.2 Governance structure

Recommended launch structure:

- multisig at launch
- timelocked parameter changes
- onchain parameter registry
- transition to limited governance once product-market fit is real

### 16.3 Emergency controls

Allowed:

- pause rewards
- raise defense fees within bounded range
- freeze new router approvals
- halt new emission streams

Not allowed:

- stop exits
- freeze user balances
- mint rescue supply outside predefined policy authority

## 17. MVP Scope

### v1 should include

- `AGC` token
- one canonical `AGC/USDC` v4 pool
- hook with dynamic LP fee logic, hook fee logic, trade classification, oracle observations, and LP anti-JIT rules
- epoch policy controller
- treasury vault
- reward distributor
- one trusted settlement router for productive x402-style flows

### v1 should not include

- custom swap curve
- AsyncSwap
- cross-chain issuance
- fully permissionless productive-flow attestation
- general lending
- governance token

This repo follows that narrower v1.

## 18. v2 Extensions

After the first market cycle proves itself:

### A. Custom curve

Uniswap v4 supports custom accounting and custom curves, but these add risk [5].

### B. Multi-router productive attestation

Allow multiple x402 facilitators and payment processors.

### C. Agent reputation layer

Higher reward multipliers for reliable long-horizon users.

### D. Cross-app sinks

Apps offer discounts or premium execution for locked `AGC`.

### E. Multi-quote discovery

Additional pools for price discovery while `AGC/USDC` remains the policy pool.

## 19. Formal Policy Pseudocode

```text
At each epoch t:

Inputs:
  P_t      = short TWAP AGC/USDC
  sigma_t  = realized vol
  D1_t     = USDC depth to 1% slippage
  X_t      = net AGC->USDC exit pressure
  U_t      = productive usage ratio
  G_t      = productive flow growth
  R_t      = repeat productive user ratio
  M_float  = liquid circulating AGC
  A_t      = current anchor
  B_t      = current half-band

Derived:
  C_t = D1_t / M_float
  dev_t = (P_t - A_t) / A_t

  E_t = w1*U_t + w2*G_t + w3*C_t + w4*R_t - w5*sigma_t - w6*X_t - w7*abs(dev_t)
  S_t = q1*sigma_t + q2*X_t + q3*max(0, C_min - C_t) + q4*max(0, (A_t - B_t) - P_t)

Regime selection:
  if S_t >= S_critical:
      regime = DEFENSE
  else if cooldown_active:
      regime = RECOVERY
  else if E_t >= E_expand and P_t >= A_t and U_t >= U_min:
      regime = EXPANSION
  else:
      regime = NEUTRAL

Actions:
  if regime == EXPANSION:
      MintBudget = M_float * k_e * clamp(E_t - E_expand, 0, cap_e)
      allocate MintBudget across agents / LPs / integrators / treasury / reserve
      stream rewards, do not release instantly

  if regime == NEUTRAL:
      no fresh discretionary mint
      continue previously scheduled streams
      low-level treasury buybacks optional

  if regime == DEFENSE:
      stop new issuance
      increase exit hook fee
      BuybackBudget = min(TreasuryUSDC * k_b * S_t, BuybackCap)
      buy AGC and burn
      widen band if stress rules require

  if regime == RECOVERY:
      keep issuance off
      keep incentives reduced
      rebuild reserve
      require cooldown before EXPANSION resumes

Anchor update:
  A_(t+1) = clamp(EMA_7d(productive settlement price), A_t*(1-eta), A_t*(1+eta))
```

## 20. Protocol Narrative

Agent Credit Protocol creates a floating, algorithmically managed transaction currency for autonomous agents. Agents hold `AGC` for incentives and working capital, and swap into `USDC` only at payment time. A Uniswap v4 hook governs fees, collects flow data, and powers the protocol's fast-path market policy, while an epoch controller adjusts issuance and contraction based on productive usage, volatility, exit pressure, and liquidity depth. The system does not promise redemption or collateral backing; instead it aims to keep `AGC` usable and profitable by stabilizing a soft `USDC` anchor, rewarding real payment activity, and using treasury revenue to absorb stress.

If you actually build and ship this, start narrower than the full theory:

- one pool
- one router
- one productive-flow category
- streamed rewards only
- no custom curve
- conservative expansion caps
- aggressive simulation before launch

The hard part is not minting.

The hard part is proving that productive machine demand is measurable enough to justify monetary expansion.

## Commands

Install:

```bash
pnpm install
forge install
```

Build contracts:

```bash
pnpm build:contracts
```

Run tests:

```bash
pnpm test
```

Build the frontend:

```bash
pnpm build:web
```

Generate ABI bindings for the frontend:

```bash
pnpm generate:abis
```

## Local Deploy

Start Anvil, then run:

```bash
export PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
pnpm deploy:local
pnpm build:web
```

The deploy script writes:

- [`/Users/nate/Desktop/agc/deployments/local.json`](/Users/nate/Desktop/agc/deployments/local.json)
- [`/Users/nate/Desktop/agc/web/.env.local`](/Users/nate/Desktop/agc/web/.env.local)

Those outputs give the wagmi app the deployed contract addresses for a local Anvil session.

## Foundry Note

This codebase currently needs split compilation:

- `AGCHook` compiles cleanly in the regular pipeline
- `PolicyController`, router-facing test suites, and reward/policy suites are run with `--via-ir`

The package scripts already encode the working build and test matrix, so use those instead of plain `forge build` / `forge test` at the repo root.

## References

[1] [Uniswap v4 Hooks](https://docs.uniswap.org/contracts/v4/concepts/hooks)  
[2] [Coinbase x402 Overview](https://docs.cdp.coinbase.com/x402/welcome)  
[3] [Access `msg.sender` Inside a Uniswap v4 Hook](https://docs.uniswap.org/contracts/v4/guides/accessing-msg.sender-using-hook)  
[4] [Uniswap v4 Overview](https://docs.uniswap.org/contracts/v4/overview)  
[5] [Uniswap v4 Custom Accounting](https://docs.uniswap.org/contracts/v4/guides/custom-accounting)  
[6] [Uniswap v4 Dynamic Fees](https://docs.uniswap.org/contracts/v4/concepts/dynamic-fees)
