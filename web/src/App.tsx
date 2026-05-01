import { useEffect, useRef, useState, type TouchEvent, type WheelEvent } from "react";
import {
  docsUpdatedAt,
  flatProtocolDocs,
  protocolDocGroups,
} from "./protocolDocs";
import { hasSolanaDeployment, solanaAddresses } from "./contracts";

const docsHref = "/docs";
const xProfileHref = "https://x.com/AgentCreditSOL";

const dashboardNavItems = [
  { label: "Landing", href: "/" },
  { label: "State", href: "#telemetry" },
  { label: "Use AGC", href: "#market-desk" },
  { label: "Expansion", href: "#policy" },
  { label: "Docs", href: docsHref },
  { label: "X", href: xProfileHref },
] as const;

const landingSections = [
  {
    id: "credit",
    video: {
      desktopMp4: "/art-deco/statue_city-loop-1080.mp4",
      mobileMp4: "/art-deco/statue_city-loop-720.mp4",
    },
    playbackRate: 1,
    poster: "/art-deco/statue_city_poster.webp",
    eyebrow: "Agent Credit Protocol",
    title: "Credit inventory for machine economies.",
    text:
      "AGC is spendable working capital for agents, apps, and automated markets. It moves like a token, expands like credit, and only grows when the balance sheet earns it.",
    align: "left",
    stats: ["Machine capital", "Earned expansion", "Solana native"],
    docsId: "overview",
  },
  {
    id: "problem",
    video: {
      desktopMp4: "/art-deco/city_orbit-loop-1080.mp4",
      mobileMp4: "/art-deco/city_orbit-loop-720.mp4",
    },
    playbackRate: 1.5,
    poster: "/art-deco/city_orbit_poster.webp",
    eyebrow: "The problem",
    title: "Static dollars cannot power a live machine economy.",
    text:
      "Agents need balances for compute, APIs, data, inventory, execution, and commerce. Fully reserved stablecoins settle payments, but they do not create native credit velocity.",
    align: "right",
    stats: ["Working capital", "Real-time spend", "Credit velocity"],
    docsId: "problem",
  },
  {
    id: "growth",
    video: {
      desktopMp4: "/art-deco/statue_orbit-loop-1080.mp4",
      mobileMp4: "/art-deco/statue_orbit-loop-720.mp4",
    },
    playbackRate: 0.8,
    poster: "/art-deco/statue_orbit_poster.webp",
    eyebrow: "Upside",
    title: "Own the credit machine before it gets crowded.",
    text:
      "If AGC becomes useful credit inventory, demand deepens reserves, unlocks capacity, and routes value into xAGC. The early bet is ownership of a credit network that expands when the balance sheet earns it.",
    align: "left",
    stats: ["xAGC expansion", "BTC reserve beta", "Credit access"],
    docsId: "growth",
  },
] as const;

const footerVideo = {
  desktopMp4: "/art-deco/statue_orbit_pillars-loop-1080.mp4",
  mobileMp4: "/art-deco/statue_orbit_pillars-loop-720.mp4",
} as const;

const JUPITER_PLUGIN_SCRIPT = "https://plugin.jup.ag/plugin-v1.js";
const JUPITER_PLUGIN_TARGET_ID = "jupiter-plugin";

type JupiterPluginEvent = {
  txid?: string;
  error?: unknown;
};

type JupiterPluginConfig = {
  displayMode: "integrated" | "widget" | "modal";
  integratedTargetId?: string;
  formProps?: {
    initialInputMint?: string;
    initialOutputMint?: string;
    fixedMint?: string;
    initialAmount?: string;
    fixedAmount?: boolean;
    swapMode?: "ExactIn" | "ExactOut" | "ExactInOrOut";
  };
  branding?: {
    logoUri?: string;
    name?: string;
  };
  onSuccess?: (event: JupiterPluginEvent) => void;
  onSwapError?: (event: JupiterPluginEvent) => void;
};

