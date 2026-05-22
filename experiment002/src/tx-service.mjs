import {
  PublicKey,
  SystemProgram,
  TOKEN_PROGRAM_ID,
  anchor,
  bn,
  bnUsdc,
  bytes,
  createProgram,
  enumName,
  findPda,
  formatUsdc,
  getTokenAmount,
  loadDeployment,
  pubkey,
  pubkeyString,
  randomByteArray,
  riskGradeEnum,
  sourceEnum,
  toBuffer,
  web3,
} from "./chain.mjs";

export function createTxService() {
  const deployment = loadDeployment();
  const connection = new web3.Connection(deployment.rpcUrl, "confirmed");

  return {
    deployment,
    connection,
    async health() {
      const version = await connection.getVersion();
      return {
        ok: true,
        service: "agc-experiment-002",
        signing: "wallet",
        rpcUrl: deployment.rpcUrl,
        programId: deployment.programId,
        solanaVersion: version["solana-core"],
      };
    },
    async state() {
      return readState(deployment, connection);
    },
    async buildTransaction(kind, body) {
      return buildTransaction(deployment, connection, kind, body);
    },
    async airdrop(body) {
      const wallet = pubkey(required(body.wallet, "wallet"));
      const sol = Math.min(Number(body.sol ?? 2), 10);
      const signature = await connection.requestAirdrop(wallet, sol * web3.LAMPORTS_PER_SOL);
      await connection.confirmTransaction(signature, "confirmed");
      return { signature, wallet: wallet.toBase58(), sol };
    },
  };
}

