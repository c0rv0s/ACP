const app = document.querySelector("#app");
const web3 = window.solanaWeb3;

let state = null;
let deployment = null;
let walletAddress = window.solana?.publicKey?.toString?.() ?? null;
let walletSolBalance = null;
let txStatus = "Idle";
let txNote = "";
let activeTab = "operator";
const AIRDROP_THRESHOLD_SOL = 2;

const actions = {
  connectWallet,
  disconnectWallet,
  airdrop: () => airdropSol(),
  spendSearch: () => submitWalletTx("spend", { amountUsdc: "2.50", merchant: "search" }, "Search API spend"),
  spendReview: () => submitWalletTx("spend", { amountUsdc: "11.00", merchant: "search" }, "Review threshold spend"),
  spendBlocked: () => submitWalletTx("spend", { amountUsdc: "2.50", merchant: "blocked" }, "Blocked merchant spend"),
  revenue: () => submitWalletTx("revenue", { amountUsdc: "50.00", source: "manual" }, "Revenue sweep"),
  manualRepay: () => submitWalletTx("manual-repay", { amountUsdc: "10.00" }, "Manual repayment"),
  fundVault: () => submitWalletTx("fund-vault", { amountUsdc: "100.00" }, "Fund underwriter vault"),
  score: () => submitWalletTx("score", { score: 765, riskGrade: "B" }, "Score attestation"),
  suspend: () => submitWalletTx("suspend", {}, "Suspend credit line"),
  resume: () => submitWalletTx("resume", {}, "Resume credit line"),
  close: () => submitWalletTx("close", {}, "Close repaid credit line"),
  markDefault: () => submitWalletTx("default", {}, "Mark default"),
  refresh: () => refreshState(),
};

window.solana?.on?.("accountChanged", async (publicKey) => {
  walletAddress = publicKey?.toString?.() ?? null;
  walletSolBalance = null;
  render();
  await refreshWalletBalance();
});

await refreshState();

async function refreshState() {
  try {
    const response = await fetch("/v1/state");
    if (!response.ok) throw new Error((await response.json()).message ?? "State fetch failed");
    state = await response.json();
    deployment = state.deployment;
    await refreshWalletBalance({ rerender: false });
  } catch (error) {
    state = null;
    walletSolBalance = null;
    txNote = error.message;
  }
  render();
}

async function connectWallet() {
  try {
    if (!window.solana) {
      txNote = "No Solana wallet detected. Install Phantom or another Solana wallet.";
      render();
      return;
    }
    const result = await window.solana.connect();
    walletAddress = result.publicKey.toString();
    await refreshWalletBalance({ rerender: false });
    txNote = "";
  } catch (error) {
    txNote = extractErrorMessage(error);
  }
  render();
}

async function disconnectWallet() {
  await window.solana?.disconnect?.();
  walletAddress = null;
  walletSolBalance = null;
  render();
}

async function airdropSol() {
  if (!walletAddress) {
    txNote = "Connect a wallet first.";
    render();
    return;
  }
  txStatus = "Requesting airdrop...";
  render();
  try {
    const result = await postJson("/v1/airdrop", { wallet: walletAddress, sol: 2 });
    txStatus = "Airdrop complete";
    txNote = `Airdropped 2 SOL to ${shortKey(result.wallet)}. Signature ${shortKey(result.signature)}.`;
    await refreshWalletBalance({ rerender: false });
  } catch (error) {
    txStatus = "Idle";
    txNote = extractErrorMessage(error);
  }
  render();
}