declare global {
  interface Window {
    Jupiter?: {
      init: (config: JupiterPluginConfig) => void;
    };
    solana?: {
      isPhantom?: boolean;
      publicKey?: { toString: () => string };
      connect: () => Promise<{ publicKey: { toString: () => string } }>;
      disconnect?: () => Promise<void>;
      on?: (event: string, handler: (publicKey: unknown) => void) => void;
      off?: (event: string, handler: (publicKey: unknown) => void) => void;
    };
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
  isActive,
  shouldLoad,
  preload,
  playbackRate = 1,
}: {
  sources: { desktopMp4: string; mobileMp4: string };
  poster: string;
  isActive: boolean;
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

  function pauseVideo(video: HTMLVideoElement | null) {
    if (!video) return;
    video.pause();
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

  useEffect(() => {
    const firstVideo = firstVideoRef.current;
    const secondVideo = secondVideoRef.current;

    if (!shouldLoad) {
      isCrossfadingRef.current = false;
      pauseVideo(firstVideo);
      pauseVideo(secondVideo);
      return;
    }

    firstVideo?.load();
    secondVideo?.load();
  }, [shouldLoad, sources.desktopMp4, sources.mobileMp4]);

  useEffect(() => {
    const firstVideo = firstVideoRef.current;
    const secondVideo = secondVideoRef.current;

    if (!isActive || !shouldLoad) {
      pauseVideo(firstVideo);
      pauseVideo(secondVideo);
      return;
    }

    const activeVideo = videoForSlot(activeSlot);
    prepareVideo(activeVideo);
    void activeVideo?.play();
  }, [activeSlot, isActive, playbackRate, shouldLoad]);

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
              isActive={activeScene === index}
              shouldLoad={shouldLoadVideo}
              preload={activeScene === index ? "auto" : "metadata"}
              playbackRate={section.playbackRate}
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
                  Open AGC Console
                </a>
                <a className="landing-button" href={`${docsHref}/${section.docsId}`}>
                  Read Docs
                </a>
              </div>
            ) : (
              <div className="landing-actions">
                <a className="landing-button" href={`${docsHref}/${section.docsId}`}>
                  Read Section
                </a>
              </div>
            )}
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
          src="/art-deco/statue_orbit_pillars_poster.webp"
          alt=""
          aria-hidden="true"
        />
        <CrossfadeVideo
          sources={footerVideo}
          poster="/art-deco/statue_orbit_pillars_poster.webp"
          isActive={activeScene === landingSections.length}
          shouldLoad={activeScene >= landingSections.length - 1}
          preload={activeScene === landingSections.length ? "auto" : "metadata"}
          playbackRate={0.75}
        />
        <div className="cinema-vignette" aria-hidden="true" />
        <div className="footer-cinema-content">
          <p className="landing-eyebrow">Why it matters</p>
          <h2>Expansion with proof, not inflation with a story.</h2>
          <p>
            AGC prints only when reserves, collateral, liquidity, oracle health,
            and mint caps line up. Governance can tune the machine, but it
            cannot turn it into an unlimited faucet.
          </p>
          <div className="footer-link-row">
            <a href="/dashboard">AGC Console</a>
            <a href={`${docsHref}/defense`}>Safety Docs</a>
            <a href={`${docsHref}/governance`}>Governance Docs</a>
            <a href={xProfileHref} target="_blank" rel="noreferrer">X</a>
          </div>
        </div>
      </section>
    </main>
  );
}

const docVisuals = {
  flywheel: {
    src: "/docs/diagrams/credit-flywheel.webp",
    alt: "AGC credit flywheel diagram",
    steps: ["Demand", "Reserves", "Credit capacity", "xAGC upside"],
  },
  reserves: {
    src: "/docs/diagrams/reserve-buckets.webp",
    alt: "AGC reserve bucket diagram",
    steps: ["Cash", "BTC", "RWA", "Haircuts"],
  },
  defense: {
    src: "/docs/diagrams/defense-controls.webp",
    alt: "AGC defense controls diagram",
    steps: ["Caps", "Oracles", "Pauses", "Buybacks"],
  },
  vault: {
    src: "/docs/diagrams/credit-flywheel.webp",
    alt: "xAGC vault flow diagram",
    steps: ["Deposit AGC", "Receive xAGC", "Expansion enters vault", "Redeem with fee"],
  },
  governance: {
    src: "/docs/diagrams/defense-controls.webp",
    alt: "AGC governance control diagram",
    steps: ["Admin", "Risk", "Emergency", "Upgrade"],
  },
  integration: {
    src: "/docs/diagrams/reserve-buckets.webp",
    alt: "AGC Solana integration diagram",
    steps: ["PDAs", "Vaults", "Collateral accounts", "Oracle feeds"],
  },
} as const;