async function readState(deployment, connection) {
  const program = programFor(connection, new PublicKey(deployment.publicKeys.admin));
  const addresses = deployment.addresses;
  const account = program.account;
  const [
    protocol,
    borrower,
    agent,
    workflow,
    policy,
    vault,
    line,
    searchMerchant,
    blockedMerchant,
    spendAuthorizations,
    scores,
    vaultBalance,
    underwriterBalance,
    revenueBalance,
    merchantBalance,
    borrowerReceivableBalance,
  ] = await Promise.all([
    fetchNullable(account.protocolConfig, addresses.protocolConfig),
    fetchNullable(account.borrowerProfile, addresses.borrower),
    fetchNullable(account.agentProfile, addresses.agent),
    fetchNullable(account.workflowProfile, addresses.workflow),
    fetchNullable(account.spendPolicy, addresses.policy),
    fetchNullable(account.underwriterVault, addresses.underwriterVault),
    fetchNullable(account.creditLine, addresses.creditLine),
    fetchNullable(account.merchant, addresses.merchants.search),
    fetchNullable(account.merchant, addresses.merchants.blocked),
    account.spendAuthorization.all().catch(() => []),
    account.scoreAttestation.all().catch(() => []),
    getTokenAmount(connection, addresses.vaultUsdc),
    getTokenAmount(connection, addresses.tokenAccounts.underwriterUsdc),
    getTokenAmount(connection, addresses.tokenAccounts.revenuePayerUsdc),
    getTokenAmount(connection, addresses.tokenAccounts.merchantUsdc),
    getTokenAmount(connection, addresses.tokenAccounts.borrowerReceivableUsdc),
  ]);

  const signatures = await connection
    .getSignaturesForAddress(pubkey(addresses.creditLine), { limit: 12 }, "confirmed")
    .catch(() => []);

  return {
    deployment: publicDeployment(deployment),
    accounts: {
      protocol: protocol && {
        paused: protocol.paused,
        admin: pubkeyString(protocol.admin),
        paymentRouter: pubkeyString(protocol.paymentRouter),
        totalLiveCapitalAtRiskUsdc: formatUsdc(protocol.totalLiveCapitalAtRiskUsdc),
        totalDailySpendUsdc: formatUsdc(protocol.totalDailySpendUsdc),
        totalDefaultedUsdc: formatUsdc(protocol.totalDefaultedUsdc),
        maxLossBudgetUsdc: formatUsdc(protocol.maxLossBudgetUsdc),
      },
      borrower: borrower && {
        status: enumName(borrower.status),
        verificationStatus: enumName(borrower.verificationStatus),
        operator: pubkeyString(borrower.operator),
        totalPrincipalLimitUsdc: formatUsdc(borrower.totalPrincipalLimitUsdc),
        totalOutstandingUsdc: formatUsdc(borrower.totalOutstandingUsdc),
      },
      agent: agent && {
        status: enumName(agent.status),
        wallet: pubkeyString(agent.wallet),
        lastActiveAt: timestamp(agent.lastActiveAt),
      },
      workflow: workflow && {
        status: enumName(workflow.status),
        policy: pubkeyString(workflow.policy),
      },
      policy: policy && {
        version: policy.version,
        status: enumName(policy.status),
        maxPerTransactionUsdc: formatUsdc(policy.maxPerTransactionUsdc),
        maxDailySpendUsdc: formatUsdc(policy.maxDailySpendUsdc),
        maxWeeklySpendUsdc: formatUsdc(policy.maxWeeklySpendUsdc),
        humanApprovalThresholdUsdc: formatUsdc(policy.humanApprovalThresholdUsdc),
        revenueSweepBps: policy.revenueSweepBps,
        dailySpendUsdc: formatUsdc(policy.dailySpendUsdc),
        weeklySpendUsdc: formatUsdc(policy.weeklySpendUsdc),
      },
      underwriterVault: vault && {
        status: enumName(vault.status),
        committedCapitalUsdc: formatUsdc(vault.committedCapitalUsdc),
        availableCapitalUsdc: formatUsdc(vault.availableCapitalUsdc),
        committedToLinesUsdc: formatUsdc(vault.committedToLinesUsdc),
        principalDrawnUsdc: formatUsdc(vault.principalDrawnUsdc),
        principalRepaidUsdc: formatUsdc(vault.principalRepaidUsdc),
        interestEarnedUsdc: formatUsdc(vault.interestEarnedUsdc),
        lossRealizedUsdc: formatUsdc(vault.lossRealizedUsdc),
      },
      creditLine: line && {
        status: enumName(line.status),
        riskGrade: riskGradeName(line.riskGrade),
        principalLimitUsdc: formatUsdc(line.principalLimitUsdc),
        availableLimitUsdc: formatUsdc(line.availableLimitUsdc),
        reservedSpendUsdc: formatUsdc(line.reservedSpendUsdc),
        principalOutstandingUsdc: formatUsdc(line.principalOutstandingUsdc),
        accruedInterestUsdc: formatUsdc(line.accruedInterestUsdc),
        feesDueUsdc: formatUsdc(line.feesDueUsdc),
        aprBps: line.aprBps,
        maturityAt: timestamp(line.maturityAt),
      },
      merchants: {
        search: searchMerchant && { address: addresses.merchants.search, status: enumName(searchMerchant.status), category: searchMerchant.category },
        blocked: blockedMerchant && { address: addresses.merchants.blocked, status: enumName(blockedMerchant.status), category: blockedMerchant.category },
      },
    },
    tokenBalances: {
      vaultUsdc: balance(vaultBalance),
      underwriterUsdc: balance(underwriterBalance),
      revenuePayerUsdc: balance(revenueBalance),
      merchantUsdc: balance(merchantBalance),
      borrowerReceivableUsdc: balance(borrowerReceivableBalance),
    },
    spendAuthorizations: spendAuthorizations.map(({ publicKey, account: item }) => ({
      address: publicKey.toBase58(),
      amountUsdc: formatUsdc(item.amountUsdc),
      merchant: pubkeyString(item.merchant),
      status: enumName(item.status),
      reasonCode: enumName(item.reasonCode),
      createdAt: timestamp(item.createdAt),
    })),
    scoreAttestations: scores.map(({ publicKey, account: item }) => ({
      address: publicKey.toBase58(),
      score: item.score,
      riskGrade: riskGradeName(item.riskGrade),
      recommendedLimitUsdc: formatUsdc(item.recommendedLimitUsdc),
      recommendedAprBps: item.recommendedAprBps,
      confidenceBps: item.confidenceBps,
      createdAt: timestamp(item.createdAt),
    })),
    signatures: signatures.map((item) => ({
      signature: item.signature,
      slot: item.slot,
      blockTime: item.blockTime,
      confirmationStatus: item.confirmationStatus,
    })),
  };
}

