import { useEffect, useRef, useState, type TouchEvent, type WheelEvent } from "react";
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

const docsHref = "https://github.com/c0rv0s/ACP";

const dashboardNavItems = [
  { label: "Landing", href: "/" },
  { label: "Telemetry", href: "#telemetry" },
  { label: "Market", href: "#market-desk" },
  { label: "Policy", href: "#policy" },
  { label: "Docs", href: docsHref },
] as const;

const landingSections = [
  {
    id: "credit",
    video: {
      desktopMp4: "/art-deco/statue_city-loop-1080.mp4",
      mobileMp4: "/art-deco/statue_city-loop-720.mp4",
    },
    poster: "/art-deco/statue_city_poster.jpg",
    eyebrow: "Agent Credit Protocol",
    title: "Credit for autonomous markets.",
    text:
      "AGC is working capital for software systems: liquid enough to move through markets, disciplined enough to be governed by policy, and connected to USDC where final settlement needs a reserve rail.",
    align: "left",
    stats: ["AGC inventory", "USDC settlement", "xAGC duration"],
  },
  {
    id: "problem",
    video: {
      desktopMp4: "/art-deco/city_orbit-loop-1080.mp4",
      mobileMp4: "/art-deco/city_orbit-loop-720.mp4",
    },
    poster: "/art-deco/city_orbit_poster.jpg",
    eyebrow: "The gap",
    title: "Autonomous demand needs balance sheet motion.",
    text:
      "Static dollar inventory is a poor fit for agents that buy compute, APIs, data, execution, and bandwidth in real time. AGC turns productive market activity into a measured credit surface instead of forcing every unit to be a passive receipt.",
    align: "right",
    stats: ["Demand measured at the venue", "Stress visible in flow", "Expansion bounded by rules"],
  },
  {
    id: "mechanism",
    video: {
      desktopMp4: "/art-deco/statue_orbit-loop-1080.mp4",
      mobileMp4: "/art-deco/statue_orbit-loop-720.mp4",
    },
    poster: "/art-deco/statue_orbit_poster.jpg",
    eyebrow: "Mechanism",
    title: "The market is the policy surface.",
    text:
      "A canonical AGC/USDC pool records buys, sells, volatility, fees, and exit pressure. The controller settles that tape into regime changes, issuance limits, vault flow, and treasury defense.",
    align: "left",
    stats: ["Hook observes", "Controller settles", "Treasury defends"],
  },
] as const;

const footerVideo = {
  desktopMp4: "/art-deco/statue_orbit_pillars-loop-1080.mp4",
  mobileMp4: "/art-deco/statue_orbit_pillars-loop-720.mp4",
} as const;

const architectureSteps = [
  {
    step: "01",
    title: "Hold",
    text:
      "Operators and agents hold AGC as liquid inventory for near-term autonomous execution.",
  },
  {
    step: "02",
    title: "Clear",
    text:
      "Inventory is bought and sold through the canonical pool, where the hook measures demand and withdrawal pressure.",
  },
  {
    step: "03",
    title: "Lock",
    text:
      "Longer-duration holders deposit AGC into xAGC to participate in the savings layer and absorb expansion flow.",
  },
  {
    step: "04",
    title: "Govern",
    text:
      "The controller settles each epoch, keeps issuance within hard caps, and moves the regime when conditions change.",
  },
] as const;

const regimeNarrative = [
  {
    name: "Expansion",
    signal: "Measured growth",
    text:
      "Supply can expand only when demand, liquidity depth, volatility, and exit pressure remain inside configured bounds.",
  },
  {
    name: "Neutral",
    signal: "Hold posture",
    text:
      "The protocol continues measuring the market while withholding discretionary growth.",
  },
  {
    name: "Defense",
    signal: "Treasury response",
    text:
      "Issuance halts, costs rise, and queued buybacks can spend reserve strength back into disorder.",
  },
  {
    name: "Recovery",
    signal: "Cooldown",
    text:
      "The system rebuilds credibility before returning to a growth posture.",
  },
] as const;

const DEFAULT_RECIPIENT = "0x000000000000000000000000000000000000dEaD";
const MAX_UINT256 = (1n << 256n) - 1n;

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