function activeDocFromPath() {
  if (typeof window === "undefined") return flatProtocolDocs[0];
  const [, , pageId] = window.location.pathname.split("/");
  return flatProtocolDocs.find((page) => page.id === pageId) ?? flatProtocolDocs[0];
}

function normalizedAppPath() {
  if (typeof window === "undefined") return "";
  return window.location.pathname.replace(/\/$/, "");
}

function isAppRoute(pathname: string) {
  return pathname === "/" || pathname === "/dashboard" || pathname === "/docs" || pathname.startsWith("/docs/");
}

function scrollToHash(hash: string) {
  if (!hash) {
    window.scrollTo(0, 0);
    return;
  }

  const targetId = decodeURIComponent(hash.slice(1));
  window.setTimeout(() => {
    document.getElementById(targetId)?.scrollIntoView({ block: "start" });
  }, 0);
}

function useClientRoutePath() {
  const [path, setPath] = useState(normalizedAppPath);

  useEffect(() => {
    function syncPath() {
      setPath(normalizedAppPath());
    }

    function handleClick(event: globalThis.MouseEvent) {
      if (
        event.defaultPrevented ||
        event.button !== 0 ||
        event.metaKey ||
        event.altKey ||
        event.ctrlKey ||
        event.shiftKey
      ) {
        return;
      }

      if (!(event.target instanceof Element)) return;
      const anchor = event.target.closest<HTMLAnchorElement>("a[href]");
      if (!anchor || anchor.target || anchor.hasAttribute("download")) return;

      const url = new URL(anchor.href, window.location.href);
      if (url.origin !== window.location.origin || !isAppRoute(url.pathname)) return;

      if (url.pathname === window.location.pathname && url.search === window.location.search) {
        return;
      }

      event.preventDefault();
      window.history.pushState(null, "", `${url.pathname}${url.search}${url.hash}`);
      syncPath();
      scrollToHash(url.hash);
    }

    window.addEventListener("popstate", syncPath);
    document.addEventListener("click", handleClick);
    return () => {
      window.removeEventListener("popstate", syncPath);
      document.removeEventListener("click", handleClick);
    };
  }, []);

  return path;
}

function DocVisual({ page }: { page: (typeof flatProtocolDocs)[number] }) {
  const visual = docVisuals[page.visual];

  return (
    <figure className="doc-visual">
      <img src={visual.src} alt={visual.alt} />
      <figcaption>
        {visual.steps.map((step, index) => (
          <span key={step}>
            {String(index + 1).padStart(2, "0")} {step}
          </span>
        ))}
      </figcaption>
    </figure>
  );
}

