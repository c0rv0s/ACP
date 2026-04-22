import {
  Fragment,
  useEffect,
  useState,
  type CSSProperties,
} from "react";
import {
  formatUnits,
  keccak256,
  parseUnits,
  stringToHex,
  type Hex,
} from "viem";
import {
  useAccount,
  useConnect,
  useDisconnect,
  usePublicClient,
  useReadContract,
  useWriteContract,
} from "wagmi";
import {
  addresses,
  agcAbi,
  hookAbi,
  policyControllerAbi,
  erc20Abi,
  settlementRouterAbi,
  stabilityVaultAbi,
  xagcVaultAbi,
} from "./contracts";

const regimeLabels = ["Neutral", "Expansion", "Defense", "Recovery"] as const;
const regimeKeys = ["neutral", "expansion", "defense", "recovery"] as const;

const marqueeItems = [
  "FLOATING CREDIT",
  "SOFT ANCHOR",
  "NO REDEMPTION FICTION",
  "MACHINE GDP",
  "UNISWAP v4 HOOK",
  "xAGC SAVINGS LAYER",
  "WORKING CAPITAL FOR AGENTS",
];

const heroAscii = String.raw`
      ___    ______   ______      AGC / AGENT CREDIT
     /   |  / ____/  / ____/      HOLD -> TRADE -> LOCK
    / /| | / / __   / /          FLOATING CREDIT / SOFT ANCHOR
   / ___ |/ /_/ /  / /___        NOT A STABLECOIN
  /_/  |_|\____/   \____/         NO DOLLAR CLAIM

      [ AGENTS ] => [ AGC / USDC POOL ] => [ xAGC + TREASURY ]
`;

const heroStickers = [
  "NO REDEMPTION FICTION",
  "xAGC SAVINGS LAYER",
  "SOFT ANCHOR / HARD TOLLS",
  "MACHINE GDP FEVER",
  "HOOK SEES EVERYTHING",
  "DEMAND WITHOUT WHITELISTS",
  "USDC ONLY AT THE EDGE",
  "CREDIT WITH ATTITUDE",
  "ANTI-BANK-RUN POSTURE",
  "WORKING CAPITAL MAXXED",
  "EXIT FEE TO TREASURY",
  "BUYBACK THE PANIC",
  "TWAP DREAM LOGIC",
  "FLOAT FIRST / LOCK LATER",
  "TREASURY DEFENDS THE RANGE",
];

const heroStickersTop = heroStickers.slice(0, 8);
const heroStickersBottom = heroStickers.slice(8);

const manifestoCards = [
  {
    kicker: "The problem",
    title: "Agents need working capital, not just a warehouse receipt for bank dollars.",
    text:
      "Stablecoins settle payments well, but they do not expand with machine demand and they do not reward the activity that makes autonomous commerce useful. The result is a static monetary base for a dynamic software economy.",
  },
  {
    kicker: "The bet",
    title: "Short-horizon utility matters more than a legal redemption promise.",
    text:
      "Agents do not need every unit to be redeemable at a permanent dollar floor. They need low enough volatility, reliable exit liquidity, and incentives that make holding transaction inventory rational over the next few hours or days.",
  },
  {
    kicker: "The machine",
    title: "The market venue is also the policy surface.",
    text:
      "AGC lives inside a canonical AGC/USDC Uniswap v4 pool with a dedicated hook. Fees, buy and sell flow, stress tolls, and oracle observations all happen at the venue where the currency actually clears.",
  },
  {
    kicker: "The point",
    title: "USDC is the reserve rail. AGC is the elastic transaction inventory.",
    text:
      "Agents can hold AGC as working capital and lock AGC into xAGC to own the expansion path. That lets the protocol create machine-native purchasing power without pretending every unit is a dollar IOU.",
  },
] as const;

const explainerSections = [
  {
    kicker: "What it is",
    title: "AGC is a floating transaction-credit unit optimized for autonomous commerce.",
    body:
      "The protocol does not try to hide behind collateral theater. AGC is not a bank deposit, not a treasury wrapper, and not a promise of offchain redemption. It is a policy-managed credit instrument that tries to stay usable enough for machine holding periods while routing final payment into USDC when that actually matters.",
    detail:
      "That framing changes the design space. You stop asking whether every token is backed by a matching dollar and start asking whether the network can keep volatility, conversion costs, and exit depth inside tolerable bounds for agents that buy compute, APIs, bandwidth, or execution.",
    bullets: [
      "Fixed-balance ERC-20, no rebases",
      "Soft anchor to USDC, not a hard peg",
      "Supply expands and contracts through policy, buybacks, fees, and vault flow",
    ],
    ascii: String.raw`
+------------------------------+
| AGC = FLOATING CREDIT UNIT   |
| USDC = LAST-MILE SETTLEMENT  |
+------------------------------+
`,
  },
  {
    kicker: "The problem it solves",
    title: "Autonomous buyers need liquid inventory between tasks, but stablecoin supply is static and economically inert.",
    body:
      "If agents can only hold inert stablecoins, the machine economy inherits a hard ceiling from the existing stock of external dollars. The protocol's core claim is that productive machine demand should be able to mint new working capital when the system is healthy, then shrink when stress rises, rather than forcing every expansion phase to wait for fresh outside balance sheet capacity.",
    detail:
      "That makes AGC a monetary network, not just a payment token. The hook, the treasury, and the issuance logic are there to convert actual usage into controlled elastic credit, while still preserving a clean escape hatch into USDC at payment time.",
    bullets: [
      "Agents want a transaction buffer",
      "Merchants want stablecoin settlement",
      "The protocol bridges those needs without claiming 1:1 redemption",
    ],
    ascii: String.raw`
STATIC DOLLAR INVENTORY  !=  ELASTIC MACHINE DEMAND
AGC tries to fill the gap without pretending to be cash.
`,
  },
  {
    kicker: "How it works",
    title: "The Uniswap v4 hook is the fast path; the policy controller is the slow path.",
    body:
      "On the fast path, the hook classifies flows, adjusts LP fees, charges hook fees, updates epoch counters, and penalizes short-lived liquidity. On the slow path, the controller settles epochs, enforces mint and buyback caps, sets regime state, routes expansion across xAGC, growth, LP, integrator, and treasury buckets, then queues defense buybacks for separate execution.",
    detail:
      "The launch architecture keeps that split intentionally conservative. The hook accumulates market data. The controller validates bounded policy actions against hard guardrails. The monetary loop stays disciplined on purpose so the network can expand only when generic demand is actually showing up in the tape.",
    bullets: [
      "beforeSwap / afterSwap dynamic fee surface",
      "Oracle-style epoch snapshots from hook data",
      "Bounded mint, buyback, and cooldown logic",
    ],
    ascii: String.raw`
swap -> classify -> fee -> observe -> accumulate
epoch -> settle -> mint to xAGC | queue buyback -> execute (chunked)
`,
  },
  {
    kicker: "Why agents hold it",
    title: "AGC is for liquid inventory. xAGC is for owning the credit machine.",
    body:
      "USDC is an inert reserve asset. AGC is designed to be active monetary inventory. xAGC is the locked savings layer that collects protocol expansion via a rising AGC-per-share exchange rate. The result is one reason to hold AGC for execution and another to lock AGC for upside.",
    detail:
      "The protocol only works if that economics is real. If AGC is not useful inventory and xAGC is not attractive savings, users will just sit in USDC. The entire design is a bet that reserve-efficient credit plus a strong upside layer can make holding rational without promising redemption.",
    bullets: [
      "Hold AGC as working capital for markets and autonomous execution",
      "Lock AGC into xAGC to capture expansion over time",
      "Exit through the canonical AGC/USDC pool when you need liquidity",
    ],
    ascii: String.raw`
HOLD AGC      -> liquid inventory
LOCK xAGC     -> rising claim on expansion
`,
  },
  {
    kicker: "Risk controls",
    title: "When stress rises, the system is supposed to get mean, not pretend everything is fine.",
    body:
      "Defense mode is the anti-bank-run posture. Issuance stops. Exit fees rise. Treasury USDC is earmarked for buybacks (queued on epoch settlement, executed in separate swaps). The band can widen. Mercenary flow becomes more expensive. The point is not to freeze users; it is to preserve utility long enough that the currency can survive reflexive sell pressure.",
    detail:
      "This is why the protocol narrative has to stay honest. There is no redemption guarantee hiding under the hood. The safety story is dynamic policy, disciplined caps, durable liquidity incentives, and a treasury that can spend into disorder.",
    bullets: [
      "No new growth mint while weak",
      "Defense buybacks: queued on settle, executed via router in chunks with sqrt price limits",
      "Exit fee on xAGC redemptions feeds treasury dry powder",
    ],
    ascii: String.raw`
if price weak or exits spike:
  stop mint
  raise tolls
  buy back
  cool down
`,
  },
] as const;

