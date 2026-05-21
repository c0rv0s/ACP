import { useEffect, useState } from "react";
import { PublicKey } from "@solana/web3.js";

import {
  explorerProgramUrl,
  hasProgramDeployment,
  hasSolanaDeployment,
  showJupiterSwap,
  solanaAddresses,
  solanaCluster,
} from "./contracts";
import {
  formatAgc,
  formatBps,
  formatPrice,
  formatUsdc,
  regimeLabel,
  useProtocolSnapshot,
} from "./useProtocolState";
import {
  depositCollateral,
  depositXagc,
  drawCredit,
  explorerTxUrl,
  redeemXagc,
  repayCredit,
  underwriteAgc,
} from "./transactions";

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
  const [stakeAgcAmount, setStakeAgcAmount] = useState("");
  const [redeemXagcShares, setRedeemXagcShares] = useState("");
  const [underwriteAmount, setUnderwriteAmount] = useState("");
  const [collateralAmount, setCollateralAmount] = useState("");
  const [drawAmount, setDrawAmount] = useState("");
  const [repayAmount, setRepayAmount] = useState("");
  const [txStatus, setTxStatus] = useState("Idle");
  const [txNote, setTxNote] = useState<string | null>(null);

  const ready = hasSolanaDeployment;
  const snapshot = useProtocolSnapshot(walletAddress);
  const regimeKey = snapshot.state?.regime ?? "neutral";
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

  async function submitTx(
    label: string,
    builder: (wallet: PublicKey) => Promise<string>,
  ) {
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
    setTxStatus(`${label}...`);
    setTxNote(null);
    try {
      const sig = await builder(new PublicKey(walletAddress));
      setTxStatus(`${label} complete`);
      setTxNote(
        `Signature ${sig.slice(0, 8)}...${sig.slice(-8)} — ` +
          `view on Solana Explorer (${solanaCluster}).`,
      );
      // Surface the explorer URL via the browser console; clicking the note
      // text in the UI is left to a follow-up.
      console.log(`${label} tx:`, explorerTxUrl(sig, solanaCluster));
    } catch (error) {
      setTxStatus("Idle");
      setTxNote(extractErrorMessage(error));
    }
  }

  const treasuryDollarValue = (() => {
    if (!snapshot.balances || !snapshot.state) return null;
    // Treasury USDC at face value + treasury AGC at anchor price (both → quote x18 → dollars)
    const usdcDollars = snapshot.balances.treasuryUsdc / 1_000_000n; // USDC has 6 decimals
    const agcUnits = snapshot.balances.treasuryAgc / 1_000_000_000n; // AGC has 9 decimals
    const agcDollars = (agcUnits * snapshot.state.anchorPriceX18) / 10n ** 18n;
    return usdcDollars + agcDollars;
  })();

  const telemetry = [
    {
      label: "Regime",
      value: regimeLabel(snapshot.state?.regime),
      detail: "policy account",
    },
    {
      label: "Anchor",
      value: snapshot.state ? formatPrice(snapshot.state.anchorPriceX18) : " - ",
      detail: "soft reference",
    },
    {
      label: "Coverage",
      value: snapshot.state ? formatBps(snapshot.state.lastReserveCoverageBps) : " - ",
      detail: "risk-weighted",
    },
    {
      label: "Credit",
      value: snapshot.state
        ? `${formatAgc(snapshot.state.creditPrincipalOutstandingAgc)} AGC`
        : " - ",
      detail: "principal outstanding",
    },
    {
      label: "Treasury",
      value:
        treasuryDollarValue !== null && treasuryDollarValue >= 0n
          ? `$${treasuryDollarValue >= 1_000_000n
              ? `${(Number(treasuryDollarValue) / 1_000_000).toFixed(2)}M`
              : treasuryDollarValue >= 1_000n
                ? `${(Number(treasuryDollarValue) / 1_000).toFixed(2)}k`
                : treasuryDollarValue.toString()}`
          : " - ",
      detail: "USDC + AGC (anchor-valued)",
    },
  ];

  const operatingMetrics: readonly (readonly [string, string])[] = [
    ["Cluster", solanaCluster],
    ["Last settled epoch", snapshot.state?.lastSettledEpoch.toString() ?? " - "],
    ["AGC supply", snapshot.balances ? `${formatAgc(snapshot.balances.agcSupply)} AGC` : " - "],
    ["xAGC supply", snapshot.balances ? `${formatAgc(snapshot.balances.xagcSupply)} xAGC` : " - "],
    [
      "xAGC vault assets",
      snapshot.balances ? `${formatAgc(snapshot.balances.xagcVaultAgc)} AGC` : " - ",
    ],
    [
      "Underwriter vault",
      snapshot.balances ? `${formatAgc(snapshot.balances.underwriterVaultAgc)} AGC` : " - ",
    ],
    [
      "Treasury AGC",
      snapshot.balances ? `${formatAgc(snapshot.balances.treasuryAgc)} AGC` : " - ",
    ],
    [
      "Treasury USDC",
      snapshot.balances ? formatUsdc(snapshot.balances.treasuryUsdc) : " - ",
    ],
    [
      "Credit drawn (cumulative)",
      snapshot.state ? `${formatAgc(snapshot.state.creditDrawnAgc)} AGC` : " - ",
    ],
    [
      "Credit repaid (cumulative)",
      snapshot.state ? `${formatAgc(snapshot.state.creditRepaidAgc)} AGC` : " - ",
    ],
    [
      "Premium (last epoch)",
      snapshot.state ? formatBps(snapshot.state.lastPremiumBps) : " - ",
    ],
    [
      "Locked share (last epoch)",
      snapshot.state ? formatBps(snapshot.state.lastLockedShareBps) : " - ",
    ],
  ];

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
          {hasProgramDeployment ? (
            <>
              <strong>
                Program deployed to {solanaCluster}, protocol not yet initialized.
              </strong>
              <span>
                The Solana program is live at{" "}
                {explorerProgramUrl(solanaAddresses.programId) ? (
                  <a
                    href={explorerProgramUrl(solanaAddresses.programId) ?? "#"}
                    target="_blank"
                    rel="noreferrer"
                  >
                    {shortKey(solanaAddresses.programId)}
                  </a>
                ) : (
                  shortKey(solanaAddresses.programId)
                )}
                . Live telemetry and wallet operations come online after{" "}
                <code>initialize_protocol</code> creates the AGC/xAGC mints and
                treasury accounts.
              </span>
            </>
          ) : (
            <>
              <strong>Deployment addresses are not configured in this environment.</strong>
              <span>
                Live telemetry and wallet operations populate after the Solana
                program addresses are connected.
              </span>
            </>
          )}
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
            {showJupiterSwap ? (
              <JupiterSwapPanel />
            ) : (
              <article className="operation-panel">
                <div className="panel-header">
                  <span className="card-label">Market</span>
                  <h3>Swap AGC</h3>
                </div>
                <div className="plugin-empty-state">
                  <strong>Mainnet only</strong>
                  <span>
                    Jupiter routes against mainnet liquidity. On {solanaCluster} the
                    protocol mechanics below — xAGC vault, credit facility, settlement
                    distribution — are the live demo surface.
                  </span>
                </div>
                <p className="panel-meta">
                  Connect a {solanaCluster} wallet, then use the credit and xAGC
                  panels to interact with the deployed program directly.
                </p>
              </article>
            )}

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
                  placeholder="100"
                />
                <Field
                  id="redeem-xagc"
                  label="Redeem xAGC shares"
                  value={redeemXagcShares}
                  onChange={setRedeemXagcShares}
                  placeholder="50"
                />
              </div>
              <div className="vault-metrics">
                <span>
                  Your AGC:{" "}
                  <strong>
                    {snapshot.userBalances
                      ? formatAgc(snapshot.userBalances.agc)
                      : walletAddress
                        ? "Loading..."
                        : " - "}
                  </strong>
                </span>
                <span>
                  Your xAGC:{" "}
                  <strong>
                    {snapshot.userBalances
                      ? formatAgc(snapshot.userBalances.xagc)
                      : walletAddress
                        ? "Loading..."
                        : " - "}
                  </strong>
                </span>
                <span>Share px: <strong>On-chain</strong></span>
                <span>Exit fee: <strong>Policy state</strong></span>
              </div>
              <div className="panel-actions">
                <button
                  className="button button-primary"
                  disabled={!walletAddress || !ready}
                  onClick={() =>
                    submitTx("Deposit AGC", (wallet) => depositXagc(wallet, stakeAgcAmount))
                  }
                >
                  Deposit AGC
                </button>
                <button
                  className="button button-secondary"
                  disabled={!walletAddress || !ready}
                  onClick={() =>
                    submitTx("Redeem xAGC", (wallet) => redeemXagc(wallet, redeemXagcShares))
                  }
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
                  placeholder="1000"
                />
                <Field
                  id="credit-collateral"
                  label="Collateral"
                  value={collateralAmount}
                  onChange={setCollateralAmount}
                  placeholder="0.1"
                />
                <Field
                  id="credit-draw"
                  label="Draw AGC"
                  value={drawAmount}
                  onChange={setDrawAmount}
                  placeholder="200"
                />
                <Field
                  id="credit-repay"
                  label="Repay AGC"
                  value={repayAmount}
                  onChange={setRepayAmount}
                  placeholder="100"
                />
              </div>
              <div className="vault-metrics">
                <span>
                  Your BTC:{" "}
                  <strong>
                    {snapshot.userBalances
                      ? (Number(snapshot.userBalances.btc) / 1e8).toLocaleString(undefined, {
                          maximumFractionDigits: 4,
                        })
                      : walletAddress
                        ? "Loading..."
                        : " - "}
                  </strong>
                </span>
                <span>
                  Your AGC:{" "}
                  <strong>
                    {snapshot.userBalances
                      ? formatAgc(snapshot.userBalances.agc)
                      : walletAddress
                        ? "Loading..."
                        : " - "}
                  </strong>
                </span>
                <span>Health: <strong>Collateral value / AGC debt</strong></span>
                <span>Backstop: <strong>Underwriter first loss</strong></span>
                <span>Interest: <strong>Paid to underwriters</strong></span>
                <span>Default: <strong>Burn reserve, route collateral</strong></span>
              </div>
              <div className="panel-actions">
                <button
                  className="button button-primary"
                  disabled={!walletAddress || !ready}
                  onClick={() =>
                    submitTx("Underwrite AGC", (wallet) =>
                      underwriteAgc(wallet, underwriteAmount),
                    )
                  }
                >
                  Underwrite
                </button>
                <button
                  className="button button-secondary"
                  disabled={!walletAddress || !ready}
                  onClick={() =>
                    submitTx("Deposit collateral", (wallet) =>
                      depositCollateral(wallet, collateralAmount),
                    )
                  }
                >
                  Add collateral
                </button>
                <button
                  className="button button-secondary"
                  disabled={!walletAddress || !ready}
                  onClick={() =>
                    submitTx("Draw AGC", (wallet) => drawCredit(wallet, drawAmount))
                  }
                >
                  Draw
                </button>
                <button
                  className="button button-secondary"
                  disabled={!walletAddress || !ready}
                  onClick={() =>
                    submitTx("Repay AGC", (wallet) => repayCredit(wallet, repayAmount))
                  }
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
        <span>
          {solanaCluster}:{" "}
          {explorerProgramUrl(solanaAddresses.programId) ? (
            <a
              href={explorerProgramUrl(solanaAddresses.programId) ?? "#"}
              target="_blank"
              rel="noreferrer"
            >
              {shortKey(solanaAddresses.programId)}
            </a>
          ) : (
            shortKey(solanaAddresses.programId)
          )}
        </span>
      </footer>
    </main>
  );
}

export default DashboardPage;