function DocsPage() {
  const activeDoc = activeDocFromPath();
  const activeIndex = flatProtocolDocs.findIndex((page) => page.id === activeDoc.id);
  const previousDoc = activeIndex > 0 ? flatProtocolDocs[activeIndex - 1] : undefined;
  const nextDoc =
    activeIndex >= 0 && activeIndex < flatProtocolDocs.length - 1
      ? flatProtocolDocs[activeIndex + 1]
      : undefined;

  return (
    <main className="site docs-page">
      <div className="ambient-grid" aria-hidden="true" />

      <header className="dashboard-topbar docs-topbar">
        <a className="brand" href="/" aria-label="Agent Credit Protocol home">
          <span className="brand-mark">
            <img src="/agc-mark.svg" alt="" />
          </span>
          <span className="brand-copy">
            <span className="brand-name">Agent Credit Protocol</span>
            <span className="brand-subtitle">Docs updated {docsUpdatedAt}</span>
          </span>
        </a>
        <nav className="nav" aria-label="Docs navigation" />
        <div className="wallet">
          <a className="button button-secondary" href={xProfileHref} target="_blank" rel="noreferrer">
            X
          </a>
          <a className="button button-secondary" href="/dashboard">
            AGC Console
          </a>
        </div>
      </header>

      <section className="docs-layout">
        <aside className="docs-sidebar" aria-label="Docs table of contents">
          {protocolDocGroups.map((group) => (
            <div key={group.title} className="docs-sidebar-group">
              <strong>{group.title}</strong>
              {group.pages.map((section) => (
                <a
                  key={section.id}
                  className={activeDoc.id === section.id ? "is-active" : ""}
                  href={`${docsHref}/${section.id}`}
                >
                  {section.title}
                </a>
              ))}
            </div>
          ))}
        </aside>

        <div className="docs-content">
          <article key={activeDoc.id} id={activeDoc.id} className="doc-section">
            <p className="eyebrow">AGC Docs</p>
            <h1>{activeDoc.title}</h1>
            <p className="doc-summary">{activeDoc.summary}</p>
            <DocVisual page={activeDoc} />
            {activeDoc.body.map((paragraph) => (
              <p key={paragraph}>{paragraph}</p>
            ))}
            {activeDoc.example ? (
              <div className="doc-callout">
                <strong>{activeDoc.example.title}</strong>
                {activeDoc.example.lines.map((line) => (
                  <span key={line}>{line}</span>
                ))}
              </div>
            ) : null}
            {activeDoc.bullets ? (
              <ul>
                {activeDoc.bullets.map((bullet) => (
                  <li key={bullet}>{bullet}</li>
                ))}
              </ul>
            ) : null}
            {activeDoc.technical ? (
              <div className="doc-technical">
                <strong>Under the hood</strong>
                {activeDoc.technical.map((note) => (
                  <span key={note}>{note}</span>
                ))}
              </div>
            ) : null}
            <nav className="doc-next-prev" aria-label="Docs pagination">
              {previousDoc ? (
                <a href={`${docsHref}/${previousDoc.id}`}>Previous: {previousDoc.title}</a>
              ) : (
                <span />
              )}
              {nextDoc ? (
                <a href={`${docsHref}/${nextDoc.id}`}>Next: {nextDoc.title}</a>
              ) : (
                <span />
              )}
            </nav>
          </article>
        </div>
      </section>

      <footer className="footer">
        <span>AGC docs</span>
        <span>Updated {docsUpdatedAt}</span>
      </footer>
    </main>
  );
}