function Field({
  id,
  label,
  value,
  placeholder,
  onChange,
}: {
  id: string;
  label: string;
  value: string;
  placeholder: string;
  onChange: (value: string) => void;
}) {
  return (
    <div className="field">
      <label className="field-label" htmlFor={id}>
        {label}
      </label>
      <input
        id={id}
        className="field-input"
        value={value}
        onChange={(event) => onChange(event.target.value)}
        placeholder={placeholder}
      />
    </div>
  );
}

function CrossfadeVideo({
  sources,
  poster,
  shouldLoad,
  preload,
  playbackRate = 1,
}: {
  sources: { desktopMp4: string; mobileMp4: string };
  poster: string;
  shouldLoad: boolean;
  preload: "auto" | "metadata";
  playbackRate?: number;
}) {
  const [activeSlot, setActiveSlot] = useState<0 | 1>(0);
  const firstVideoRef = useRef<HTMLVideoElement | null>(null);
  const secondVideoRef = useRef<HTMLVideoElement | null>(null);
  const isCrossfadingRef = useRef(false);
  const fadeDurationMs = 1200;
  const fadeLeadSeconds = 1.2;

  function videoForSlot(slot: 0 | 1) {
    return slot === 0 ? firstVideoRef.current : secondVideoRef.current;
  }

  function prepareVideo(video: HTMLVideoElement | null) {
    if (!video) return;
    video.playbackRate = playbackRate;
  }

  function startCrossfade(fromSlot: 0 | 1) {
    if (isCrossfadingRef.current || !shouldLoad) return;

    const fromVideo = videoForSlot(fromSlot);
    const nextSlot = fromSlot === 0 ? 1 : 0;
    const nextVideo = videoForSlot(nextSlot);
    if (!fromVideo || !nextVideo) return;

    isCrossfadingRef.current = true;
    nextVideo.currentTime = 0;
    nextVideo.playbackRate = playbackRate;
    void nextVideo.play();
    setActiveSlot(nextSlot);

    window.setTimeout(() => {
      fromVideo.pause();
      fromVideo.currentTime = 0;
      isCrossfadingRef.current = false;
    }, fadeDurationMs);
  }

  function handleTimeUpdate(slot: 0 | 1) {
    if (slot !== activeSlot) return;
    const video = videoForSlot(slot);
    if (!video || !Number.isFinite(video.duration)) return;
    if (video.duration - video.currentTime <= fadeLeadSeconds) {
      startCrossfade(slot);
    }
  }

  useEffect(() => {
    prepareVideo(firstVideoRef.current);
    prepareVideo(secondVideoRef.current);
  }, [playbackRate]);

  return (
    <div className="cinema-video-stack" aria-hidden="true">
      {[0, 1].map((slot) => (
        <video
          key={slot}
          ref={(video) => {
            if (slot === 0) {
              firstVideoRef.current = video;
            } else {
              secondVideoRef.current = video;
            }
            prepareVideo(video);
          }}
          className={`cinema-media cinema-video ${
            activeSlot === slot ? "is-visible" : "is-hidden"
          }`}
          autoPlay={slot === 0}
          muted
          playsInline
          poster={poster}
          preload={preload}
          onTimeUpdate={() => handleTimeUpdate(slot as 0 | 1)}
          onEnded={() => startCrossfade(slot as 0 | 1)}
        >
          {shouldLoad ? (
            <>
              <source media="(max-width: 820px)" src={sources.mobileMp4} type="video/mp4" />
              <source src={sources.desktopMp4} type="video/mp4" />
            </>
          ) : null}
        </video>
      ))}
    </div>
  );
}

