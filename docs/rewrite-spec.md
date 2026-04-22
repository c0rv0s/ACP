# AGC Rewrite Spec

This is the clean-slate pre-launch spec.

This document uses `AGC` as the canonical ticker for Agent Credit Protocol.

Current repo identifiers may still say `ACP`, but that should be treated as legacy naming rather than the target product language.

## 1. Core Framing

AGC should not be framed primarily as:

- a token you hold briefly before swapping to `USDC`
- a checkout reward coin
- a disguised dollar stablecoin

AGC should be framed as:

- reserve-efficient private credit for the agent economy
- a synthetic monetary layer that expands against liquidity the way banking systems expand credit against reserves
- a floating credit asset whose policy tries to maximize credit outstanding while keeping withdrawals absorbable

Internal mental model:

- the pool liquidity is the reserve base
- the circulating AGC float is credit outstanding
- selling AGC is a withdrawal request
- policy should expand credit when reserve conditions and demand allow it
- policy should contract or defend when reserve coverage weakens

This is much closer to fractional reserve banking than to a stablecoin with full backing.

## 2. One-Sentence Product

AGC is a reserve-efficient credit asset for agents: the protocol expands AGC against available liquidity and real demand, keeps the market inside a stable working range, and lets long-term holders capture the upside through `xAGC`.

Short version:

- `AGC` = circulating credit
- `xAGC` = savings / equity-like claim on growth
- `USDC` = settlement and reserve asset

## 3. What The Protocol Is Trying To Maximize

The goal is not to maximize a peg score.

The goal is to maximize safe credit creation.

Define:

- `CreditOutstanding = circulating AGC * anchor price`
- `ReserveBase = USDC exit liquidity at target slippage`
- `ReserveEfficiency = CreditOutstanding / ReserveBase`

The system should try to make `ReserveEfficiency` as high as possible while keeping:

- price stability inside the target band
- orderly exits
- acceptable volatility
- confidence that typical withdrawals can be met

This is the right high-level objective:

- create as much agent-native purchasing power as possible
- with as little reserve liquidity as possible
- without letting the system tip into disorder

## 4. Assets

### 4.1 `AGC`

Liquid monetary unit.

What it is:

- circulating private credit for agents
- the thing agents hold as inventory
- the thing policy expands and contracts

What it is not:

- a redeemable dollar claim
- a fully backed stablecoin
- a hard-peg wrapper

### 4.2 `xAGC`

Locked savings receipt.

Purpose:

- capture the majority of AGC expansion
- later capture a share of protocol fee carry
- function like the equity-like upside layer of the system

If AGC is the circulating bank money, `xAGC` is the retained-earnings claim.

Design requirements:

- non-rebasing share token
- transferable
- backed by locked AGC
- redeemable with an exit fee

That gives the protocol two holder stories:

- hold AGC for agent inventory
- hold xAGC to own the growth machine

### 4.2.1 How `xAGC` accrues value

`xAGC` should not be a rebasing token.

It should work like a vault share:

- users deposit AGC
- the vault mints `xAGC` shares
- policy-directed AGC expansion is minted into the vault
- vault AGC per `xAGC` share rises over time

So the holder experience is:

- share count stays fixed
- claim on AGC grows

That is cleaner than rebasing because:

- it is easier to integrate
- it is easier to reason about
- it keeps the upside concentrated in exchange rate growth rather than balance mutation

Launch rule:

- `xAGC` gets expansion by direct AGC mint into the vault
- `xAGC` does not take routine negative rebases during ordinary contraction
- `xAGC` redemption uses an exit fee, not a cooldown

Recommended launch setting:

- `xAGC` redemption fee = `3%`
- fee is charged in AGC
- fee is sent to treasury

Why:

- it makes exits costly enough to reward patience
- it gives treasury more dry powder over time
- it avoids the UX awkwardness of a hard unlock queue

Treasury handling rule:

