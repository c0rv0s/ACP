# AGC Next Build Spec: Agent Credit Control Plane

**Working name:** AGC Credit Control Plane
**Version:** Experiment 002
**Purpose:** Design and ship the next AGC experiment after the hackathon balance-sheet token prototype.
**Core question:** Can policy-constrained agent workflows be underwritten safely enough to create a real credit market?

The next build should not be “AGC mainnet launch.” It should be a focused experiment that produces new knowledge about **agent-native credit**, using small USDC credit lines, agent spend controls, repayment automation, and underwriting telemetry.

The shortest version:

> Build a closed-beta credit control plane where AI-native businesses or agent workflows can receive small USDC credit lines, spend only through policy-controlled rails, repay automatically from revenue inflows, and generate a portable credit history. The experiment is not to prove that a token can be minted; it is to learn what agent activity deserves credit.

This intentionally narrows the hackathon system. Your current AGC repo frames the protocol as “reserve-efficient agent credit” or “private credit for autonomous markets,” with AGC as liquid credit inventory, xAGC as the long-duration expansion layer, and credit facilities as controlled sleeves backed by underwriters and risk gates.  The next build should preserve the **credit-facility insight** but defer the **balance-sheet token** until you have repayment data.

---

# 1. Strategic framing

## 1.1 Why this build exists

AGC’s hackathon prototype explored the maximal version:

```text
policy-managed credit asset
+ reserve-efficient expansion
+ xAGC long-duration capital
+ underwriter-backed credit facilities
+ Solana-native balance-sheet mechanics
```

That was useful as a research instrument, but it bundled too many unsolved problems:

```text
agent demand
+ borrower identity
+ underwriting
+ repayment enforcement
+ default risk
+ token liquidity
+ reserve policy
+ legal classification
+ market manipulation
```

The next experiment should isolate the most important unknown:

> Can an autonomous or semi-autonomous workflow be trusted with bounded spending power if the lender can observe, constrain, and sweep its cashflows?

That is the core of credit in the agentic economy.

## 1.2 The market context

x402 is a useful “why now” primitive because it frames itself as an open standard for internet-native payments that enables agentic payments at scale. Its own site says x402 makes payments possible between clients and servers and “empower[s] agentic payments at scale.” ([x402][1]) Coinbase’s x402 docs describe it as an HTTP-native stablecoin payment protocol for APIs and digital content where both human and machine clients can programmatically pay without accounts or manual payment flows. ([Coinbase Developer Docs][2])

Floe Labs is already validating the adjacent direction. Their site positions Floe as “The Financial OS for AI Agent Developers,” offering x402 credit lines, programmable spend controls, per-call vendor payments, and no required crypto knowledge. ([Floe Labs][3]) They describe secured working capital, unsecured working capital underwritten against receivables and proprietary signals, an x402 payment facilitator, and a credit/trust bureau. ([Floe Labs][3]) They also describe automatic repayment through a revenue lien on inbound payments and portable credit signals. ([Floe Labs][3])

So the market is real enough that you should move fast, but Floe also clarifies your differentiation.

## 1.3 AGC’s differentiated angle

Do not try to beat Floe by building the same “agent financial OS” faster.

AGC’s wedge should be:

> **The underwriting and credit-market layer for autonomous workflows, starting on Solana.**

Floe appears to be going full-stack: wallet, fiat ramps, x402 payment proxy, credit lines, trust bureau, SDKs, and institutional capital. AGC should go narrower and deeper:

```text
Floe: agent financial OS
AGC: agent credit underwriting protocol / credit control plane
```

The key distinction:

```text
Payments product:
"Your agent can pay."

Credit control plane:
"Your agent can borrow under constraints, generate repayment data, and become underwritable."
```

---

# 2. Product thesis

## 2.1 Core thesis

AI-native businesses will increasingly operate through software workflows that spend money, consume APIs, buy compute, purchase data, and generate revenue.

Those workflows need credit, but they should not be treated like ordinary anonymous crypto borrowers.

They are potentially more underwritable than humans because they can be:

```text
instrumented
constrained
rate-limited
merchant-limited
revenue-swept
paused
audited
scored continuously
```

AGC’s next build should test whether these properties make undercollateralized or partially collateralized credit viable.

## 2.2 The primitive

The primitive is not a “loan” in the generic sense.

The primitive is a **policy-constrained credit line for an agent workflow**.

A credit line has:

```text
borrower/operator identity
agent identity
workflow purpose
credit limit
spend policy
allowed merchants/tools
repayment source
revenue sweep rule
risk grade
underwriter capital
default procedure
credit history
```

## 2.3 The central experiment

The experiment is:

> Given a workflow with observable spend and revenue, can AGC safely extend small amounts of working capital and recover it through automated repayment while producing useful underwriting data?

This is more valuable than proving another DeFi lending interface can exist.

---

# 3. Scope

## 3.1 In scope for V0

Build a closed-beta system with:

1. **USDC-denominated credit lines**
2. **Agent/workflow registration**
3. **Spend policy engine**
4. **Payment router or mock x402/payment adapter**
5. **Revenue escrow or repayment sweep**
6. **Risk scoring and limit assignment**
7. **Underwriter capital ledger**
8. **Repayment/default event history**
9. **Portable credit record**
10. **Experiment analytics**

## 3.2 Out of scope for V0

Do not build these yet:

```text
public AGC token launch
xAGC yield product
floating credit asset in production
open anonymous borrowing
unsecured lending to unknown wallets
retail consumer credit
large credit lines
permissionless underwriter marketplace
complex tokenized tranches
full fiat on/off-ramp stack
general-purpose x402 competitor
```

## 3.3 Explicit design choice

V0 uses **USDC** as the credit asset.

AGC/xAGC remain in the research roadmap.

Reason:

```text
USDC credit line = tests underwriting
AGC balance-sheet token = tests monetary design + underwriting + liquidity + reserves all at once
```

The next experiment should test one hard thing, not six.

---

# 4. Product overview

## 4.1 User-facing description

For an AI-native business:

> Register an agent workflow, define where it is allowed to spend, receive a small USDC credit line, route spend through AGC, repay automatically from revenue inflows, and build a credit profile that unlocks larger limits.