function LandingPage() {
  const [activeScene, setActiveScene] = useState(0);
  const transitionLockRef = useRef(false);
  const touchStartYRef = useRef<number | null>(null);
  const sceneCount = landingSections.length + 1;

  function releaseTransitionLock() {
    window.setTimeout(() => {
      transitionLockRef.current = false;
    }, 850);
  }

  function goToScene(nextScene: number) {
    const boundedScene = Math.max(0, Math.min(sceneCount - 1, nextScene));

    setActiveScene((currentScene) => {
      if (currentScene === boundedScene) return currentScene;
      transitionLockRef.current = true;
      releaseTransitionLock();
      return boundedScene;
    });
  }

  function stepScene(direction: 1 | -1) {
    if (transitionLockRef.current) return;
    goToScene(activeScene + direction);
  }

  function handleWheel(event: WheelEvent<HTMLElement>) {
    event.preventDefault();
    if (Math.abs(event.deltaY) < 18) return;
    stepScene(event.deltaY > 0 ? 1 : -1);
  }

  function handleTouchStart(event: TouchEvent<HTMLElement>) {
    touchStartYRef.current = event.touches[0]?.clientY ?? null;
  }

  function handleTouchEnd(event: TouchEvent<HTMLElement>) {
    const startY = touchStartYRef.current;
    const endY = event.changedTouches[0]?.clientY;
    touchStartYRef.current = null;
    if (startY === null || endY === undefined) return;

    const deltaY = startY - endY;
    if (Math.abs(deltaY) < 42) return;
    stepScene(deltaY > 0 ? 1 : -1);
  }

  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      if (["ArrowDown", "PageDown", " "].includes(event.key)) {
        event.preventDefault();
        stepScene(1);
      }
      if (["ArrowUp", "PageUp"].includes(event.key)) {
        event.preventDefault();
        stepScene(-1);
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [activeScene]);

  return (
    <main
      className="landing-page"
      onWheel={handleWheel}
      onTouchStart={handleTouchStart}
      onTouchEnd={handleTouchEnd}
    >
      <div className="scroll-progress" aria-label="Landing sections">
        {landingSections.map((section, index) => (
          <button
            key={section.id}
            className={activeScene === index ? "is-active" : ""}
            type="button"
            aria-label={section.title}
            aria-current={activeScene === index ? "step" : undefined}
            onClick={() => goToScene(index)}
          />
        ))}
        <button
          className={activeScene === landingSections.length ? "is-active" : ""}
          type="button"
          aria-label="Footer"
          aria-current={activeScene === landingSections.length ? "step" : undefined}
          onClick={() => goToScene(landingSections.length)}
        />
      </div>

      {landingSections.map((section, index) => {
        const shouldLoadVideo = Math.abs(activeScene - index) <= 1;

        return (
          <section
            key={section.id}
            id={section.id}
            className={`cinema-section cinema-${section.align} ${
              activeScene === index ? "is-active" : activeScene > index ? "is-before" : "is-after"
            }`}
            aria-hidden={activeScene !== index}
          >
            <img
              className="cinema-poster"
              src={section.poster}
              alt=""
              aria-hidden="true"
            />
            <CrossfadeVideo
              sources={section.video}
              poster={section.poster}
              shouldLoad={shouldLoadVideo}
              preload={activeScene === index ? "auto" : "metadata"}
            />
          <div className="cinema-vignette" aria-hidden="true" />
          <div className="cinema-content">
            <p className="landing-eyebrow">{section.eyebrow}</p>
            <h1>{section.title}</h1>
            <p>{section.text}</p>
            <div className="landing-pillars" aria-label="Protocol pillars">
              {section.stats.map((stat) => (
                <span key={stat}>{stat}</span>
              ))}
            </div>
            {index === 0 ? (
              <div className="landing-actions">
                <a className="landing-button landing-button-primary" href="/dashboard">
                  Open Dashboard
                </a>
                <a className="landing-button" href={docsHref}>
                  Read Docs
                </a>
              </div>
            ) : null}
          </div>
          </section>
        );
      })}

      <section
        id="footer"
        className={`cinema-section footer-cinema ${
          activeScene === landingSections.length ? "is-active" : "is-after"
        }`}
        aria-hidden={activeScene !== landingSections.length}
      >
        <img
          className="cinema-poster"
          src="/art-deco/statue_orbit_pillars_poster.jpg"
          alt=""
          aria-hidden="true"
        />
        <CrossfadeVideo
          sources={footerVideo}
          poster="/art-deco/statue_orbit_pillars_poster.jpg"
          shouldLoad={activeScene >= landingSections.length - 1}
          preload={activeScene === landingSections.length ? "auto" : "metadata"}
        />
        <div className="cinema-vignette" aria-hidden="true" />
        <div className="footer-cinema-content">
          <p className="landing-eyebrow">Why it matters</p>
          <h2>Useful credit becomes infrastructure when issuance, liquidity, and defense share one rulebook.</h2>
          <p>
            AGC is built to keep autonomous commerce liquid without hiding the
            policy surface. The full mechanics live in the dashboard and docs.
          </p>
          <div className="footer-link-row">
            <a href="/dashboard">Dashboard</a>
            <a href={docsHref}>GitHub / Docs</a>
            <a href="https://x.com">X</a>
          </div>
        </div>
      </section>
    </main>
  );
}