const asciiInterludes = [
  {
    after: 0,
    label: "inventory pulse",
    caption:
      "AGC is meant to move like charged inventory, not sit there like a dead warehouse receipt.",
    frames: [
      String.raw`
      .         .        .         .        .
   AGC o     AGC o    AGC o     AGC o    AGC o
        \        \      |      /        /
         \        \     |     /        /
          \        \    |    /        /
         ~ ~ ~ ~ [ UNISWAP v4 POOL ] ~ ~ ~ ~
                     ||        ||
                     ||        ||------> xAGC
                     ||------> treasury
`,
      String.raw`
   .         .       .        .       .        .
 AGC o    AGC o   AGC o    AGC o   AGC o    AGC o
      \       \      \      |      /       /
       \       \      \     |     /       /
       \       \      \    |    /       /
       ~ ~ ~ ~ ~ [ UNISWAP v4 POOL ] ~ ~ ~ ~ ~
                      ||        ||
                 treasury dry     ||----------> xAGC
`,
      String.raw`
     .        .        .        .        .
  AGC o    AGC o    AGC o    AGC o    AGC o
       \        \     / \      /       /
        \        \   /   \    /       /
         \        \ /     \  /       /
        ~ ~ ~ ~ [ UNISWAP v4 POOL ] ~ ~ ~ ~
                    ||            ||
                    ||------> treasury hum
                    ||------> xAGC grows
`,
    ],
  },
  {
    after: 1,
    label: "strange attractor",
    caption:
      "Not every break has to explain itself. Some of them can just feel like the protocol is dreaming in ANSI.",
    frames: [
      String.raw`
              *           .            *
        .          .--------------.        .
     *        .---/ rainbow void /---.        *
            /::::/::AGC comet::/::::\ 
     .     |::::|:::::::::::::|:::::|      .
           |::::|:::<><><>::::|:::::|
           |::::|:::::::__::::|:::::|    *
      *    |::::|::::::/  \:::|:::::|
           \::::\:::::\__/:::/:::::/ 
        .    '---\ machine hum /---'     .
                *  *       *  *
`,
      String.raw`
           .                *               .
      *         .--------------------.        *
             .-/ psychedelic buffer /-. 
            /::/::glitter daemon:::\::\
      .    |::|::::::::::::::::::::|::|     .
           |::|::::::(\_/ )::::::::|::|
           |::|::::::( o.o)::::::::|::|   *
        *  |::|::::::(> ^ <):::::::|::|
           \::\::::::::::::::::::::/::/
        .    '-\ rainbow static /-'    .
               *      *      *      *
`,
      String.raw`
           *             .               *
      .        .------------------.         .
             ./ spectral billboard \.
      *     /::::::::::::::::::::::\
           |::::: AGC DREAMS ::::::|
           |::::: IN GLITCHES :::::|
      .    |::::: AND HOT NOISE :::|     *
           |:::::  <><><><><>  ::::|
           \::::::::::::::::::::::/
        *    '------------------'      .
                *      .      *
`,
    ],
  },
  {
    after: 2,
    label: "hook cinema",
    caption:
      "The hook is the surveillance camera, toll booth, and weird little arcade cabinet welded into the market itself.",
    frames: [
      String.raw`
[ beforeSwap ] --> [ fee ] --> [ classify ] --> [ vibes ]
                         || 
                         \/
                  .---------------.
                  |   afterSwap   |
                  |  oracle tape   |
                  |  demand meter  |
                  '---------------'
                         ||
                       tape *
`,
      String.raw`
[ beforeSwap ] ==> [ fee ] ==> [ classify ] ==> [ vibes ]
                          ||
                          \/
                   .---------------.
                   |   afterSwap   |
                   |  oracle tape   |
                   |  demand meter  |
                   '---------------'
                          ||
                        tape **
`,
      String.raw`
[ beforeSwap ] --> [ fee ] --> [ classify ] --> [ vibes ]
                         ||
                         \/
                  .---------------.
                  |   afterSwap   |
                  |  oracle tape   |
                  |  demand meter  |
                  '---------------'
                         ||
                       tape ***
`,
      String.raw`
[ beforeSwap ] ==> [ fee ] ==> [ classify ] ==> [ vibes ]
                          ||
                          \/
                   .---------------.
                   |   afterSwap   |
                   |  oracle tape   |
                   |  demand meter  |
                   '---------------'
                          ||
                        tape ****
`,
    ],
  },
  {
    after: 3,
    label: "daemon lounge",
    caption:
      "A little whimsical break before the risk section. The protocol has earned one.",
    frames: [
      String.raw`
      _________________________________
     /  AGC DAEMON LOUNGE            /|
    /_______________________________/ |
    |  .-.    .-.    .-.    .-.     | |
    | (o o)  (o o)  (o o)  (o o)    | |
    |  |=|    |=|    |=|    |=|     | |
    | __|__ __|__ __|__ __|__      | |
    |/__=__\__=__\__=__\__=__\     | |
    |  looping productive volume    | /
    '-------------------------------'/
`,
      String.raw`
      _________________________________
     /  AGC DAEMON LOUNGE            /|
    /_______________________________/ |
    |  \_/    \_/    \_/    \_/     | |
    | (o.o)  (o.o)  (o.o)  (o.o)    | |
    |  /|\    /|\    /|\    /|\     | |
    | _/ \_ _/ \_ _/ \_ _/ \_      | |
    |   routing swaps all night      | /
    '-------------------------------'/
`,
      String.raw`
      _________________________________
     /  AGC DAEMON LOUNGE            /|
    /_______________________________/ |
    |  <*>    <*>    <*>    <*>     | |
    | (^-^)  (^-^)  (^-^)  (^-^)    | |
    | _/ \_  _/ \_  _/ \_  _/ \_    | |
    |  treasury glows in the dark    | /
    '-------------------------------'/
`,
    ],
  },
  {
    after: 4,
    label: "defense mode",
    caption:
      "When stress hits, the protocol should feel different: tighter, louder, more expensive to exit, and more willing to spend treasury strength.",
    frames: [
      String.raw`
  .-------------------------------.
  | stress meter: [###.....]      |
  | mint         : OFF            |
  | exit toll    : +              |
  | treasury     : --> buyback    |
  |                    --> burn   |
  '-------------------------------'
`,
      String.raw`
  .-------------------------------.
  | stress meter: [#####...]      |
  | mint         : OFF            |
  | exit toll    : ++             |
  | treasury     : ==> buyback    |
  |                    ==> burn   |
  '-------------------------------'
`,
      String.raw`
  .-------------------------------.
  | stress meter: [#######.]      |
  | mint         : OFF            |
  | exit toll    : +++            |
  | treasury     : ===> buyback   |
  |                    ===> burn  |
  '-------------------------------'
`,
      String.raw`
  .-------------------------------.
  | stress meter: [########]      |
  | mint         : OFF            |
  | exit toll    : ++++           |
  | treasury     : ====> buyback  |
  |                    ====> burn |
  '-------------------------------'
`,
    ],
  },
  {
    after: 4,
    label: "credits roll",
    caption:
      "One last little hallucination at the end of the dense part, just to keep the scroll generous.",
    frames: [
      String.raw`
      ___         ___         ___         ___                  ___         ___     
     /\  \       /\  \       /\__\       /\  \                /\  \       /\__\    
    |::\  \     /::\  \     /:/  /       \:\  \     ___       \:\  \     /:/ _/_   
    |:|:\  \   /:/\:\  \   /:/  /         \:\  \   /\__\       \:\  \   /:/ /\__\  
  __|:|\:\  \ /:/ /::\  \ /:/  /  ___ ___ /::\  \ /:/__/   _____\:\  \ /:/ /:/ _/_ 
 /::::|_\:\__Y:/_/:/\:\__Y:/__/  /\__Y\  /:/\:\__Y::\  \  /::::::::\__Y:/_/:/ /\__\
 \:\~~\  \/__|:\/:/  \/__|:\  \ /:/  |:\/:/  \/__|/\:\  \_\:\~~\~~\/__|:\/:/ /:/  /
  \:\  \      \::/__/     \:\  /:/  / \::/__/     ~~\:\/\__\:\  \      \::/_/:/  / 
   \:\  \      \:\  \      \:\/:/  /   \:\  \        \::/  /\:\  \      \:\/:/  /  
    \:\__\      \:\__\      \::/  /     \:\__\       /:/  /  \:\__\      \::/  /   
     \/__/       \/__/       \/__/       \/__/       \/__/    \/__/       \/__/     

      AGC >>> USDC >>> xAGC >>> GLOW
`,
      String.raw`
      ___         ___         ___         ___      ___     
     /\  \       /\  \       /\  \       /\__\    /\  \    
    /::\  \     /::\  \     /::\  \     /::|  |   \:\  \   
   /:/\:\  \   /:/\:\  \   /:/\:\  \   /:|:|  |    \:\  \  
  /::\~\:\  \ /:/  \:\  \ /::\~\:\  \ /:/|:|  |__  /::\  \ 
 /:/\:\ \:\__Y:/__/_\:\__Y:/\:\ \:\__Y:/ |:| /\__\/:/\:\__\
 \/__\:\/:/  |:\  /\ \/__|:\~\:\ \/__|/__|:|/:/  /:/  \/__/
      \::/  / \:\ \:\__\  \:\ \:\__\     |:/:/  /:/  /     
      /:/  /   \:\/:/  /   \:\ \/__/     |::/  /\/__/      
     /:/  /     \::/  /     \:\__\       /:/  /            
     \/__/       \/__/       \/__/       \/__/                                                

      GLOW >>> xAGC >>> USDC >>> AGC
`,
      String.raw`
      ___         ___         ___         ___         ___         ___         ___              
     /\__\       /\  \       /\  \       /\  \       /\__\       /\  \       /\__\             
    /:/  /       \:\  \     /::\  \     /::\  \     /:/ _/_      \:\  \     /:/  /       ___   
   /:/  /         \:\  \   /:/\:\__\   /:/\:\__\   /:/ /\__\      \:\  \   /:/  /       /|  |  
  /:/  /  ___ ___  \:\  \ /:/ /:/  /  /:/ /:/  /  /:/ /:/ _/_ _____\:\  \ /:/  /  ___  |:|  |  
 /:/__/  /\__Y\  \  \:\__Y:/_/:/__/__/:/_/:/__/__/:/_/:/ /\__Y::::::::\__Y:/__/  /\__\ |:|  |  
 \:\  \ /:/  |:\  \ /:/  |:\/:::::/  |:\/:::::/  |:\/:/ /:/  |:\~~\~~\/__|:\  \ /:/  /_|:|__|  
  \:\  /:/  / \:\  /:/  / \::/~~/~~~~ \::/~~/~~~~ \::/_/:/  / \:\  \      \:\  /:/  /::::\  \  
   \:\/:/  /   \:\/:/  /   \:\~~\      \:\~~\      \:\/:/  /   \:\  \      \:\/:/  /~~~~\:\  \ 
    \::/  /     \::/  /     \:\__\      \:\__\      \::/  /     \:\__\      \::/  /      \:\__\
     \/__/       \/__/       \/__/       \/__/       \/__/       \/__/       \/__/        \/__/           

      xAGC >>> GLOW >>> AGC >>> USDC
`,
    ],
  },
] as const;