For a capital provider:

> Fund isolated credit lines or underwriter vaults, receive interest, and view repayment/default data tied to specific workflow policies and risk grades.

For AGC:

> Learn which signals predict repayment in agentic workflows and build the first agent-credit graph.

## 4.2 System diagram

```text
                      ┌────────────────────────┐
                      │  Underwriter Capital   │
                      │  USDC vault / ledger   │
                      └───────────┬────────────┘
                                  │
                                  ▼
┌─────────────┐        ┌────────────────────────┐
│  Operator   │───────▶│  Credit Line Engine    │
│  / Business │        │  limit, APR, tenor     │
└──────┬──────┘        └───────────┬────────────┘
       │                           │
       ▼                           ▼
┌─────────────┐        ┌────────────────────────┐
│ Agent /     │───────▶│  Spend Policy Engine   │
│ Workflow    │        │  caps, allowlists      │
└──────┬──────┘        └───────────┬────────────┘
       │                           │
       ▼                           ▼
┌─────────────┐        ┌────────────────────────┐
│ Payment     │───────▶│  Vendors / APIs /      │
│ Router      │        │  Compute / Tools       │
└──────┬──────┘        └───────────┬────────────┘
       │                           │
       ▼                           ▼
┌─────────────┐        ┌────────────────────────┐
│ Revenue     │◀───────│  Customer / API /      │
│ Inflows     │        │  Marketplace Revenue   │
└──────┬──────┘        └────────────────────────┘
       │
       ▼
┌────────────────────────────────────────────────┐
│ Repayment Sweep + Credit History + Risk Model  │
└────────────────────────────────────────────────┘
```

---

# 5. Personas

## 5.1 Borrower/operator

The borrower is not initially a fully autonomous agent.

The first borrower is:

```text
a human founder
or small company
or AI-native business
or agent developer
operating one or more agent workflows
```

They want:

```text
working capital
API/compute/data spend limits
postpaid usage
less prefunding
automated repayment
proof that their agent is financially reliable
```

## 5.2 Agent/workflow

An agent/workflow is the underwritten object.

Examples:

```text
AI research agent buying data/API calls
lead-generation agent buying enrichment/search tools
trading/research agent buying market data
content-generation agent buying inference and distribution tools
customer-support agent using paid APIs
agent marketplace bot with repeatable revenue
```

Important: the workflow must have a bounded purpose.

Bad:

```text
"General agent that can do anything."
```

Good:

```text
"Lead enrichment workflow that spends up to $50/day on search and data APIs and receives revenue from completed reports."
```

## 5.3 Underwriter/capital provider

In V0 this can be simulated, internal, or invite-only.

They want:

```text
principal protection
known max loss
fixed tenor
clear APR
repayment data
isolated risk
no socialized pool losses
default visibility
```

Floe claims bilateral isolated positions with no shared pools and no socialized risk, funded by institutional capital. ([Floe Labs][3]) AGC should adopt the same seriousness around isolated risk, but use it to generate a deeper underwriting dataset.

## 5.4 Merchant/vendor

A merchant/vendor is any destination where borrowed funds can be spent.

Examples:

```text
x402 API
LLM provider
data provider
compute provider
browser automation service
SaaS endpoint
internal AGC test service
```

In V0, merchant access should be allowlisted.

## 5.5 AGC operator

The AGC operator manages:

```text
borrower approval
risk policy
merchant registry
credit limits
underwriter capital allocation
delinquency/default handling
experiment analysis
```

This is manual at first. The goal is not to automate judgment prematurely; the goal is to learn what judgment should be automated.

---

# 6. Core hypotheses

## H1 — Demand hypothesis

AI-native businesses and agent developers want postpaid or credit-based access to APIs, tools, compute, and data because prefunding every vendor is operationally annoying.

Success signal:

```text
5–10 serious design partners ask to use it
or
developers route real spend through the system despite small limits
```

## H2 — Control hypothesis

Policy constraints reduce loss severity enough to make small undercollateralized or partially collateralized lines viable.

Controls include:

```text
daily spend caps
per-call caps
merchant allowlists
velocity limits
repayment sweeps
human approval thresholds
workflow-specific limits
automatic suspension
```

Success signal:

```text
policy violations are blocked before spend
losses remain below predefined experiment budget
```

## H3 — Underwriting hypothesis

Workflow telemetry predicts repayment better than wallet history alone.

Useful signals may include:

```text
revenue consistency
spend-to-revenue conversion
merchant mix
repayment latency
usage regularity
operator verification
collateral/first-loss deposit
policy violation rate
workflow age
customer concentration
```

Success signal:

```text
risk grades become directionally predictive across cohorts
```

## H4 — Repayment automation hypothesis

A revenue sweep or receivables lien materially improves repayment behavior.

Success signal:

```text
autopay repayment rate > manual repayment rate
time-to-repayment decreases
delinquencies decrease
```

## H5 — Credit graph hypothesis

Every repayment creates reusable credit data that can improve future limits and pricing.

Success signal:

```text
borrowers with good repayment history receive larger limits at lower APR
underwriters trust AGC’s score more over time
```

---

# 7. Product requirements

## 7.1 Must-have V0 requirements

### Borrower registration

The system must register:

```text
operator identity
organization
wallets
agent/workflow
intended use case
expected spend destinations
expected repayment source
```

For real money beta, do not allow anonymous borrowers. Pseudonymous-only borrowing can exist in simulation, but not with live underwriter capital.

### Agent/workflow profile

Each workflow must have:

```text
workflow_id
operator_id
wallet_address
description
allowed_use_case
risk_grade
credit_status
created_at
last_active_at
```

### Credit line

Each credit line must have:

```text
line_id
borrower_id
agent_id
principal_limit_usdc
available_limit_usdc
drawn_principal_usdc
accrued_interest_usdc
apr_bps
origination_fee_bps
tenor_seconds
maturity_at
grace_period_seconds
status
underwriter_id
risk_grade
policy_id
repayment_rule_id
```

### Policy engine

The system must evaluate every spend request before funds move.

Policy dimensions:

