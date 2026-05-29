import { keyPaths, loadDeployment, loadKeypair, web3 } from "../src/chain.mjs";

const deployment = loadDeployment();
const wallet = loadKeypair(keyPaths().admin);
const baseUrl = process.env.AGC_DASHBOARD_URL ?? "http://127.0.0.1:8082";

const draw = await submitWalletTransaction("draw", {
  wallet: wallet.publicKey.toBase58(),
  amountUsdc: process.env.AGC_SMOKE_DRAW_USDC ?? "75.00",
});
console.log(`Submitted wallet-signed credit draw: ${draw.signature}`);

const repay = await submitWalletTransaction("repay", {
  wallet: wallet.publicKey.toBase58(),
  amountUsdc: process.env.AGC_SMOKE_REPAY_USDC ?? "25.00",
});
console.log(`Submitted wallet-signed repayment: ${repay.signature}`);

const attestation = await submitWalletTransaction("score", {
  wallet: wallet.publicKey.toBase58(),
  trustScore: 872,
  riskGrade: "A",
});
console.log(`Submitted wallet-signed score attestation: ${attestation.signature}`);

const state = await getJson(`${baseUrl}/v1/state`);
console.log(`Credit line available: ${state.accounts.creditLine.availableLimitUsdc}`);
console.log(`Credit line outstanding: ${state.accounts.creditLine.principalOutstandingUsdc}`);
console.log(`Pool USDC balance: ${state.tokenBalances.poolUsdc.uiAmount}`);
console.log(`Borrower USDC balance: ${state.tokenBalances.borrowerUsdc.uiAmount}`);
console.log(`Fee vault USDC balance: ${state.tokenBalances.feeVaultUsdc.uiAmount}`);

async function submitWalletTransaction(kind, body) {
  const built = await postJson(`${baseUrl}/v1/transactions/${kind}`, body);
  const transaction = web3.Transaction.from(Buffer.from(built.transaction, "base64"));
  transaction.partialSign(wallet);

  const connection = new web3.Connection(deployment.rpcUrl, "confirmed");
  const signature = await connection.sendRawTransaction(transaction.serialize());
  await connection.confirmTransaction(
    {
      signature,
      blockhash: built.blockhash,
      lastValidBlockHeight: built.lastValidBlockHeight,
    },
    "confirmed",
  );

  return { signature };
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

async function getJson(url) {
  const response = await fetch(url);
  const payload = await response.json();
  if (!response.ok) throw new Error(payload.message ?? "Request failed");
  return payload;
}