const finalAsciiInterlude = asciiInterludes.find(
  (interlude) => interlude.label === "credits roll",
);

const regimeNarrative = [
  {
    name: "Expansion",
    signal: "mint allowed",
    text:
      "Demand is healthy, liquidity is deep enough, volatility is controlled, and exits are not dominating the tape. New supply can be distributed into xAGC, growth programs, LPs, integrators, and treasury.",
  },
  {
    name: "Neutral",
    signal: "hold posture",
    text:
      "The protocol keeps the machine running without discretionary growth. It keeps measuring demand, but it does not print fresh credit just because price held together for a single epoch.",
  },
  {
    name: "Defense",
    signal: "stress tolls",
    text:
      "Price is weak or stress metrics are flashing red. Mint shuts off, exits get more expensive, and treasury USDC can be drawn down through queued buybacks executed over time with on-chain slippage and price-limit guardrails. The protocol spends stored strength to defend utility.",
  },
  {
    name: "Recovery",
    signal: "cooldown",
    text:
      "Stress has cooled but trust has not been fully rebuilt. Incentives return slowly, the reserve gets priority, and expansion stays locked until the cooldown expires.",
  },
] as const;

const flowSteps = [
  {
    title: "1. Hold AGC",
    text:
      "An agent, sponsor, or operator keeps AGC as transaction inventory instead of holding only inert stablecoins between jobs.",
  },
  {
    title: "2. Buy or receive more AGC",
    text:
      "Users add inventory through the canonical AGC/USDC pool. Gross buys, gross sells, volatility, and hook fees all accumulate in the same venue.",
  },
  {
    title: "3. Lock AGC into xAGC",
    text:
      "When users want the upside layer instead of liquid inventory, they deposit AGC into xAGC and hold a fixed share count against a growing asset base.",
  },
  {
    title: "4. Settle the epoch",
    text:
      "The controller reads the hook tape plus vault flow, then decides whether the protocol should expand, hold neutral, enter defense, or remain in recovery.",
  },
  {
    title: "5. Expand or defend",
    text:
      "Expansion mints mostly into xAGC and other target buckets. Defense queues treasury buybacks and keeps the monetary posture tight.",
  },
  {
    title: "6. Exit when needed",
    text:
      "Users can always sell AGC back into the pool, or redeem xAGC shares and pay the exit fee that replenishes treasury dry powder.",
  },
] as const;

const vendingCoinCount = 6;
const vendingLaneWidth = 31;
const vendingPanelWidth = 28;

type CursorStamp = {
  id: number;
  x: number;
  y: number;
  hue: number;
  scale: number;
  rotation: number;
};

type VendingMachineStage = {
  key: string;
  statusCopy: string;
  durationMs?: number;
  trailTop: string;
  trailBottom: string;
  display: string;
  slot: string;
  gears: string;
  clacker: string;
  chute: string;
  candyOut: string;
};

const idleVendingStage: VendingMachineStage = {
  key: "idle",
  statusCopy: "ʘ‿ʘ",
  trailTop: "",
  trailBottom: "",
  display: "CANDY BANK",
  slot: "COIN SLOT [ ]",
  gears: "GEARS   o-.-o   o-.-o",
  clacker: "RATCHET   _/   \\_",
  chute: "CHUTE    [      ]",
  candyOut: "",
};