```text
max_per_transaction_usdc
max_daily_spend_usdc
max_weekly_spend_usdc
allowed_merchants
blocked_merchants
allowed_categories
max_apr_bps
max_duration_seconds
human_approval_threshold_usdc
revenue_sweep_percent
min_available_limit_after_spend
cooldown_after_policy_violation
```

### Payment router

The payment router must:

```text
receive spend request
identify merchant
quote cost
check credit availability
check policy
authorize or reject
execute payment if approved
record spend event
update utilization
```

V0 can support two adapters:

```text
MockMerchantAdapter
x402Adapter
```

Start with the mock adapter if x402 integration slows you down.

### Revenue/repayment flow

The system must support:

```text
manual repayment
automatic repayment from escrow
automatic repayment from inbound revenue
partial repayment
full repayment
early repayment
late repayment
```

Revenue sweep example:

```text
sweep_percent = 3000 bps
incoming_revenue = 100 USDC
30 USDC goes to repay credit line
70 USDC goes to borrower wallet
```

### Risk score

Each borrower/workflow must have a score snapshot.

Minimum score fields:

```text
score_version
risk_grade
recommended_limit_usdc
recommended_apr_bps
pd_estimate_bps
lgd_estimate_bps
confidence_bps
features_used
created_at
```

### Event ledger

Every important action must produce an event:

```text
agent_registered
policy_created
credit_line_approved
spend_requested
spend_approved
spend_rejected
payment_settled
revenue_received
repayment_swept
manual_repayment
line_delinquent
line_defaulted
line_closed
score_updated
limit_increased
limit_decreased
policy_violation
```

This event ledger is the beginning of the AGC moat.

---

# 8. Non-functional requirements

## 8.1 Safety

The system must never allow:

```text
spend above credit limit
spend above policy limit
spend to blocked merchants
spend after suspension
spend after maturity unless explicitly allowed
underwriter exposure above committed capital
silent policy changes
silent limit increases
```

## 8.2 Observability

Every decision should be explainable.

For every spend approval/rejection:

```text
input request
policy version
line state
risk flags
decision
reason codes
timestamp
```

## 8.3 Auditability

The experiment should be publishable as research.

That means every credit outcome should be reconstructable:

```text
what was the limit?
what was the policy?
what did the agent spend on?
what revenue arrived?
what was repaid?
what became delinquent?
what signals predicted the outcome?
```

## 8.4 Loss caps

The system must have global experiment loss limits.

Example:

```text
max_total_live_capital_at_risk = 5,000 USDC
max_single_borrower_limit = 250 USDC
max_unsecured_exposure_per_borrower = 100 USDC
max_daily_total_spend = 1,000 USDC
max_loss_budget = 500 USDC
```

The exact numbers can change, but V0 must have hard limits.

---

# 9. Credit product design

## 9.1 Product type

V0 product:

```text
short-duration revolving USDC credit line
```

Not a term loan initially.

Why revolving?

Agent workflows need repeated small spend events, not one big loan.

## 9.2 Suggested first terms

For simulation:

```text
limit: 10–1,000 virtual USDC
APR: simulated
tenor: 7–30 days
repayment: manual or automatic
```

For live closed beta:

```text
starter limit: 25–250 USDC
max first-cycle unsecured line: 100 USDC
tenor: 7 days
APR: 10–40% annualized, depending on risk
origination fee: 0–2%
sweep: 20–50% of inbound revenue
grace period: 24–72 hours
```

Do not optimize APR at first. Optimize learning and loss control.

## 9.3 Credit line states

```text
Draft
Applied
ManualReview
Approved
Active
Suspended
Matured
GracePeriod
Delinquent
Defaulted
Repaid
Closed
```

State transitions:

```text
Draft -> Applied
Applied -> ManualReview
ManualReview -> Approved
Approved -> Active
Active -> Suspended
Active -> Matured
Matured -> GracePeriod
GracePeriod -> Delinquent
Delinquent -> Defaulted
Active -> Repaid
Repaid -> Closed
Suspended -> Active
Suspended -> Defaulted
```

## 9.4 Draw/spend model

Do not let borrowers withdraw USDC freely in V0.

Instead, borrowed funds should be spent through the AGC payment router.

This is crucial.

Bad V0:

```text
borrower draws 100 USDC to arbitrary wallet
borrower disappears
```

Good V0:

```text
borrower gets 100 USDC credit limit
agent requests 2.50 USDC API call
AGC checks policy
AGC pays allowlisted vendor
repayment is swept from revenue
```

Credit should be **usable**, not fully withdrawable.

This is the central anti-default insight.

---

# 10. Underwriting model

## 10.1 Basic formula

Use the standard credit frame:

```text
Expected Loss = PD × LGD × EAD
```

Where:

```text
PD  = probability of default
LGD = loss given default
EAD = exposure at default
```

Pricing:

```text
APR = cost_of_capital + expected_loss + servicing_cost + risk_margin
```

For V0, this can be heuristic.

## 10.2 Limit assignment

Initial line limit:

```text
base_limit = min(
  revenue_based_limit,
  spend_based_limit,
  collateral_based_limit,
  risk_budget_limit,
  underwriter_limit,
  global_experiment_limit
)
```

Where:

```text
revenue_based_limit = rolling_7d_verified_revenue * revenue_advance_rate
spend_based_limit = projected_7d_approved_spend
collateral_based_limit = first_loss_deposit / required_first_loss_ratio
risk_budget_limit = max_loss_per_borrower / estimated_lgd
underwriter_limit = underwriter_remaining_capital_for_grade
```

Example:

```text
rolling_7d_verified_revenue = 500 USDC
revenue_advance_rate = 20%
revenue_based_limit = 100 USDC

first_loss_deposit = 25 USDC
required_first_loss_ratio = 25%
collateral_based_limit = 100 USDC

max_loss_per_borrower = 50 USDC
estimated_lgd = 50%
risk_budget_limit = 100 USDC

approved limit = 100 USDC
```

## 10.3 Risk features

### Identity/operator features

```text
operator_verified
business_verified
domain_verified
github_age_days
company_age_days
prior_relationship
manual_review_score
```

### Wallet/onchain features