async function submitWalletTx(kind, body, label) {
  if (!walletAddress) {
    txNote = "Connect the local dev wallet first.";
    render();
    return;
  }
  if (!web3) {
    txNote = "Solana web3 bundle did not load.";
    render();
    return;
  }

  txStatus = `${label}...`;
  txNote = "";
  render();

  try {
    const built = await postJson(`/v1/transactions/${kind}`, { ...body, wallet: walletAddress });
    const txBytes = Uint8Array.from(atob(built.transaction), (char) => char.charCodeAt(0));
    const tx = web3.Transaction.from(txBytes);
    const provider = window.solana;
    if (!provider) throw new Error("No Solana wallet connected.");

    let signature;
    if (provider.signAndSendTransaction) {
      signature = (await provider.signAndSendTransaction(tx)).signature;
    } else if (provider.signTransaction) {
      const signed = await provider.signTransaction(tx);
      const connection = new web3.Connection(deployment.rpcUrl, "confirmed");
      signature = await connection.sendRawTransaction(signed.serialize());
    } else {
      throw new Error("Connected wallet cannot sign transactions.");
    }

    const connection = new web3.Connection(deployment.rpcUrl, "confirmed");
    await connection.confirmTransaction(
      {
        signature,
        blockhash: built.blockhash,
        lastValidBlockHeight: built.lastValidBlockHeight,
      },
      "confirmed",
    );

    txStatus = `${label} complete`;
    txNote = `Signature ${shortKey(signature)} confirmed on ${deployment.rpcUrl}.`;
    await refreshState();
  } catch (error) {
    txStatus = "Idle";
    txNote = extractErrorMessage(error);
    render();
  }
}

async function refreshWalletBalance({ rerender = true } = {}) {
  if (!walletAddress || !deployment || !web3) {
    walletSolBalance = null;
    if (rerender) render();
    return;
  }

  const requestedWallet = walletAddress;
  try {
    const connection = new web3.Connection(deployment.rpcUrl, "confirmed");
    const lamports = await connection.getBalance(new web3.PublicKey(requestedWallet), "confirmed");
    if (walletAddress === requestedWallet) {
      walletSolBalance = lamports / web3.LAMPORTS_PER_SOL;
    }
  } catch {
    if (walletAddress === requestedWallet) walletSolBalance = null;
  }

  if (rerender) render();
}

function render() {
  if (!state) {
    app.innerHTML = `
      <main class="empty-shell">
        <section class="setup-panel">
          <p class="eyebrow">AGC Credit Control</p>
          <h1>Local onchain sandbox is not ready.</h1>
          <p>${escapeHtml(txNote || "Run npm run setup:local, then npm run dev.")}</p>
          <code>cd experiment002 && npm run setup:local && npm run dev</code>
        </section>
      </main>
    `;
    return;
  }

  const { accounts, tokenBalances } = state;
  const expectedWallet = deployment.publicKeys.admin;
  const walletMatches = walletAddress && walletAddress === expectedWallet;

  app.innerHTML = `
    <div class="shell">
      <header class="topbar">
        <a class="brand" href="./">
          <span>
            <strong>AGC Credit Control</strong>
            <small>Wallet-signed local Anchor transactions</small>
          </span>
        </a>
        <div class="role-tabs" role="tablist" aria-label="View">
          <button type="button" data-tab="operator" data-active="${activeTab === "operator"}">Operator</button>
          <button type="button" data-tab="underwriter" data-active="${activeTab === "underwriter"}">Underwriter</button>
          <button type="button" data-tab="chain" data-active="${activeTab === "chain"}">Onchain state</button>
        </div>
        <div class="wallet-controls">
          ${topbarAirdropButton()}
          ${walletAddress ? walletChip() : ""}
          <button class="button ${walletAddress ? "button-secondary" : "button-primary"}" data-action="${walletAddress ? "disconnectWallet" : "connectWallet"}">
            <span>${walletAddress ? "Disconnect" : "Connect wallet"}</span>
          </button>
        </div>
      </header>

      <section class="ticker-strip" aria-label="Live credit telemetry">
        ${tickerItem("credit line", accounts.creditLine?.availableLimitUsdc ?? "-", accounts.creditLine?.status ?? "missing")}
        ${tickerItem("outstanding", accounts.creditLine?.principalOutstandingUsdc ?? "-", `${(accounts.creditLine?.aprBps ?? 0) / 100}% APR`)}
        ${tickerItem("vault USDC", tokenBalances.vaultUsdc.uiAmount, accounts.underwriterVault?.status ?? "missing")}
        ${tickerItem("merchant received", tokenBalances.merchantUsdc.uiAmount, "settled spend")}
      </section>

      <main>
        ${walletMatches ? "" : walletWarning(expectedWallet)}
        ${activeTab === "operator" ? operatorView() : activeTab === "underwriter" ? underwriterView() : chainView()}
      </main>
    </div>
  `;

  app.querySelectorAll("[data-tab]").forEach((button) => {
    button.addEventListener("click", () => {
      activeTab = button.dataset.tab;
      render();
    });
  });
  app.querySelectorAll("[data-action]").forEach((button) => {
    button.addEventListener("click", () => actions[button.dataset.action]?.());
  });
}

