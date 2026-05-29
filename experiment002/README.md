# Agent Credit Market

Experiment 002 is now the local, wallet-signed Solana/Anchor app for the Agent Credit Market.

It models the first production shape of the product:

- LPs create and fund USDC liquidity pools.
- Borrowers register verified credit profiles.
- Borrowers submit credit applications against LP pool policy.
- Risk admins or pool managers approve and deploy programmable credit lines.
- Borrowers draw stablecoin credit.
- Repayments route principal back to the LP pool and a 0.1% platform fee to the protocol fee vault.
- Credit attestations update borrower trust score, risk grade, and recommended terms.

The browser does not mutate fake state and the API does not custody private keys. The local server reads deployed Anchor accounts, builds unsigned transactions, and the connected wallet signs and sends them to the local validator.

## Local Onchain Sandbox

```bash
cd experiment002
npm run setup:local
npm run dev
```

Open `http://127.0.0.1:8082`.

`npm run setup:local` starts a local `solana-test-validator`, builds and deploys the Anchor program, mints local USDC, initializes market accounts, creates a funded LP pool, registers a verified borrower, submits and approves a credit application, deploys a credit line, and writes deployment metadata to `.local/deployment.json`.

## Wallet Setup

The bootstrap uses one local dev wallet for market admin, risk admin, LP pool manager, borrower operator, and repayment payer roles so Phantom or another Solana wallet can sign every dashboard action.

```bash
cd experiment002
npm run dev-wallet
```

Import the printed local-only secret key into Phantom, switch the wallet RPC to `http://127.0.0.1:8899`, then connect it in the dashboard.

## Runtime Commands

```bash
npm run setup:local       # reset local validator, deploy program, bootstrap market accounts
npm run setup:local -- --keep-ledger
npm run dev               # serve API + dashboard at http://127.0.0.1:8082
npm run localnet:down     # stop validator started by setup/localnet scripts
npm run dev-wallet        # print local wallet import material
npm run smoke:local       # sign/send draw, repay, and attestation transactions
```

Useful endpoints:

```bash
curl -sS http://127.0.0.1:8082/health
curl -sS http://127.0.0.1:8082/v1/state
curl -sS -X POST http://127.0.0.1:8082/v1/transactions/draw \
  -H 'content-type: application/json' \
  --data '{"wallet":"<connected-wallet>","amountUsdc":"75.00"}'
curl -sS -X POST http://127.0.0.1:8082/v1/transactions/repay \
  -H 'content-type: application/json' \
  --data '{"wallet":"<connected-wallet>","amountUsdc":"25.00"}'
```

The transaction endpoint returns a base64 serialized unsigned Solana transaction. The browser deserializes it, asks the connected wallet to sign/send it, confirms it on the local validator, and refreshes the onchain account view.

## Verification

```bash
cd experiment002
npm test
npm run test:solana
```

`npm test` covers shared chain utilities. `npm run test:solana` runs the program unit tests. For an end-to-end local smoke test, run `npm run setup:local`, `npm run dev`, then `npm run smoke:local`.