```text
wallet_age_days
wallet_balance_usdc
wallet_inflow_volume_30d
wallet_outflow_volume_30d
counterparty_diversity
failed_transaction_rate
prior_credit_events
```

### Workflow features

```text
workflow_age_days
workflow_type
merchant_categories
allowed_merchants_count
spend_frequency
spend_volatility
task_completion_rate
human_approval_rate
policy_violation_rate
```

### Revenue features

```text
verified_revenue_7d
verified_revenue_30d
revenue_consistency
revenue_source_diversity
gross_margin_estimate
revenue_to_spend_ratio
inbound_payment_frequency
chargeback_or_reversal_rate
```

### Repayment features

```text
repayment_count
days_since_first_repayment
average_days_early
average_days_late
delinquency_count
default_count
manual_repay_rate
auto_sweep_repay_rate
```

## 10.4 Scorecard

Example scorecard:

```text
Score = 1000
      + identity_points
      + revenue_points
      + repayment_points
      + workflow_points
      + collateral_points
      - concentration_penalty
      - volatility_penalty
      - policy_violation_penalty
      - delinquency_penalty
```

Risk grades:

| Grade |   Score | Max first line |      Advance rate | Suggested APR | Required controls             |
| ----- | ------: | -------------: | ----------------: | ------------: | ----------------------------- |
| A     |    850+ |           $250 | 30% of 7d revenue |        10–15% | sweep + allowlist             |
| B     | 700–849 |           $100 | 20% of 7d revenue |        15–25% | sweep + allowlist + daily cap |
| C     | 550–699 |            $50 | 10% of 7d revenue |        25–40% | first-loss deposit + sweep    |
| D     |    <550 |        $0 live |                0% |           n/a | simulation only               |

Do not pretend this scorecard is mathematically validated. It is a starting policy that becomes empirical over time.

---

# 11. Default prevention and enforcement

## 11.1 Core principle

You cannot stop every borrower from disappearing.

You can design the system so that disappearing is less profitable than repaying, and so that losses are capped.

The V0 stack should be:

```text
1. Do not let borrowers withdraw freely.
2. Spend only through controlled rails.
3. Require verified identity for live credit.
4. Start with tiny limits.
5. Sweep repayment from revenue.
6. Use first-loss deposits when needed.
7. Suspend lines automatically on risk events.
8. Increase limits only after repayment.
```

## 11.2 Anti-disappearance design

### No arbitrary cash-out

Borrowed funds are not sent to the borrower wallet by default.

They are used to pay approved vendors.

### Merchant allowlists

Initial allowed merchants should be:

```text
AGC test merchant
known x402 APIs
compute providers
data providers
tool providers
```

### Revenue lien

Borrower agrees that some inbound revenue is routed through AGC repayment logic.

In code:

```text
if credit_line.outstanding > 0:
    repay_amount = min(incoming_revenue * sweep_bps / 10_000, outstanding)
    transfer repay_amount to credit_line
    transfer remainder to borrower
else:
    transfer full incoming_revenue to borrower
```

### Progressive limits

Limits rise only after successful repayment cycles.

Example:

```text
Cycle 1: $25
Cycle 2: $50
Cycle 3: $100
Cycle 4: $250
Cycle 5: custom/manual review
```

### First-loss borrower deposit

For riskier borrowers:

```text
borrower posts 10–30% first-loss deposit
AGC extends line above deposit
default consumes deposit first
```

This is not pure unsecured lending, but it is a bridge toward undercollateralized credit.

### Human operator liability

For live beta, tie agent credit to a verified operator.

Even if the agent is autonomous, the account owner is responsible.

## 11.3 Default states

A line becomes **delinquent** when:

```text
now > maturity_at
and outstanding_balance > 0
```

A line becomes **defaulted** when:

```text
now > maturity_at + grace_period
and outstanding_balance > 0
```

Or immediately if:

```text
fraud_flag = true
or policy_breach_severe = true
or operator_revokes_repayment_mandate
or revenue lien is bypassed
```

## 11.4 Default handling

On default:

```text
1. suspend further spend
2. freeze remaining available limit
3. consume first-loss deposit if any
4. apply available escrow balance
5. record default event
6. update credit score
7. notify underwriter
8. mark unrecovered amount
9. block new lines until manual review
```

Do not overbuild collections. The first version should be about loss containment and learning.

---

# 12. Payment and spend policy engine

## 12.1 Spend request structure

```json
{
  "agent_id": "agt_123",
  "workflow_id": "wf_456",
  "credit_line_id": "cl_789",
  "merchant_id": "m_x402_search_api",
  "amount_usdc": "2.50",
  "purpose": "lead_enrichment",
  "task_id": "task_abc",
  "metadata": {
    "url": "https://example-api.com/search",
    "method": "POST",
    "category": "data_api"
  }
}
```

## 12.2 Policy check

The policy engine evaluates:

```text
line is active
line not matured
amount <= available limit
amount <= per-transaction cap
daily spend + amount <= daily cap
merchant is allowed
category is allowed
risk flags clear
human approval not required
revenue sweep mandate active
```

Decision output:

```json
{
  "decision": "approved",
  "authorization_id": "auth_123",
  "approved_amount_usdc": "2.50",
  "policy_version": 4,
  "expires_at": "2026-05-21T12:05:00Z",
  "reason_codes": []
}
```

Rejection output:

```json
{
  "decision": "rejected",
  "policy_version": 4,
  "reason_codes": [
    "DAILY_LIMIT_EXCEEDED",
    "MERCHANT_NOT_ALLOWED"
  ]
}
```

## 12.3 Human approval thresholds

Some requests should pause for approval.

Example:

```text
amount > 25 USDC
new merchant
new category
abnormal velocity
risk score recently downgraded
```

The point is not to make agents fully autonomous immediately. The point is to measure where autonomy breaks.

---

# 13. Revenue and repayment system

## 13.1 Revenue sources

Supported V0 revenue sources:

```text
manual USDC deposit
inbound wallet transfer
AGC-hosted revenue endpoint
x402 merchant receipt
Stripe/manual offchain attestation
```

Start with what is easy to instrument.

## 13.2 Repayment waterfall

When funds arrive:

```text
1. fees due
2. accrued interest
3. principal
4. borrower balance
```

