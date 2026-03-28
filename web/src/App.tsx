import { useMemo, useState } from "react";
import { formatUnits, keccak256, parseUnits, stringToHex } from "viem";
import {
  useAccount,
  useConnect,
  useDisconnect,
  useReadContract,
  useWriteContract,
  useWaitForTransactionReceipt,
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

function fmt18(v: bigint | undefined, decimals = 4): string {
  if (v === undefined) return "—";
  return Number(formatUnits(v, 18)).toFixed(decimals);
}

function fmtBps(v: bigint | undefined): string {
  if (v === undefined) return "—";
  return `${(Number(v) / 100).toFixed(2)}%`;
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

  /* ── contract reads ── */
  const enabled = (a: string | undefined) => ({
    query: { enabled: Boolean(a) },
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

  /* ── derived ── */
  const regimeIdx =
    typeof regime.data === "number" ? regime.data : undefined;
  const regimeKey =
    regimeIdx !== undefined ? regimeKeys[regimeIdx] ?? "neutral" : "neutral";
  const regimeLabel =
    regimeIdx !== undefined ? regimeLabels[regimeIdx] ?? "Unknown" : "—";

  const status = useMemo(() => {
    if (isConfirming) return "confirming" as const;
    if (hash) return "submitted" as const;
    return "idle" as const;
  }, [hash, isConfirming]);

  const statusLabel = { idle: "Idle", confirming: "Confirming…", submitted: "Submitted" }[status];

  /* ── actions ── */
  async function approveAndPay() {
    if (!ready || !address || !addresses.agc || !addresses.settlementRouter)
      return;

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

  const shortAddr = address
    ? `${address.slice(0, 6)}…${address.slice(-4)}`
    : null;

  return (
    <main className="shell" data-regime={regimeKey}>
      {/* ── Top bar ── */}
      <header className="topbar">
        <div className="topbar-brand">
          <div className="topbar-mark">A</div>
          <span className="topbar-name">Agent Credit Protocol</span>
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
              onClick={() => connect({ connector: connectors[0] })}
            >
              Connect Wallet
            </button>
          )}
        </div>
      </header>

      {/* ── Hero ── */}
      <section className="hero-section">
        <p className="hero-eyebrow">Elastic Working Capital</p>
        <h1 className="hero-title">
          Infrastructure for <strong>autonomous</strong> commerce.
        </h1>
        <p className="hero-sub">
          Hold <code>AGC</code>, route machine payments through the canonical
          Uniswap v4 pool, and settle in <code>USDC</code> only when payment
          lands.
        </p>
      </section>

      {/* ── Missing config ── */}
      {!ready && (
        <div className="notice">
          <strong>Missing deployment addresses.</strong> Set{" "}
          <code>VITE_AGC_ADDRESS</code>,{" "}
          <code>VITE_POLICY_CONTROLLER_ADDRESS</code>,{" "}
          <code>VITE_REWARD_DISTRIBUTOR_ADDRESS</code>, and{" "}
          <code>VITE_SETTLEMENT_ROUTER_ADDRESS</code> in{" "}
          <code>.env.local</code>.
        </div>
      )}

      {/* ── Regime strip ── */}
      <div className="regime-strip">
        <div className="regime-badge">
          <span className="regime-indicator" />
          <div>
            <span className="regime-label-prefix">Regime</span>
            <div className="regime-label-value">{regimeLabel}</div>
          </div>
        </div>
        <div className="regime-divider" />
        <div className="regime-stat">
          <span className="regime-stat-label">Anchor</span>
          <span className="regime-stat-value">
            {anchor.data ? `$${fmt18(anchor.data)}` : "—"}
          </span>
        </div>
        <div className="regime-divider" />
        <div className="regime-stat">
          <span className="regime-stat-label">Band</span>
          <span className="regime-stat-value">{fmtBps(band.data)}</span>
        </div>
        <div className="regime-divider" />
        <div className="regime-stat">
          <span className="regime-stat-label">Balance</span>
          <span className="regime-stat-value">
            {balance.data ? `${fmt18(balance.data, 2)} AGC` : "—"}
          </span>
        </div>
      </div>

      {/* ── Metrics ── */}
      <div className="metrics">
        <div className="metric">
          <span className="metric-label">Anchor Price</span>
          <span className="metric-value">
            {anchor.data ? `$${fmt18(anchor.data)}` : "—"}
          </span>
          <span className="metric-hint">EMA soft floor</span>
        </div>
        <div className="metric">
          <span className="metric-label">Band Width</span>
          <span className="metric-value">{fmtBps(band.data)}</span>
          <span className="metric-hint">Half-width around anchor</span>
        </div>
        <div className="metric">
          <span className="metric-label">Productive Usage</span>
          <span className="metric-value">
            {fmtBps(productiveUsage.data)}
          </span>
          <span className="metric-hint">Payment volume share</span>
        </div>
        <div className="metric">
          <span className="metric-label">Coverage</span>
          <span className="metric-value">{fmtBps(coverage.data)}</span>
          <span className="metric-hint">Liquidity depth ratio</span>
        </div>
        <div className="metric">
          <span className="metric-label">Exit Pressure</span>
          <span className="metric-value">{fmtBps(exitPressure.data)}</span>
          <span className="metric-hint">Net sell-side flow</span>
        </div>
        <div className="metric">
          <span className="metric-label">Volatility</span>
          <span className="metric-value">{fmtBps(volatility.data)}</span>
          <span className="metric-hint">Realized epoch vol</span>
        </div>
        <div className="metric">
          <span className="metric-label">Regime</span>
          <span className="metric-value">{regimeLabel}</span>
          <span className="metric-hint">Current policy stance</span>
        </div>
        <div className="metric">
          <span className="metric-label">AGC Balance</span>
          <span className="metric-value">
            {balance.data ? fmt18(balance.data, 2) : "0.00"}
          </span>
          <span className="metric-hint">Connected wallet</span>
        </div>
      </div>

      {/* ── Action panels ── */}
      <div className="panels">
        <div className="panel">
          <div className="panel-header">
            <h2 className="panel-title">Claim Rewards</h2>
            <span className="panel-badge">Vesting</span>
          </div>
          <div className="field">
            <label className="field-label" htmlFor="stream-id">
              Stream ID
            </label>
            <input
              id="stream-id"
              className="field-input"
              value={streamId}
              onChange={(e) => setStreamId(e.target.value)}
              placeholder="0"
            />
          </div>
          <p className="panel-meta">
            Claimable:{" "}
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
              Claim vested rewards
            </button>
          </div>
        </div>

        <div className="panel">
          <div className="panel-header">
            <h2 className="panel-title">Settle Payment</h2>
            <div className="tx-status" data-status={status}>
              <span className="tx-status-dot" />
              {statusLabel}
            </div>
          </div>
          <div className="field">
            <label className="field-label" htmlFor="agc-in">
              AGC Amount
            </label>
            <input
              id="agc-in"
              className="field-input"
              value={paymentAmount}
              onChange={(e) => setPaymentAmount(e.target.value)}
              placeholder="10.0"
            />
          </div>
          <div className="field">
            <label className="field-label" htmlFor="min-usdc">
              Min USDC Out
            </label>
            <input
              id="min-usdc"
              className="field-input"
              value={minUsdcOut}
              onChange={(e) => setMinUsdcOut(e.target.value)}
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
              onChange={(e) => setRecipient(e.target.value)}
              placeholder="0x…"
            />
          </div>
          <div className="panel-actions">
            <button
              className="btn btn-primary"
              disabled={!isConnected || !ready}
              onClick={approveAndPay}
            >
              Approve & Settle
            </button>
          </div>
        </div>
      </div>

      {/* ── Footer ── */}
      <footer className="footer">
        <span className="footer-left">AGC v0 — Local Devnet</span>
      </footer>
    </main>
  );
}