const vendingMachineSequence: readonly VendingMachineStage[] = [
  {
    key: "roll-1",
    statusCopy: "ヾ(⌐■_■)ノ♪",
    durationMs: 150,
    trailTop: "  ($) -------->",
    trailBottom: "",
    display: "CANDY BANK",
    slot: "COIN SLOT [ ]",
    gears: "GEARS   o-.-o   o-.-o",
    clacker: "RATCHET   _/   \\_",
    chute: "CHUTE    [      ]",
    candyOut: "",
  },
  {
    key: "roll-2",
    statusCopy: "( ͡ᵔ ͜ʖ ͡ᵔ )",
    durationMs: 140,
    trailTop: "          ($) -------->",
    trailBottom: "",
    display: "CANDY BANK",
    slot: "COIN SLOT [ ]",
    gears: "GEARS   o-.-o   o-.-o",
    clacker: "RATCHET   _/   \\_",
    chute: "CHUTE    [      ]",
    candyOut: "",
  },
  {
    key: "roll-3",
    statusCopy: "( ͡° ͜ʖ ͡°)",
    durationMs: 130,
    trailTop: "                   ($) --->",
    trailBottom: "",
    display: "INCOMING",
    slot: "COIN SLOT [ ]",
    gears: "GEARS   o~.~o   o~.~o",
    clacker: "RATCHET   _/   \\_",
    chute: "CHUTE    [      ]",
    candyOut: "",
  },
  {
    key: "insert",
    statusCopy: "\\ (•◡•) /",
    durationMs: 160,
    trailTop: "",
    trailBottom: "                         v",
    display: "* * TINNNNG * *",
    slot: "COIN SLOT ($)",
    gears: "GEARS   o-!-o   o-!-o",
    clacker: "RATCHET   _/ ! \\_",
    chute: "CHUTE    [      ]",
    candyOut: "",
  },
  {
    key: "tumble",
    statusCopy: "╰(°▽°)╯",
    durationMs: 150,
    trailTop: "",
    trailBottom: "",
    display: "PROCESSING...",
    slot: "COIN SLOT [*]",
    gears: "GEARS   o=+=o   o=+=o",
    clacker: "TUMBLE  ($)~~~>>",
    chute: "CHUTE    [      ]",
    candyOut: "",
  },
  {
    key: "clack-1",
    statusCopy: "༼ つ ಥ_ಥ ༽つ",
    durationMs: 130,
    trailTop: "",
    trailBottom: "",
    display: "CLACK",
    slot: "COIN SLOT [ ]",
    gears: "GEARS   o<+>o   o-.-o",
    clacker: "HAMMER    _\\|!|/_",
    chute: "CHUTE    [      ]",
    candyOut: "",
  },
  {
    key: "clack-2",
    statusCopy: "ᕦ(ò_óˇ)ᕤ",
    durationMs: 130,
    trailTop: "",
    trailBottom: "",
    display: "CLACK-CLACK",
    slot: "COIN SLOT [ ]",
    gears: "GEARS   o-.-o   o<+>o",
    clacker: "HAMMER    _/|!|\\_",
    chute: "CHUTE    [      ]",
    candyOut: "",
  },
  {
    key: "clack-3",
    statusCopy: "(ノಠ益ಠ)ノ彡┻━┻",
    durationMs: 140,
    trailTop: "",
    trailBottom: "",
    display: "!!CLACK-CLACK!!",
    slot: "COIN SLOT [ ]",
    gears: "GEARS   o<+>o   o<+>o",
    clacker: "HAMMER   _\\|!!!|/_",
    chute: "CHUTE    [      ]",
    candyOut: "",
  },
  {
    key: "spin-1",
    statusCopy: "┬─┬ノ( º _ ºノ)",
    durationMs: 150,
    trailTop: "",
    trailBottom: "",
    display: "WHRRRR",
    slot: "COIN SLOT [ ]",
    gears: "GEARS   o/>/<o  o/>/<o",
    clacker: "SPINDLE   ~<#>~",
    chute: "CHUTE    [      ]",
    candyOut: "",
  },
  {
    key: "spin-2",
    statusCopy: "( ˘ ³˘)♥",
    durationMs: 140,
    trailTop: "",
    trailBottom: "",
    display: "WHRRRRRRRRR",
    slot: "COIN SLOT [ ]",
    gears: "GEARS   o<*>o   o<*>o",
    clacker: "SPINDLE  ~<<#>>~",
    chute: "CHUTE    [      ]",
    candyOut: "",
  },
  {
    key: "mix-1",
    statusCopy: "(ง°ل͜°)ง",
    durationMs: 170,
    trailTop: "",
    trailBottom: "",
    display: "MIXING...",
    slot: "COIN SLOT [ ]",
    gears: "GEARS   o@*@o   o@*@o",
    clacker: "MIXER    [==#==]",
    chute: "CHUTE    [      ]",
    candyOut: "",
  },
  {
    key: "mix-2",
    statusCopy: "(づ￣ ³￣)づ",
    durationMs: 180,
    trailTop: "",
    trailBottom: "",
    display: "~* ALCHEMY *~",
    slot: "COIN SLOT [ ]",
    gears: "GEARS   o*@*o   o*@*o",
    clacker: "MIXER    [##=##]",
    chute: "CHUTE    [      ]",
    candyOut: "",
  },
  {
    key: "assemble",
    statusCopy: "ʕ•̫͡•ʔ♡*:.✧",
    durationMs: 170,
    trailTop: "",
    trailBottom: "",
    display: "ASSEMBLING",
    slot: "COIN SLOT [ ]",
    gears: "GEARS   o-*-o   o-*-o",
    clacker: "PRESS    [>#@#<]",
    chute: "CHUTE    [      ]",
    candyOut: "",
  },
  {
    key: "drop",
    statusCopy: "(っ˘ڡ˘ς)",
    durationMs: 170,
    trailTop: "",
    trailBottom: "",
    display: "* THUNK *",
    slot: "COIN SLOT [ ]",
    gears: "GEARS   o-.-o   o-.-o",
    clacker: "PRESS    [_____]",
    chute: "CHUTE    [o={#@#}=o]",
    candyOut: "",
  },
  {
    key: "slide",
    statusCopy: "~(˘▾˘~)",
    durationMs: 160,
    trailTop: "",
    trailBottom: "",
    display: "DISPENSING",
    slot: "COIN SLOT [ ]",
    gears: "GEARS   o-.-o   o-.-o",
    clacker: "RATCHET   _/   \\_",
    chute: "CHUTE    [  >>o=]",
    candyOut: "",
  },
  {
    key: "pop-1",
    statusCopy: "(─‿‿─)",
    durationMs: 200,
    trailTop: "",
    trailBottom: "",
    display: "DELIVER",
    slot: "COIN SLOT [ ]",
    gears: "GEARS   o-.-o   o-.-o",
    clacker: "RATCHET   _/   \\_",
    chute: "CHUTE    [      ]",
    candyOut: "--> o={#@#}=o",
  },
  {
    key: "pop-2",
    statusCopy: "(ʘᗩʘ')",
    durationMs: 2340,
    trailTop: "",
    trailBottom: "",
    display: "* * ENJOY * *",
    slot: "COIN SLOT [ ]",
    gears: "GEARS   o-.-o   o-.-o",
    clacker: "RATCHET   _/   \\_",
    chute: "CHUTE    [      ]",
    candyOut: "====> (;´༎ຶД༎ຶ`)",
  },
] as const;

function padVendingLane(value: string) {
  return value.padEnd(vendingLaneWidth, " ");
}

function padVendingPanel(value: string) {
  return value.padEnd(vendingPanelWidth, " ");
}

function buildVendingMachineFrame({
  activeCoinIndex,
  stage,
}: {
  activeCoinIndex: number | null;
  stage: VendingMachineStage;
}) {
  const queue = Array.from({ length: vendingCoinCount }, (_, index) =>
    activeCoinIndex !== null && activeCoinIndex === index ? "   " : "($)",
  ).join("  ");

  return [
    `${padVendingLane(queue)}     ____________________________________________`,
    `${padVendingLane(stage.trailTop)}    / .----------------------------------------. \\`,
    `${padVendingLane(stage.trailBottom)}   / /      VEND-O-MATIC 9000                   \\ \\`,
    `${padVendingLane("")}  | |      .-------------------------------.     | |`,
    `${padVendingLane("")}  | |      | ${padVendingPanel(stage.display)} |      | |`,
    `${padVendingLane("")}  | |      | ${padVendingPanel(stage.slot)} |      | |`,
    `${padVendingLane("")}  | |      | ${padVendingPanel(stage.gears)} |      | |`,
    `${padVendingLane("")}  | |      | ${padVendingPanel(stage.clacker)} |      | |`,
    `${padVendingLane("")}  | |      | ${padVendingPanel(stage.chute)} |      | |   ${stage.candyOut}`,
    `${padVendingLane("")}  | |      |__________.--.__.--.__________|      | |`,
    `${padVendingLane("")}  | |______|____________|__||__|___________|_____| |`,
    `${padVendingLane("")}   \\______________________________________________/`,
  ].join("\n");
}

function CursorTrail() {
  const [stamps, setStamps] = useState<CursorStamp[]>([]);

  useEffect(() => {
    if (typeof window === "undefined") return;
    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;

    let stampId = 0;
    let hue = 0;
    let lastX = -1_000;
    let lastY = -1_000;
    let lastSpawnAt = 0;
    const timeouts = new Set<number>();
    const maxStamps = 18;
    const lifetimeMs = 1_350;

    const onMove = (event: PointerEvent) => {
      const now = performance.now();
      const distance = Math.hypot(event.clientX - lastX, event.clientY - lastY);

      if (distance < 18 && now - lastSpawnAt < 42) {
        return;
      }

      lastX = event.clientX;
      lastY = event.clientY;
      lastSpawnAt = now;
      hue = (hue + 31) % 360;

      const id = stampId++;
      const stamp: CursorStamp = {
        id,
        x: event.clientX,
        y: event.clientY,
        hue,
        scale: 0.78 + (id % 4) * 0.045,
        rotation: -12 + (id % 5) * 3,
      };

      setStamps((current) => [...current.slice(-(maxStamps - 1)), stamp]);

      const timeout = window.setTimeout(() => {
        setStamps((current) => current.filter((entry) => entry.id !== id));
        timeouts.delete(timeout);
      }, lifetimeMs);

      timeouts.add(timeout);
    };

    window.addEventListener("pointermove", onMove, { passive: true });

    return () => {
      window.removeEventListener("pointermove", onMove);
      timeouts.forEach((timeout) => window.clearTimeout(timeout));
    };
  }, []);

  if (stamps.length === 0) return null;

  return (
    <div className="cursor-trail" aria-hidden="true">
      {stamps.map((stamp) => {
        const style = {
          left: `${stamp.x}px`,
          top: `${stamp.y}px`,
          "--trail-scale": `${stamp.scale}`,
          "--trail-hue": `${stamp.hue}`,
          "--trail-rotate": `${stamp.rotation}deg`,
        } as CSSProperties;

        return <span key={stamp.id} className="cursor-stamp" style={style} />;
      })}
    </div>
  );
}