Pseudo:

```text
incoming = revenue_amount

fee_paid = min(incoming, fees_due)
incoming -= fee_paid

interest_paid = min(incoming, accrued_interest)
incoming -= interest_paid

principal_paid = min(incoming, principal_outstanding)
incoming -= principal_paid

borrower_receives = incoming
```

## 13.3 Sweep policies

Supported sweep modes:

```text
fixed_percent
fixed_amount_until_repaid
all_revenue_until_repaid
manual_only
hybrid
```

Recommended V0 default:

```text
30% of inbound revenue until current cycle is repaid
```

## 13.4 Repayment events

Each repayment records:

```text
repayment_id
credit_line_id
source_event_id
amount_usdc
principal_paid
interest_paid
fees_paid
remaining_balance
repayment_type
timestamp
```

---

# 14. Underwriter capital model

## 14.1 V0 model

Start with one of three underwriter modes:

### Mode A — simulated capital

No real loans. Used for integration and research.

### Mode B — internal capital

You fund tiny live lines yourself.

### Mode C — invite-only underwriter

One or more trusted capital providers fund isolated lines.

Do not launch a public pool yet.

## 14.2 Isolated risk

Each credit line should be assigned to a specific capital bucket.

Avoid socialized pool risk in V0.

```text
underwriter_commitment_id
credit_line_id
principal_committed
principal_drawn
interest_earned
loss_realized
status
```

## 14.3 Underwriter return

Interest paid by borrower flows to the underwriter minus AGC fee.

Example:

```text
borrower interest paid = 10 USDC
AGC protocol fee = 5% of interest = 0.50 USDC
underwriter receives = 9.50 USDC
```

Floe publicly states a similar pricing model where it takes 5% of interest or financing paid, not a subscription fee. ([Floe Labs][3]) AGC can adopt or test this, but the experiment should track whether the fee is meaningful at small line sizes.

## 14.4 Loss handling

Loss waterfall:

```text
1. borrower first-loss deposit
2. revenue escrow
3. underwriter principal
4. AGC loss reserve, if explicitly provided
```

Do not imply AGC backstops all losses unless it actually does.

---

# 15. Onchain/offchain architecture

## 15.1 Recommended V0 architecture

Use a hybrid architecture.

```text
Offchain:
- policy engine
- risk engine
- x402/payment adapter
- merchant metadata
- experiment analytics
- manual review

Onchain:
- USDC vaults
- credit line state
- repayment events
- underwriter commitments
- borrower/agent attestations
- optional score hash
```

Reason:

```text
HTTP payments and agent workflows are offchain.
USDC custody and repayment should be verifiable.
Underwriting logic will change quickly during the experiment.
```

Do not prematurely freeze the risk model onchain.

## 15.2 Solana program components

### Program 1: CreditLineProgram

Core accounts:

```text
ProtocolConfig
BorrowerProfile
AgentProfile
UnderwriterVault
CreditLine
SpendPolicy
RepaymentEscrow
CreditEventLog
ScoreAttestation
MerchantRegistry
```

Core instructions:

```text
initialize_protocol
register_borrower
register_agent
create_spend_policy
fund_underwriter_vault
approve_credit_line
activate_credit_line
reserve_spend
settle_spend
record_revenue
sweep_repayment
manual_repay
suspend_credit_line
mark_delinquent
mark_default
close_credit_line
update_score_attestation
```

### Program 2: optional PolicyAttestationProgram

This can come later.

Purpose:

```text
write signed policy decisions to chain
hash offchain metadata
make experiment auditable
```

## 15.3 Offchain services

### API service

Handles SDK calls and authentication.

### Policy service

Evaluates every spend request.

### Risk service

Scores borrowers and suggests limits.

### Payment service

Executes approved payments through adapters.

### Event service

Writes structured events to database and optionally onchain.

### Research/analytics service

Computes experiment metrics.

---

# 16. Data model

## 16.1 Borrower

```ts
type Borrower = {
  id: string;
  legalName?: string;
  displayName: string;
  type: "individual" | "company" | "pseudonymous_sandbox";
  verificationStatus: "none" | "email" | "wallet" | "kyc" | "kyb" | "manual";
  primaryWallet: string;
  createdAt: string;
  status: "active" | "suspended" | "blocked";
};
```

## 16.2 Agent

```ts
type Agent = {
  id: string;
  borrowerId: string;
  name: string;
  walletAddress: string;
  framework?: "openai" | "langchain" | "agentkit" | "custom" | "other";
  description: string;
  status: "pending" | "active" | "suspended";
  createdAt: string;
};
```

## 16.3 Workflow

```ts
type Workflow = {
  id: string;
  agentId: string;
  name: string;
  purpose: string;
  expectedSpendCategories: string[];
  expectedRevenueSources: string[];
  policyId: string;
  status: "draft" | "approved" | "active" | "suspended";
};
```

## 16.4 Credit line

```ts
type CreditLine = {
  id: string;
  borrowerId: string;
  agentId: string;
  workflowId: string;
  underwriterId: string;
  principalLimitUsdc: string;
  availableLimitUsdc: string;
  principalOutstandingUsdc: string;
  accruedInterestUsdc: string;
  aprBps: number;
  originationFeeBps: number;
  maturityAt: string;
  gracePeriodSeconds: number;
  policyId: string;
  repaymentRuleId: string;
  scoreSnapshotId: string;
  status:
    | "draft"
    | "approved"
    | "active"
    | "suspended"
    | "matured"
    | "delinquent"
    | "defaulted"
    | "repaid"
    | "closed";
};
```

## 16.5 Spend policy

```ts
type SpendPolicy = {
  id: string;
  version: number;
  maxPerTransactionUsdc: string;
  maxDailySpendUsdc: string;
  maxWeeklySpendUsdc: string;
  allowedMerchantIds: string[];
  blockedMerchantIds: string[];
  allowedCategories: string[];
  humanApprovalThresholdUsdc: string;
  revenueSweepBps: number;
  validFrom: string;
  validTo?: string;
};
```

## 16.6 Spend event

