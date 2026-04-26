export type ProtocolDocPage = {
  id: string;
  title: string;
  summary: string;
  visual: "flywheel" | "reserves" | "defense" | "vault" | "governance" | "integration";
  body: string[];
  bullets?: string[];
  example?: {
    title: string;
    lines: string[];
  };
  technical?: string[];
};

export type ProtocolDocGroup = {
  title: string;
  pages: ProtocolDocPage[];
};

export const protocolDocGroups: ProtocolDocGroup[] = [
  {
    title: "Start Here",
    pages: [
      {
        id: "overview",
        title: "Credit inventory for machine economies",
        summary:
          "AGC is spendable working capital for agents, apps, and automated markets: liquid like a token, expandable like credit, and constrained by balance-sheet proof.",
        visual: "flywheel",
        body: [
          "AGC is money-like inventory for autonomous activity. Agents need to pay for compute, APIs, data, execution, inventory, advertising, and other services. Fully reserved dollars settle payments, but they do not create native credit velocity.",
          "The protocol creates additional purchasing power only when the balance sheet can carry it. AGC is not a fully backed stablecoin; it is a managed credit asset with reserves, collateral, mint caps, and defense controls.",
        ],
        bullets: [
          "AGC is liquid credit inventory.",
          "xAGC owns the long-duration expansion layer.",
          "Stablecoins provide defense cash.",
          "BTC and isolated RWAs add capacity only through haircuts.",
        ],
      },
      {
        id: "problem",
        title: "The problem AGC solves",
        summary:
          "Crypto has liquidity and speculation, but it does not have a good native credit unit for agent-driven economic activity.",
        visual: "reserves",
        body: [
          "Most onchain assets are either volatile collateral, fully reserved stablecoins, or governance tokens. Agents need something different: working capital that can circulate, compound, and expand with the network.",
          "A normal lending market rents out money that already exists. AGC is different: when reserves, collateral, and credit quality improve, the system can create more usable credit against that stronger base.",
        ],
        example: {
          title: "Plain English",
          lines: [
            "A lending market rents existing money.",
            "AGC creates earned credit.",
            "The machine gets bigger when the balance sheet gets stronger.",
          ],
        },
      },
      {
        id: "growth",
        title: "Why holders care",
        summary:
          "The upside is ownership of future credit capacity. The bet is that AGC becomes a larger credit network and xAGC captures much of that growth.",
        visual: "flywheel",
        body: [
          "The holder thesis is simple: if AGC becomes useful credit inventory, the network needs more of it. Demand can deepen reserves, stronger reserves can unlock more credit capacity, and higher-quality credit activity can feed value back into the system.",
          "xAGC is the long-duration version of that bet. It receives most expansion flow and is the natural place for revenue to accrue over time, so holders own the credit machine rather than only the liquid inventory.",
        ],
        bullets: [
          "AGC holders own scarce credit inventory.",
          "xAGC holders own the expansion layer.",
          "Underwriters earn spread by backing credit pools.",
          "Scarcity comes from epoch caps, daily caps, reserve requirements, and credit access tiers.",
        ],
      },
      {
        id: "defense",
        title: "Why this is not rug-prone",
        summary:
          "The protocol is built so growth depends on rules and reserves, not on unlimited discretion from one wallet.",
        visual: "defense",
        body: [
          "The trust model starts from a conservative assumption: no single key deserves unlimited control over a credit system. Administration, risk changes, and emergency response are separated so each authority has a narrower job.",
          "The most dangerous actions are boxed in by code-level limits. Governance can tune the machine, pause risk, migrate control, and update collateral settings, but it does not get an unlimited faucet.",
        ],
        bullets: [
          "Emergency authority pauses risky surfaces; it cannot mint freely.",
          "Risk authority changes parameters only inside hard-coded bounds.",
          "Admin control migrates through a two-step handoff instead of a silent key swap.",
          "AGC and xAGC mints reject external freeze authorities at initialization.",
        ],
      },
    ],
  },
  {
    title: "Protocol Flows",
    pages: [
      {
        id: "flow-reserves",
        title: "How reserves work",
        summary:
          "USDC, USDT, BTC wrappers, and future RWAs sit in different risk buckets. They do not count equally.",
        visual: "reserves",
        body: [
          "The reserve model is intentionally uneven. A dollar stablecoin, a wrapped BTC mint, and a tokenized stock do not give the machine the same kind of protection, so they do not receive the same treatment.",
          "USDC and USDT are the cash layer for defense, buybacks, liquidity support, and the main AGC/USDC market. BTC wrappers are strategic reserves with upside, but they receive haircuts because they are volatile and carry wrapper-specific risk.",
          "RWAs and tokenized stocks start isolated. They become global reserve collateral only after issuer, legal, oracle, liquidity, and market-hours risks are proven manageable.",
        ],
        technical: [
          "Every supported mint has its own collateral account, oracle feed, reserve weight, collateral factor, liquidation threshold, concentration cap, staleness limit, and confidence limit.",
          "Stable cash coverage is tracked separately from total risk-weighted reserve coverage, so the protocol can distinguish immediate defense cash from longer-term collateral strength.",
          "This is the main difference between a simple treasury balance and a credit balance sheet.",
        ],
      },
      {
        id: "flow-expansion",
        title: "How expansion works",
        summary:
          "AGC expansion is gated by multiple independent limits, so one strong metric cannot carry the whole system.",
        visual: "flywheel",
        body: [
          "AGC grows only when several independent checks agree. The system looks at reserve coverage, stable cash, liquidity depth, oracle health, market conditions, and xAGC lock flow before allowing new supply.",
          "This matters because price alone is not enough. A token can pump for a day and still be unsafe to expand. AGC treats price as a demand signal, while reserves and collateral decide whether new credit is earned.",
        ],
        example: {
          title: "Expansion example",
          lines: [
            "1. AGC trades above the anchor for several epochs.",
            "2. Stable cash and BTC-backed reserve coverage are above target.",
            "3. Liquidity depth is high and exit pressure is low.",
            "4. The epoch cap allows a limited mint.",
            "5. Most new AGC flows to xAGC, with smaller allocations to growth, liquidity, integrations, and treasury.",
          ],
        },
      },
      {
        id: "flow-defense",
        title: "How defense works",
        summary:
          "Defense is what happens when the system stops printing and protects the remaining balance sheet.",
        visual: "defense",
        body: [
          "Defense is the part of the system that matters most when conditions get ugly. If price falls below the stressed floor, stable cash gets too low, reserve coverage weakens, oracle data goes stale, volatility spikes, or exit pressure rises, expansion turns off.",
          "Once defense is active, the protocol protects the balance sheet first. It can queue buybacks, burn treasury AGC, pause risky surfaces, and reduce risk limits until the system earns its way back to a healthier posture.",
        ],
        bullets: [
          "No expansion minting during defense.",
          "Buybacks execute as atomic swap-and-burn through an approved executor.",
          "Collateral can be disabled or haircut further.",
          "Recovery requires a cooldown before expansion can resume.",
        ],
      },
      {
        id: "flow-xagc",
        title: "How xAGC works",
        summary:
          "xAGC is the savings and upside layer for holders who want exposure to the growth machine rather than only liquid AGC inventory.",
        visual: "vault",
        body: [
          "Users deposit AGC and receive xAGC shares. The vault holds AGC. When policy mints to xAGC, the amount of AGC backing each xAGC share rises.",
          "That keeps holder balances simple. xAGC does not rebase; its exchange rate grows as the expansion layer receives more AGC.",
        ],
        example: {
          title: "Vault example",
          lines: [
            "Alice deposits 1,000 AGC and receives xAGC shares.",
            "Later, expansion sends more AGC into the vault.",
            "Alice owns the same number of shares, but each share claims more AGC.",
            "If she exits, an exit fee routes AGC back to treasury.",
          ],
        },
      },
      {
        id: "flow-credit",
        title: "How credit facilities work",
        summary:
          "Credit facilities mint AGC against collateral, underwriter reserve, and explicit debt limits.",
        visual: "vault",
        body: [
          "A credit facility is a controlled credit sleeve. It has one collateral mint, one collateral reserve, one AGC first-loss reserve, debt caps, health thresholds, oracle limits, and pause controls.",
          "Borrowers do not receive AGC because a market is excited. They receive AGC when a credit line has approved collateral, a fresh oracle price, enough underwriter reserve behind it, and room inside the facility's debt caps.",
        ],
        example: {
          title: "Credit line flow",
          lines: [
            "1. Risk governance opens a facility for a supported collateral mint.",
            "2. Underwriters deposit AGC into the first-loss reserve.",
            "3. A borrower deposits collateral and draws AGC inside the limit.",
            "4. Principal repayment burns AGC.",
            "5. Interest repayment flows to underwriters.",
            "6. Default burns underwriter reserve and moves seized collateral to the configured reserve account.",
          ],
        },
      },
    ],
  },
  {
    title: "Governance & Risk",
    pages: [
      {
        id: "governance",
        title: "Who can change parameters",
        summary:
          "Governance is structured for a live credit protocol: fast enough to reduce risk, constrained enough to avoid arbitrary control.",
        visual: "governance",
        body: [
          "AGC governance is designed around accountable multisig control, not slow token voting and not a single founder wallet. The protocol uses separate authority lanes so the same key does not control upgrades, risk settings, emergency pauses, and day-to-day operations.",
          "The practical result is simple: risk-reducing actions can happen quickly, while risk-increasing actions are constrained by process and by code. A holder can inspect who controls each lane, what each lane can do, and what the program refuses to allow.",
        ],
        bullets: [
          "High-level control sits behind Solana multisigs rather than one hot wallet.",
          "Emergency powers pause risk; they do not create new supply.",
          "Risk governance tunes collateral and expansion settings only inside hard limits.",
          "Risk-increasing changes are operationally slower than risk-reducing changes.",
        ],
      },
      {
        id: "risk-parameters",
        title: "Risk parameters",
        summary:
          "The most important parameters are the ones that decide how much credit can exist against the current balance sheet.",
        visual: "defense",
        body: [
          "Risk parameters are the public rulebook for how aggressive the credit machine can be. Reserve weights say how much of an asset counts toward backing. Collateral factors say how much credit an asset can support. Concentration caps stop one asset from dominating the reserve story.",
          "Oracle staleness and confidence limits keep expansion from relying on bad prices. For a holder, these settings are where the protocol's growth story meets its safety story.",
        ],
        technical: [
          "Stable cash coverage shows how much immediate defense capacity exists.",
          "Risk-weighted reserve coverage shows broader balance-sheet strength after haircuts.",
          "Liquidity-depth coverage measures whether markets can absorb exits without disorder.",
          "Daily and epoch mint caps remain absolute supply limits even when every other signal is healthy.",
        ],
      },
    ],
  },
  {
    title: "Technical Diligence",
    pages: [
      {
        id: "integration-solana",
        title: "Solana program accounts",
        summary:
          "The Solana program is built around PDA-owned mints and vaults, plus registry accounts for collateral assets.",
        visual: "integration",
        body: [
          "Under the hood, the Solana program uses PDAs for the assets that matter most. Mint authority, treasury movement, and xAGC vault accounting live under program rules instead of depending on an external private key.",
          "Collateral is configured per mint. That means USDC, USDT, each BTC wrapper, and each future RWA can carry its own risk settings instead of being blended into one undifferentiated collateral bucket.",
        ],
        technical: [
          "ProtocolState stores authority lanes, mint addresses, vault addresses, policy params, pause flags, regime state, and last epoch telemetry.",
          "Keeper accounts have scoped permissions for market reporting, oracle reporting, settlement, buyback execution, treasury burns, and credit operations.",
          "CollateralAsset accounts are derived from the collateral mint, making the reserve registry explicit and inspectable.",
          "CreditFacility accounts own collateral vaults and underwriter AGC vaults through facility PDAs.",
          "CreditLine accounts track borrower limits, principal debt, accrued interest, collateral, maturity, default state, and seized collateral.",
        ],
      },
      {
        id: "integration-oracles",
        title: "Oracle and reserve data",
        summary:
          "Production reserve metrics come from verified oracle and vault integrations.",
        visual: "integration",
        body: [
          "The protocol does not treat a keeper's spreadsheet as the balance sheet. Reserve values come from actual token accounts and verified price feeds, with freshness and confidence checks before they count toward expansion.",
          "Market-volume adapters can still provide useful venue telemetry, but they are not the source of truth for global demand. On Solana, users can trade through wallets, aggregators, and bots, so AGC policy is anchored in reserves, liquidity, oracles, and credit quality.",
        ],
        bullets: [
          "Stale oracle prices do not count toward expansion.",
          "One wrapped BTC or RWA mint cannot dominate the reserve base.",
          "Adapter-reported volume is telemetry, not global truth.",
          "Buybacks swap and burn atomically through approved executors.",
        ],
      },
    ],
  },
];

export const flatProtocolDocs = protocolDocGroups.flatMap((group) => group.pages);

export const docsUpdatedAt = "2026-04-26";