function JupiterSwapPanel() {
  const [pluginStatus, setPluginStatus] = useState(
    solanaAddresses.agcMint ? "Loading Jupiter swap..." : "AGC mint is not configured.",
  );
  const canLoadPlugin = Boolean(solanaAddresses.agcMint && solanaAddresses.usdcMint);

  useEffect(() => {
    if (!canLoadPlugin || !solanaAddresses.agcMint) return;

    let disposed = false;

    const initializePlugin = () => {
      if (disposed || !window.Jupiter) return;

      window.Jupiter.init({
        displayMode: "integrated",
        integratedTargetId: JUPITER_PLUGIN_TARGET_ID,
        formProps: {
          initialInputMint: solanaAddresses.usdcMint,
          initialOutputMint: solanaAddresses.agcMint,
          fixedMint: solanaAddresses.agcMint,
          initialAmount: "10",
          fixedAmount: false,
          swapMode: "ExactInOrOut",
        },
        branding: {
          logoUri: "/agc-mark.svg",
          name: "Agent Credit Protocol",
        },
        onSuccess: ({ txid }) => {
          setPluginStatus(txid ? `Swap confirmed: ${txid.slice(0, 8)}...` : "Swap confirmed.");
        },
        onSwapError: ({ error }) => {
          setPluginStatus(extractErrorMessage(error));
        },
      });
      setPluginStatus("Jupiter swap ready.");
    };

    const existingScript = document.querySelector<HTMLScriptElement>(
      `script[src="${JUPITER_PLUGIN_SCRIPT}"]`,
    );

    if (window.Jupiter) {
      initializePlugin();
    } else if (existingScript) {
      existingScript.addEventListener("load", initializePlugin, { once: true });
    } else {
      const script = document.createElement("script");
      script.src = JUPITER_PLUGIN_SCRIPT;
      script.defer = true;
      script.dataset.preload = "true";
      script.addEventListener("load", initializePlugin, { once: true });
      script.addEventListener(
        "error",
        () => setPluginStatus("Jupiter swap could not load."),
        { once: true },
      );
      document.head.appendChild(script);
    }

    return () => {
      disposed = true;
    };
  }, [canLoadPlugin]);

  return (
    <article className="operation-panel jupiter-swap-panel">
      <div className="panel-header">
        <span className="card-label">Market</span>
        <h3>Swap AGC</h3>
      </div>
      <div className="jupiter-plugin-frame">
        {canLoadPlugin ? (
          <div id={JUPITER_PLUGIN_TARGET_ID} className="jupiter-plugin-target" />
        ) : (
          <div className="plugin-empty-state">
            <strong>AGC mint not configured</strong>
            <span>Set VITE_SOLANA_AGC_MINT to enable the Jupiter swap panel.</span>
          </div>
        )}
      </div>
      <p className="panel-meta">
        Jupiter handles routing, slippage, wallet connection, and swap submission.
      </p>
      <p className="tx-note" data-state={pluginStatus.includes("confirmed") ? "complete" : "idle"}>
        {pluginStatus}
      </p>
    </article>
  );
}

function shortKey(value: string | null | undefined) {
  if (!value) return " - ";
  if (value.length <= 12) return value;
  return `${value.slice(0, 4)}...${value.slice(-4)}`;
}