function operatorView() {
  const { accounts, tokenBalances } = state;
  return `
    <section class="stage-strip" aria-label="Borrower workflow">
      ${stageItem("Protocol", accounts.protocol?.paused ? "paused" : "active", accounts.protocol?.paused ? "current" : "complete")}
      ${stageItem("Borrower", accounts.borrower?.verificationStatus ?? "missing", "complete")}
      ${stageItem("Workflow", accounts.workflow?.status ?? "missing", "complete")}
      ${stageItem("Credit line", accounts.creditLine?.status ?? "missing", "current")}
      ${stageItem("Wallet signer", walletAddress ? shortKey(walletAddress) : "not connected", walletAddress ? "complete" : "current")}
    </section>

    <section class="role-grid">
      <section class="panel">
        <div class="panel-heading">
          <div>
            <p class="eyebrow">Local deployment</p>
            <h1>AGC Credit Sandbox</h1>
          </div>
          <span class="state-pill" data-state="${accounts.creditLine?.status ?? "missing"}">${accounts.creditLine?.status ?? "missing"}</span>
        </div>
        <div class="summary-grid">
          ${detail("Program", shortKey(deployment.programId))}
          ${detail("RPC", deployment.rpcUrl)}
          ${detail("Dev wallet", shortKey(deployment.publicKeys.admin))}
          ${detail("USDC mint", shortKey(deployment.addresses.usdcMint))}
          ${detail("Credit line", shortKey(deployment.addresses.creditLine))}
          ${detail("Policy", shortKey(deployment.addresses.policy))}
        </div>
      </section>

      <section class="panel">
        <div class="panel-heading panel-heading-inline">
          <div>
            <p class="eyebrow">Wallet</p>
            <h2>Signer requirements</h2>
          </div>
          <button class="icon-button" data-action="refresh" type="button" aria-label="Refresh onchain state" title="Refresh onchain state">
            ${refreshIcon()}
          </button>
        </div>
        <div class="contract-list">
          ${contractRow("admin", "risk actions", shortKey(deployment.publicKeys.admin))}
          ${contractRow("payment_router", "spend reserve + settle", shortKey(deployment.publicKeys.router))}
          ${contractRow("underwriter", "vault funding", shortKey(deployment.publicKeys.underwriter))}
          ${contractRow("revenue_payer", "sweep + repay", shortKey(deployment.publicKeys.revenuePayer))}
        </div>
      </section>
    </section>

    <section class="role-grid">
      <section class="panel">
        <div class="panel-heading">
          <div>
            <p class="eyebrow">Credit line</p>
            <h2>${shortKey(deployment.addresses.creditLine)}</h2>
          </div>
          <span class="state-pill" data-state="${accounts.creditLine?.status ?? "missing"}">${accounts.creditLine?.status ?? "missing"}</span>
        </div>
        <div class="summary-grid">
          ${detail("Limit", accounts.creditLine?.principalLimitUsdc ?? "-")}
          ${detail("Available", accounts.creditLine?.availableLimitUsdc ?? "-")}
          ${detail("Outstanding", accounts.creditLine?.principalOutstandingUsdc ?? "-")}
          ${detail("Risk grade", accounts.creditLine?.riskGrade ?? "-")}
          ${detail("Daily spend", accounts.policy?.dailySpendUsdc ?? "-")}
          ${detail("Revenue payer USDC", tokenBalances.revenuePayerUsdc.uiAmount)}
        </div>
      </section>

      <section class="panel">
        <div class="panel-heading">
          <div>
            <p class="eyebrow">Wallet-signed actions</p>
            <h2>Spend and repay</h2>
          </div>
        </div>
        <div class="action-section">
          <span>Use credit</span>
          <div class="action-grid">
            ${actionButton("spendSearch", "Search API spend", "$2.50", "reserve_spend + settle_spend", true)}
            ${actionButton("spendReview", "Review threshold", "$11.00", "expected rejection")}
            ${actionButton("spendBlocked", "Blocked merchant", "$2.50", "expected rejection", false, true)}
          </div>
        </div>
        <div class="action-section">
          <span>Revenue and repayment</span>
          <div class="action-grid">
            ${actionButton("revenue", "Record revenue", "$50.00", "record_revenue_and_sweep")}
            ${actionButton("manualRepay", "Manual repay", "$10.00", "manual_repay")}
          </div>
        </div>
        ${decisionView()}
      </section>
    </section>

    ${policyAndHistoryView()}
  `;
}

