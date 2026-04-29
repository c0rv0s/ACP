# Upgrade And Migration Runbook

AGC is deployed as an upgradeable Solana program so the protocol can evolve. The upgrade path is treated as a production control surface, not an informal developer convenience.

## Authority Model

The upgrade authority belongs to a production multisig at launch. Admin, risk, emergency, and keeper roles remain separate inside protocol state. The multisig can upgrade the program, but runtime authorities still limit what any operational key can do.

Governance setup is finalized during deployment prep. Until then, local and devnet deployments can use a temporary admin key, but production does not rely on a single hot wallet.

## Pre-Upgrade Checklist

- Build the program from a clean commit.
- Run unit tests, local-validator tests, and web build.
- Diff the generated IDL against the deployed IDL.
- Confirm account-size changes and migration requirements.
- Confirm every new instruction has authority checks and pause behavior.
- Simulate the upgrade on a fresh local validator and on devnet.
- Prepare a public upgrade note explaining the behavior change.

## Account Migration Rules

Account migrations use explicit versioning. New account fields are added with reserved space where possible. If an account cannot be safely extended, the program adds a new account version and a migration instruction that copies state under admin or risk authority.

Migration instructions are single-purpose. They validate the old account, initialize the new account, copy only intended fields, and emit an event that external indexers can verify.

## Upgrade Execution

1. Build the verified artifact with `anchor build`.
2. Publish the IDL and binary hash in the internal release note.
3. Submit the upgrade transaction through the upgrade multisig.
4. Execute the upgrade during a low-activity window.
5. Immediately verify the deployed program data, IDL, and critical PDAs.
6. Run smoke transactions: read state, refresh oracle, settle a no-op epoch, and execute a harmless paused-surface check.

## Rollback And Pause

Solana upgrades cannot be reversed by wishful thinking; rollback is another upgrade. The emergency response is therefore:

- Pause risky surfaces first.
- Stop keepers from executing expansion or buyback campaigns if needed.
- Prepare a rollback binary from the previous audited commit.
- Upgrade back through the multisig.
- Re-run smoke tests and publish the incident note.

The core safety principle is simple: when something is unclear, the protocol stops taking new risk before it attempts to resume growth.