function DashboardPage() {
  const [walletAddress, setWalletAddress] = useState<string | null>(null);
  const [stakeAgcAmount, setStakeAgcAmount] = useState("50");
  const [redeemXagcShares, setRedeemXagcShares] = useState("10");
  const [underwriteAmount, setUnderwriteAmount] = useState("1000");
  const [collateralAmount, setCollateralAmount] = useState("0.25");
  const [drawAmount, setDrawAmount] = useState("250");
  const [repayAmount, setRepayAmount] = useState("100");
  const [txStatus, setTxStatus] = useState("Idle");
  const [txNote, setTxNote] = useState<string | null>(null);

  const ready = hasSolanaDeployment;
  const regimeKey = "neutral";
  const shortAddr = walletAddress ? shortKey(walletAddress) : null;

  useEffect(() => {
    const provider = window.solana;
    if (provider?.publicKey) {
      setWalletAddress(provider.publicKey.toString());
    }

    const handleAccountChanged = (publicKey: unknown) => {
      if (publicKey && typeof publicKey === "object" && "toString" in publicKey) {
        setWalletAddress((publicKey as { toString: () => string }).toString());
      } else {
        setWalletAddress(null);
      }
    };

    provider?.on?.("accountChanged", handleAccountChanged);
    return () => provider?.off?.("accountChanged", handleAccountChanged);
  }, []);

  async function connectWallet() {
    try {
      setTxNote(null);
      const provider = window.solana;
      if (!provider) {
        setTxNote("No Solana wallet detected. Install Phantom or another Solana wallet.");
        return;
      }
      const response = await provider.connect();
      setWalletAddress(response.publicKey.toString());
    } catch (error) {
      setTxNote(extractErrorMessage(error));
    }
  }

  async function disconnectWallet() {
    await window.solana?.disconnect?.();
    setWalletAddress(null);
  }

  function runSolanaAction(status: string) {
    if (!ready) {
      setTxStatus("Idle");
      setTxNote("Deployment addresses are not configured in this environment.");
      return;
    }
    if (!walletAddress) {
      setTxStatus("Idle");
      setTxNote("Connect a Solana wallet first.");
      return;
    }
    setTxStatus(status);
    setTxNote(
      "Live Solana transactions activate after the deployed IDL client is configured.",
    );
  }

  const telemetry = [
    {
      label: "Regime",
      value: " - ",
      detail: "policy account",
    },
    {
      label: "Anchor",
      value: " - ",
      detail: "soft reference",
    },
    {
      label: "Coverage",
      value: " - ",
      detail: "risk-weighted",
    },
    {
      label: "Credit",
      value: " - ",
      detail: "principal outstanding",
    },
    {
      label: "Treasury",
      value: " - ",
      detail: "USDC / USDT",
    },
    {
      label: "Program",
      value: shortKey(solanaAddresses.programId),
      detail: "Solana",
    },
  ];

  const operatingMetrics = [
    ["Program ID", shortKey(solanaAddresses.programId)],
    ["Protocol state", shortKey(solanaAddresses.state)],
    ["AGC mint", shortKey(solanaAddresses.agcMint)],
    ["xAGC mint", shortKey(solanaAddresses.xagcMint)],
    ["USDC mint", shortKey(solanaAddresses.usdcMint)],
    ["Treasury USDC", shortKey(solanaAddresses.treasuryUsdc)],
    ["Credit facilities", "Collateralized AGC lines"],
    ["Underwriter vaults", "First-loss AGC reserves"],
    ["Oracle cache", "Pyth per collateral mint"],
    ["Buyback path", "TWAP campaign burn escrow"],
  ] as const;

  const txState =
    txStatus === "Idle" ? "idle" : txStatus.includes("complete") ? "complete" : "active";

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

        <nav className="nav" aria-label="AGC console navigation">
          {dashboardNavItems.map((item) => (
            <a
              key={item.label}
              href={item.href}
              target={item.href.startsWith("https://") ? "_blank" : undefined}
              rel={item.href.startsWith("https://") ? "noreferrer" : undefined}
            >
              {item.label}
            </a>
          ))}
        </nav>

        <div className="wallet">
          {walletAddress ? (
            <>
              <span className="wallet-address">{shortAddr}</span>
              <button className="button button-secondary" onClick={disconnectWallet}>
                Disconnect
              </button>
            </>
          ) : (
            <button
              className="button button-primary"
              onClick={connectWallet}
            >
              Connect Wallet
            </button>
          )}
        </div>
      </header>

      <section className="dashboard-hero">
        <p className="eyebrow">AGC console</p>
        <h1>Credit inventory, expansion state, and xAGC duration.</h1>
        <p>
          Trade AGC, lock into the expansion layer, underwrite credit, and watch
          the balance sheet decide when new supply is earned.
        </p>
      </section>

      <section id="telemetry" className="telemetry-band dashboard-telemetry" aria-label="AGC state">
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
            Live telemetry and wallet operations populate after the Solana
            program addresses are connected.
          </span>
        </section>
      )}

      <section id="market-desk" className="section console-section">
        <div className="section-heading console-heading">
          <p className="eyebrow">Use AGC</p>
          <h2>Trade, lock, borrow, and underwrite in one place.</h2>
        </div>

        <div className="market-console">
          <div className="status-rail">
            <span className="status-dot" />
            <span className="status-label">Transaction status</span>
            <strong>{txStatus}</strong>
          </div>

          <div className="operation-grid">
            <JupiterSwapPanel />

            <article id="vaults" className="operation-panel">
              <div className="panel-header">
                <span className="card-label">Savings layer</span>
                <h3>xAGC expansion layer</h3>
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
                <span>Wallet: <strong>{shortKey(walletAddress)}</strong></span>
                <span>AGC mint: <strong>{shortKey(solanaAddresses.agcMint)}</strong></span>
                <span>xAGC mint: <strong>{shortKey(solanaAddresses.xagcMint)}</strong></span>
                <span>Reserve: <strong>{shortKey(solanaAddresses.xagcVaultAgc)}</strong></span>
                <span>Share px: <strong>On-chain</strong></span>
                <span>Exit fee: <strong>Policy state</strong></span>
                <span>Deposit: <strong>{stakeAgcAmount || " - "} AGC</strong></span>
                <span>Redeem: <strong>{redeemXagcShares || " - "} xAGC</strong></span>
              </div>
              <div className="panel-actions">
                <button
                  className="button button-primary"
                  disabled={!walletAddress || !ready}
                  onClick={() => runSolanaAction("Depositing AGC")}
                >
                  Deposit AGC
                </button>
                <button
                  className="button button-secondary"
                  disabled={!walletAddress || !ready}
                  onClick={() => runSolanaAction("Redeeming xAGC")}
                >
                  Redeem xAGC
                </button>
              </div>
            </article>

            <article className="operation-panel credit-panel">
              <div className="panel-header">
                <span className="card-label">Credit facility</span>
                <h3>Borrow and back credit</h3>
              </div>
              <div className="field-pair">
                <Field
                  id="underwrite-agc"
                  label="Underwrite AGC"
                  value={underwriteAmount}
                  onChange={setUnderwriteAmount}
                  placeholder="1000.0"
                />
                <Field
                  id="credit-collateral"
                  label="Collateral"
                  value={collateralAmount}
                  onChange={setCollateralAmount}
                  placeholder="0.25"
                />
                <Field
                  id="credit-draw"
                  label="Draw AGC"
                  value={drawAmount}
                  onChange={setDrawAmount}
                  placeholder="250.0"
                />
                <Field
                  id="credit-repay"
                  label="Repay AGC"
                  value={repayAmount}
                  onChange={setRepayAmount}
                  placeholder="100.0"
                />
              </div>
              <div className="vault-metrics">
                <span>Facility: <strong>Per collateral mint</strong></span>
                <span>Collateral: <strong>USDC / USDT / BTC / isolated RWA</strong></span>
                <span>Health: <strong>Collateral value / AGC debt</strong></span>
                <span>Backstop: <strong>Underwriter first loss</strong></span>
                <span>Interest: <strong>Paid to underwriters</strong></span>
                <span>Default: <strong>Burn reserve, route collateral</strong></span>
              </div>
              <div className="panel-actions">
                <button
                  className="button button-primary"
                  disabled={!walletAddress || !ready}
                  onClick={() => runSolanaAction("Depositing underwriter AGC")}
                >
                  Underwrite
                </button>
                <button
                  className="button button-secondary"
                  disabled={!walletAddress || !ready}
                  onClick={() => runSolanaAction("Depositing collateral")}
                >
                  Add collateral
                </button>
                <button
                  className="button button-secondary"
                  disabled={!walletAddress || !ready}
                  onClick={() => runSolanaAction("Drawing AGC credit")}
                >
                  Draw
                </button>
                <button
                  className="button button-secondary"
                  disabled={!walletAddress || !ready}
                  onClick={() => runSolanaAction("Repaying AGC credit")}
                >
                  Repay
                </button>
              </div>
            </article>
          </div>

          {txNote && <p className="tx-note" data-state={txState}>{txNote}</p>}
        </div>
      </section>

      <section id="policy" className="section policy-section">
        <div className="policy-visual">
          <img src="/art-deco/policy-engine.webp" alt="" />
        </div>
        <div className="policy-copy">
          <p className="eyebrow">Expansion control</p>
          <h2>The machine grows only when the balance sheet proves it.</h2>
          <p>
            Reserves, collateral, liquidity, oracle health, and credit quality
            decide capacity. When those signals weaken, issuance stops and
            defense takes priority.
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

      <footer className="footer">
        <span>AGC / Jupiter market / expansion control / xAGC vault / credit facilities</span>
        <span>Program: {shortKey(solanaAddresses.programId)}</span>
      </footer>
    </main>
  );
}

export default function App() {
  const path = useClientRoutePath();
  const isDashboard = path === "/dashboard";
  const isDocs = path === "/docs" || path.startsWith("/docs/");

  if (isDashboard) return <DashboardPage />;
  if (isDocs) return <DocsPage />;
  return <LandingPage />;
}