function underwriterView() {
  const { accounts, tokenBalances } = state;
  return `
    <section class="role-grid">
      <section class="panel">
        <div class="panel-heading">
          <div>
            <p class="eyebrow">Underwriter vault</p>
            <h1>${shortKey(deployment.addresses.underwriterVault)}</h1>
          </div>
          <span class="state-pill" data-state="${accounts.underwriterVault?.status ?? "missing"}">${accounts.underwriterVault?.status ?? "missing"}</span>
        </div>
        <div class="summary-grid">
          ${detail("Vault token balance", tokenBalances.vaultUsdc.uiAmount)}
          ${detail("Available capital", accounts.underwriterVault?.availableCapitalUsdc ?? "-")}
          ${detail("Committed", accounts.underwriterVault?.committedToLinesUsdc ?? "-")}
          ${detail("Principal drawn", accounts.underwriterVault?.principalDrawnUsdc ?? "-")}
          ${detail("Principal repaid", accounts.underwriterVault?.principalRepaidUsdc ?? "-")}
          ${detail("Loss realized", accounts.underwriterVault?.lossRealizedUsdc ?? "-")}
        </div>
      </section>

      <section class="panel">
        <div class="panel-heading">
          <div>
            <p class="eyebrow">Risk controls</p>
            <h2>Admin-signed line actions</h2>
          </div>
        </div>
        <div class="action-grid">
          ${actionButton("fundVault", "Fund vault", "$100.00", "fund_underwriter_vault", true)}
          ${actionButton("score", "Update score", "765 / B", "update_score_attestation")}
          ${actionButton("suspend", "Suspend line", "active only", "suspend_credit_line", false, true)}
          ${actionButton("resume", "Resume line", "suspended only", "resume_credit_line")}
          ${actionButton("markDefault", "Mark default", "suspended + debt", "mark_default", false, true)}
          ${actionButton("close", "Close repaid line", "zero balance", "close_repaid_credit_line")}
        </div>
        ${decisionView()}
      </section>
    </section>
    ${policyAndHistoryView()}
  `;
}

function chainView() {
  return `
    <section class="lower-grid">
      <section class="panel">
        <div class="panel-heading">
          <div>
            <p class="eyebrow">Addresses</p>
            <h2>Bootstrapped local accounts</h2>
          </div>
        </div>
        <div class="contract-list">
          ${Object.entries(flattenAddresses(deployment.addresses)).map(([label, value]) => contractRow(label, shortKey(value), value)).join("")}
        </div>
      </section>
      <section class="panel">
        <div class="panel-heading">
          <div>
            <p class="eyebrow">Raw state</p>
            <h2>Read from local validator</h2>
          </div>
        </div>
        <pre class="json-panel">${escapeHtml(JSON.stringify(state.accounts, null, 2))}</pre>
      </section>
    </section>
    ${policyAndHistoryView()}
  `;
}