async function buildTransaction(deployment, connection, kind, body) {
  const wallet = pubkey(required(body.wallet, "wallet"));
  const program = programFor(connection, wallet);
  const addresses = deployment.addresses;
  const instructions = [];
  const meta = { kind };

  if (kind === "spend") {
    const spendId = randomByteArray(16);
    const merchantKey = body.merchant === "blocked" ? addresses.merchants.blocked : addresses.merchants.search;
    const spendAuthorization = findPda(
      [Buffer.from("spend-auth"), pubkey(addresses.creditLine).toBuffer(), toBuffer(spendId)],
      program.programId,
    );
    instructions.push(
      await program.methods
        .reserveSpend({
          spendId,
          amountUsdc: bnUsdc(body.amountUsdc ?? "2.50"),
          purposeHash: bytes(32, 31),
          authorizationTtlSeconds: bn(300),
        })
        .accounts({
          protocolConfig: addresses.protocolConfig,
          borrower: addresses.borrower,
          creditLine: addresses.creditLine,
          policy: addresses.policy,
          merchant: merchantKey,
          spendAuthorization,
          router: wallet,
          systemProgram: SystemProgram.programId,
        })
        .instruction(),
    );
    instructions.push(
      await program.methods
        .settleSpend()
        .accounts({
          protocolConfig: addresses.protocolConfig,
          borrower: addresses.borrower,
          creditLine: addresses.creditLine,
          spendAuthorization,
          underwriterVault: addresses.underwriterVault,
          vaultAuthority: addresses.vaultAuthority,
          underwriterUsdcVault: addresses.vaultUsdc,
          merchantUsdc: addresses.tokenAccounts.merchantUsdc,
          router: wallet,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .instruction(),
    );
    meta.spendAuthorization = spendAuthorization.toBase58();
  } else if (kind === "revenue") {
    instructions.push(
      await program.methods
        .recordRevenueAndSweep(bnUsdc(body.amountUsdc ?? "50.00"), sourceEnum(body.source ?? "manual"), randomByteArray(32))
        .accounts({
          protocolConfig: addresses.protocolConfig,
          borrower: addresses.borrower,
          creditLine: addresses.creditLine,
          policy: addresses.policy,
          underwriterVault: addresses.underwriterVault,
          revenueSourceUsdc: addresses.tokenAccounts.revenuePayerUsdc,
          underwriterUsdcVault: addresses.vaultUsdc,
          borrowerReceivableUsdc: addresses.tokenAccounts.borrowerReceivableUsdc,
          revenuePayer: wallet,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .instruction(),
    );
  } else if (kind === "manual-repay") {
    instructions.push(
      await program.methods
        .manualRepay(bnUsdc(body.amountUsdc ?? "10.00"))
        .accounts({
          protocolConfig: addresses.protocolConfig,
          borrower: addresses.borrower,
          creditLine: addresses.creditLine,
          underwriterVault: addresses.underwriterVault,
          payerUsdc: addresses.tokenAccounts.revenuePayerUsdc,
          underwriterUsdcVault: addresses.vaultUsdc,
          payer: wallet,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .instruction(),
    );
  } else if (kind === "fund-vault") {
    instructions.push(
      await program.methods
        .fundUnderwriterVault(bnUsdc(body.amountUsdc ?? "100.00"))
        .accounts({
          protocolConfig: addresses.protocolConfig,
          underwriterVault: addresses.underwriterVault,
          underwriterUsdc: addresses.tokenAccounts.underwriterUsdc,
          vaultUsdc: addresses.vaultUsdc,
          underwriter: wallet,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .instruction(),
    );
  } else if (kind === "score") {
    const scoreVersionHash = randomByteArray(32);
    const scoreAttestation = findPda(
      [Buffer.from("score"), pubkey(addresses.creditLine).toBuffer(), toBuffer(scoreVersionHash)],
      program.programId,
    );
    const score = Number(body.score ?? 765);
    const riskGrade = body.riskGrade ?? (score >= 850 ? "A" : score >= 700 ? "B" : score >= 550 ? "C" : "D");
    instructions.push(
      await program.methods
        .updateScoreAttestation({
          scoreVersionHash,
          score,
          riskGrade: riskGradeEnum(riskGrade),
          recommendedLimitUsdc: bnUsdc(body.recommendedLimitUsdc ?? "100.00"),
          recommendedAprBps: Number(body.recommendedAprBps ?? 1800),
          pdEstimateBps: 900,
          lgdEstimateBps: 4200,
          confidenceBps: 6200,
          featuresHash: randomByteArray(32),
        })
        .accounts({
          protocolConfig: addresses.protocolConfig,
          creditLine: addresses.creditLine,
          scoreAttestation,
          authority: wallet,
          systemProgram: SystemProgram.programId,
        })
        .instruction(),
    );
    meta.scoreAttestation = scoreAttestation.toBase58();
  } else if (["suspend", "resume", "close", "default"].includes(kind)) {
    if (kind === "suspend") {
      instructions.push(await program.methods.suspendCreditLine().accounts({ protocolConfig: addresses.protocolConfig, creditLine: addresses.creditLine, authority: wallet }).instruction());
    } else if (kind === "resume") {
      instructions.push(await program.methods.resumeCreditLine().accounts({ protocolConfig: addresses.protocolConfig, creditLine: addresses.creditLine, authority: wallet }).instruction());
    } else if (kind === "close") {
      instructions.push(
        await program.methods
          .closeRepaidCreditLine()
          .accounts({
            protocolConfig: addresses.protocolConfig,
            borrower: addresses.borrower,
            creditLine: addresses.creditLine,
            underwriterVault: addresses.underwriterVault,
            authority: wallet,
          })
          .instruction(),
      );
    } else {
      instructions.push(
        await program.methods
          .markDefault()
          .accounts({
            protocolConfig: addresses.protocolConfig,
            creditLine: addresses.creditLine,
            underwriterVault: addresses.underwriterVault,
            authority: wallet,
          })
          .instruction(),
      );
    }
  } else {
    throw new Error(`Unknown transaction kind: ${kind}`);
  }

  const transaction = new web3.Transaction();
  for (const instruction of instructions) transaction.add(instruction);
  transaction.feePayer = wallet;
  const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash("confirmed");
  transaction.recentBlockhash = blockhash;

  return {
    transaction: transaction.serialize({ requireAllSignatures: false, verifySignatures: false }).toString("base64"),
    blockhash,
    lastValidBlockHeight,
    feePayer: wallet.toBase58(),
    meta,
  };
}

function programFor(connection, walletPublicKey) {
  const wallet = {
    publicKey: walletPublicKey,
    signTransaction: async (transaction) => transaction,
    signAllTransactions: async (transactions) => transactions,
  };
  const provider = new anchor.AnchorProvider(connection, wallet, {
    commitment: "confirmed",
    preflightCommitment: "confirmed",
  });
  return createProgram(provider);
}

function publicDeployment(deployment) {
  return {
    generatedAt: deployment.generatedAt,
    rpcUrl: deployment.rpcUrl,
    programId: deployment.programId,
    publicKeys: deployment.publicKeys,
    addresses: deployment.addresses,
  };
}

async function fetchNullable(namespace, address) {
  if (!namespace || !address) return null;
  return namespace.fetchNullable(pubkey(address));
}

function required(value, field) {
  if (!value) throw new Error(`${field} is required`);
  return value;
}

function balance(raw) {
  return { raw: raw.toString(), uiAmount: formatUsdc(raw) };
}

function timestamp(value) {
  const raw = value?.toString?.() ?? String(value ?? "0");
  if (raw === "0") return null;
  return new Date(Number(raw) * 1000).toISOString();
}

function riskGradeName(value) {
  return enumName(value).toUpperCase();
}