```ts
type SpendEvent = {
  id: string;
  creditLineId: string;
  agentId: string;
  workflowId: string;
  merchantId: string;
  amountUsdc: string;
  status: "requested" | "approved" | "rejected" | "settled" | "failed";
  policyVersion: number;
  reasonCodes: string[];
  authorizationId?: string;
  settlementTx?: string;
  createdAt: string;
};
```

## 16.7 Revenue event

```ts
type RevenueEvent = {
  id: string;
  borrowerId: string;
  agentId: string;
  workflowId?: string;
  amountUsdc: string;
  source: "manual" | "wallet" | "x402" | "stripe_attested" | "test_merchant";
  txHash?: string;
  sweepApplied: boolean;
  createdAt: string;
};
```

## 16.8 Repayment event

```ts
type RepaymentEvent = {
  id: string;
  creditLineId: string;
  amountUsdc: string;
  principalPaidUsdc: string;
  interestPaidUsdc: string;
  feesPaidUsdc: string;
  source: "manual" | "auto_sweep" | "escrow";
  txHash?: string;
  createdAt: string;
};
```

## 16.9 Score snapshot

```ts
type ScoreSnapshot = {
  id: string;
  borrowerId: string;
  agentId: string;
  workflowId?: string;
  scoreVersion: string;
  score: number;
  riskGrade: "A" | "B" | "C" | "D";
  recommendedLimitUsdc: string;
  recommendedAprBps: number;
  pdEstimateBps: number;
  lgdEstimateBps: number;
  confidenceBps: number;
  featureValues: Record<string, number | string | boolean>;
  createdAt: string;
};
```

---

# 17. API spec

## 17.1 Register borrower

`POST /v1/borrowers`

Request:

```json
{
  "displayName": "Acme Agents",
  "type": "company",
  "primaryWallet": "So111...",
  "email": "founder@example.com"
}
```

Response:

```json
{
  "borrowerId": "brw_123",
  "verificationStatus": "email",
  "status": "active"
}
```

## 17.2 Register agent

`POST /v1/agents`

Request:

```json
{
  "borrowerId": "brw_123",
  "name": "Lead Research Agent",
  "walletAddress": "So222...",
  "framework": "custom",
  "description": "Researches leads using paid data APIs and produces reports."
}
```

Response:

```json
{
  "agentId": "agt_123",
  "status": "pending"
}
```

## 17.3 Create workflow

`POST /v1/workflows`

```json
{
  "agentId": "agt_123",
  "name": "Lead enrichment workflow",
  "purpose": "Buy search/data API calls and generate paid lead reports.",
  "expectedSpendCategories": ["search_api", "data_api"],
  "expectedRevenueSources": ["customer_usdc", "manual_invoice"]
}
```

## 17.4 Create spend policy

`POST /v1/policies`

```json
{
  "workflowId": "wf_123",
  "maxPerTransactionUsdc": "5.00",
  "maxDailySpendUsdc": "25.00",
  "maxWeeklySpendUsdc": "100.00",
  "allowedMerchantIds": ["m_search_api", "m_data_api"],
  "allowedCategories": ["search_api", "data_api"],
  "humanApprovalThresholdUsdc": "10.00",
  "revenueSweepBps": 3000
}
```

## 17.5 Apply for credit

`POST /v1/credit-lines/apply`

```json
{
  "borrowerId": "brw_123",
  "agentId": "agt_123",
  "workflowId": "wf_123",
  "requestedLimitUsdc": "100.00",
  "requestedTenorDays": 7,
  "repaymentSource": "revenue_sweep"
}
```

Response:

```json
{
  "applicationId": "app_123",
  "status": "manual_review",
  "preliminaryRiskGrade": "B",
  "recommendedLimitUsdc": "75.00"
}
```

## 17.6 Get credit line

`GET /v1/credit-lines/{creditLineId}`

Response:

```json
{
  "creditLineId": "cl_123",
  "status": "active",
  "principalLimitUsdc": "75.00",
  "availableLimitUsdc": "62.50",
  "principalOutstandingUsdc": "12.50",
  "accruedInterestUsdc": "0.04",
  "aprBps": 1800,
  "maturityAt": "2026-05-28T00:00:00Z"
}
```

## 17.7 Authorize spend

`POST /v1/spend/authorize`

```json
{
  "creditLineId": "cl_123",
  "agentId": "agt_123",
  "workflowId": "wf_123",
  "merchantId": "m_search_api",
  "amountUsdc": "2.50",
  "purpose": "lead_enrichment",
  "taskId": "task_123"
}
```

## 17.8 Execute x402 fetch

`POST /v1/x402/fetch`

```json
{
  "creditLineId": "cl_123",
  "agentId": "agt_123",
  "url": "https://merchant.example/api/search",
  "method": "POST",
  "body": {
    "query": "AI infrastructure companies in New York"
  },
  "maxPaymentUsdc": "2.50"
}
```

Response:

```json
{
  "status": "settled",
  "amountPaidUsdc": "1.20",
  "spendEventId": "spn_123",
  "merchantResponse": {
    "results": []
  }
}
```

## 17.9 Record revenue

`POST /v1/revenue`

```json
{
  "agentId": "agt_123",
  "workflowId": "wf_123",
  "amountUsdc": "50.00",
  "source": "manual",
  "txHash": "optional"
}
```

Response:

```json
{
  "revenueEventId": "rev_123",
  "sweepApplied": true,
  "repaymentAmountUsdc": "15.00",
  "borrowerReceivesUsdc": "35.00"
}
```

## 17.10 Manual repay

`POST /v1/repayments`

```json
{
  "creditLineId": "cl_123",
  "amountUsdc": "20.00"
}
```

---

# 18. SDK spec

## 18.1 TypeScript example