function DashboardPage() {
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

  const ready = Boolean(
    addresses.agc &&
      addresses.usdc &&
      addresses.hook &&
      addresses.policyController &&
      addresses.settlementRouter &&
      addresses.treasuryVault &&
      addresses.xagcVault,
  );

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

  const telemetry = [
    {
      label: "Regime",
      value: regimeLabel,
      detail: "policy posture",
    },
    {
      label: "Anchor",
      value: anchor.data ? `$${fmt18(anchor.data)}` : " - ",
      detail: "soft reference",
    },
    {
      label: "Coverage",
      value: fmtBps(coverage.data),
      detail: "reserve depth",
    },
    {
      label: "Premium",
      value: fmtBps(premium.data),
      detail: "seven-day state",
    },
    {
      label: "Treasury",
      value: treasuryUsdc.data ? `${fmt6(treasuryUsdc.data)} USDC` : " - ",
      detail: "available reserve",
    },
    {
      label: "Epoch",
      value: currentEpochId.data ? `${currentEpochId.data}` : " - ",
      detail: "hook tape",
    },
  ];

  const operatingMetrics = [
    ["Exit pressure", fmtBps(exitPressure.data)],
    ["Volatility", fmtBps(volatility.data)],
    ["Locked share", fmtBps(lockedShare.data)],
    ["Lock flow", fmtBps(lockFlow.data)],
    ["Gross buys", fmtQuote(grossBuyVolume)],
    ["Gross sells", fmtQuote(grossSellVolume)],
    ["Hook fees", fmtQuote(hookFeesQuote)],
    [
      "Pending buyback",
      pendingBuyback.data ? `${fmt6(pendingBuyback.data)} USDC` : " - ",
    ],
  ] as const;

  const txState =
    txStatus === "Idle" ? "idle" : txStatus.includes("complete") ? "complete" : "active";

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
      "Approving AGC",
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
      "Approving vault access",
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
      "Depositing xAGC",
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
    <main className="site dashboard-page" data-regime={regimeKey}>
      <div className="ambient-grid" aria-hidden="true" />

      <header className="dashboard-topbar">
        <a className="brand" href="/" aria-label="Agent Credit Protocol home">
          <span className="brand-mark">
            <img src="/agc-mark.svg" alt="" />
          </span>
          <span className="brand-copy">
            <span className="brand-name">Agent Credit Protocol</span>
            <span className="brand-subtitle">Autonomous credit infrastructure</span>
          </span>
        </a>

        <nav className="nav" aria-label="Dashboard navigation">
          {dashboardNavItems.map((item) => (
            <a key={item.label} href={item.href}>
              {item.label}
            </a>
          ))}
        </nav>

        <div className="wallet">
          {isConnected ? (
            <>
              <span className="wallet-address">{shortAddr}</span>
              <button className="button button-secondary" onClick={() => disconnect()}>
                Disconnect
              </button>
            </>
          ) : (
            <button
              className="button button-primary"
              disabled={connectors.length === 0}
              onClick={() => connect({ connector: connectors[0] })}
            >
              Connect Wallet
            </button>
          )}
        </div>
      </header>

      <section className="dashboard-hero">
        <p className="eyebrow">Operator dashboard</p>
        <h1>Protocol telemetry and market controls.</h1>
        <p>
          This surface is for live interaction: wallet state, policy readings,
          canonical AGC swaps, and xAGC vault operations.
        </p>
      </section>

      <section id="telemetry" className="telemetry-band dashboard-telemetry" aria-label="Protocol telemetry">
        {telemetry.map((metric) => (
          <article key={metric.label} className="telemetry-item">
            <span className="metric-label">{metric.label}</span>
            <strong>{metric.value}</strong>
            <span>{metric.detail}</span>
          </article>
        ))}
      </section>

      {!ready && (
        <section className="notice">
          <strong>Deployment addresses are not configured in this environment.</strong>
          <span>
            Live telemetry and wallet operations will populate after the v1
            contract addresses are connected.
          </span>
        </section>
      )}

      <section id="market-desk" className="section two-column">
        <div className="section-heading">
          <p className="eyebrow">Operator console</p>
          <h2>Trade inventory, lock duration, and watch policy state in one place.</h2>
          <p>
            Canonical swaps, vault duration, and regime telemetry share one
            operating context for measured credit activity.
          </p>
        </div>

        <div className="market-console">
          <div className="status-rail">
            <span className="status-dot" />
            <span className="status-label">Transaction status</span>
            <strong>{txStatus}</strong>
          </div>

          <div className="operation-grid">
            <article className="operation-panel">
              <div className="panel-header">
                <span className="card-label">Router swap</span>
                <h3>Buy AGC</h3>
              </div>
              <Field
                id="buy-usdc-in"
                label="USDC amount"
                value={buyUsdcAmount}
                onChange={setBuyUsdcAmount}
                placeholder="10.0"
              />
              <Field
                id="buy-min-agc"
                label="Minimum AGC out"
                value={buyMinAgcOut}
                onChange={setBuyMinAgcOut}
                placeholder="18.0"
              />
              <Field
                id="recipient"
                label="Recipient"
                value={recipient}
                onChange={setRecipient}
                placeholder="0x..."
              />
              <p className="panel-meta">
                Wallet balance: <strong>{usdcBalance.data ? `${fmt6(usdcBalance.data)} USDC` : " - "}</strong>
              </p>
              <div className="panel-actions">
                <button
                  className="button button-secondary"
                  disabled={!isConnected || !addresses.usdc || !addresses.settlementRouter}
                  onClick={handleApproveUsdc}
                >
                  Approve USDC
                </button>
                <button
                  className="button button-primary"
                  disabled={!isConnected || !ready}
                  onClick={handleBuyAgc}
                >
                  Buy AGC
                </button>
              </div>
            </article>

            <article className="operation-panel">
              <div className="panel-header">
                <span className="card-label">Canonical exit</span>
                <h3>Sell AGC</h3>
              </div>
              <Field
                id="sell-agc-in"
                label="AGC amount"
                value={sellAgcAmount}
                onChange={setSellAgcAmount}
                placeholder="20.0"
              />
              <Field
                id="sell-min-usdc"
                label="Minimum USDC out"
                value={sellMinUsdcOut}
                onChange={setSellMinUsdcOut}
                placeholder="9.9"
              />
              <p className="panel-meta">
                Wallet balance: <strong>{agcBalance.data ? `${fmt18(agcBalance.data, 2)} AGC` : " - "}</strong>
              </p>
              <p className="panel-meta">
                Hook fees tracked: <strong>{fmtQuote(hookFeesQuote)}</strong>
              </p>
              <div className="panel-actions">
                <button
                  className="button button-secondary"
                  disabled={!isConnected || !addresses.agc || !addresses.settlementRouter}
                  onClick={handleApproveAgcForSell}
                >
                  Approve AGC
                </button>
                <button
                  className="button button-primary"
                  disabled={!isConnected || !ready}
                  onClick={handleSellAgc}
                >
                  Sell AGC
                </button>
              </div>
            </article>

            <article id="vaults" className="operation-panel operation-panel-wide">
              <div className="panel-header">
                <span className="card-label">Savings layer</span>
                <h3>xAGC vault</h3>
              </div>
              <div className="field-pair">
                <Field
                  id="stake-agc"
                  label="Deposit AGC"
                  value={stakeAgcAmount}
                  onChange={setStakeAgcAmount}
                  placeholder="50.0"
                />
                <Field
                  id="redeem-xagc"
                  label="Redeem xAGC shares"
                  value={redeemXagcShares}
                  onChange={setRedeemXagcShares}
                  placeholder="10.0"
                />
              </div>
              <div className="vault-metrics">
                <span>Wallet: <strong>{xagcBalance.data ? `${fmt18(xagcBalance.data, 2)} xAGC` : " - "}</strong></span>
                <span>Vault: <strong>{xagcTotalAssets.data ? `${fmt18(xagcTotalAssets.data, 2)} AGC` : " - "}</strong></span>
                <span>Share px: <strong>{xagcExchangeRate ? `${fmt18(xagcExchangeRate, 4)} AGC` : " - "}</strong></span>
                <span>Exit fee: <strong>{fmtBps(xagcExitFee.data)}</strong></span>
                <span>Deposit preview: <strong>{previewDeposit.data ? `${fmt18(previewDeposit.data, 4)} xAGC` : " - "}</strong></span>
                <span>Redeem preview: <strong>{previewRedeemNet ? `${fmt18(previewRedeemNet, 4)} AGC` : " - "}</strong></span>
                <span>Redeem fee: <strong>{previewRedeemFee ? `${fmt18(previewRedeemFee, 4)} AGC` : " - "}</strong></span>
              </div>
              <div className="panel-actions">
                <button
                  className="button button-secondary"
                  disabled={!isConnected || !addresses.agc || !addresses.xagcVault}
                  onClick={handleApproveAgcForVault}
                >
                  Approve Vault
                </button>
                <button
                  className="button button-primary"
                  disabled={!isConnected || !ready}
                  onClick={handleDepositXagc}
                >
                  Deposit AGC
                </button>
                <button
                  className="button button-secondary"
                  disabled={!isConnected || !ready}
                  onClick={handleRedeemXagc}
                >
                  Redeem xAGC
                </button>
              </div>
            </article>
          </div>

          {txNote && <p className="tx-note" data-state={txState}>{txNote}</p>}
        </div>
      </section>

      <section id="policy" className="section policy-section">
        <div className="policy-visual">
          <img src="/art-deco/policy-engine.png" alt="" />
        </div>
        <div className="policy-copy">
          <p className="eyebrow">Policy chamber</p>
          <h2>Measured expansion. Visible defense. No hidden promise.</h2>
          <p>
            The protocol is built around a controlled credit loop: demand is
            observed at the market, state is settled by the controller, and
            treasury action is reserved for stress.
          </p>
          <div className="metric-table">
            {operatingMetrics.map(([label, value]) => (
              <div key={label} className="metric-row">
                <span>{label}</span>
                <strong>{value}</strong>
              </div>
            ))}
          </div>
        </div>
      </section>

      <section className="section architecture-section">
        <div className="section-heading">
          <p className="eyebrow">Monetary architecture</p>
          <h2>The protocol reads the market before it changes the money.</h2>
        </div>
        <div className="architecture-grid">
          {architectureSteps.map((item) => (
            <article key={item.step} className="architecture-card">
              <span>{item.step}</span>
              <h3>{item.title}</h3>
              <p>{item.text}</p>
            </article>
          ))}
        </div>
      </section>

      <section className="section regime-section">
        <div className="section-heading">
          <p className="eyebrow">Regime discipline</p>
          <h2>Four states, each with a different obligation.</h2>
        </div>
        <div className="regime-grid">
          {regimeNarrative.map((item) => (
            <article key={item.name} className="regime-card">
              <h3>{item.name}</h3>
              <span>{item.signal}</span>
              <p>{item.text}</p>
            </article>
          ))}
        </div>
      </section>

      <footer className="footer">
        <span>AGC / canonical pool / policy engine / xAGC vault</span>
        <span>Treasury AGC: {treasuryAgc.data ? `${fmt18(treasuryAgc.data, 2)} AGC` : " - "}</span>
      </footer>
    </main>
  );
}

export default function App() {
  const path =
    typeof window !== "undefined" ? window.location.pathname.replace(/\/$/, "") : "";
  const isDashboard = path === "/dashboard";

  return isDashboard ? <DashboardPage /> : <LandingPage />;
}
