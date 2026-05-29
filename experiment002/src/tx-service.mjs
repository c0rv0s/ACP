import {
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
        product: "agent-credit-market",
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
  const program = programFor(connection, new web3.PublicKey(deployment.publicKeys.admin));
  const addresses = deployment.addresses;
  const account = program.account;
  const [
    market,
    borrower,
    liquidityPool,
    creditApplication,
    creditLine,
    creditAttestation,
    allApplications,
    allAttestations,
    poolBalance,
    feeBalance,
    managerBalance,
    borrowerBalance,
    repaymentPayerBalance,
  ] = await Promise.all([
    fetchNullable(account.marketConfig, addresses.marketConfig),
    fetchNullable(account.borrowerProfile, addresses.borrower),
    fetchNullable(account.liquidityPool, addresses.liquidityPool),
    fetchNullable(account.creditApplication, addresses.creditApplication),
    fetchNullable(account.creditLine, addresses.creditLine),
    fetchNullable(account.creditAttestation, addresses.creditAttestation),
    account.creditApplication.all().catch(() => []),
    account.creditAttestation.all().catch(() => []),
    getTokenAmount(connection, addresses.poolUsdc),
    getTokenAmount(connection, addresses.feeVault),
    getTokenAmount(connection, addresses.tokenAccounts.poolManagerUsdc),
    getTokenAmount(connection, addresses.tokenAccounts.borrowerUsdc),
    getTokenAmount(connection, addresses.tokenAccounts.repaymentPayerUsdc),
  ]);

  const signatures = await connection
    .getSignaturesForAddress(pubkey(addresses.creditLine), { limit: 12 }, "confirmed")
    .catch(() => []);

  return {
    deployment: publicDeployment(deployment),
    accounts: {
      market: market && {
        paused: market.paused,
        admin: pubkeyString(market.admin),
        riskAdmin: pubkeyString(market.riskAdmin),
        platformFeeBps: market.platformFeeBps,
        maxTotalCreditUsdc: formatUsdc(market.maxTotalCreditUsdc),
        maxSingleBorrowerUsdc: formatUsdc(market.maxSingleBorrowerUsdc),
        totalLiquidityUsdc: formatUsdc(market.totalLiquidityUsdc),
        totalCreditLimitUsdc: formatUsdc(market.totalCreditLimitUsdc),
        totalOutstandingUsdc: formatUsdc(market.totalOutstandingUsdc),
        totalPlatformFeesUsdc: formatUsdc(market.totalPlatformFeesUsdc),
      },
      borrower: borrower && {
        status: enumName(borrower.status),
        verificationStatus: enumName(borrower.verificationStatus),
        borrowerType: enumName(borrower.borrowerType),
        operator: pubkeyString(borrower.operator),
        primaryWallet: pubkeyString(borrower.primaryWallet),
        trustScore: borrower.trustScore,
        totalCreditLimitUsdc: formatUsdc(borrower.totalCreditLimitUsdc),
        totalOutstandingUsdc: formatUsdc(borrower.totalOutstandingUsdc),
        repaymentCount: borrower.repaymentCount,
        delinquencyCount: borrower.delinquencyCount,
      },
      liquidityPool: liquidityPool && {
        status: enumName(liquidityPool.status),
        approvalMode: enumName(liquidityPool.approvalMode),
        manager: pubkeyString(liquidityPool.manager),
        committedCapitalUsdc: formatUsdc(liquidityPool.committedCapitalUsdc),
        availableCapitalUsdc: formatUsdc(liquidityPool.availableCapitalUsdc),
        committedToLinesUsdc: formatUsdc(liquidityPool.committedToLinesUsdc),
        principalDrawnUsdc: formatUsdc(liquidityPool.principalDrawnUsdc),
        principalRepaidUsdc: formatUsdc(liquidityPool.principalRepaidUsdc),
        platformFeesPaidUsdc: formatUsdc(liquidityPool.platformFeesPaidUsdc),
        maxSingleLineUsdc: formatUsdc(liquidityPool.maxSingleLineUsdc),
        minTrustScore: liquidityPool.minTrustScore,
        maxAprBps: liquidityPool.maxAprBps,
        autoApproveUnderUsdc: formatUsdc(liquidityPool.autoApproveUnderUsdc),
      },
      creditApplication: creditApplication && {
        status: enumName(creditApplication.status),
        approvalMode: enumName(creditApplication.approvalMode),
        requestedLimitUsdc: formatUsdc(creditApplication.requestedLimitUsdc),
        proposedAprBps: creditApplication.proposedAprBps,
        collateralizationBps: creditApplication.collateralizationBps,
        trustScoreSnapshot: creditApplication.trustScoreSnapshot,
        reviewedAt: timestamp(creditApplication.reviewedAt),
        reviewer: pubkeyString(creditApplication.reviewer),
      },
      creditLine: creditLine && {
        status: enumName(creditLine.status),
        riskGrade: enumName(creditLine.riskGrade),
        principalLimitUsdc: formatUsdc(creditLine.principalLimitUsdc),
        availableLimitUsdc: formatUsdc(creditLine.availableLimitUsdc),
        principalOutstandingUsdc: formatUsdc(creditLine.principalOutstandingUsdc),
        principalRepaidUsdc: formatUsdc(creditLine.principalRepaidUsdc),
        platformFeesPaidUsdc: formatUsdc(creditLine.platformFeesPaidUsdc),
        aprBps: creditLine.aprBps,
        platformFeeBps: creditLine.platformFeeBps,
        collateralizationBps: creditLine.collateralizationBps,
        maturityAt: timestamp(creditLine.maturityAt),
        lastDrawAt: timestamp(creditLine.lastDrawAt),
        lastRepaymentAt: timestamp(creditLine.lastRepaymentAt),
      },
      creditAttestation: creditAttestation && {
        score: creditAttestation.score,
        trustScore: creditAttestation.trustScore,
        riskGrade: enumName(creditAttestation.riskGrade),
        recommendedLimitUsdc: formatUsdc(creditAttestation.recommendedLimitUsdc),
        recommendedAprBps: creditAttestation.recommendedAprBps,
        pdEstimateBps: creditAttestation.pdEstimateBps,
        lgdEstimateBps: creditAttestation.lgdEstimateBps,
        confidenceBps: creditAttestation.confidenceBps,
        createdAt: timestamp(creditAttestation.createdAt),
      },
    },
    tokenBalances: {
      poolUsdc: balance(poolBalance),
      feeVaultUsdc: balance(feeBalance),
      poolManagerUsdc: balance(managerBalance),
      borrowerUsdc: balance(borrowerBalance),
      repaymentPayerUsdc: balance(repaymentPayerBalance),
    },
    applications: allApplications.map(({ publicKey, account: item }) => ({
      address: publicKey.toBase58(),
      status: enumName(item.status),
      approvalMode: enumName(item.approvalMode),
      requestedLimitUsdc: formatUsdc(item.requestedLimitUsdc),
      proposedAprBps: item.proposedAprBps,
      trustScoreSnapshot: item.trustScoreSnapshot,
      createdAt: timestamp(item.createdAt),
    })),
    attestations: allAttestations.map(({ publicKey, account: item }) => ({
      address: publicKey.toBase58(),
      score: item.score,
      trustScore: item.trustScore,
      riskGrade: enumName(item.riskGrade),
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

  if (kind === "fund-pool") {
    instructions.push(
      await program.methods
        .fundLiquidityPool(bnUsdc(body.amountUsdc ?? "100.00"))
        .accounts({
          marketConfig: addresses.marketConfig,
          liquidityPool: addresses.liquidityPool,
          managerUsdc: addresses.tokenAccounts.poolManagerUsdc,
          poolUsdcVault: addresses.poolUsdc,
          manager: wallet,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .instruction(),
    );
  } else if (kind === "draw") {
    instructions.push(
      await program.methods
        .drawCredit(bnUsdc(body.amountUsdc ?? "75.00"))
        .accounts({
          marketConfig: addresses.marketConfig,
          borrower: addresses.borrower,
          liquidityPool: addresses.liquidityPool,
          creditLine: addresses.creditLine,
          vaultAuthority: addresses.poolAuthority,
          poolUsdcVault: addresses.poolUsdc,
          borrowerUsdc: addresses.tokenAccounts.borrowerUsdc,
          operator: wallet,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .instruction(),
    );
  } else if (kind === "repay") {
    instructions.push(
      await program.methods
        .repayCredit(bnUsdc(body.amountUsdc ?? "25.00"))
        .accounts({
          marketConfig: addresses.marketConfig,
          borrower: addresses.borrower,
          liquidityPool: addresses.liquidityPool,
          creditLine: addresses.creditLine,
          payerUsdc: addresses.tokenAccounts.repaymentPayerUsdc,
          poolUsdcVault: addresses.poolUsdc,
          feeVault: addresses.feeVault,
          payer: wallet,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .instruction(),
    );
  } else if (kind === "score") {
    const scoreHash = randomByteArray(32);
    const creditAttestation = findPda(
      [Buffer.from("attestation"), pubkey(addresses.borrower).toBuffer(), toBuffer(scoreHash)],
      program.programId,
    );
    const trustScore = Number(body.trustScore ?? body.score ?? 872);
    const riskGrade = body.riskGrade ?? (trustScore >= 820 ? "A" : trustScore >= 680 ? "B" : trustScore >= 540 ? "C" : "D");
    instructions.push(
      await program.methods
        .updateCreditAttestation({
          scoreHash,
          score: Number(body.score ?? trustScore),
          trustScore,
          riskGrade: riskGradeEnum(riskGrade),
          recommendedLimitUsdc: bnUsdc(body.recommendedLimitUsdc ?? "650.00"),
          recommendedAprBps: Number(body.recommendedAprBps ?? 825),
          pdEstimateBps: 125,
          lgdEstimateBps: 2100,
          confidenceBps: 9400,
          featuresHash: bytes(32, 44),
        })
        .accounts({
          marketConfig: addresses.marketConfig,
          borrower: addresses.borrower,
          creditLine: addresses.creditLine,
          creditAttestation,
          authority: wallet,
          systemProgram: SystemProgram.programId,
        })
        .instruction(),
    );
    meta.creditAttestation = creditAttestation.toBase58();
  } else if (["suspend", "resume", "default", "close"].includes(kind)) {
    const accounts = {
      marketConfig: addresses.marketConfig,
      borrower: addresses.borrower,
      liquidityPool: addresses.liquidityPool,
      creditLine: addresses.creditLine,
      authority: wallet,
    };
    if (kind === "suspend") {
      instructions.push(await program.methods.suspendCreditLine().accounts(accounts).instruction());
    } else if (kind === "resume") {
      instructions.push(await program.methods.resumeCreditLine().accounts(accounts).instruction());
    } else if (kind === "default") {
      instructions.push(await program.methods.markDefault().accounts(accounts).instruction());
    } else {
      instructions.push(await program.methods.closeCreditLine().accounts(accounts).instruction());
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