```ts
import { AGC } from "@agc/credit";

const agc = new AGC({
  apiKey: process.env.AGC_API_KEY,
});

const agent = await agc.registerAgent({
  name: "Lead Research Agent",
  walletAddress: "So222...",
  framework: "custom",
});

const workflow = await agc.createWorkflow({
  agentId: agent.id,
  name: "Lead enrichment",
  purpose: "Use paid APIs to produce lead reports",
  expectedSpendCategories: ["search_api", "data_api"],
});

const policy = await agc.createPolicy({
  workflowId: workflow.id,
  maxPerTransactionUsdc: "5.00",
  maxDailySpendUsdc: "25.00",
  allowedMerchantIds: ["m_search_api"],
  revenueSweepBps: 3000,
});

const creditLine = await agc.applyForCredit({
  agentId: agent.id,
  workflowId: workflow.id,
  requestedLimitUsdc: "100.00",
  requestedTenorDays: 7,
});

const response = await agc.x402Fetch({
  creditLineId: creditLine.id,
  agentId: agent.id,
  url: "https://merchant.example/search",
  method: "POST",
  body: { query: "Solana AI startups" },
  maxPaymentUsdc: "2.50",
});
```

## 18.2 Python example

```python
from agc_credit import AGC

agc = AGC(api_key="...")

agent = agc.register_agent(
    name="Lead Research Agent",
    wallet_address="So222...",
    framework="custom",
)

workflow = agc.create_workflow(
    agent_id=agent.id,
    name="Lead enrichment",
    purpose="Use paid APIs to produce lead reports",
    expected_spend_categories=["search_api", "data_api"],
)

line = agc.apply_for_credit(
    agent_id=agent.id,
    workflow_id=workflow.id,
    requested_limit_usdc="100.00",
    requested_tenor_days=7,
)

result = agc.x402_fetch(
    credit_line_id=line.id,
    agent_id=agent.id,
    url="https://merchant.example/search",
    method="POST",
    body={"query": "Solana AI startups"},
    max_payment_usdc="2.50",
)
```

---

# 19. Experiment design

## 19.1 Phase 0 — offline simulation

Goal:

```text
test policies and scoring without real money
```

Build:

```text
synthetic agents
synthetic merchants
synthetic revenue
synthetic defaults
risk model simulator
```

Output:

```text
policy sensitivity report
loss simulation
limit progression model
```

## 19.2 Phase 1 — sandbox with real developers, no real credit

Goal:

```text
observe agent spend behavior without lending real money
```

Mechanic:

```text
developers use AGC payment router
AGC gives virtual credit
all spend is simulated or prepaid
```

Output:

```text
spend telemetry
policy violation data
developer demand
workflow taxonomy
```

## 19.3 Phase 2 — closed beta with tiny live USDC lines

Goal:

```text
test real repayment behavior with capped losses
```

Parameters:

```text
5–10 borrowers
$25–$250 credit lines
7-day tenor
allowlisted merchants
manual approval
operator verification
revenue sweep required
global loss budget <= $500
```

Output:

```text
first repayment/default dataset
validated risk features
customer feedback
capital provider memo
```

## 19.4 Phase 3 — invite-only underwriter capital

Goal:

```text
test whether capital providers trust AGC underwriting
```

Parameters:

```text
1–3 underwriters
isolated commitments
fixed risk grades
no public pool
transparent loss reporting
```

Output:

```text
underwriter term sheet
risk-adjusted yield data
capital allocation model
```

---

# 20. Metrics

## 20.1 Borrower demand

```text
number of design partners
number of registered workflows
number of active workflows
credit applications submitted
credit applications approved
activation rate
```

## 20.2 Usage

```text
spend requests
approved spend
rejected spend
policy violations
average transaction size
merchant distribution
daily utilization
weekly utilization
```

## 20.3 Credit performance

```text
principal originated
principal outstanding
repayment rate
automatic repayment share
manual repayment share
average time to repayment
delinquency rate
default rate
loss rate
recovery rate
```

## 20.4 Risk model quality

```text
score distribution
grade migration
predicted vs actual repayment
false approvals
false rejections
policy-violation correlation
revenue-sweep effectiveness
```

## 20.5 Business model

```text
interest paid
AGC fee revenue
underwriter yield
servicing cost
gross margin
cost per borrower
cost per credit line
```

## 20.6 Research outputs

```text
agent workflow taxonomy
underwriting feature importance
repayment behavior patterns
default case studies
policy design recommendations
whitepaper sections validated
```

---

# 21. Acceptance criteria

## 21.1 MVP acceptance

The MVP is done when:

```text
a borrower can register an agent/workflow
a policy can be attached to the workflow
a credit line can be approved
an agent can request spend
the policy engine can approve/reject spend
approved spend can settle through a mock or real adapter
revenue can be recorded
repayment can be swept automatically
a credit history updates after repayment
```

## 21.2 Experiment success

The experiment is successful if it answers:

```text
Who wants this?
What do they spend on?
What repayment source is credible?
Which policies prevent bad behavior?
What signals predict repayment?
What losses occur even with controls?
Would anyone fund this besides AGC?
```

## 21.3 Investor-facing success

For CyberFund/Colosseum, the successful result is not:

```text
"I shipped more code."
```

It is:

```text
"I created the first structured dataset of agent workflow credit behavior and used it to design the next version of autonomous credit markets."
```

---

# 22. Implementation roadmap

## Week 1 — Research-to-spec freeze

Deliverables:

```text
final product spec
risk policy v0
data schema
experiment plan
design partner target list
```

Decision:

```text
mock payments first or x402 adapter first
```

Recommendation:

```text
mock payments first, x402 adapter second
```

## Week 2 — Event ledger + workflow registry

Build:

```text
borrower registration
agent registration
workflow registration
merchant registry
event schema
database
basic API
```

Output:

```text
AGC knows who/what it is underwriting
```

## Week 3 — Policy engine

Build:

```text
spend policy creation
policy evaluation
approval/rejection reason codes
daily/weekly limits
merchant allowlist
manual approval thresholds
```

Output:

```text
AGC can block bad spend before funds move
```

## Week 4 — Credit line engine

Build:

```text
credit application
manual approval
line states
limit tracking
APR/fee accrual
maturity/grace period
suspension
```

Output:

```text
AGC can issue bounded credit lines
```

## Week 5 — Payment adapter

Build:

```text
mock merchant adapter
payment settlement event
optional x402 fetch adapter
spend receipts
```

Output:

```text
agents can use credit to buy something measurable
```

## Week 6 — Revenue and repayment

Build:

```text
revenue recording
repayment waterfall
revenue sweep
manual repay
line closure
```

Output:

