const app = document.querySelector("#app");
const web3 = window.solanaWeb3;

const NAV = [
  { id: "control", label: "Control Plane" },
  { id: "applications", label: "Applications" },
  { id: "lines", label: "Credit Lines" },
  { id: "analytics", label: "Analytics" },
];
const AIRDROP_THRESHOLD_SOL = 2;

let state = null;
let deployment = null;
let walletAddress = window.solana?.publicKey?.toString?.() ?? null;
let walletSolBalance = null;
let txStatus = "Idle";
let txNote = "";
let demoData = loadBoolPref("agc.demoData", true);
let activeSection = normalizeSection(loadPref("agc.section", "control"));
let amountInput = "75.00";
let copyResetTimer = null;
let referenceOpen = false;
let sidebarCollapsed = loadBoolPref("agc.sidebarCollapsed", false);

const actions = {
  connectWallet,
  disconnectWallet,
  airdrop: () => airdropSol(),
  fundPool: () => submitWalletTx("fund-pool", { amountUsdc: currentAmount() }, "Fund pool"),
  draw: () => submitWalletTx("draw", { amountUsdc: currentAmount() }, "Draw credit"),
  repay: () => submitWalletTx("repay", { amountUsdc: currentAmount() }, "Repay credit"),
  score: () => submitWalletTx("score", { trustScore: 872, riskGrade: "A" }, "Update attestation"),
  suspend: () => submitWalletTx("suspend", {}, "Suspend line"),
  resume: () => submitWalletTx("resume", {}, "Resume line"),
  close: () => submitWalletTx("close", {}, "Close line"),
  markDefault: () => submitWalletTx("default", {}, "Mark default"),
  refresh: () => refreshState(),
  toggleDemo: () => {
    demoData = !demoData;
    savePref("agc.demoData", String(demoData));
    render();
  },
  toggleReference: () => {
    referenceOpen = !referenceOpen;
    render();
  },
  toggleSidebar: () => {
    sidebarCollapsed = !sidebarCollapsed;
    savePref("agc.sidebarCollapsed", String(sidebarCollapsed));
    render();
  },
};

document.addEventListener("keydown", (event) => {
  if (event.key === "Escape" && referenceOpen) {
    referenceOpen = false;
    render();
  }
});

window.solana?.on?.("accountChanged", async (publicKey) => {
  walletAddress = publicKey?.toString?.() ?? null;
  walletSolBalance = null;
  render();
  await refreshWalletBalance();
});

window.addEventListener?.("popstate", () => render());