- treasury should not dump redemption-fee AGC immediately during stress
- it can hold AGC, market-make with it, or gradually convert part of it into `USDC` during healthy regimes

Contraction should mostly be handled by:

- halted issuance
- defense fees
- treasury buybacks and burns

Only in true insolvency-style scenarios should the protocol even consider forcing downside into the `xAGC` layer, and that should be treated as exceptional recapitalization, not routine policy.

### 4.3 `USDC`

Settlement and reserve asset.

Use cases:

- final payment rail
- treasury defense asset
- the practical withdrawal venue

`USDC` is a reserve buffer, not full backing.

## 5. Demand Sources

All of these can be examples of monetary demand:

- API settlement
- trading inventory
- prediction-market inventory
- general agent execution balances
- future use cases we cannot name today

The big conceptual change is this:

- payments are not the whole economy
- inventory is the whole economy
- payment is only one way inventory gets used

Policy does not need to know what demand is for.

It only needs to know:

- that demand exists
- that demand is persistent
- that liquidity can absorb withdrawals

## 6. Why The Dollar Level Does Not Matter

The actual token price is not important.

What matters is:

- that policy keeps AGC fairly stable around its chosen anchor
- that users do not interpret AGC as a 1:1 dollar claim
- that the band is believable and durable

So the launch anchor can be:

- `$0.50`
- `$1.00`
- `$3.00`

or something else entirely.

The choice should be made for narrative and psychology, not because the protocol has any natural right to sit at one dollar.

Recommended design principle:

- choose a price that makes it obvious AGC is its own unit, not "another dollar stablecoin"

## 7. Price Frame

### 7.1 Anchor

Start with:

- `A_0 = chosen launch reference price`

The anchor is soft and crawling.

It should follow a smoothed clearing price of real demand, not raw spot speculation.

Anchor source:

- EMA of clearing price over a sufficiently long window
- not just one narrow payment lane
- not a single raw spot print

### 7.2 Bands

Recommended normal band:

- plus or minus `3%`

Recommended stressed band:

- plus or minus `7%`

Interpretation:

- inside the normal band, policy is healthy
- outside the normal band, attention rises
- below the stressed floor, defense activates
- above the anchor with persistent demand, expansion is allowed

Again, the relevant question is not "is it one dollar?"

The relevant question is "is the credit unit stable enough to be useful?"

## 8. Signals The Protocol Should Care About

### 8.1 Reserve and stress signals

- `P`: short-horizon TWAP
- `sigma`: realized volatility
- `X`: exit pressure = `AGC -> USDC` outflow / total volume
- `R`: reserve coverage = exit liquidity at target slippage / credit outstanding

`R` is the onchain reserve ratio analog.

This is the key mental shift.

The current code calls this "coverage."

Economically, it is the reserve ratio.

### 8.2 Demand signals

- `B`: gross buy volume = `USDC -> AGC` volume over the epoch
- `S`: gross sell volume = `AGC -> USDC` volume over the epoch
- `N`: net buy pressure = `max(B - S, 0)`
- `G`: buy growth = change in gross buy volume from last epoch

### 8.3 Holder signals

- `H`: locked share = AGC locked behind `xAGC` / float
- `L`: lock flow = net new AGC entering `xAGC` this epoch / float

Expansion must require all three classes of evidence:

- market premium
- real buying
- real locking

If one leg is missing, the mint should stay small or go to zero.

Interpretation:

- `B` tells us whether people are actually reaching for AGC
- `N` tells us whether buys are dominating sells
- `L` tells us whether some of that demand is being committed into the long-term capital layer

This is enough for launch.

Transfer volume should not be a core policy input.

Why not:

- it is noisy
- it is easy to spoof
- many transfers are just wallet shuffling
- transfers do not necessarily express demand

## 9. Regimes

Keep four regimes.

- Expansion
- Neutral
- Defense
- Recovery

### 9.1 Defense