function policyAndHistoryView() {
  const { accounts } = state;
  return `
    <section class="lower-grid">
      <section class="panel">
        <div class="panel-heading">
          <div>
            <p class="eyebrow">Spend policy</p>
            <h2>Policy v${accounts.policy?.version ?? "-"}</h2>
          </div>
          <span class="state-pill" data-state="${accounts.policy?.status ?? "missing"}">${accounts.policy?.status ?? "missing"}</span>
        </div>
        <div class="policy-grid">
          ${policyItem("Per transaction", accounts.policy?.maxPerTransactionUsdc ?? "-")}
          ${policyItem("Daily cap", accounts.policy?.maxDailySpendUsdc ?? "-")}
          ${policyItem("Weekly cap", accounts.policy?.maxWeeklySpendUsdc ?? "-")}
          ${policyItem("Review threshold", accounts.policy?.humanApprovalThresholdUsdc ?? "-")}
        </div>
        <div class="merchant-row">
          ${merchantPill("Search API", accounts.merchants.search?.status ?? "missing")}
          ${merchantPill("Blocked wallet", accounts.merchants.blocked?.status ?? "missing")}
        </div>
      </section>
      <section class="panel">
        <div class="panel-heading">
          <div>
            <p class="eyebrow">Onchain records</p>
            <h2>Spend authorizations and score attestations</h2>
          </div>
        </div>
        <div class="telemetry-columns">
          <div class="analytics-list">
            ${state.spendAuthorizations.slice(0, 6).map((item) => analyticsItem(`${item.status} ${shortKey(item.address)}`, item.amountUsdc)).join("") || analyticsItem("Spend authorizations", "None yet")}
          </div>
          <div class="analytics-list">
            ${state.scoreAttestations.slice(0, 6).map((item) => analyticsItem(`Score ${item.score}`, `${item.riskGrade} / ${item.recommendedLimitUsdc}`)).join("") || analyticsItem("Score attestations", "None yet")}
          </div>
        </div>
      </section>
    </section>
    <section class="panel event-panel">
      <div class="panel-heading">
        <div>
          <p class="eyebrow">Recent transactions</p>
          <h2>Credit line signatures</h2>
        </div>
      </div>
      <div class="event-list">
        ${state.signatures.map((item) => eventRow(item)).join("") || `<article class="event-row"><strong>No signatures yet</strong></article>`}
      </div>
    </section>
  `;
}

function walletWarning(expectedWallet) {
  return `
    <section class="notice-band">
      <strong>${walletAddress ? "Connected wallet is not the bootstrapped dev wallet." : "Connect the bootstrapped dev wallet."}</strong>
      <span>The local bootstrap uses ${shortKey(expectedWallet)} as admin, payment router, underwriter, and revenue payer. Run <code>npm run dev-wallet</code>, import the printed local-only secret into Phantom, and point the wallet RPC at <code>http://127.0.0.1:8899</code>.</span>
    </section>
  `;
}

function topbarAirdropButton() {
  if (!walletAddress || walletSolBalance === null || walletSolBalance >= AIRDROP_THRESHOLD_SOL) return "";
  return `
    <button class="button button-primary topbar-airdrop" data-action="airdrop" type="button" title="${formatSol(walletSolBalance)} SOL available">
      <span>Airdrop SOL</span>
    </button>
  `;
}

function walletChip() {
  const balance = walletSolBalance === null ? "..." : `${formatSol(walletSolBalance)} SOL`;
  return `
    <span class="wallet-address" title="${escapeHtml(walletAddress)}">
      <span>${shortKey(walletAddress)}</span>
      <strong>${escapeHtml(balance)}</strong>
    </span>
  `;
}