renderBoot();
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
    txNote = "Connect the bootstrapped local wallet first.";
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
          <p class="eyebrow">Agent Credit Market</p>
          <h1>Local onchain market is not ready.</h1>
          <p>${escapeHtml(txNote || "Run npm run setup:local, then npm run dev.")}</p>
          <code>cd experiment002 &amp;&amp; npm run setup:local &amp;&amp; npm run dev</code>
        </section>
      </main>
    `;
    return;
  }

  const route = currentRoute();
  app.innerHTML = `
    <div class="site-frame ${route}-page">
      ${route === "dashboard" ? dashboardPage() : landingPage()}
      ${route === "dashboard" ? "" : siteFooter()}
    </div>
    ${referenceOpen ? referenceOverlay() : ""}
  `;

  bindEvents();
  hydrateReveals();
}

function currentRoute() {
  return window.location.pathname === "/dashboard" || window.location.pathname === "/dashboard/" ? "dashboard" : "landing";
}

function renderBoot() {
  app.innerHTML = `
    <main class="boot-shell" aria-busy="true" aria-label="Loading control plane">
      <div class="boot-card">
        <span class="brand-mark" aria-hidden="true">AG</span>
        <div class="boot-spinner"></div>
        <p>Connecting to onchain state…</p>
      </div>
    </main>
  `;
}

function bindEvents() {
  app.querySelectorAll("[data-route]").forEach((el) => {
    el.addEventListener("click", (event) => {
      event.preventDefault();
      const path = el.dataset.route === "dashboard" ? "/dashboard" : "/";
      if (window.location.pathname !== path) window.history.pushState({}, "", path);
      render();
      window.scrollTo?.({ top: 0, behavior: "auto" });
    });
  });
  app.querySelectorAll("[data-section]").forEach((el) => {
    el.addEventListener("click", () => {
      activeSection = normalizeSection(el.dataset.section);
      savePref("agc.section", activeSection);
      render();
    });
  });
  app.querySelectorAll("[data-action]").forEach((el) => {
    el.addEventListener("click", () => actions[el.dataset.action]?.());
  });
  app.querySelectorAll("[data-copy]").forEach((el) => {
    el.addEventListener("click", () => copyValue(el));
  });
  const amount = app.querySelector("#amount");
  if (amount) amount.addEventListener("input", (event) => { amountInput = event.target.value; });
  const overlay = app.querySelector(".reference-overlay");
  if (overlay) {
    overlay.addEventListener("click", (event) => {
      if (event.target === overlay) {
        referenceOpen = false;
        render();
      }
    });
  }
}

function hydrateReveals() {
  const targets = app.querySelectorAll(".reveal");
  if (!targets.length) return;
  if (!("IntersectionObserver" in window)) {
    targets.forEach((target) => target.classList.add("visible"));
    return;
  }
  const observer = new IntersectionObserver(
    (entries) => {
      entries.forEach((entry) => {
        if (entry.isIntersecting) {
          entry.target.classList.add("visible");
          observer.unobserve(entry.target);
        }
      });
    },
    { rootMargin: "0px 0px -12% 0px", threshold: 0.12 },
  );
  targets.forEach((target) => observer.observe(target));
}

function landingPage() {
  const market = state.accounts.market;
  const creditLine = state.accounts.creditLine;
  const pool = state.accounts.liquidityPool;
  const utilization = poolUtilization();
  const outstanding = market?.totalOutstandingUsdc ?? creditLine?.principalOutstandingUsdc ?? "-";
  const liquidity = market?.totalLiquidityUsdc ?? pool?.committedCapitalUsdc ?? "-";
  const limit = market?.totalCreditLimitUsdc ?? creditLine?.principalLimitUsdc ?? "-";

  return `
    <header class="site-nav" aria-label="Primary">
      <a class="brand-lockup" href="/" data-route="landing" aria-label="Agent Credit Market home">
        <span class="brand-mark" aria-hidden="true">AG</span>
        <span><strong>Agent Credit</strong><small>Market Control</small></span>
      </a>
      <nav class="nav-pills" aria-label="Landing navigation">
        <a href="#market">Market</a>
        <a href="#underwriting">Underwriting</a>
        <a href="#verifiable-credit">Credit</a>
        <a href="#assurance">Assurance</a>
        <a href="#waitlist">Waitlist</a>
        <a href="/dashboard" data-route="dashboard">Dashboard</a>
      </nav>
      <a class="cta-button compact" href="/dashboard" data-route="dashboard"><span>Open Control</span><i aria-hidden="true">-&gt;</i></a>
    </header>

    <main class="landing" id="top">
      <section class="hero-panel reveal" aria-labelledby="hero-title">
        <div class="hero-copy">
          <p class="eyebrow">Stablecoin credit infrastructure</p>
          <h1 id="hero-title">Programmable credit lines for agent-run markets.</h1>
          <p class="hero-lede">Experiment 002 turns a local Solana validator into a live credit control plane: LP pools, verified borrowers, policy-priced applications, drawdowns, repayments, and account proofs in one interface.</p>
          <div class="hero-actions">
            <a class="cta-button" href="/dashboard" data-route="dashboard"><span>Operate the market</span><i aria-hidden="true">-&gt;</i></a>
            <button class="ghost-button" data-action="toggleReference" type="button"><span>Inspect accounts</span><i aria-hidden="true">~</i></button>
          </div>
        </div>
        <div class="hero-visual" aria-label="Market signal preview">
          <div class="orbit-card primary">
            <span>Local liquidity</span>
            <strong>USDC ${escapeHtml(liquidity)}</strong>
            <small>${utilization}% committed to active lines</small>
          </div>
          <div class="signal-board">
            <div class="signal-head">
              <span>Policy engine</span>
              <b>${prettyEnum(pool?.approvalMode ?? "Manual")}</b>
            </div>
            <div class="signal-grid" aria-hidden="true">
              ${signalBars().map((height) => `<i style="height: ${height}%"></i>`).join("")}
            </div>
            <div class="signal-row"><span>Credit limit</span><strong>USDC ${escapeHtml(limit)}</strong></div>
            <div class="signal-row"><span>Outstanding</span><strong>USDC ${escapeHtml(outstanding)}</strong></div>
          </div>
          <div class="orbit-card secondary">
            <span>Risk grade</span>
            <strong>${riskGradeLabel(creditLine?.riskGrade)} ${riskLabel(riskGradeLabel(creditLine?.riskGrade))}</strong>
            <small>${creditLine ? `${(creditLine.aprBps ?? 0) / 100}% APR` : "Awaiting line"}</small>
          </div>
        </div>
      </section>

      <section class="market-bento reveal" id="market" aria-label="Market architecture">
        <article class="bento-card tall">
          <span>Liquidity side</span>
          <h2>Capital allocators set the policy, not the spreadsheet.</h2>
          <p>Pool managers can fund USDC, cap APR, define minimum trust score, and require admin review before deployment.</p>
        </article>
        <article class="bento-card">
          <span>Borrower side</span>
          <h3>Credit profiles travel with verifiable state.</h3>
          <p>Borrowers carry risk grade, repayment count, trust score, and operator authorization into each line.</p>
        </article>
        <article class="bento-card accent">
          <span>Execution side</span>
          <h3>Wallet signatures are the operating boundary.</h3>
          <p>The server builds unsigned transactions; the browser wallet signs, sends, confirms, and refreshes onchain state.</p>
        </article>
      </section>

      <section class="underwriting-section reveal" id="underwriting" aria-labelledby="underwriting-title">
        <div class="section-copy">
          <p class="eyebrow">Underwriting engine</p>
          <h2 id="underwriting-title">Automated risk work, lender-controlled policy.</h2>
          <p>LPs can run mechanical rules, agentic document review, or human escalation thresholds. The market generates caps, rates, covenants, and repayment requirements before capital is deployed.</p>
        </div>
        <div class="policy-panel">
          <div class="panel-title">
            <strong>Atlas approval policy</strong>
            <span>Live simulation</span>
          </div>
          <div class="policy-steps">
            ${policyStep("Auto approve", "Trust score above 820, cash-flow coverage above 1.4x")}
            ${policyStep("Agent review", "Unstructured statements, invoices, bank exports, tax docs")}
            ${policyStep("Escalate", "Jurisdiction risk, weak repayment history, covenant breach")}
          </div>
          <div class="facility-card">
            <span>Recommended facility</span>
            <strong>USDC 5.0M at 8.75% APR</strong>
          </div>
        </div>
      </section>

      <section class="credit-flow-section reveal" id="verifiable-credit" aria-labelledby="credit-flow-title">
        <div class="section-copy">
          <p class="eyebrow">Verifiable credit</p>
          <h2 id="credit-flow-title">Prove what matters. Keep sensitive data out of custody.</h2>
          <p>Borrowers can submit business history, financials, ownership data, collateral, and repayment records. The market verifies the signal, produces credit attestations, and lets borrowers authorize temporary lender access when deeper review is needed.</p>
        </div>
        <div class="credit-flow" aria-label="Credit verification flow">
          ${flowNode("Business data")}
          ${flowNode("zk proof")}
          ${flowNode("Trust score")}
          ${flowNode("Credit line")}
        </div>
      </section>

      <section class="assurance-section reveal" id="assurance">
        <div>
          <p class="eyebrow">Assurance model</p>
          <h2>Trust starts with controls you can inspect.</h2>
        </div>
        <div class="assurance-stack">
          ${assuranceItem("Wallet custody boundary", "The server builds unsigned transactions. The connected wallet signs, sends, and confirms execution.")}
          ${assuranceItem("Policy-bound credit", "Pool caps, APR limits, trust-score floors, drawdowns, repayments, and fee routing are reflected in program state.")}
          ${assuranceItem("Observable accounting", "Liquidity, outstanding principal, repayment history, and fee-vault balances are read back from local validator accounts.")}
        </div>
      </section>

      <section class="waitlist-section reveal" id="waitlist" aria-labelledby="waitlist-title">
        <div class="waitlist-copy">
          <p class="eyebrow">Private beta</p>
          <h2 id="waitlist-title">Join the Experiment 002 waitlist.</h2>
          <p>Get the next credit-market sandbox drop, deployment notes, and early access to hosted control-plane builds.</p>
        </div>
        <form class="waitlist-form" name="waitlist" method="post" data-netlify="true" netlify-honeypot="bot-field">
          <input type="hidden" name="form-name" value="waitlist" />
          <label class="bot-field">Do not fill this out<input name="bot-field" /></label>
          <label for="waitlist-email">Email</label>
          <div class="email-row">
            <input id="waitlist-email" name="email" type="email" placeholder="you@company.com" required />
            <button type="submit">Request access</button>
          </div>
          <span>No spam. Just product access and experiment notes.</span>
        </form>
      </section>
    </main>
  `;
}

function dashboardPage() {
  return `
    <main class="dashboard-app" id="dashboard" aria-label="Experiment 002 dashboard">
      <div class="shell${sidebarCollapsed ? " rail" : ""}">
        ${currentView()}
      </div>
    </main>
  `;
}

function signalBars() {
  return [46, 62, 38, 78, 54, 86, 68, 42, 74, 58, 92, 64];
}

function assuranceItem(title, body) {
  return `
    <article class="assurance-item">
      <span>${escapeHtml(title)}</span>
      <p>${escapeHtml(body)}</p>
    </article>
  `;
}

function policyStep(label, body) {
  return `
    <div class="policy-step">
      <b>${escapeHtml(label)}</b>
      <span>${escapeHtml(body)}</span>
    </div>
  `;
}

function flowNode(label) {
  return `<div class="flow-node">${escapeHtml(label)}</div>`;
}

function siteFooter() {
  return `
    <footer class="site-footer">
      <a href="https://x.com/AgentCreditSOL" target="_blank" rel="noreferrer">X</a>
      <a href="https://github.com/c0rv0s/agc" target="_blank" rel="noreferrer">Github</a>
      <a href="/dashboard" data-route="dashboard">Dashboard</a>
    </footer>
  `;
}

function currentView() {
  switch (activeSection) {
    case "applications":
      return applicationsView();
    case "lines":
      return linesView();
    case "analytics":
      return analyticsView();
    default:
      return controlView();
  }
}

function demoToggle() {
  return `
    <button class="demo-toggle ${demoData ? "on" : ""}" data-action="toggleDemo" type="button" role="switch" aria-checked="${demoData}" title="${demoData ? "Demo data is shown. Click to show only real onchain data." : "Showing only real onchain data. Click to add demo content."}">
      <span class="demo-track"><span class="demo-dot"></span></span>
      <span class="demo-label">Demo data</span>
    </button>
  `;
}

function controlView() {
  return `
    <section class="control-plane" aria-label="Credit control plane">
      ${consoleSidebar()}
      <div class="console-main">
        ${sectionHeader("Credit Control Plane", "Applications priced by policy, proofs, and risk agents.")}
        ${queueTabs()}
        ${borrowerRows()}
        <div class="console-lower">
          ${proofStrip()}
        </div>
      </div>
      ${termSheet()}
    </section>
  `;
}

function applicationsView() {
  const apps = state.applications ?? [];
  const rows = apps.length
    ? apps.map(applicationRow).join("")
    : `<div class="data-empty">No credit applications onchain yet.</div>`;
  return `
    <section class="control-plane wide-plane" aria-label="Applications">
      ${consoleSidebar()}
      <div class="wide-main">
        ${sectionHeader("Applications", "Credit applications submitted against pool policy.")}
        <section class="panel">
          <div class="data-table">
            <div class="data-head">
              <span>Application</span><span>Status</span><span>Requested</span><span>Proposed APR</span><span>Trust</span><span>Submitted</span>
            </div>
            ${rows}
          </div>
        </section>
      </div>
    </section>
  `;
}

function applicationRow(application) {
  return `
    <div class="data-row">
      <code title="${escapeHtml(application.address)}">${shortKey(application.address)}</code>
      <span class="status-pill ${statusTone(application.status)}">${prettyEnum(application.status)}</span>
      <span>USDC ${escapeHtml(application.requestedLimitUsdc)}</span>
      <span>${(application.proposedAprBps ?? 0) / 100}%</span>
      <span>${application.trustScoreSnapshot ?? "-"}</span>
      <span>${formatDate(application.createdAt)}</span>
    </div>
  `;
}

function linesView() {
  const creditLine = state.accounts.creditLine;
  return `
    <section class="control-plane" aria-label="Credit lines">
      ${consoleSidebar()}
      <div class="console-main">
        ${sectionHeader("Credit Line", "Deployed revolving facility and its guardrails.")}
        ${creditLineDetail(creditLine)}
      </div>
      ${termSheet()}
    </section>
  `;
}

function creditLineDetail(creditLine) {
  if (!creditLine) return `<div class="data-empty tall">No credit line deployed yet.</div>`;
  const grade = riskGradeLabel(creditLine.riskGrade);
  return `
    <div class="line-banner ${statusTone(creditLine.status)}">
      <div><span>Status</span><strong>${prettyEnum(creditLine.status)}</strong></div>
      <div><span>Risk grade</span><strong>${grade} ${riskLabel(grade)}</strong></div>
      <div><span>APR</span><strong>${(creditLine.aprBps ?? 0) / 100}%</strong></div>
      <div><span>Coverage</span><strong>${(creditLine.collateralizationBps ?? 0) / 100}%</strong></div>
    </div>
    <div class="metric-grid">
      ${metric("Principal limit", `USDC ${creditLine.principalLimitUsdc}`)}
      ${metric("Available", `USDC ${creditLine.availableLimitUsdc}`)}
      ${metric("Outstanding", `USDC ${creditLine.principalOutstandingUsdc}`)}
      ${metric("Repaid", `USDC ${creditLine.principalRepaidUsdc}`)}
      ${metric("Platform fees", `USDC ${creditLine.platformFeesPaidUsdc}`)}
      ${metric("Maturity", formatDate(creditLine.maturityAt))}
      ${metric("Last draw", formatDate(creditLine.lastDrawAt))}
      ${metric("Last repayment", formatDate(creditLine.lastRepaymentAt))}
    </div>
  `;
}

function analyticsView() {
  const market = state.accounts.market;
  return `
    <section class="control-plane wide-plane" aria-label="Analytics">
      ${consoleSidebar()}
      <div class="wide-main">
        ${sectionHeader("Analytics", "Market exposure, utilization, and onchain activity.")}
        <div class="kpi-grid">
          ${kpi("Total liquidity", market ? `USDC ${market.totalLiquidityUsdc}` : "-")}
          ${kpi("Total credit limit", market ? `USDC ${market.totalCreditLimitUsdc}` : "-")}
          ${kpi("Outstanding", market ? `USDC ${market.totalOutstandingUsdc}` : "-")}
          ${kpi("Platform fees", market ? `USDC ${market.totalPlatformFeesUsdc}` : "-")}
        </div>
        <div class="analytics-lower">
          <section class="panel">
            ${panelHead("Utilization", "Pool deployment")}
            ${utilizationBlock()}
          </section>
          <section class="panel">
            ${panelHead("Activity", "Recent onchain transactions")}
            ${activityFeed(state.signatures ?? [])}
          </section>
        </div>
      </div>
    </section>
  `;
}

function referenceOverlay() {
  return `
    <div class="reference-overlay">
      <aside class="reference-panel" role="dialog" aria-modal="true" aria-label="Onchain reference">
        <header class="reference-head">
          <div>
            <p class="eyebrow">Reference</p>
            <h2>Onchain accounts</h2>
            <span>Bootstrapped local validator state for the Agent Credit program.</span>
          </div>
          <button class="icon-button" data-action="toggleReference" type="button" aria-label="Close reference" title="Close">${closeIcon()}</button>
        </header>
        <div class="reference-body">
          <section class="panel">
            ${panelHead("Addresses", "Market accounts")}
            <div class="contract-list">
              ${Object.entries(flattenAddresses(deployment.addresses)).map(([label, value]) => contractRow(label, value)).join("")}
            </div>
          </section>
          <section class="panel">
            ${panelHead("Raw state", "Read from local validator")}
            <pre class="json-panel">${escapeHtml(JSON.stringify(state.accounts, null, 2))}</pre>
          </section>
        </div>
      </aside>
    </div>
  `;
}

function consoleSidebar() {
  const pool = state.accounts.liquidityPool;
  const counts = {
    applications: Math.max(state.applications.length, 1),
    lines: state.accounts.creditLine ? 1 : 0,
  };
  const poolName = demoData ? "Atlas Growth Fund" : "Liquidity Pool";
  const utilization = poolUtilization();
  const toggleLabel = sidebarCollapsed ? "Expand sidebar" : "Collapse sidebar";
  return `
    <aside class="console-sidebar">
      <div class="sidebar-head">
        <div class="sidebar-title">
          <strong>Agent Credit Market</strong>
        </div>
        <button class="rail-toggle" data-action="toggleSidebar" type="button" aria-label="${toggleLabel}" title="${toggleLabel}">${collapseIcon()}</button>
      </div>
      <div class="pool-card">
        <span>Your pool</span>
        <strong>${escapeHtml(poolName)}</strong>
        ${walletBlock()}
      </div>
      <ul>
        ${NAV.map((item) => `
          <li class="${activeSection === item.id ? "active" : ""}">
            <button type="button" data-section="${item.id}" title="${escapeHtml(item.label)}">
              <span class="nav-icon">${navIcon(item.id)}</span>
              <span class="nav-label">${escapeHtml(item.label)}</span>
              ${counts[item.id] != null ? `<b>${counts[item.id]}</b>` : ""}
            </button>
          </li>
        `).join("")}
      </ul>
      <div class="sidebar-foot">
        <div class="sidebar-ledger">
          <span>Policy mode</span>
          <strong>${prettyEnum(pool?.approvalMode ?? "missing")}</strong>
          <small>Min score ${pool?.minTrustScore ?? "-"} / Max APR ${((pool?.maxAprBps ?? 0) / 100).toFixed(2)}%</small>
        </div>
        <div class="health-card">
          <span>Pool utilization</span>
          <strong>${utilization}%</strong>
          <div class="meter"><i style="width: ${utilization}%"></i></div>
        </div>
        <div class="sidebar-tools">
          ${demoToggle()}
          <button class="icon-button" data-action="toggleReference" type="button" aria-label="Onchain reference" title="Onchain reference">${infoIcon()}</button>
          <button class="icon-button" data-action="refresh" type="button" aria-label="Refresh state" title="Refresh state">${refreshIcon()}</button>
        </div>
      </div>
    </aside>
  `;
}

function queueTabs() {
  const creditLine = state.accounts.creditLine;
  const queueCount = Math.max(state.applications.length, 1);
  const needsReviewCount = creditLine?.status === "suspended" ? 1 : 0;
  const escalationsCount = creditLine?.status === "defaulted" ? 1 : 0;
  return `
    <div class="queue-tabs">
      <span class="selected">Application Queue <b>${queueCount}</b></span>
      <span>Needs Review <b>${needsReviewCount}</b></span>
      <span>Escalations <b>${escalationsCount}</b></span>
    </div>
  `;
}

function borrowerRows() {
  const creditLine = state.accounts.creditLine;
  const borrower = state.accounts.borrower;
  const grade = riskGradeLabel(creditLine?.riskGrade);
  const name = demoData ? "Apex Global Ltd." : "Verified Borrower";
  const sub = demoData
    ? "Wholesale trade - UAE"
    : `${prettyEnum(borrower?.borrowerType ?? "-")} - ${shortKey(deployment.addresses.borrower)}`;
  const rows = [`
    <button class="borrower-row selected" type="button" data-section="lines">
      <span class="avatar">${demoData ? "AG" : "VB"}</span>
      <div>
        <strong>${escapeHtml(name)}</strong>
        <small>${escapeHtml(sub)}</small>
      </div>
      <span class="grade ${riskGradeTone(grade)}">${grade} ${riskLabel(grade)}</span>
      <span class="amount">USDC ${creditLine?.principalLimitUsdc ?? "-"}</span>
    </button>
  `];
  if (demoData) {
    rows.push(demoBorrowerRow("NB", "NileBridge Solutions", "Logistics - Egypt", "mid", "B+ Moderate", "USDC 3,750,000"));
    rows.push(demoBorrowerRow("PT", "PrimeTextile Co.", "Manufacturing - Vietnam", "high", "C+ Review", "USDC 2,250,000"));
  }
  return rows.join("");
}

function demoBorrowerRow(initials, name, industry, tone, gradeLabel, amount) {
  return `
    <article class="borrower-row muted" title="Demo data">
      <span class="avatar">${initials}</span>
      <div>
        <strong>${escapeHtml(name)}</strong>
        <small>${escapeHtml(industry)}</small>
      </div>
      <span class="grade ${tone}">${escapeHtml(gradeLabel)}</span>
      <span class="amount">${escapeHtml(amount)}</span>
    </article>
  `;
}

function proofStrip() {
  const borrower = state.accounts.borrower;
  return `
    <section class="proof-strip" aria-label="Borrower proofs">
      ${proofItem("Verification", prettyEnum(borrower?.verificationStatus ?? "-"))}
      ${proofItem("Trust score", `${borrower?.trustScore ?? "-"} / 1000`)}
      ${proofItem("Repayments", `${borrower?.repaymentCount ?? 0} onchain`)}
      ${proofItem("Fee vault", `${state.tokenBalances.feeVaultUsdc.uiAmount} USDC`)}
    </section>
  `;
}

function termSheet() {
  const creditLine = state.accounts.creditLine;
  const status = prettyEnum(creditLine?.status ?? "missing");
  return `
    <aside class="term-sheet">
      <div class="term-header">
        <p>Credit Term Sheet</p>
        <span>${demoData ? "AI generated" : "Onchain"}</span>
      </div>
      <dl>
        ${term("Facility", "Revolving credit line")}
        ${term("Limit", `USDC ${creditLine?.principalLimitUsdc ?? "-"}`)}
        ${term("Available", `USDC ${creditLine?.availableLimitUsdc ?? "-"}`)}
        ${term("APR", `${(creditLine?.aprBps ?? 0) / 100}%`)}
        ${term("Coverage", `${(creditLine?.collateralizationBps ?? 0) / 100}% receivables`)}
      </dl>
      ${repaymentVisual(creditLine)}
      <div class="contract-card">
        <span>Onchain contract</span>
        <strong>${status}</strong>
        <small>${shortKey(deployment.addresses.creditLine)}</small>
      </div>
      ${actionPanel()}
    </aside>
  `;
}

function actionPanel() {
  return `
    <div class="action-panel">
      <label class="amount-field">
        <span>Amount (USDC)</span>
        <input id="amount" type="text" inputmode="decimal" autocomplete="off" spellcheck="false" value="${escapeHtml(amountInput)}" />
      </label>
      <div class="action-grid">
        ${termAction("draw", "Draw", "Borrow against line", true)}
        ${termAction("repay", "Repay", "Principal + fee")}
        ${termAction("fundPool", "Fund pool", "Add liquidity")}
        ${termAction("score", "Update score", "AI attestation")}
      </div>
      <div class="guardrail-actions" aria-label="Guardrail actions">
        ${guardrailAction("suspend", "Suspend")}
        ${guardrailAction("resume", "Resume")}
        ${guardrailAction("close", "Close")}
        ${guardrailAction("markDefault", "Default", true)}
      </div>
    </div>
  `;
}

function repaymentVisual(creditLine) {
  if (demoData) {
    const heights = [38, 48, 44, 56, 62, 70, 76];
    return `<div class="repayment-chart" aria-hidden="true">${heights.map((height) => `<span style="height: ${height}%"></span>`).join("")}</div>`;
  }
  const limit = parseNumber(creditLine?.principalLimitUsdc) || 1;
  const bar = (label, raw, tone) => {
    const pct = Math.max(2, Math.min(100, Math.round((parseNumber(raw) / limit) * 100)));
    return `
      <div class="usage-row">
        <span>${escapeHtml(label)}</span>
        <div class="usage-track"><i class="${tone}" style="width: ${pct}%"></i></div>
        <b>USDC ${escapeHtml(raw ?? "-")}</b>
      </div>
    `;
  };
  return `
    <div class="usage-chart">
      ${bar("Outstanding", creditLine?.principalOutstandingUsdc, "out")}
      ${bar("Available", creditLine?.availableLimitUsdc, "avail")}
      ${bar("Repaid", creditLine?.principalRepaidUsdc, "repaid")}
    </div>
  `;
}

function utilizationBlock() {
  const pool = state.accounts.liquidityPool;
  if (!pool) return `<p class="empty-note">No liquidity pool deployed.</p>`;
  const capital = parseNumber(pool.committedCapitalUsdc) || 1;
  const bar = (label, raw, tone) => {
    const pct = Math.max(2, Math.min(100, Math.round((parseNumber(raw) / capital) * 100)));
    return `
      <div class="usage-row">
        <span>${escapeHtml(label)}</span>
        <div class="usage-track"><i class="${tone}" style="width: ${pct}%"></i></div>
        <b>USDC ${escapeHtml(raw ?? "-")}</b>
      </div>
    `;
  };
  return `
    <div class="usage-chart wide">
      ${bar("Committed to lines", pool.committedToLinesUsdc, "out")}
      ${bar("Available capital", pool.availableCapitalUsdc, "avail")}
      ${bar("Principal drawn", pool.principalDrawnUsdc, "repaid")}
    </div>
  `;
}

function activityFeed(signatures) {
  if (!signatures.length) {
    return `<p class="empty-note">No onchain transactions yet. Connect the wallet and run an action to see signatures here.</p>`;
  }
  return `
    <div class="activity-feed">
      ${signatures.map((item) => `
        <div class="activity-row">
          <code title="${escapeHtml(item.signature)}">${shortKey(item.signature)}</code>
          <span class="status-pill ${item.confirmationStatus === "finalized" || item.confirmationStatus === "confirmed" ? "low" : "mid"}">${prettyEnum(item.confirmationStatus ?? "processed")}</span>
          <span>Slot ${item.slot ?? "-"}</span>
          <span>${item.blockTime ? formatDate(new Date(item.blockTime * 1000).toISOString()) : "-"}</span>
          <button class="copy-button" type="button" data-copy="${escapeHtml(item.signature)}" aria-label="Copy signature">Copy</button>
        </div>
      `).join("")}
    </div>
  `;
}

function sectionHeader(title, subtitle) {
  return `
    <div class="section-header">
      <div>
        <p>${escapeHtml(title)}</p>
        <span>${escapeHtml(subtitle)}</span>
        ${operationNote()}
      </div>
    </div>
  `;
}

function panelHead(eyebrow, title) {
  return `
    <div class="panel-heading">
      <div>
        <p class="eyebrow">${escapeHtml(eyebrow)}</p>
        <h2>${escapeHtml(title)}</h2>
      </div>
    </div>
  `;
}

function operationNote() {
  if (txStatus === "Idle" && !txNote) return "";
  return `<span class="operation-note" role="status">${escapeHtml(txStatus)}${txNote ? ` - ${escapeHtml(txNote)}` : ""}</span>`;
}

function walletBlock() {
  if (!walletAddress) {
    return `<button class="button button-primary wallet-connect" data-action="connectWallet" type="button">Connect wallet</button>`;
  }
  const solBalance = walletSolBalance === null ? "..." : `${formatSol(walletSolBalance)} SOL`;
  const usdcBalance = walletUsdcBalance();
  const lowSol = walletSolBalance !== null && walletSolBalance < AIRDROP_THRESHOLD_SOL;
  return `
    <div class="wallet-block">
      <div class="wallet-balances">
        <div class="asset-line">
          <span>SOL</span>
          <strong>${escapeHtml(solBalance)}</strong>
        </div>
        <div class="asset-line">
          <span>USDC</span>
          <strong>${escapeHtml(usdcBalance)}</strong>
        </div>
      </div>
      <div class="wallet-address" title="${escapeHtml(walletAddress)}">
        <span>Wallet</span>
        <code>${shortKey(walletAddress)}</code>
      </div>
      <div class="wallet-actions">
        ${lowSol ? `<button class="button button-primary" data-action="airdrop" type="button">Airdrop SOL</button>` : ""}
        <button class="button button-secondary" data-action="disconnectWallet" type="button">Disconnect</button>
      </div>
    </div>
  `;
}

function walletUsdcBalance() {
  const keys = deployment.publicKeys ?? {};
  const balances = state.tokenBalances ?? {};
  const wallet = walletAddress ?? "";
  if (wallet === keys.poolManager) return `${balances.poolManagerUsdc?.uiAmount ?? "-"} USDC`;
  if (wallet === keys.borrowerOperator) return `${balances.borrowerUsdc?.uiAmount ?? "-"} USDC`;
  if (wallet === keys.repaymentPayer) return `${balances.repaymentPayerUsdc?.uiAmount ?? "-"} USDC`;
  return "-";
}

function term(label, value) {
  return `<div><dt>${escapeHtml(label)}</dt><dd>${escapeHtml(value)}</dd></div>`;
}

function metric(label, value) {
  return `<div class="metric"><span>${escapeHtml(label)}</span><strong>${escapeHtml(value)}</strong></div>`;
}

function kpi(label, value) {
  return `<div class="kpi-card"><span>${escapeHtml(label)}</span><strong>${escapeHtml(value)}</strong></div>`;
}

function contractRow(label, value) {
  return `
    <div class="contract-row">
      <code title="${escapeHtml(label)}">${escapeHtml(label)}</code>
      <span class="addr" title="${escapeHtml(value)}">${escapeHtml(value)}</span>
      <button class="copy-button" type="button" data-copy="${escapeHtml(value)}" aria-label="Copy ${escapeHtml(label)}">Copy</button>
    </div>
  `;
}

function termAction(action, label, sub, primary = false) {
  return `<button class="term-action${primary ? " primary" : ""}" data-action="${escapeHtml(action)}" type="button"><span>${escapeHtml(sub)}</span><strong>${escapeHtml(label)}</strong></button>`;
}

function guardrailAction(action, label, isDanger = false) {
  return `<button class="guardrail-action${isDanger ? " danger" : ""}" data-action="${escapeHtml(action)}" type="button">${escapeHtml(label)}</button>`;
}

function proofItem(label, value) {
  return `<div class="proof-item"><span>${escapeHtml(label)}</span><strong>${escapeHtml(value)}</strong></div>`;
}

function navIcon(id) {
  const paths = {
    control: `<path d="M4 7h7"></path><path d="M16 7h4"></path><circle cx="13" cy="7" r="2.4"></circle><path d="M4 12h3"></path><path d="M12 12h8"></path><circle cx="9" cy="12" r="2.4"></circle><path d="M4 17h9"></path><path d="M18 17h2"></path><circle cx="15" cy="17" r="2.4"></circle>`,
    applications: `<path d="M6 3h8l4 4v14H6z"></path><path d="M14 3v4h4"></path><path d="M9 13h6"></path><path d="M9 17h5"></path>`,
    lines: `<rect x="3" y="6" width="18" height="12" rx="2"></rect><path d="M3 10h18"></path><path d="M7 14h4"></path>`,
    analytics: `<path d="M5 19V11"></path><path d="M10 19V5"></path><path d="M15 19v-6"></path><path d="M20 19H4"></path>`,
  };
  return `<svg aria-hidden="true" viewBox="0 0 24 24" focusable="false">${paths[id] ?? ""}</svg>`;
}

function collapseIcon() {
  return `
    <svg aria-hidden="true" viewBox="0 0 24 24" focusable="false">
      <rect x="3" y="3" width="18" height="18" rx="2"></rect>
      <path d="M9 3v18"></path>
    </svg>
  `;
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

function infoIcon() {
  return `
    <svg aria-hidden="true" viewBox="0 0 24 24" focusable="false">
      <circle cx="12" cy="12" r="9"></circle>
      <path d="M12 11v5"></path>
      <path d="M12 8h.01"></path>
    </svg>
  `;
}

function closeIcon() {
  return `
    <svg aria-hidden="true" viewBox="0 0 24 24" focusable="false">
      <path d="M6 6l12 12"></path>
      <path d="M18 6L6 18"></path>
    </svg>
  `;
}

async function copyValue(element) {
  const value = element.dataset.copy ?? "";
  try {
    await navigator.clipboard.writeText(value);
    const original = element.textContent;
    element.textContent = "Copied";
    element.classList.add("copied");
    clearTimeout(copyResetTimer);
    copyResetTimer = setTimeout(() => {
      element.textContent = original;
      element.classList.remove("copied");
    }, 1200);
  } catch {
    txNote = "Clipboard blocked by the browser.";
    render();
  }
}

function poolUtilization() {
  const pool = state.accounts.liquidityPool;
  if (!pool) return 0;
  const committed = parseNumber(pool.committedToLinesUsdc);
  const capital = parseNumber(pool.committedCapitalUsdc);
  if (!capital) return 0;
  return Math.max(0, Math.min(100, Math.round((committed / capital) * 100)));
}

function parseNumber(value) {
  return Number(String(value ?? "0").replaceAll(",", ""));
}

function currentAmount() {
  const value = (amountInput ?? "").trim();
  return /^\d+(\.\d{1,6})?$/.test(value) ? value : "75.00";
}

function statusTone(status) {
  const raw = String(status ?? "").toLowerCase();
  if (["active", "approved", "finalized", "confirmed"].includes(raw)) return "low";
  if (["pending", "suspended", "processing", "processed"].includes(raw)) return "mid";
  if (["defaulted", "rejected", "closed"].includes(raw)) return "high";
  return "mid";
}

function formatDate(value) {
  if (!value) return "-";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "-";
  return date.toLocaleDateString(undefined, { year: "numeric", month: "short", day: "numeric" });
}

function prettyEnum(value) {
  const raw = String(value ?? "");
  if (!raw || raw === "-") return raw || "-";
  const withSpaces = raw.replace(/([a-z])([A-Z])/g, "$1 $2");
  if (withSpaces.toLowerCase() === "zk verified") return "ZK Verified";
  return withSpaces
    .split(" ")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}

function riskGradeLabel(value) {
  const raw = String(value ?? "-");
  return raw === "-" ? raw : raw.toUpperCase();
}

function riskGradeTone(value) {
  if (value.startsWith("A")) return "low";
  if (value.startsWith("B")) return "mid";
  return "high";
}

function riskLabel(value) {
  if (value.startsWith("A")) return "Low Risk";
  if (value.startsWith("B")) return "Moderate";
  if (value === "-") return "Unscored";
  return "Review";
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

function normalizeSection(value) {
  if (value === "market") return "control";
  return NAV.some((item) => item.id === value) ? value : "control";
}

function loadPref(key, fallback) {
  try {
    const value = localStorage.getItem(key);
    return value ?? fallback;
  } catch {
    return fallback;
  }
}

function loadBoolPref(key, fallback) {
  try {
    const value = localStorage.getItem(key);
    return value === null ? fallback : value === "true";
  } catch {
    return fallback;
  }
}

function savePref(key, value) {
  try {
    localStorage.setItem(key, value);
  } catch {
    /* ignore storage failures */
  }
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