```text
AGC can observe repayment behavior
```

## Week 7 — Risk score v0

Build:

```text
score snapshots
limit recommendation
risk grades
feature extraction
score history
```

Output:

```text
AGC starts forming the agent-credit graph
```

## Week 8 — Closed sandbox

Run:

```text
3–5 internal/sandbox workflows
virtual or tiny credit
mock and/or real API spend
manual review of all approvals
```

Output:

```text
first experiment report
```

## Weeks 9–10 — Live closed beta

Run:

```text
5–10 external design partners
tiny USDC lines
strict limits
operator verification
revenue sweep required
```

Output:

```text
real repayment/default data
```

## Weeks 11–12 — Whitepaper and accelerator demo

Deliver:

```text
agent credit whitepaper v0
experiment results memo
underwriting model v1
live demo
investor deck
capital provider memo
```

---

# 23. Relationship to existing AGC prototype

## 23.1 Reuse

Keep these ideas from the hackathon build:

```text
controlled credit facilities
underwriter first-loss logic
debt caps
interest accrual
repayment waterfall
default states
pause/suspend controls
risk-admin configuration
simulation mindset
```

The AGC context already describes credit facilities as controlled credit sleeves, with underwriters depositing into facility-specific vaults, earning interest, and acting as first-loss capital on default. 

## 23.2 Defer

Defer these:

```text
AGC as draw asset
xAGC as expansion layer
policy-managed minting
floating anchor
treasury buybacks
reserve-efficiency ratio
public liquidity
```

## 23.3 Translate

Translate the hackathon mechanics like this:

| Hackathon AGC               | Next experiment                             |
| --------------------------- | ------------------------------------------- |
| AGC minted to borrower      | USDC spent through controlled line          |
| xAGC long-duration capital  | underwriter commitment / first-loss capital |
| policy expansion controller | credit policy and risk engine               |
| price/reserve defense       | exposure limits and automatic suspension    |
| collateralized BTC facility | revenue-swept workflow credit line          |
| protocol telemetry          | agent credit event ledger                   |

---

# 24. Security and adversarial cases

## 24.1 Borrower takes money and disappears

Mitigation:

```text
no free withdrawal
allowlisted spend only
tiny starter lines
verified operator
revenue sweep
progressive limits
first-loss deposit
```

## 24.2 Fake revenue

Mitigation:

```text
only count verified revenue sources
ignore self-funded circular payments
detect related wallets
score revenue diversity
manual review large increases
```

## 24.3 Merchant collusion

Borrower creates fake merchant and pays themselves.

Mitigation:

```text
merchant allowlist
merchant verification
category limits
no arbitrary URL spend for live credit
delayed merchant onboarding
```

## 24.4 Agent prompt injection causes spend

Mitigation:

```text
policy engine outside agent context
hard spend caps
human approval above threshold
merchant allowlist
task-specific budgets
```

## 24.5 Wash repayment

Borrower borrows, repays with own funds, builds fake score.

Mitigation:

```text
separate repayment score from revenue score
score verified operating revenue higher
cap limit increases from self-funded repayment
look for circular flows
```

## 24.6 Underwriter drain

Mitigation:

```text
isolated commitments
max exposure per line
explicit underwriter approval
no socialized losses
withdrawal restrictions only for committed capital
```

## 24.7 Policy tampering

Mitigation:

```text
policy versioning
signed policy updates
audit log
no silent limit increases
borrower-visible policy state
underwriter-visible policy state
```

---

# 25. Legal/compliance posture

For live credit, assume this needs legal review.

Key issues:

```text
business lending vs consumer lending
KYC/KYB
money transmission
AML/sanctions screening
usury limits
loan disclosure
data privacy
credit reporting rules
securities implications for underwriter capital
```

Recommended V0 posture:

```text
closed beta
business users only
small limits
manual approvals
no public lending pool
no retail investor underwriters
no consumer credit
no promise of yield to the public
clear experimental terms
```

Do not frame this as “anonymous DeFi undercollateralized lending.”

Frame it as:

```text
controlled working capital for verified AI-native businesses and workflows
```

---

# 26. What makes this new knowledge?

This experiment creates new knowledge if it produces answers to questions like:

```text
Can workflow-level telemetry predict repayment?
Which agent spend categories create repayable value?
Are revenue sweeps enough to reduce default?
How much do policy controls reduce loss severity?
Do developers actually want credit, or just easier payments?
Do underwriters care about portable agent credit records?
What is the minimum viable identity requirement?
Can credit limits increase algorithmically after repayment?
```

That is the path to a whitepaper.

The whitepaper should not start with:

```text
"Here is a token."
```

It should start with:

```text
"Here is what we learned from the first controlled credit experiment for autonomous workflows."
```

---

# 27. Recommended first experiment configuration

I would make the first live version extremely small and controlled.

```text
Name: AGC Credit Sandbox
Asset: USDC
Borrowers: 5–10 verified AI-native builders
Line size: $25–$250
Tenor: 7 days
Repayment: 30% revenue sweep + manual repay
Spend: allowlisted merchants only
Capital: internal or invite-only
Loss budget: $500 max
Main output: agent credit dataset + underwriting memo
```

Example first workflow:

```text
Workflow:
Lead enrichment agent

Spend:
Search API, data API, LLM inference

Revenue:
Customer pays per completed lead report

Credit line:
$100 for 7 days

Policy:
$5 max transaction
$25 max daily spend
allowed merchants only
30% revenue sweep

Success:
agent spends $60
generates $200 revenue
repays automatically
limit increases to $150
```

That is a real credit loop.

---

# 28. The spec in one sentence

> Build AGC Credit Sandbox: a closed-beta USDC credit control plane where verified AI-native businesses register agent workflows, receive tiny policy-constrained credit lines, spend only through approved rails, repay automatically from revenue, and generate the first underwriting dataset for autonomous credit markets.

[1]: https://www.x402.org/ "x402 - Payment Required | Internet-Native Payments Standard"
[2]: https://docs.cdp.coinbase.com/x402/welcome "Welcome to x402 - Coinbase Developer Documentation"
[3]: https://www.floelabs.xyz/ "Credit and payments for AI agent developers."