function AsciiInterlude({
  label,
  caption,
  frames,
  backgroundVideoSrc,
}: {
  label: string;
  caption: string;
  frames: readonly string[];
  backgroundVideoSrc?: string;
}) {
  const [frameIndex, setFrameIndex] = useState(0);

  useEffect(() => {
    const interval = window.setInterval(() => {
      setFrameIndex((current) => (current + 1) % frames.length);
    }, 420);

    return () => window.clearInterval(interval);
  }, [frames]);

  return (
    <section className="ascii-interlude">
      {backgroundVideoSrc ? (
        <>
          <video
            className="ascii-interlude-video"
            autoPlay
            muted
            loop
            playsInline
            preload="auto"
            aria-hidden="true"
          >
            <source src={backgroundVideoSrc} type="video/mp4" />
          </video>
          <div className="ascii-interlude-video-overlay" aria-hidden="true" />
        </>
      ) : null}
      <div className="ascii-interlude-top">
        <p className="ascii-interlude-label">{label}</p>
        <span className="ascii-interlude-pulse" />
      </div>
      <pre
        className={`ascii-interlude-frame${
          backgroundVideoSrc ? " ascii-interlude-frame-video" : ""
        }`}
        aria-hidden="true"
      >
        {frames[frameIndex]}
      </pre>
      <p className="ascii-interlude-caption">{caption}</p>
    </section>
  );
}

function AsciiVendingMachine() {
  const [activeCoinIndex, setActiveCoinIndex] = useState<number | null>(null);
  const [nextCoinIndex, setNextCoinIndex] = useState(0);
  const [stageIndex, setStageIndex] = useState<number | null>(null);
  const activeStage =
    stageIndex === null
      ? idleVendingStage
      : vendingMachineSequence[stageIndex] ?? idleVendingStage;

  useEffect(() => {
    if (stageIndex === null) return;

    const timeout = window.setTimeout(() => {
      if (stageIndex >= vendingMachineSequence.length - 1) {
        setStageIndex(null);
        setActiveCoinIndex(null);
        return;
      }

      setStageIndex(stageIndex + 1);
    }, activeStage.durationMs ?? 0);

    return () => window.clearTimeout(timeout);
  }, [activeStage.durationMs, stageIndex]);

  function handleVend() {
    if (stageIndex !== null) return;

    setActiveCoinIndex(nextCoinIndex);
    setNextCoinIndex((current) => (current + 1) % vendingCoinCount);
    setStageIndex(0);
  }

  const frame = buildVendingMachineFrame({ activeCoinIndex, stage: activeStage });

  return (
    <section className="vending-machine-section">
      <div className="dash-heading">
        <p className="section-kicker">sugar break</p>
      </div>

      <div className="vending-machine-layout">
        <pre className="vending-machine-art" aria-hidden="true">
          {frame}
        </pre>

        <div className="vending-machine-console">
          <div className="vending-machine-controls">
            <span className="vending-machine-arrow" aria-hidden="true">
              -------------&gt;
            </span>
            <button
              type="button"
              className="btn btn-primary vending-machine-button"
              onClick={handleVend}
              disabled={stageIndex !== null}
            >
              Click me
            </button>
          </div>

          <p className="vending-machine-status" aria-live="polite">
            {activeStage.statusCopy}
          </p>
        </div>
      </div>
    </section>
  );
}

const MAX_UINT256 = (1n << 256n) - 1n;
const DEFAULT_RECIPIENT = "0x000000000000000000000000000000000000dEaD";

function fmtAmount(
  value: bigint | undefined,
  decimals: number,
  precision = 4,
): string {
  if (value === undefined) return " - ";
  return Number(formatUnits(value, decimals)).toFixed(precision);
}

function fmt18(v: bigint | undefined, decimals = 4): string {
  return fmtAmount(v, 18, decimals);
}

function fmt6(v: bigint | undefined, decimals = 2): string {
  return fmtAmount(v, 6, decimals);
}

function fmtQuote(v: bigint | undefined, decimals = 2): string {
  if (v === undefined) return " - ";
  return `$${fmt18(v, decimals)}`;
}

function fmtBps(v: bigint | undefined): string {
  if (v === undefined) return " - ";
  return `${(Number(v) / 100).toFixed(2)}%`;
}

function tupleField<T>(
  value: unknown,
  key: string,
  index: number,
): T | undefined {
  if (value && typeof value === "object" && key in (value as Record<string, unknown>)) {
    return (value as Record<string, T>)[key];
  }
  if (Array.isArray(value)) {
    return value[index] as T;
  }
  return undefined;
}

function safeParseUnits(value: string, decimals: number) {
  try {
    return parseUnits(value || "0", decimals);
  } catch {
    return 0n;
  }
}

function extractErrorMessage(error: unknown) {
  if (error instanceof Error) return error.message;
  if (typeof error === "string") return error;
  if (error && typeof error === "object") {
    const shortMessage = (error as { shortMessage?: string }).shortMessage;
    if (shortMessage) return shortMessage;
    const message = (error as { message?: string }).message;
    if (message) return message;
  }
  return "Transaction failed.";
}