Enter defense if any are true:

- price breaks below the stressed floor
- realized volatility is too high
- reserve coverage is too weak
- exit pressure is too high

Effects:

- all new minting stops
- exit fees rise
- treasury buybacks begin
- market-growth campaigns pause

### 9.2 Recovery

After defense, require a cooldown.

Effects:

- no fresh expansion
- rebuild confidence
- rebuild reserve strength

### 9.3 Expansion

Enter expansion only if all are true:

- price is above anchor by a minimum premium
- that premium has persisted for multiple epochs
- gross buy volume is above a minimum floor
- net buy pressure is positive
- net lock flow is positive
- locked share is above a floor
- reserve coverage is healthy
- volatility is low
- exit pressure is low
- buy growth is positive

This is intentionally strict.

The protocol should only print more private credit when the market is clearly asking for it and the reserve base can support it.

### 9.4 Neutral

Everything else is neutral.

## 10. Expansion Rule

Expansion should happen after the fact at the epoch boundary.

Never inline inside user swaps.

### 10.1 Why delayed expansion is necessary

If users feel like their principal is being mutated in the swap path:

- the venue becomes hostile
- routing gets weird
- arbitrage quality gets worse
- the demand signal becomes corrupted

The swap path should measure.

The controller should decide later.

### 10.2 Demand score

Define:

- `premium = max(P - A, 0) / A`
- `demandSignal = min(premium, N, L, positive G)`

Meaning:

- premium must agree with net buys
- net buys must agree with new locking
- growth must be positive

### 10.3 Health score

Define:

- `healthSignal = min(R headroom, volatility headroom, exit headroom, buy floor)`

Meaning:

- reserve ratio must be healthy
- volatility must be acceptable
- withdrawals cannot already be stressing the system
- raw buy appetite must be present and not just one isolated burst

### 10.4 Mint formula

Use:

`Mint_t = FloatSupply * k * demandSignal * healthSignal`

with hard caps:

- max mint per epoch
- max mint per day
- zero mint in Defense
- zero mint in Recovery

This is the money-creation engine.

It should be viewed exactly that way.

## 11. Expansion Distribution

Do not route most of the upside through payment receipts.

Recommended default split:

- 50% to `xAGC`
- 20% to market-growth programs
- 10% to LPs
- 5% to integrators
- 15% to treasury

Why:

- `xAGC` becomes the main savings asset
- market-growth programs still accelerate adoption
- LPs still deepen the reserve base
- treasury still compounds for defense

Hard rule:

- the `xAGC` bucket must be the largest bucket

If the protocol does not primarily pay the equity-like upside layer, then the holder narrative is not strong enough.

## 12. Contraction Rule

Contraction should not mean negative rebasing every holder.

That is not needed.

Do this instead:

- stop minting
- raise defense fees on exits
- route those fees to treasury
- use treasury `USDC` to buy back and burn AGC
- if needed, widen the band temporarily during stress

This keeps downside simple:

- no new credit creation
- reserve defense turns on
- float is reduced by buybacks and burns

## 13. Why People Hold AGC

This needs to be explicit.

People hold AGC for two different reasons.

### 13.1 Hold AGC as inventory

Agents hold AGC because it is the monetary inventory of the agent economy.

This is the operational reason.

### 13.2 Hold xAGC as upside

Long-term holders hold xAGC because it owns most of the growth machine.

This is the speculative and savings reason.

That split is healthy.

AGC does not need to do every job by itself.

It needs:

- liquid usefulness
- stable range

xAGC needs:

- compounding upside
- narrative intensity

## 14. Why This Can Actually Get Big

The system becomes large if it behaves like reserve-efficient private credit for agents.

That means:

- every new reserve dollar of liquidity can support multiple dollars of circulating AGC
- policy carefully increases that multiplier over time
- long-term holders get paid when the multiplier expands
- the market comes to believe the system can safely sustain more credit than it has today

