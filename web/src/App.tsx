import {
  Fragment,
  useEffect,
  useMemo,
  useState,
  type CSSProperties,
} from "react";
import { formatUnits, keccak256, parseUnits, stringToHex } from "viem";
import {
  useAccount,
  useConnect,
  useDisconnect,
  useReadContract,
  useWaitForTransactionReceipt,
  useWriteContract,
} from "wagmi";
import {
  addresses,
  agcAbi,
  policyControllerAbi,
  rewardDistributorAbi,
  settlementRouterAbi,
} from "./contracts";

const regimeLabels = ["Neutral", "Expansion", "Defense", "Recovery"] as const;
const regimeKeys = ["neutral", "expansion", "defense", "recovery"] as const;

const marqueeItems = [
  "FLOATING CREDIT",
  "SOFT ANCHOR",
  "NO REDEMPTION FICTION",
  "MACHINE GDP",
  "UNISWAP v4 HOOK",
  "LAST-MILE USDC",
  "WORKING CAPITAL FOR AGENTS",
];

const heroAscii = String.raw`
      ___    ______   ______      AGC / AGENT CREDIT
     /   |  / ____/  / ____/      HOLD -> ROUTE -> SETTLE
    / /| | / / __   / /          FLOATING CREDIT / SOFT ANCHOR
   / ___ |/ /_/ /  / /___        NOT A STABLECOIN
  /_/  |_|\____/   \____/         NO DOLLAR CLAIM

      [ AGENTS ] => [ AGC / USDC POOL ] => [ x402 + USDC ]
`;

