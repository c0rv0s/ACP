import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  TOKEN_PROGRAM_ID,
  createInitializeAccountInstruction,
  createMint,
  getAccount,
  getMint,
} from "@solana/spl-token";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import { strict as assert } from "assert";
import type { AgcSolana } from "../target/types/agc_solana";

const PRICE_SCALE = new anchor.BN("1000000000000000000");

describe("agc_solana local validator", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const provider = anchor.getProvider() as anchor.AnchorProvider;
  const program = anchor.workspace.AgcSolana as Program<AgcSolana>;

  it("rejects externally freezable AGC mints", async () => {
    const admin = Keypair.generate();
    await airdrop(admin.publicKey);

    const [state] = pda("state");
    const [mintAuthority] = pda("mint-authority");
    const [treasuryAuthority] = pda("treasury-authority");
    const [xagcAuthority] = pda("xagc-authority");
    const [treasuryAgc] = pda("treasury-agc");
    const [treasuryUsdc] = pda("treasury-usdc");
    const [xagcVaultAgc] = pda("xagc-vault-agc");

    const agcMint = await createMint(
      provider.connection,
      admin,
      mintAuthority,
      admin.publicKey,
      9,
    );
    const xagcMint = await createMint(
      provider.connection,
      admin,
      mintAuthority,
      null,
      9,
    );
    const usdcMint = await createMint(
      provider.connection,
      admin,
      admin.publicKey,
      null,
      6,
    );

    const growth = Keypair.generate();
    const lp = Keypair.generate();
    const integrators = Keypair.generate();
    await Promise.all([
      createTokenAccount(growth, agcMint, admin.publicKey, admin),
      createTokenAccount(lp, agcMint, admin.publicKey, admin),
      createTokenAccount(integrators, agcMint, admin.publicKey, admin),
    ]);

    await assert.rejects(
      program.methods
        .initializeProtocol({
        initialAnchorPriceX18: PRICE_SCALE,
        policyParams: policyParams(),
        mintDistribution: {
          xagcBps: 3000,
          growthProgramsBps: 2000,
          lpBps: 2000,
          integratorsBps: 1000,
          treasuryBps: 2000,
        },
        settlementRecipients: {
          growthProgramsAgc: growth.publicKey,
          lpAgc: lp.publicKey,
          integratorsAgc: integrators.publicKey,
        },
        exitFeeBps: 100,
        growthProgramsEnabled: true,
      })
        .accountsStrict({
          payer: provider.wallet.publicKey,
          admin: admin.publicKey,
          state,
          agcMint,
          xagcMint,
          usdcMint,
          treasuryAgc,
          treasuryUsdc,
          xagcVaultAgc,
          mintAuthority,
          treasuryAuthority,
          xagcAuthority,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([admin])
        .rpc(),
    );
  });

  it("initializes PDA-owned mints and vaults", async () => {
    const admin = Keypair.generate();
    await airdrop(admin.publicKey);

    const [state] = pda("state");
    const [mintAuthority] = pda("mint-authority");
    const [treasuryAuthority] = pda("treasury-authority");
    const [xagcAuthority] = pda("xagc-authority");
    const [treasuryAgc] = pda("treasury-agc");
    const [treasuryUsdc] = pda("treasury-usdc");
    const [xagcVaultAgc] = pda("xagc-vault-agc");

    const agcMint = await createMint(
      provider.connection,
      admin,
      mintAuthority,
      null,
      9,
    );
    const xagcMint = await createMint(
      provider.connection,
      admin,
      mintAuthority,
      null,
      9,
    );
    const usdcMint = await createMint(
      provider.connection,
      admin,
      admin.publicKey,
      null,
      6,
    );
    const growth = Keypair.generate();
    const lp = Keypair.generate();
    const integrators = Keypair.generate();
    await Promise.all([
      createTokenAccount(growth, agcMint, admin.publicKey, admin),
      createTokenAccount(lp, agcMint, admin.publicKey, admin),
      createTokenAccount(integrators, agcMint, admin.publicKey, admin),
    ]);

    await program.methods
      .initializeProtocol({
          initialAnchorPriceX18: PRICE_SCALE,
          policyParams: policyParams(),
          mintDistribution: {
            xagcBps: 3000,
            growthProgramsBps: 2000,
            lpBps: 2000,
            integratorsBps: 1000,
            treasuryBps: 2000,
          },
          settlementRecipients: {
            growthProgramsAgc: growth.publicKey,
            lpAgc: lp.publicKey,
            integratorsAgc: integrators.publicKey,
          },
          exitFeeBps: 100,
          growthProgramsEnabled: true,
      })
      .accountsStrict({
        payer: provider.wallet.publicKey,
        admin: admin.publicKey,
        state,
        agcMint,
        xagcMint,
        usdcMint,
        treasuryAgc,
        treasuryUsdc,
        xagcVaultAgc,
        mintAuthority,
        treasuryAuthority,
        xagcAuthority,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([admin])
      .rpc();

    const stateAccount = await program.account.protocolState.fetch(state);
    assert.equal(stateAccount.admin.toBase58(), admin.publicKey.toBase58());
    assert.equal(stateAccount.agcMint.toBase58(), agcMint.toBase58());
    assert.equal(stateAccount.xagcMint.toBase58(), xagcMint.toBase58());
    assert.equal(stateAccount.usdcMint.toBase58(), usdcMint.toBase58());

    const treasuryAgcAccount = await getAccount(provider.connection, treasuryAgc);
    const treasuryUsdcAccount = await getAccount(provider.connection, treasuryUsdc);
    const xagcVaultAccount = await getAccount(provider.connection, xagcVaultAgc);
    assert.equal(treasuryAgcAccount.owner.toBase58(), treasuryAuthority.toBase58());
    assert.equal(treasuryUsdcAccount.owner.toBase58(), treasuryAuthority.toBase58());
    assert.equal(xagcVaultAccount.owner.toBase58(), xagcAuthority.toBase58());

    const initializedAgcMint = await getMint(provider.connection, agcMint);
    assert.equal(initializedAgcMint.freezeAuthority, null);
  });

  async function airdrop(address: PublicKey) {
    const signature = await provider.connection.requestAirdrop(
      address,
      5 * anchor.web3.LAMPORTS_PER_SOL,
    );
    const blockhash = await provider.connection.getLatestBlockhash();
    await provider.connection.confirmTransaction({ signature, ...blockhash });
  }

  async function createTokenAccount(
    account: Keypair,
    mint: PublicKey,
    owner: PublicKey,
    payer: Keypair,
  ) {
    const space = 165;
    const lamports = await provider.connection.getMinimumBalanceForRentExemption(
      space,
    );
    const tx = new anchor.web3.Transaction().add(
      SystemProgram.createAccount({
        fromPubkey: payer.publicKey,
        newAccountPubkey: account.publicKey,
        lamports,
        space,
        programId: TOKEN_PROGRAM_ID,
      }),
      createInitializeAccountInstruction(
        account.publicKey,
        mint,
        owner,
      ),
    );
    await provider.sendAndConfirm(tx, [payer, account]);
  }

  function pda(seed: string): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [Buffer.from(seed)],
      program.programId,
    );
  }
});

function policyParams() {
  return {
    normalBandBps: 300,
    stressedBandBps: 700,
    anchorEmaBps: 500,
    maxAnchorCrawlBps: 100,
    minPremiumBps: 100,
    premiumPersistenceRequired: 2,
    minGrossBuyFloorBps: 50,
    minLockedShareBps: 1000,
    targetGrossBuyBps: 500,
    targetNetBuyBps: 250,
    targetLockFlowBps: 100,
    targetBuyGrowthBps: 500,
    targetLockedShareBps: 3000,
    expansionReserveCoverageBps: 3000,
    targetReserveCoverageBps: 8000,
    neutralReserveCoverageBps: 2000,
    defenseReserveCoverageBps: 1500,
    hardDefenseReserveCoverageBps: 800,
    minStableCashCoverageBps: 1200,
    targetStableCashCoverageBps: 2500,
    defenseStableCashCoverageBps: 800,
    minLiquidityDepthCoverageBps: 2000,
    targetLiquidityDepthCoverageBps: 5000,
    maxReserveConcentrationBps: 6000,
    maxOracleConfidenceBps: 150,
    maxStaleOracleCount: 0,
    maxExpansionVolatilityBps: 300,
    defenseVolatilityBps: 1000,
    maxExpansionExitPressureBps: 3000,
    defenseExitPressureBps: 7000,
    expansionKappaBps: 1000,
    maxMintPerEpochBps: 100,
    maxMintPerDayBps: 250,
    buybackKappaBps: 5000,
    mildDefenseSpendBps: 500,
    severeDefenseSpendBps: 1500,
    severeStressThresholdBps: 1000,
    recoveryCooldownEpochs: 2,
    policyEpochDuration: new anchor.BN(3600),
  };
}