export default function App() {
  const { address, isConnected } = useAccount();
  const { connect, connectors } = useConnect();
  const { disconnect } = useDisconnect();
  const publicClient = usePublicClient();
  const { writeContractAsync } = useWriteContract();

  const [buyUsdcAmount, setBuyUsdcAmount] = useState("10");
  const [buyMinAgcOut, setBuyMinAgcOut] = useState("18");
  const [sellAgcAmount, setSellAgcAmount] = useState("20");
  const [sellMinUsdcOut, setSellMinUsdcOut] = useState("9.9");
  const [stakeAgcAmount, setStakeAgcAmount] = useState("50");
  const [redeemXagcShares, setRedeemXagcShares] = useState("10");
  const [recipient, setRecipient] = useState(DEFAULT_RECIPIENT);
  const [txStatus, setTxStatus] = useState("Idle");
  const [txNote, setTxNote] = useState<string | null>(null);

  const ready =
    addresses.agc &&
    addresses.usdc &&
    addresses.hook &&
    addresses.policyController &&
    addresses.settlementRouter &&
    addresses.treasuryVault &&
    addresses.xagcVault;

  const enabled = (addressValue: string | undefined) => ({
    query: { enabled: Boolean(addressValue) },
  });

  const anchor = useReadContract({
    address: addresses.policyController,
    abi: policyControllerAbi,
    functionName: "anchorPriceX18",
    ...enabled(addresses.policyController),
  });
  const regime = useReadContract({
    address: addresses.policyController,
    abi: policyControllerAbi,
    functionName: "regime",
    ...enabled(addresses.policyController),
  });
  const premium = useReadContract({
    address: addresses.policyController,
    abi: policyControllerAbi,
    functionName: "lastPremiumBps",
    ...enabled(addresses.policyController),
  });
  const coverage = useReadContract({
    address: addresses.policyController,
    abi: policyControllerAbi,
    functionName: "lastCoverageBps",
    ...enabled(addresses.policyController),
  });
  const exitPressure = useReadContract({
    address: addresses.policyController,
    abi: policyControllerAbi,
    functionName: "lastExitPressureBps",
    ...enabled(addresses.policyController),
  });
  const volatility = useReadContract({
    address: addresses.policyController,
    abi: policyControllerAbi,
    functionName: "lastVolatilityBps",
    ...enabled(addresses.policyController),
  });
  const lockedShare = useReadContract({
    address: addresses.policyController,
    abi: policyControllerAbi,
    functionName: "lastLockedShareBps",
    ...enabled(addresses.policyController),
  });
  const lockFlow = useReadContract({
    address: addresses.policyController,
    abi: policyControllerAbi,
    functionName: "lastLockFlowBps",
    ...enabled(addresses.policyController),
  });
  const pendingBuyback = useReadContract({
    address: addresses.policyController,
    abi: policyControllerAbi,
    functionName: "pendingTreasuryBuybackUsdc",
    ...enabled(addresses.policyController),
  });
  const currentEpochId = useReadContract({
    address: addresses.hook,
    abi: hookAbi,
    functionName: "currentEpochId",
    ...enabled(addresses.hook),
  });
  const currentAccumulator = useReadContract({
    address: addresses.hook,
    abi: hookAbi,
    functionName: "currentAccumulator",
    ...enabled(addresses.hook),
  });
  const treasuryUsdc = useReadContract({
    address: addresses.treasuryVault,
    abi: stabilityVaultAbi,
    functionName: "availableUsdc",
    ...enabled(addresses.treasuryVault),
  });
  const treasuryAgc = useReadContract({
    address: addresses.treasuryVault,
    abi: stabilityVaultAbi,
    functionName: "availableAGC",
    ...enabled(addresses.treasuryVault),
  });
  const agcBalance = useReadContract({
    address: addresses.agc,
    abi: agcAbi,
    functionName: "balanceOf",
    args: address ? [address] : undefined,
    query: { enabled: Boolean(addresses.agc && address) },
  });
  const usdcBalance = useReadContract({
    address: addresses.usdc,
    abi: erc20Abi,
    functionName: "balanceOf",
    args: address ? [address] : undefined,
    query: { enabled: Boolean(addresses.usdc && address) },
  });
  const xagcBalance = useReadContract({
    address: addresses.xagcVault,
    abi: xagcVaultAbi,
    functionName: "balanceOf",
    args: address ? [address] : undefined,
    query: { enabled: Boolean(addresses.xagcVault && address) },
  });
  const xagcTotalAssets = useReadContract({
    address: addresses.xagcVault,
    abi: xagcVaultAbi,
    functionName: "totalAssets",
    ...enabled(addresses.xagcVault),
  });
  const xagcTotalSupply = useReadContract({
    address: addresses.xagcVault,
    abi: xagcVaultAbi,
    functionName: "totalSupply",
    ...enabled(addresses.xagcVault),
  });
  const xagcExitFee = useReadContract({
    address: addresses.xagcVault,
    abi: xagcVaultAbi,
    functionName: "exitFeeBps",
    ...enabled(addresses.xagcVault),
  });
  const previewDeposit = useReadContract({
    address: addresses.xagcVault,
    abi: xagcVaultAbi,
    functionName: "previewDeposit",
    args: [safeParseUnits(stakeAgcAmount, 18)],
    ...enabled(addresses.xagcVault),
  });
  const previewRedeem = useReadContract({
    address: addresses.xagcVault,
    abi: xagcVaultAbi,
    functionName: "previewRedeem",
    args: [safeParseUnits(redeemXagcShares, 18)],
    ...enabled(addresses.xagcVault),
  });

  const regimeIdx =
    typeof regime.data === "number"
      ? regime.data
      : typeof regime.data === "bigint"
        ? Number(regime.data)
        : undefined;
  const regimeKey =
    regimeIdx !== undefined ? regimeKeys[regimeIdx] ?? "neutral" : "neutral";
  const regimeLabel =
    regimeIdx !== undefined ? regimeLabels[regimeIdx] ?? "Unknown" : " - ";

  const shortAddr = address
    ? `${address.slice(0, 6)}...${address.slice(-4)}`
    : null;

  const grossBuyVolume =
    tupleField<bigint>(currentAccumulator.data, "grossBuyVolumeQuoteX18", 5);
  const grossSellVolume =
    tupleField<bigint>(currentAccumulator.data, "grossSellVolumeQuoteX18", 6);
  const hookFeesQuote =
    tupleField<bigint>(currentAccumulator.data, "totalHookFeesQuoteX18", 11);

  const previewRedeemNet =
    Array.isArray(previewRedeem.data) ? (previewRedeem.data[0] as bigint) : undefined;
  const previewRedeemFee =
    Array.isArray(previewRedeem.data) ? (previewRedeem.data[1] as bigint) : undefined;

  const xagcExchangeRate =
    xagcTotalAssets.data && xagcTotalSupply.data && xagcTotalSupply.data > 0n
      ? (xagcTotalAssets.data * 10n ** 18n) / xagcTotalSupply.data
      : undefined;

  useEffect(() => {
    if (address && recipient === DEFAULT_RECIPIENT) {
      setRecipient(address);
    }
  }, [address, recipient]);

  async function waitForHash(hash: Hex) {
    if (!publicClient) {
      throw new Error("Wallet client is connected, but no public client is configured.");
    }
    return publicClient.waitForTransactionReceipt({ hash });
  }

  async function runAction(
    status: string,
    action: () => Promise<Hex>,
    successNote: string,
  ) {
    try {
      setTxNote(null);
      setTxStatus(status);
      const hash = await action();
      await waitForHash(hash);
      setTxStatus(`${status} complete`);
      setTxNote(successNote);
    } catch (error) {
      setTxStatus("Idle");
      setTxNote(extractErrorMessage(error));
    }
  }

  async function handleApproveUsdc() {
    if (!addresses.usdc || !addresses.settlementRouter) return;

    await runAction(
      "Approving USDC",
      () =>
        writeContractAsync({
          address: addresses.usdc!,
          abi: erc20Abi,
          functionName: "approve",
          args: [addresses.settlementRouter!, MAX_UINT256],
        }),
      "USDC approved for router buys.",
    );
  }

  async function handleApproveAgcForSell() {
    if (!addresses.agc || !addresses.settlementRouter) return;

    await runAction(
      "Approving AGC for sells",
      () =>
        writeContractAsync({
          address: addresses.agc!,
          abi: agcAbi,
          functionName: "approve",
          args: [addresses.settlementRouter!, MAX_UINT256],
        }),
      "AGC approved for router sells.",
    );
  }

  async function handleApproveAgcForVault() {
    if (!addresses.agc || !addresses.xagcVault) return;

    await runAction(
      "Approving AGC for xAGC",
      () =>
        writeContractAsync({
          address: addresses.agc!,
          abi: agcAbi,
          functionName: "approve",
          args: [addresses.xagcVault!, MAX_UINT256],
        }),
      "AGC approved for xAGC deposits.",
    );
  }

  async function handleBuyAgc() {
    if (!ready || !address || !addresses.settlementRouter) return;

    await runAction(
      "Buying AGC",
      () =>
        writeContractAsync({
          address: addresses.settlementRouter!,
          abi: settlementRouterAbi,
          functionName: "buyAGC",
          args: [
            parseUnits(buyUsdcAmount, 6),
            parseUnits(buyMinAgcOut, 18),
            (recipient || address) as `0x${string}`,
            keccak256(stringToHex(`${address}:${Date.now()}:buy`)),
          ],
        }),
      "AGC buy submitted through the canonical pool.",
    );
  }

  async function handleSellAgc() {
    if (!ready || !address || !addresses.settlementRouter) return;

    await runAction(
      "Selling AGC",
      () =>
        writeContractAsync({
          address: addresses.settlementRouter!,
          abi: settlementRouterAbi,
          functionName: "sellAGC",
          args: [
            parseUnits(sellAgcAmount, 18),
            parseUnits(sellMinUsdcOut, 6),
            (recipient || address) as `0x${string}`,
            keccak256(stringToHex(`${address}:${Date.now()}:sell`)),
          ],
        }),
      "AGC sell submitted through the canonical pool.",
    );
  }

  async function handleDepositXagc() {
    if (!ready || !address || !addresses.xagcVault) return;

    await runAction(
      "Depositing into xAGC",
      () =>
        writeContractAsync({
          address: addresses.xagcVault!,
          abi: xagcVaultAbi,
          functionName: "deposit",
          args: [
            parseUnits(stakeAgcAmount, 18),
            (recipient || address) as `0x${string}`,
          ],
        }),
      "AGC deposited into xAGC.",
    );
  }

  async function handleRedeemXagc() {
    if (!ready || !address || !addresses.xagcVault) return;

    await runAction(
      "Redeeming xAGC",
      () =>
        writeContractAsync({
          address: addresses.xagcVault!,
          abi: xagcVaultAbi,
          functionName: "redeem",
          args: [
            parseUnits(redeemXagcShares, 18),
            (recipient || address) as `0x${string}`,
            address,
          ],
        }),
      "xAGC redeemed back into AGC.",
    );
  }

  return (
    <main className="shell" data-regime={regimeKey}>
      <CursorTrail />
      <div className="fx fx-grid" aria-hidden="true" />
      <div className="fx fx-wash" aria-hidden="true" />
      <div className="ticker" aria-hidden="true">
        <div className="ticker-track">
          {marqueeItems.concat(marqueeItems).map((item, index) => (
            <span key={`${item}-${index}`}>{item}</span>
          ))}
        </div>
      </div>

      <header className="topbar">
        <div className="topbar-brand">
          <div className="topbar-mark" aria-hidden="true">
            <img className="topbar-mark-image" src="/agc-mark.svg" alt="" />
          </div>
          <div>
            <p className="topbar-name">Agent Credit Protocol</p>
            <p className="topbar-caption">
              floating credit / soft anchor / machine-native money
            </p>
          </div>
        </div>

        <div className="topbar-wallet">
          {isConnected ? (
            <>
              <span className="wallet-dot" />
              <span className="wallet-address">{shortAddr}</span>
              <button className="btn" onClick={() => disconnect()}>
                Disconnect
              </button>
            </>
          ) : (
            <button
              className="btn btn-primary"
              disabled={connectors.length === 0}
              onClick={() => connect({ connector: connectors[0] })}
            >
              Connect Wallet
            </button>
          )}
        </div>
      </header>

      <section className="hero-section">
        <div className="hero-copy">
          <p className="hero-eyebrow">
            ELASTIC WORKING CAPITAL / CANONICAL v4 POLICY POOL
          </p>
          <h1 className="hero-title">
            autonomous
            <br />
            commerce needs a currency that behaves like software
          </h1>
          <p className="hero-sub">
            Hold <code>AGC</code> as machine-native inventory. Trade against the
            canonical pool, lock into <code>xAGC</code> when you want the upside
            layer, and let the hook decide fees while the treasury spends into
            stress instead of pretending every token is a warehouse receipt for
            dollars.
          </p>

          <div className="hero-actions">
            <a className="btn btn-primary" href="#dashboard">
              Scan The Dashboard
            </a>
            <a className="btn btn-secondary" href="#explainer">
              Read The Protocol
            </a>
          </div>

          <div className="hero-chip-row">
            <span className="hero-chip">AGC != stablecoin collateral claim</span>
            <span className="hero-chip">USDC = settlement substrate</span>
            <span className="hero-chip">Hook fees + seigniorage + spread</span>
          </div>
        </div>

        <div className="hero-art">
          <video
            className="hero-art-video"
            autoPlay
            muted
            loop
            playsInline
            preload="auto"
            aria-hidden="true"
          >
            <source src="/ascii_green_smoke_psych_720_web.mp4" type="video/mp4" />
          </video>
          <div className="hero-art-overlay" aria-hidden="true" />
          <div className="sticker-band sticker-band-top" aria-hidden="true">
            {heroStickersTop.map((sticker, index) => (
              <div
                key={sticker}
                className={`sticker sticker-${String.fromCharCode(97 + index)}`}
              >
                {sticker}
              </div>
            ))}
          </div>
          <pre className="hero-ascii">{heroAscii}</pre>
          <div className="sticker-band sticker-band-bottom" aria-hidden="true">
            {heroStickersBottom.map((sticker, index) => (
              <div
                key={sticker}
                className={`sticker sticker-${String.fromCharCode(105 + index)}`}
              >
                {sticker}
              </div>
            ))}
          </div>
        </div>
      </section>

      {!ready && (
        <div className="notice">
          <strong>Live contract addresses are not configured in this environment.</strong>{" "}
          Protocol telemetry and wallet actions will light up once the live
          v1 deployment is connected.
        </div>
      )}

      <section id="dashboard" className="dashboard-zone">
        <div className="dash-heading">
          <p className="section-kicker">live operator surface</p>
          <h2 className="section-title">Dashboard / policy telemetry / wallet actions</h2>
        </div>

        <div className="regime-strip">
          <div className="regime-badge">
            <span className="regime-indicator" />
            <div>
              <span className="regime-label-prefix">regime</span>
              <div className="regime-label-value">{regimeLabel}</div>
            </div>
          </div>
          <div className="regime-divider" />
          <div className="regime-stat">
            <span className="regime-stat-label">anchor</span>
            <span className="regime-stat-value">
              {anchor.data ? `$${fmt18(anchor.data)}` : " - "}
            </span>
          </div>
          <div className="regime-divider" />
          <div className="regime-stat">
            <span className="regime-stat-label">treasury usdc</span>
            <span className="regime-stat-value">
              {treasuryUsdc.data ? `${fmt6(treasuryUsdc.data)} USDC` : " - "}
            </span>
          </div>
          <div className="regime-divider" />
          <div className="regime-stat">
            <span className="regime-stat-label">pending buyback</span>
            <span className="regime-stat-value">
              {pendingBuyback.data ? `${fmt6(pendingBuyback.data)} USDC` : " - "}
            </span>
          </div>
        </div>

        <div className="metrics">
          <div className="metric">
            <span className="metric-label">Premium</span>
            <span className="metric-value">{fmtBps(premium.data)}</span>
            <span className="metric-hint">price over soft anchor</span>
          </div>
          <div className="metric">
            <span className="metric-label">Coverage</span>
            <span className="metric-value">{fmtBps(coverage.data)}</span>
            <span className="metric-hint">depth versus circulating float</span>
          </div>
          <div className="metric">
            <span className="metric-label">Exit pressure</span>
            <span className="metric-value">{fmtBps(exitPressure.data)}</span>
            <span className="metric-hint">net AGC to USDC stress</span>
          </div>
          <div className="metric">
            <span className="metric-label">Volatility</span>
            <span className="metric-value">{fmtBps(volatility.data)}</span>
            <span className="metric-hint">realized epoch variance</span>
          </div>
          <div className="metric">
            <span className="metric-label">Locked share</span>
            <span className="metric-value">{fmtBps(lockedShare.data)}</span>
            <span className="metric-hint">share of float locked in xAGC</span>
          </div>
          <div className="metric">
            <span className="metric-label">Lock flow</span>
            <span className="metric-value">{fmtBps(lockFlow.data)}</span>
            <span className="metric-hint">net xAGC deposit pressure</span>
          </div>
          <div className="metric">
            <span className="metric-label">Gross buys</span>
            <span className="metric-value">{fmtQuote(grossBuyVolume)}</span>
            <span className="metric-hint">epoch demand observed by hook</span>
          </div>
          <div className="metric">
            <span className="metric-label">Gross sells</span>
            <span className="metric-value">{fmtQuote(grossSellVolume)}</span>
            <span className="metric-hint">epoch withdrawal signal</span>
          </div>
        </div>

        <div className="panels">
          <div className="panel">
            <div className="panel-header">
              <h3 className="panel-title">Buy AGC</h3>
              <div className="panel-header-side">
                <span className="panel-badge">router swap</span>
                <div className="panel-info">
                  <button
                    className="panel-info-trigger"
                    type="button"
                    aria-label="How AGC buys work"
                  >
                    i
                  </button>
                  <div className="panel-info-card" role="note">
                    Approve USDC once, then buy AGC through the canonical AGC/USDC pool.
                    Gross buys feed directly into the hook's demand tape.
                  </div>
                </div>
              </div>
            </div>

            <div className="field">
              <label className="field-label" htmlFor="buy-usdc-in">
                USDC amount
              </label>
              <input
                id="buy-usdc-in"
                className="field-input"
                value={buyUsdcAmount}
                onChange={(event) => setBuyUsdcAmount(event.target.value)}
                placeholder="10.0"
              />
            </div>

            <div className="field">
              <label className="field-label" htmlFor="buy-min-agc">
                Min AGC out
              </label>
              <input
                id="buy-min-agc"
                className="field-input"
                value={buyMinAgcOut}
                onChange={(event) => setBuyMinAgcOut(event.target.value)}
                placeholder="18.0"
              />
            </div>

            <div className="field">
              <label className="field-label" htmlFor="recipient">
                Recipient
              </label>
              <input
                id="recipient"
                className="field-input"
                value={recipient}
                onChange={(event) => setRecipient(event.target.value)}
                placeholder="0x..."
              />
            </div>

            <p className="panel-meta">
              wallet: <strong>{usdcBalance.data ? `${fmt6(usdcBalance.data)} USDC` : " - "}</strong>
            </p>

            <div className="panel-actions panel-actions-spacious">
              <button
                className="btn btn-secondary"
                disabled={!isConnected || !addresses.usdc || !addresses.settlementRouter}
                onClick={handleApproveUsdc}
              >
                Approve USDC
              </button>
              <button
                className="btn btn-primary"
                disabled={!isConnected || !ready}
                onClick={handleBuyAgc}
              >
                Buy AGC
              </button>
            </div>
          </div>

          <div className="panel">
            <div className="panel-header">
              <h3 className="panel-title">Sell AGC</h3>
              <div className="tx-status" data-status={txStatus === "Idle" ? "idle" : "confirming"}>
                <span className="tx-status-dot" />
                {txStatus}
              </div>
            </div>

            <div className="field">
              <label className="field-label" htmlFor="sell-agc-in">
                AGC amount
              </label>
              <input
                id="sell-agc-in"
                className="field-input"
                value={sellAgcAmount}
                onChange={(event) => setSellAgcAmount(event.target.value)}
                placeholder="20.0"
              />
            </div>

            <div className="field">
              <label className="field-label" htmlFor="sell-min-usdc">
                Min USDC out
              </label>
              <input
                id="sell-min-usdc"
                className="field-input"
                value={sellMinUsdcOut}
                onChange={(event) => setSellMinUsdcOut(event.target.value)}
                placeholder="9.9"
              />
            </div>

            <p className="panel-meta">
              wallet: <strong>{agcBalance.data ? `${fmt18(agcBalance.data, 2)} AGC` : " - "}</strong>
              {" / "}
              hook fees tracked: <strong>{fmtQuote(hookFeesQuote)}</strong>
            </p>

            <div className="panel-actions panel-actions-spacious">
              <button
                className="btn btn-secondary"
                disabled={!isConnected || !addresses.agc || !addresses.settlementRouter}
                onClick={handleApproveAgcForSell}
              >
                Approve AGC
              </button>
              <button
                className="btn btn-primary"
                disabled={!isConnected || !ready}
                onClick={handleSellAgc}
              >
                Sell AGC
              </button>
            </div>
          </div>

          <div className="panel">
            <div className="panel-header">
              <h3 className="panel-title">xAGC vault</h3>
              <span className="panel-badge">lock / redeem</span>
            </div>

            <div className="field">
              <label className="field-label" htmlFor="stake-agc">
                Deposit AGC
              </label>
              <input
                id="stake-agc"
                className="field-input"
                value={stakeAgcAmount}
                onChange={(event) => setStakeAgcAmount(event.target.value)}
                placeholder="50.0"
              />
            </div>

            <div className="field">
              <label className="field-label" htmlFor="redeem-xagc">
                Redeem xAGC shares
              </label>
              <input
                id="redeem-xagc"
                className="field-input"
                value={redeemXagcShares}
                onChange={(event) => setRedeemXagcShares(event.target.value)}
                placeholder="10.0"
              />
            </div>

            <p className="panel-meta">
              wallet: <strong>{xagcBalance.data ? `${fmt18(xagcBalance.data, 2)} xAGC` : " - "}</strong>
              {" / "}
              vault: <strong>{xagcTotalAssets.data ? `${fmt18(xagcTotalAssets.data, 2)} AGC` : " - "}</strong>
              {" / "}
              share px: <strong>{xagcExchangeRate ? `${fmt18(xagcExchangeRate, 4)} AGC` : " - "}</strong>
              {" / "}
              exit fee: <strong>{fmtBps(xagcExitFee.data)}</strong>
            </p>

            <p className="panel-meta">
              preview deposit: <strong>{previewDeposit.data ? `${fmt18(previewDeposit.data, 4)} xAGC` : " - "}</strong>
              {" / "}
              preview redeem: <strong>{previewRedeemNet ? `${fmt18(previewRedeemNet, 4)} AGC` : " - "}</strong>
              {" / "}
              fee: <strong>{previewRedeemFee ? `${fmt18(previewRedeemFee, 4)} AGC` : " - "}</strong>
            </p>

            <div className="panel-actions panel-actions-spacious">
              <button
                className="btn btn-secondary"
                disabled={!isConnected || !addresses.agc || !addresses.xagcVault}
                onClick={handleApproveAgcForVault}
              >
                Approve AGC For xAGC
              </button>
              <button
                className="btn btn-primary"
                disabled={!isConnected || !ready}
                onClick={handleDepositXagc}
              >
                Deposit AGC
              </button>
              <button
                className="btn btn-secondary"
                disabled={!isConnected || !ready}
                onClick={handleRedeemXagc}
              >
                Redeem xAGC
              </button>
            </div>
          </div>
        </div>
        {txNote && <p className="panel-meta">{txNote}</p>}
      </section>

      <section className="alarm-banner">
        <p>
          not a stablecoin / not a bank claim / not collateral theater / a
          machine-native credit loop welded directly into the exchange venue
        </p>
      </section>

      <section className="manifesto-grid">
        {manifestoCards.map((card, index) => (
          <article key={card.title} className="manifesto-card">
            <span className="manifesto-index">{`${index + 1}`.padStart(2, "0")}</span>
            <p className="manifesto-kicker">{card.kicker}</p>
            <h3 className="manifesto-title">{card.title}</h3>
            <p className="manifesto-text">{card.text}</p>
          </article>
        ))}
      </section>

      <section id="explainer" className="explainer-zone">
        <div className="dash-heading">
          <p className="section-kicker">scroll deeper</p>
          <h2 className="section-title">What, Why and the exact problem AGC solves</h2>
        </div>

        {explainerSections.map((section, index) => (
          <Fragment key={section.title}>
            <article className="explainer-panel">
              <div className="explainer-head">
                <div>
                  <p className="explainer-kicker">{section.kicker}</p>
                  <h3 className="explainer-title">{section.title}</h3>
                </div>
                <span className="explainer-tag">{`0${index + 1}`}</span>
              </div>

              <div className="explainer-grid">
                <div className="explainer-copy">
                  <p>{section.body}</p>
                  <p>{section.detail}</p>
                </div>

                <div className="explainer-side">
                  <pre className="mini-ascii">{section.ascii}</pre>
                  <ul className="explainer-list">
                    {section.bullets.map((bullet) => (
                      <li key={bullet}>{bullet}</li>
                    ))}
                  </ul>
                </div>
              </div>
            </article>

            {asciiInterludes
              .filter(
                (interlude) =>
                  interlude.label !== "credits roll" && interlude.after === index,
              )
              .map((interlude) => (
                <AsciiInterlude
                  key={interlude.label}
                  label={interlude.label}
                  caption={interlude.caption}
                  frames={interlude.frames}
                />
              ))}
          </Fragment>
        ))}
      </section>

      <AsciiVendingMachine />

      <section className="regime-theater">
        <div className="dash-heading">
          <p className="section-kicker">policy posture</p>
          <h2 className="section-title">
            The currency changes personality as conditions change
          </h2>
        </div>

        <div className="regime-grid">
          {regimeNarrative.map((item) => (
            <article key={item.name} className="regime-card">
              <p className="regime-card-name">{item.name}</p>
              <p className="regime-card-signal">{item.signal}</p>
              <p className="regime-card-text">{item.text}</p>
            </article>
          ))}
        </div>
      </section>

      <section className="flow-section">
        <div className="dash-heading">
          <p className="section-kicker">settlement path</p>
          <h2 className="section-title">
            The last-mile payment flow is the protocol thesis in one loop
          </h2>
        </div>

        <div className="flow-grid">
          {flowSteps.map((step) => (
            <article key={step.title} className="flow-card">
              <h3 className="flow-title">{step.title}</h3>
              <p className="flow-text">{step.text}</p>
            </article>
          ))}
        </div>
      </section>

      <section className="closing-panel">
        <pre className="closing-ascii">
          {String.raw`
AGC thesis:
  productive machine demand -> measured by hook + router flow
  measured flow             -> can justify bounded monetary expansion
  stress                    -> shuts mint, raises tolls, spends treasury
  settlement                -> still lands in USDC
          `}
        </pre>
        <p className="closing-copy">
          The entire protocol stands or falls on one question: can productive
          machine demand be measured well enough to justify elastic supply
          without opening the door to farming, spoofing, or reflexive collapse?
          Everything else is implementation detail.
        </p>
      </section>

      {finalAsciiInterlude && (
        <AsciiInterlude
          label={finalAsciiInterlude.label}
          caption={finalAsciiInterlude.caption}
          frames={finalAsciiInterlude.frames}
          backgroundVideoSrc="/ascii_video_sunset_colors_2x_720p.mp4"
        />
      )}

      <footer className="footer">
        <span className="footer-tag footer-left">
          {"hold AGC -> route through v4 -> settle in USDC"}
        </span>
        <div className="footer-right">
          <a
            className="footer-link"
            href="https://x.com"
            target="_blank"
            rel="noreferrer"
          >
            X
          </a>
          <a
            className="footer-link"
            href="https://github.com/c0rv0s/ACP"
            target="_blank"
            rel="noreferrer"
          >
            GitHub
          </a>
        </div>
      </footer>
    </main>
  );
}