const heroStickers = [
  "NO REDEMPTION FICTION",
  "x402 LAST MILE",
  "SOFT ANCHOR / HARD TOLLS",
  "MACHINE GDP FEVER",
  "HOOK SEES EVERYTHING",
  "FLOAT FIRST / SETTLE LATER",
  "USDC ONLY AT THE EDGE",
  "CREDIT WITH ATTITUDE",
  "ANTI-BANK-RUN POSTURE",
  "WORKING CAPITAL MAXXED",
  "PRODUCTIVE FLOW ONLY",
  "BUYBACK THE PANIC",
  "TWAP DREAM LOGIC",
  "EXIT TOLLS GET WEIRD",
  "DOLLAR LEGIBLE / AGENT NATIVE",
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
      "AGC lives inside a canonical AGC/USDC Uniswap v4 pool with a dedicated hook. Fees, flow tagging, stress tolls, reward receipts, and oracle observations all happen at the venue where the currency actually clears.",
  },
  {
    kicker: "The point",
    title: "USDC is the settlement rail. AGC is the elastic transaction inventory.",
    text:
      "Merchants can stay legible in USDC while agents carry AGC between purchases. That lets the protocol create machine-native purchasing power without pretending every unit is a dollar IOU.",
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
      "Supply expands and contracts through policy, buybacks, fees, and reward streams",
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
      "On the fast path, the hook classifies flows, adjusts LP fees, charges hook fees, records productive receipts, updates epoch counters, and penalizes short-lived liquidity. On the slow path, the controller settles epochs, enforces mint and buyback caps, sets regime state, routes reward budgets, and triggers treasury actions through the canonical settlement router.",
    detail:
      "The launch architecture keeps that split intentionally conservative. The hook accumulates the market data. The controller validates bounded policy actions against hard guardrails. The monetary loop stays disciplined on purpose so the network can expand only when productive demand is actually showing up in the flow.",
    bullets: [
      "beforeSwap / afterSwap dynamic fee surface",
      "Oracle-style epoch snapshots from hook data",
      "Bounded policy settlement with hard caps and cooldowns",
    ],
    ascii: String.raw`
swap -> classify -> fee -> observe -> receipt
epoch -> settle -> mint/buyback -> stream rewards
`,
  },
  {
    kicker: "Why agents hold it",
    title: "AGC is useful when expected rebates, network upside, and working-capital convenience beat the friction of staying fully in USDC.",
    body:
      "USDC is an inert settlement asset. AGC is designed to be active monetary inventory. Productive routing can earn streamed rewards, durable liquidity can be subsidized, and future sinks can create utility that only exists inside the protocol. The result is a reason to hold AGC between purchases rather than only touching it at the exact instant of settlement.",
    detail:
      "The protocol only works if that economics is real. If rewards do not offset risk and conversion cost, agents will just hold USDC. The whole protocol is effectively a bet that machine-native incentives can make elastic credit inventory attractive without promising redemption.",
    bullets: [
      "Lower effective payment cost through productive rebates",
      "Monetary upside from network growth instead of inert balances",
      "Future sinks: discounts, priority, reputation, partner access",
    ],
    ascii: String.raw`
HOLD AGC      -> earn and route
SWAP TO USDC  -> only when the invoice hits
`,
  },
  {
    kicker: "Risk controls",
    title: "When stress rises, the system is supposed to get mean, not pretend everything is fine.",
    body:
      "Defense mode is the anti-bank-run posture. Issuance stops. Exit fees rise. Treasury USDC can be spent on buybacks. The band can widen. Mercenary flow becomes more expensive. The point is not to freeze users; it is to preserve utility long enough that the currency can survive reflexive sell pressure.",
    detail:
      "This is why the protocol narrative has to stay honest. There is no redemption guarantee hiding under the hood. The safety story is dynamic policy, disciplined caps, durable liquidity incentives, and a treasury that can spend into disorder.",
    bullets: [
      "No new growth mint while weak",
      "Defense-only buybacks through the router",
      "Anti-JIT liquidity fees and trusted-router gating",
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
                     ||        ||------> USDC
                     ||------> receipts
`,
      String.raw`
   .         .       .        .       .        .
 AGC o    AGC o   AGC o    AGC o   AGC o    AGC o
      \       \      \      |      /       /
       \       \      \     |     /       /
        \       \      \    |    /       /
       ~ ~ ~ ~ ~ [ UNISWAP v4 POOL ] ~ ~ ~ ~ ~
                      ||        ||
                   receipts     ||----------> USDC
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
                    ||------> USDC out
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
      "The hook is the surveillance camera, toll booth, receipt printer, and weird little arcade cabinet welded into the market itself.",
    frames: [
      String.raw`
[ beforeSwap ] --> [ fee ] --> [ classify ] --> [ vibes ]
                         || 
                         \/
                  .---------------.
                  |   afterSwap   |
                  |  oracle tape  |
                  | reward receipt |
                  '---------------'
                         ||
                      receipt *
`,
      String.raw`
[ beforeSwap ] ==> [ fee ] ==> [ classify ] ==> [ vibes ]
                          ||
                          \/
                   .---------------.
                   |   afterSwap   |
                   |  oracle tape  |
                   | reward receipt |
                   '---------------'
                          ||
                       receipt **
`,
      String.raw`
[ beforeSwap ] --> [ fee ] --> [ classify ] --> [ vibes ]
                         ||
                         \/
                  .---------------.
                  |   afterSwap   |
                  |  oracle tape  |
                  | reward receipt |
                  '---------------'
                         ||
                      receipt ***
`,
      String.raw`
[ beforeSwap ] ==> [ fee ] ==> [ classify ] ==> [ vibes ]
                          ||
                          \/
                   .---------------.
                   |   afterSwap   |
                   |  oracle tape  |
                   | reward receipt |
                   '---------------'
                          ||
                       receipt ****
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
    |  routing receipts all night   | /
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

      AGC >>> USDC >>> RECEIPTS >>> GLOW
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

      GLOW >>> RECEIPTS >>> USDC >>> AGC
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

      RECEIPTS >>> GLOW >>> AGC >>> USDC
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
      "Productive demand is healthy, liquidity is deep enough, volatility is controlled, and exits are not dominating the tape. New supply can be streamed into agents, LPs, integrators, treasury, and reserve buckets.",
  },
  {
    name: "Neutral",
    signal: "hold posture",
    text:
      "The protocol keeps the machine running without discretionary growth. Previously funded streams continue, but the system does not spray new credit just because price held together for a single epoch.",
  },
  {
    name: "Defense",
    signal: "stress tolls",
    text:
      "Price is weak or stress metrics are flashing red. Mint shuts off, exits get more expensive, and treasury USDC is available for buybacks. The protocol spends stored strength to defend utility.",
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
    title: "2. Hit the paid endpoint",
    text:
      "The merchant or service still wants x402-style stablecoin settlement, because that is what accounting and receipts understand.",
  },
  {
    title: "3. Route through SettlementRouter",
    text:
      "The trusted router packages metadata for the hook, recovers the original user identity, and swaps through the canonical AGC/USDC pool.",
  },
  {
    title: "4. Settle in USDC",
    text:
      "USDC leaves the router to the recipient. The merchant sees the settlement asset they wanted. The agent spent AGC inventory rather than warehousing only USDC.",
  },
  {
    title: "5. Record productive flow",
    text:
      "The hook records productive volume, updates epoch counters, and mints a receipt instead of spraying rewards inline into the swap path.",
  },
  {
    title: "6. Stream rewards later",
    text:
      "The distributor converts receipts into time-vested AGC streams next epoch so incentives land gradually rather than detonating the market immediately.",
  },
] as const;

type CursorStamp = {
  id: number;
  x: number;
  y: number;
  hue: number;
  scale: number;
  rotation: number;
};

function fmt18(v: bigint | undefined, decimals = 4): string {
  if (v === undefined) return " - ";
  return Number(formatUnits(v, 18)).toFixed(decimals);
}

function fmtBps(v: bigint | undefined): string {
  if (v === undefined) return " - ";
  return `${(Number(v) / 100).toFixed(2)}%`;
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
}: {
  label: string;
  caption: string;
  frames: readonly string[];
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
      <div className="ascii-interlude-top">
        <p className="ascii-interlude-label">{label}</p>
        <span className="ascii-interlude-pulse" />
      </div>
      <pre className="ascii-interlude-frame" aria-hidden="true">
        {frames[frameIndex]}
      </pre>
      <p className="ascii-interlude-caption">{caption}</p>
    </section>
  );
}

export default function App() {
  const { address, isConnected } = useAccount();
  const { connect, connectors } = useConnect();
  const { disconnect } = useDisconnect();
  const { writeContractAsync, data: hash } = useWriteContract();
  const { isLoading: isConfirming } = useWaitForTransactionReceipt({ hash });

  const [streamId, setStreamId] = useState("1");
  const [paymentAmount, setPaymentAmount] = useState("10");
  const [minUsdcOut, setMinUsdcOut] = useState("9.9");
  const [recipient, setRecipient] = useState(
    "0x000000000000000000000000000000000000dEaD",
  );

  const ready =
    addresses.agc &&
    addresses.policyController &&
    addresses.rewardDistributor &&
    addresses.settlementRouter;

  const enabled = (addressValue: string | undefined) => ({
    query: { enabled: Boolean(addressValue) },
  });

  const anchor = useReadContract({
    address: addresses.policyController,
    abi: policyControllerAbi,
    functionName: "anchorPriceX18",
    ...enabled(addresses.policyController),
  });
  const band = useReadContract({
    address: addresses.policyController,
    abi: policyControllerAbi,
    functionName: "bandWidthBps",
    ...enabled(addresses.policyController),
  });
  const regime = useReadContract({
    address: addresses.policyController,
    abi: policyControllerAbi,
    functionName: "regime",
    ...enabled(addresses.policyController),
  });
  const productiveUsage = useReadContract({
    address: addresses.policyController,
    abi: policyControllerAbi,
    functionName: "lastProductiveUsageBps",
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
  const claimable = useReadContract({
    address: addresses.rewardDistributor,
    abi: rewardDistributorAbi,
    functionName: "previewClaimable",
    args: [BigInt(streamId || "0")],
    query: {
      enabled: Boolean(addresses.rewardDistributor) && Boolean(streamId),
    },
  });
  const balance = useReadContract({
    address: addresses.agc,
    abi: agcAbi,
    functionName: "balanceOf",
    args: address ? [address] : undefined,
    query: { enabled: Boolean(addresses.agc && address) },
  });

  const regimeIdx =
    typeof regime.data === "number" ? regime.data : undefined;
  const regimeKey =
    regimeIdx !== undefined ? regimeKeys[regimeIdx] ?? "neutral" : "neutral";
  const regimeLabel =
    regimeIdx !== undefined ? regimeLabels[regimeIdx] ?? "Unknown" : " - ";

  const status = useMemo(() => {
    if (isConfirming) return "confirming" as const;
    if (hash) return "submitted" as const;
    return "idle" as const;
  }, [hash, isConfirming]);

  const statusLabel = {
    idle: "Idle",
    confirming: "Confirming...",
    submitted: "Submitted",
  }[status];

  const shortAddr = address
    ? `${address.slice(0, 6)}...${address.slice(-4)}`
    : null;

  async function approveAndPay() {
    if (!ready || !address || !addresses.agc || !addresses.settlementRouter) {
      return;
    }

    const amount = parseUnits(paymentAmount, 18);
    const minOut = parseUnits(minUsdcOut, 6);
    const paymentId = keccak256(stringToHex(`${address}:${Date.now()}`));

    await writeContractAsync({
      address: addresses.agc,
      abi: agcAbi,
      functionName: "approve",
      args: [addresses.settlementRouter, amount],
    });

    await writeContractAsync({
      address: addresses.settlementRouter,
      abi: settlementRouterAbi,
      functionName: "settlePayment",
      args: [amount, minOut, recipient as `0x${string}`, paymentId, 10_000],
    });
  }

  async function handleClaim() {
    if (!addresses.rewardDistributor) return;

    await writeContractAsync({
      address: addresses.rewardDistributor,
      abi: rewardDistributorAbi,
      functionName: "claimStream",
      args: [BigInt(streamId)],
    });
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
            Hold <code>AGC</code> as machine-native inventory. Convert to{" "}
            <code>USDC</code> only when the payment actually fires. Let the
            hook decide fees, the controller decide posture, and the treasury
            spend into stress instead of pretending every token is a warehouse
            receipt for dollars.
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
          Protocol telemetry and settlement actions will light up once the live
          deployment is connected.
        </div>
      )}

      <section id="dashboard" className="dashboard-zone">
        <div className="dash-heading">
          <p className="section-kicker">live operator surface</p>
          <h2 className="section-title">Dashboard / policy telemetry / payment path</h2>
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
            <span className="regime-stat-label">band</span>
            <span className="regime-stat-value">{fmtBps(band.data)}</span>
          </div>
          <div className="regime-divider" />
          <div className="regime-stat">
            <span className="regime-stat-label">wallet balance</span>
            <span className="regime-stat-value">
              {balance.data ? `${fmt18(balance.data, 2)} AGC` : " - "}
            </span>
          </div>
        </div>

        <div className="metrics">
          <div className="metric">
            <span className="metric-label">Anchor price</span>
            <span className="metric-value">
              {anchor.data ? `$${fmt18(anchor.data)}` : " - "}
            </span>
            <span className="metric-hint">crawling soft anchor</span>
          </div>
          <div className="metric">
            <span className="metric-label">Band width</span>
            <span className="metric-value">{fmtBps(band.data)}</span>
            <span className="metric-hint">policy half-width around anchor</span>
          </div>
          <div className="metric">
            <span className="metric-label">Productive usage</span>
            <span className="metric-value">{fmtBps(productiveUsage.data)}</span>
            <span className="metric-hint">payment volume share</span>
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
            <span className="metric-label">Regime</span>
            <span className="metric-value">{regimeLabel}</span>
            <span className="metric-hint">current monetary posture</span>
          </div>
          <div className="metric">
            <span className="metric-label">AGC balance</span>
            <span className="metric-value">
              {balance.data ? fmt18(balance.data, 2) : "0.00"}
            </span>
            <span className="metric-hint">connected wallet inventory</span>
          </div>
        </div>

        <div className="panels">
          <div className="panel">
            <div className="panel-header">
              <h3 className="panel-title">Claim reward stream</h3>
              <span className="panel-badge">vested emissions</span>
            </div>

            <div className="field">
              <label className="field-label" htmlFor="stream-id">
                Stream ID
              </label>
              <input
                id="stream-id"
                className="field-input"
                value={streamId}
                onChange={(event) => setStreamId(event.target.value)}
                placeholder="0"
              />
            </div>

            <p className="panel-meta">
              claimable now:{" "}
              <strong>
                {claimable.data ? fmt18(claimable.data) : "0.0000"} AGC
              </strong>
            </p>

            <div className="panel-actions">
              <button
                className="btn btn-primary"
                disabled={!isConnected || !addresses.rewardDistributor}
                onClick={handleClaim}
              >
                Claim vested AGC
              </button>
            </div>
          </div>

          <div className="panel">
            <div className="panel-header">
              <h3 className="panel-title">Settle payment</h3>
              <div className="tx-status" data-status={status}>
                <span className="tx-status-dot" />
                {statusLabel}
              </div>
            </div>

            <div className="field">
              <label className="field-label" htmlFor="agc-in">
                AGC amount
              </label>
              <input
                id="agc-in"
                className="field-input"
                value={paymentAmount}
                onChange={(event) => setPaymentAmount(event.target.value)}
                placeholder="10.0"
              />
            </div>

            <div className="field">
              <label className="field-label" htmlFor="min-usdc">
                Min USDC out
              </label>
              <input
                id="min-usdc"
                className="field-input"
                value={minUsdcOut}
                onChange={(event) => setMinUsdcOut(event.target.value)}
                placeholder="9.9"
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

            <div className="panel-actions">
              <button
                className="btn btn-primary"
                disabled={!isConnected || !ready}
                onClick={approveAndPay}
              >
                Approve + settle
              </button>
            </div>
          </div>
        </div>
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
