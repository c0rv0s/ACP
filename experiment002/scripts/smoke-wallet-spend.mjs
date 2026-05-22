import { keyPaths, loadDeployment, loadKeypair, web3 } from "../src/chain.mjs";

const deployment = loadDeployment();
const wallet = loadKeypair(keyPaths().admin);
const baseUrl = process.env.AGC_DASHBOARD_URL ?? "http://127.0.0.1:8082";
const amountUsdc = process.env.AGC_SMOKE_AMOUNT_USDC ?? "2.50";

const spend = await submitWalletTransaction("spend", {
  wallet: wallet.publicKey.toBase58(),
  amountUsdc,
  merchant: "search",
});
console.log(`Submitted wallet-signed spend: ${spend.signature}`);

const revenue = await submitWalletTransaction("revenue", {
  wallet: wallet.publicKey.toBase58(),
  amountUsdc: "50.00",
  source: "manual",
});
console.log(`Submitted wallet-signed revenue sweep: ${revenue.signature}`);

const state = await getJson(`${baseUrl}/v1/state`);
console.log(`Credit line available: ${state.accounts.creditLine.availableLimitUsdc}`);
console.log(`Credit line outstanding: ${state.accounts.creditLine.principalOutstandingUsdc}`);
console.log(`Vault USDC balance: ${state.tokenBalances.vaultUsdc.uiAmount}`);
console.log(`Merchant USDC balance: ${state.tokenBalances.merchantUsdc.uiAmount}`);
console.log(`Borrower receivable USDC balance: ${state.tokenBalances.borrowerReceivableUsdc.uiAmount}`);

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
