# AGC Protocol TODOs

This file tracks the remaining work to move the Solana protocol from current implementation to production launch.

## Solana Program

- [ ] Add direct Pyth or Switchboard adapter validation for collateral oracle cache updates.
- [ ] Aggregate reserve value from actual configured reserve token accounts instead of passing reserve metrics manually into settlement.
- [ ] Add a venue-specific atomic buyback executor that swaps reserved USDC and burns AGC in one controlled flow.
- [ ] Add integration tests for credit draw, repayment, default, underwriter loss, and collateral seizure across real SPL token accounts.
- [ ] Add migration playbooks for future account-version upgrades.
- [ ] Decide final launch multisig structure for admin, risk, emergency, and upgrade authorities.
- [ ] Define production parameter presets for USDC, USDT, the first BTC wrapper, and disabled RWA assets.

## Frontend

- [ ] Wire the dashboard to the deployed Solana IDL/client.
- [ ] Read live `ProtocolState`, collateral registry, credit facility, xAGC vault, and treasury token account data.
- [ ] Submit live xAGC deposit/redeem transactions.
- [ ] Submit live credit facility transactions: underwrite, deposit collateral, draw, repay, and withdraw where allowed.
- [ ] Add facility detail pages for collateral health, maturity, debt, reserve coverage, underwriter reserve, and default state.
- [ ] Keep Jupiter swap panel enabled once the AGC mint is deployed and routed.

## Docs

- [ ] Add a deployment guide with exact Solana accounts, PDA seeds, authority setup, and environment variables.
- [ ] Add a risk-parameter reference for stablecoins, BTC wrappers, and isolated RWAs.
- [ ] Add user-facing examples for xAGC, credit borrowers, underwriters, and liquidations.
- [ ] Add an upgrade/governance transparency page explaining who controls each authority and what each authority can do.
- [ ] Keep `/llms.txt` and `/llms-full.txt` in sync with product docs after every protocol change.

## Launch Readiness

- [ ] Complete an external Solana security review.
- [ ] Run adversarial tests for stale oracle data, wrong mint accounts, underwriter reserve drain, overdraw attempts, and collateral seizure routing.
- [ ] Publish deployed program ID, IDL, mint addresses, treasury addresses, and multisig addresses.
- [ ] Run a staged devnet launch before mainnet.
- [ ] Define operational monitoring for policy settlement, oracle freshness, reserve balances, credit health, and emergency pause events.