function tickerItem(label, value, meta) {
  return `<div class="ticker-item"><span>${escapeHtml(label)}</span><strong>${escapeHtml(value)}</strong><small>${escapeHtml(meta)}</small></div>`;
}

function stageItem(label, value, state) {
  return `<div class="stage-item" data-state="${escapeHtml(state)}"><span>${escapeHtml(label)}</span><strong>${escapeHtml(value)}</strong></div>`;
}

function detail(label, value) {
  return `<div class="detail-item"><span>${escapeHtml(label)}</span><strong>${escapeHtml(value)}</strong></div>`;
}

function policyItem(label, value) {
  return `<div class="policy-item"><span>${escapeHtml(label)}</span><strong>${escapeHtml(value)}</strong></div>`;
}

function contractRow(instruction, label, value) {
  return `<div class="contract-row"><code>${escapeHtml(instruction)}</code><strong title="${escapeHtml(value)}">${escapeHtml(label)}</strong><span>${escapeHtml(value)}</span></div>`;
}

function actionButton(action, label, value, instruction, isPrimary = false, isDanger = false) {
  const classes = ["button"];
  if (isPrimary) classes.push("button-primary");
  if (isDanger) classes.push("button-danger");
  return `<button class="${classes.join(" ")}" data-action="${escapeHtml(action)}"><span>${escapeHtml(label)}</span><strong>${escapeHtml(value)}</strong><small>${escapeHtml(instruction)}</small></button>`;
}

function merchantPill(label, state) {
  return `<span class="merchant-pill" data-state="${state === "active" ? "allowed" : "blocked"}">${escapeHtml(label)} / ${escapeHtml(state)}</span>`;
}

function analyticsItem(label, value) {
  return `<div class="analytics-item"><span>${escapeHtml(label)}</span><strong>${escapeHtml(String(value))}</strong></div>`;
}

function eventRow(item) {
  const time = item.blockTime ? new Date(item.blockTime * 1000).toLocaleString() : `slot ${item.slot}`;
  return `<article class="event-row"><time>${escapeHtml(time)}</time><strong>${escapeHtml(shortKey(item.signature))}</strong><code>${escapeHtml(item.confirmationStatus ?? "confirmed")}</code></article>`;
}

function refreshIcon() {
  return `
    <svg aria-hidden="true" viewBox="0 0 24 24" focusable="false">
      <path d="M20 11a8 8 0 0 0-14.7-4.4L4 8"></path>
      <path d="M4 4v4h4"></path>
      <path d="M4 13a8 8 0 0 0 14.7 4.4L20 16"></path>
      <path d="M20 20v-4h-4"></path>
    </svg>
  `;
}

function decisionView() {
  return `<div class="decision" data-decision="${txStatus.includes("complete") ? "approved" : txNote ? "manual_review" : "idle"}"><span>Transaction status</span><strong>${escapeHtml(txStatus)}</strong><p>${escapeHtml(txNote || "Waiting for a wallet-signed transaction.")}</p></div>`;
}

function flattenAddresses(addresses, prefix = "") {
  return Object.fromEntries(
    Object.entries(addresses).flatMap(([key, value]) => {
      const label = prefix ? `${prefix}.${key}` : key;
      if (value && typeof value === "object") return Object.entries(flattenAddresses(value, label));
      return [[label, value]];
    }),
  );
}

async function postJson(url, body) {
  const response = await fetch(url, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(body),
  });
  const payload = await response.json();
  if (!response.ok) throw new Error(payload.message ?? "Request failed");
  return payload;
}

function shortKey(value) {
  if (!value) return "-";
  return value.length <= 14 ? value : `${value.slice(0, 6)}...${value.slice(-6)}`;
}

function formatSol(value) {
  return Number(value).toFixed(value < 1 ? 3 : 2);
}

function extractErrorMessage(error) {
  if (error instanceof Error) return error.message;
  if (typeof error === "string") return error;
  return "Transaction failed.";
}

function escapeHtml(value) {
  return String(value ?? "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
}