That belief is what drives the "classic crypto craze" reflexivity.

Not because AGC is pegged.

Not because checkout volume is growing.

Because the market starts pricing in:

- larger safe credit outstanding
- higher future seigniorage
- more value accruing to xAGC

## 15. What Gets Rewarded

Reward directly:

- locking AGC into xAGC
- providing durable liquidity
- bringing durable demand and depth
- productive payment flow

Do not reward directly:

- raw speculative churn
- same-epoch round trips
- plain buys as instant claim receipts

A buy proves demand.

A lock proves conviction.

That is enough.

The holder gets paid through xAGC.

## 16. Demand Attribution

The protocol should not depend on approved-route accounting as a core monetary input.

That is too brittle and too easy to turn into a governance bottleneck.

The policy engine should be able to operate on generic market observables:

- price
- premium persistence
- net buy versus sell pressure
- reserve coverage
- lock flow into `xAGC`
- realized volatility
- exit pressure

If the protocol later wants marketing programs or partner incentives, those can exist as peripheral programs.

They should not be required for the money engine to work.

## 17. Minimal Mechanical Design

From first principles, the minimum system is:

1. one liquid token: `AGC`
2. one locked upside token: `xAGC`
3. one canonical `AGC/USDC` market per deployment
4. one controller that reads price, reserve coverage, demand, and lock data
5. one treasury that accumulates `USDC` and executes buybacks
6. one generic demand-accounting layer based on market data, not whitelisted intent types

Everything else is secondary.

## 18. What Can Be Thrown Out

These ideas are not sacred:

- x402 as the main expansion source
- "hold before spend" as the lead narrative
- productive payment share as the main expansion gate
- payment receipts as the main way value accrues
- the assumption that the existing contracts define the right system boundaries

What should be preserved is the deeper idea:

- floating credit
- reserve-efficient issuance
- stable range rather than hard peg
- long-term holder upside
- treasury defense

## 19. Launch Parameters

Recommended starting posture:

- launch anchor: choose for narrative, not dogma
- normal band: plus or minus `3%`
- stressed band: plus or minus `7%`
- expansion premium threshold: `1%`
- minimum premium persistence: `3 epochs`
- minimum gross buy floor: start at `1% of float per epoch`
- minimum locked-share floor: `10% of float`
- minimum reserve coverage for expansion: `20%`
- neutral reserve coverage band: `12% to 20%`
- defense trigger reserve coverage: `below 12%`
- hard defense reserve coverage: `below 8%`
- epoch mint cap: `1% of float`
- daily mint cap: `5% of float`

Why these reserve numbers:

- `20%` coverage means the system is expanding against roughly `5x` reserve efficiency
- `12%` coverage means the system is already leaning aggressive and should stop expanding
- `8%` coverage means the system is in genuine defense territory

That is aggressive enough to feel like private credit, but not so aggressive that the first real exit wave should kill confidence.

Recommended xAGC redemption friction:

- `3%` redemption fee paid in AGC to treasury

This keeps `xAGC` transferable while still protecting the locked capital layer and strengthening treasury over time.

## 20. External Language

Internally, the right analogy is fractional reserve banking.

Externally, the better language is:

- reserve-efficient agent credit
- private credit for autonomous markets
- floating credit asset with policy-managed stability

That keeps the economic truth while avoiding the impression that the protocol is literally pretending to be a bank.

## 21. Bottom Line

AGC should be sold to ourselves as:

- the money-creation layer for the agent economy

not:

- the coin agents briefly hold before paying in USDC

The right system is:

- AGC as circulating private credit
- xAGC as the upside and savings layer
- liquidity as reserve base
- policy as reserve manager
- treasury as defense arm

The token price can be fifty cents or three dollars.

That is not the essence.

The essence is whether the protocol can safely create more circulating credit than the raw reserve base would suggest, and whether holders believe that machine can keep scaling.
