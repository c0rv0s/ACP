#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;
use anchor_lang::solana_program::program_option::COption;
use anchor_spl::token::{self, Burn, Mint, MintTo, Token, TokenAccount, Transfer};

declare_id!("H1n8VTp6pMY5WFfVfi4MNkQ9q5szkMpVWcHQ21JRETXC");

const STATE_SEED: &[u8] = b"state";
const KEEPER_SEED: &[u8] = b"keeper";
const MINT_AUTHORITY_SEED: &[u8] = b"mint-authority";
const TREASURY_AUTHORITY_SEED: &[u8] = b"treasury-authority";
const XAGC_AUTHORITY_SEED: &[u8] = b"xagc-authority";
const TREASURY_AGC_SEED: &[u8] = b"treasury-agc";
const TREASURY_USDC_SEED: &[u8] = b"treasury-usdc";
const XAGC_VAULT_AGC_SEED: &[u8] = b"xagc-vault-agc";

const BPS: u128 = 10_000;
const SECONDS_PER_DAY: u64 = 86_400;

#[program]
pub mod agc_solana {
    use super::*;

    pub fn initialize_protocol(
        ctx: Context<InitializeProtocol>,
        args: InitializeProtocolArgs,
    ) -> Result<()> {
        validate_mint_authority(&ctx.accounts.agc_mint, ctx.accounts.mint_authority.key())?;
        validate_mint_authority(&ctx.accounts.xagc_mint, ctx.accounts.mint_authority.key())?;
        validate_no_freeze_authority(&ctx.accounts.agc_mint)?;
        validate_no_freeze_authority(&ctx.accounts.xagc_mint)?;
        require_keys_eq!(
            ctx.accounts.agc_mint.key(),
            ctx.accounts.xagc_vault_agc.mint,
            AgcError::InvalidTokenAccount
        );
        require!(
            ctx.accounts.agc_mint.decimals == ctx.accounts.xagc_mint.decimals,
            AgcError::UnsupportedDecimalConfig
        );
        require!(
            ctx.accounts.usdc_mint.decimals <= 18,
            AgcError::UnsupportedDecimalConfig
        );
        require!(
            ctx.accounts.agc_mint.decimals <= 18,
            AgcError::UnsupportedDecimalConfig
        );
        require!(args.initial_anchor_price_x18 > 0, AgcError::InvalidPrice);
        validate_policy_params(args.policy_params)?;
        validate_distribution(args.mint_distribution)?;
        require!(args.exit_fee_bps < BPS as u16, AgcError::InvalidFee);

        let now = current_timestamp()?;
        let state = &mut ctx.accounts.state;
        state.admin = ctx.accounts.admin.key();
        state.pending_admin = Pubkey::default();
        state.agc_mint = ctx.accounts.agc_mint.key();
        state.xagc_mint = ctx.accounts.xagc_mint.key();
        state.usdc_mint = ctx.accounts.usdc_mint.key();
        state.treasury_agc = ctx.accounts.treasury_agc.key();
        state.treasury_usdc = ctx.accounts.treasury_usdc.key();
        state.xagc_vault_agc = ctx.accounts.xagc_vault_agc.key();
        state.growth_programs_agc = args.settlement_recipients.growth_programs_agc;
        state.lp_agc = args.settlement_recipients.lp_agc;
        state.integrators_agc = args.settlement_recipients.integrators_agc;
        state.buyback_usdc_escrow = Pubkey::default();
        state.market_adapter_authority = Pubkey::default();
        state.state_bump = ctx.bumps.state;
        state.mint_authority_bump = ctx.bumps.mint_authority;
        state.treasury_authority_bump = ctx.bumps.treasury_authority;
        state.xagc_authority_bump = ctx.bumps.xagc_authority;
        state.treasury_agc_bump = ctx.bumps.treasury_agc;
        state.treasury_usdc_bump = ctx.bumps.treasury_usdc;
        state.xagc_vault_agc_bump = ctx.bumps.xagc_vault_agc;
        state.agc_decimals = ctx.accounts.agc_mint.decimals;
        state.xagc_decimals = ctx.accounts.xagc_mint.decimals;
        state.usdc_decimals = ctx.accounts.usdc_mint.decimals;
        state.agc_unit = pow10_u64(ctx.accounts.agc_mint.decimals)?;
        state.quote_scale = pow10_u128(18_u8 - ctx.accounts.usdc_mint.decimals)?;
        state.exit_fee_bps = args.exit_fee_bps;
        state.growth_programs_enabled = args.growth_programs_enabled;
        state.pause_flags = PauseFlags::default();
        state.policy_params = args.policy_params;
        state.mint_distribution = args.mint_distribution;
        state.regime = Regime::Neutral;
        state.anchor_price_x18 = args.initial_anchor_price_x18;
        state.accumulator = EpochAccumulator {
            epoch_id: 1,
            started_at: now,
            updated_at: now,
            last_observed_at: now,
            observation_count: 1,
            gross_buy_volume_quote_x18: 0,
            gross_sell_volume_quote_x18: 0,
            total_volume_quote_x18: 0,
            last_mid_price_x18: args.initial_anchor_price_x18,
            cumulative_mid_price_time_x18: 0,
            cumulative_abs_mid_price_change_bps: 0,
            total_hook_fees_quote_x18: 0,
            total_hook_fees_agc: 0,
        };

        emit!(ProtocolInitialized {
            admin: state.admin,
            agc_mint: state.agc_mint,
            xagc_mint: state.xagc_mint,
            usdc_mint: state.usdc_mint,
            initial_anchor_price_x18: state.anchor_price_x18,
        });

        Ok(())
    }

    pub fn set_keeper(ctx: Context<SetKeeper>, allowed: bool) -> Result<()> {
        let permissions = if allowed {
            KeeperPermissions::all()
        } else {
            KeeperPermissions::default()
        };
        set_keeper_permissions_inner(ctx, permissions)
    }

    pub fn set_keeper_permissions(
        ctx: Context<SetKeeper>,
        permissions: KeeperPermissions,
    ) -> Result<()> {
        set_keeper_permissions_inner(ctx, permissions)
    }

    pub fn set_market_adapter_authority(
        ctx: Context<SetMarketAdapterAuthority>,
        authority: Pubkey,
    ) -> Result<()> {
        ctx.accounts.state.market_adapter_authority = authority;
        emit!(MarketAdapterAuthorityUpdated { authority });
        Ok(())
    }

    pub fn set_buyback_usdc_escrow(ctx: Context<SetBuybackUsdcEscrow>) -> Result<()> {
        ctx.accounts.state.buyback_usdc_escrow = ctx.accounts.buyback_usdc_escrow.key();
        emit!(BuybackUsdcEscrowUpdated {
            escrow: ctx.accounts.buyback_usdc_escrow.key(),
        });
        Ok(())
    }

    pub fn transfer_admin(ctx: Context<TransferAdmin>, next_admin: Pubkey) -> Result<()> {
        require!(next_admin != Pubkey::default(), AgcError::InvalidAdmin);
        require_keys_neq!(next_admin, ctx.accounts.state.admin, AgcError::InvalidAdmin);
        ctx.accounts.state.pending_admin = next_admin;
        emit!(AdminTransferStarted {
            current_admin: ctx.accounts.state.admin,
            pending_admin: next_admin,
        });
        Ok(())
    }

    pub fn accept_admin(ctx: Context<AcceptAdmin>) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.pending_admin.key(),
            ctx.accounts.state.pending_admin,
            AgcError::Unauthorized
        );
        let previous_admin = ctx.accounts.state.admin;
        ctx.accounts.state.admin = ctx.accounts.pending_admin.key();
        ctx.accounts.state.pending_admin = Pubkey::default();
        emit!(AdminTransferred {
            previous_admin,
            new_admin: ctx.accounts.state.admin,
        });
        Ok(())
    }

    pub fn set_pause_flags(ctx: Context<SetPauseFlags>, pause_flags: PauseFlags) -> Result<()> {
        ctx.accounts.state.pause_flags = pause_flags;
        emit!(PauseFlagsUpdated { pause_flags });
        Ok(())
    }

    pub fn set_policy_params(ctx: Context<SetPolicyParams>, params: PolicyParams) -> Result<()> {
        validate_policy_params(params)?;
        ctx.accounts.state.policy_params = params;
        emit!(PolicyParametersUpdated {
            normal_band_bps: params.normal_band_bps,
            stressed_band_bps: params.stressed_band_bps,
            policy_epoch_duration: params.policy_epoch_duration,
        });
        Ok(())
    }

    pub fn set_mint_distribution(
        ctx: Context<SetMintDistribution>,
        distribution: MintDistribution,
    ) -> Result<()> {
        validate_distribution(distribution)?;
        ctx.accounts.state.mint_distribution = distribution;
        emit!(MintDistributionUpdated { distribution });
        Ok(())
    }

    pub fn set_settlement_recipients(
        ctx: Context<SetSettlementRecipients>,
        recipients: SettlementRecipients,
    ) -> Result<()> {
        let state = &mut ctx.accounts.state;
        state.growth_programs_agc = recipients.growth_programs_agc;
        state.lp_agc = recipients.lp_agc;
        state.integrators_agc = recipients.integrators_agc;
        emit!(SettlementRecipientsUpdated { recipients });
        Ok(())
    }

    pub fn set_growth_programs_enabled(
        ctx: Context<SetGrowthProgramsEnabled>,
        enabled: bool,
    ) -> Result<()> {
        ctx.accounts.state.growth_programs_enabled = enabled;
        emit!(GrowthProgramsEnabledUpdated { enabled });
        Ok(())
    }

    pub fn set_exit_fee_bps(ctx: Context<SetExitFeeBps>, exit_fee_bps: u16) -> Result<()> {
        require!(exit_fee_bps < BPS as u16, AgcError::InvalidFee);
        ctx.accounts.state.exit_fee_bps = exit_fee_bps;
        emit!(ExitFeeUpdated { exit_fee_bps });
        Ok(())
    }

    pub fn deposit_xagc(ctx: Context<DepositXagc>, assets: u64) -> Result<()> {
        require!(
            !ctx.accounts.state.pause_flags.xagc_deposits_paused,
            AgcError::Paused
        );
        require!(assets > 0, AgcError::ZeroAmount);

        let state = &mut ctx.accounts.state;
        if ctx.accounts.xagc_mint.supply == 0 {
            state.xagc_unaccounted_assets = ctx.accounts.xagc_vault_agc.amount;
        }

        let shares = convert_to_shares(
            assets,
            ctx.accounts.xagc_mint.supply,
            ctx.accounts.xagc_vault_agc.amount,
            state.xagc_unaccounted_assets,
        )?;
        require!(shares > 0, AgcError::ZeroAmount);

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.key(),
                Transfer {
                    from: ctx.accounts.depositor_agc.to_account_info(),
                    to: ctx.accounts.xagc_vault_agc.to_account_info(),
                    authority: ctx.accounts.depositor.to_account_info(),
                },
            ),
            assets,
        )?;

        mint_with_pda(
            &ctx.accounts.xagc_mint,
            &ctx.accounts.receiver_xagc,
            &ctx.accounts.mint_authority,
            &ctx.accounts.token_program,
            state.mint_authority_bump,
            shares,
        )?;

        state.xagc_gross_deposits_total = checked_add_u128(
            state.xagc_gross_deposits_total,
            assets as u128,
            AgcError::MathOverflow,
        )?;

        emit!(XagcDeposited {
            caller: ctx.accounts.depositor.key(),
            receiver_xagc: ctx.accounts.receiver_xagc.key(),
            assets,
            shares,
        });

        Ok(())
    }

    pub fn redeem_xagc(ctx: Context<RedeemXagc>, shares: u64) -> Result<()> {
        require!(
            !ctx.accounts.state.pause_flags.xagc_redemptions_paused,
            AgcError::Paused
        );
        require!(shares > 0, AgcError::ZeroAmount);
        require!(
            ctx.accounts.owner_xagc.amount >= shares,
            AgcError::InsufficientShares
        );

        let state = &mut ctx.accounts.state;
        let gross_assets = convert_to_assets(
            shares,
            ctx.accounts.xagc_mint.supply,
            ctx.accounts.xagc_vault_agc.amount,
            state.xagc_unaccounted_assets,
        )?;
        require!(gross_assets > 0, AgcError::ZeroAmount);

        let fee_assets = checked_div_u128(
            checked_mul_u128(gross_assets as u128, state.exit_fee_bps as u128)?,
            BPS,
        )? as u64;
        let net_assets = gross_assets
            .checked_sub(fee_assets)
            .ok_or(AgcError::MathOverflow)?;

        token::burn(
            CpiContext::new(
                ctx.accounts.token_program.key(),
                Burn {
                    mint: ctx.accounts.xagc_mint.to_account_info(),
                    from: ctx.accounts.owner_xagc.to_account_info(),
                    authority: ctx.accounts.owner.to_account_info(),
                },
            ),
            shares,
        )?;

        if fee_assets > 0 {
            transfer_from_xagc_vault(
                &ctx.accounts.xagc_vault_agc,
                &ctx.accounts.treasury_agc,
                &ctx.accounts.xagc_authority,
                &ctx.accounts.token_program,
                state.xagc_authority_bump,
                fee_assets,
            )?;
        }

        transfer_from_xagc_vault(
            &ctx.accounts.xagc_vault_agc,
            &ctx.accounts.receiver_agc,
            &ctx.accounts.xagc_authority,
            &ctx.accounts.token_program,
            state.xagc_authority_bump,
            net_assets,
        )?;

        state.xagc_gross_redemptions_total = checked_add_u128(
            state.xagc_gross_redemptions_total,
            gross_assets as u128,
            AgcError::MathOverflow,
        )?;

        emit!(XagcRedeemed {
            caller: ctx.accounts.owner.key(),
            receiver_agc: ctx.accounts.receiver_agc.key(),
            shares,
            gross_assets,
            fee_assets,
            net_assets,
        });

        Ok(())
    }

    pub fn record_swap(ctx: Context<RecordSwap>, args: RecordSwapArgs) -> Result<()> {
        require!(
            !ctx.accounts.state.pause_flags.market_reporting_paused,
            AgcError::Paused
        );
        assert_market_reporter_or_admin(
            &ctx.accounts.state,
            ctx.accounts.authority.key(),
            ctx.accounts.keeper.to_account_info(),
        )?;
        require!(args.price_x18 > 0, AgcError::InvalidPrice);

        let now = current_timestamp()?;
        let quote_amount_x18 = quote_to_x18(&ctx.accounts.state, args.usdc_amount)?;
        let state = &mut ctx.accounts.state;

        observe_mid_price(state, args.price_x18, now)?;
        state.accumulator.total_volume_quote_x18 = checked_add_u128(
            state.accumulator.total_volume_quote_x18,
            quote_amount_x18,
            AgcError::MathOverflow,
        )?;

        if args.agc_to_usdc {
            state.accumulator.gross_sell_volume_quote_x18 = checked_add_u128(
                state.accumulator.gross_sell_volume_quote_x18,
                quote_amount_x18,
                AgcError::MathOverflow,
            )?;
        } else {
            state.accumulator.gross_buy_volume_quote_x18 = checked_add_u128(
                state.accumulator.gross_buy_volume_quote_x18,
                quote_amount_x18,
                AgcError::MathOverflow,
            )?;
        }

        state.accumulator.total_hook_fees_quote_x18 = checked_add_u128(
            state.accumulator.total_hook_fees_quote_x18,
            quote_to_x18(state, args.hook_fee_usdc)?,
            AgcError::MathOverflow,
        )?;
        state.accumulator.total_hook_fees_agc = checked_add_u128(
            state.accumulator.total_hook_fees_agc,
            args.hook_fee_agc as u128,
            AgcError::MathOverflow,
        )?;

        emit!(SwapRecorded {
            epoch_id: state.accumulator.epoch_id,
            agc_amount: args.agc_amount,
            usdc_amount: args.usdc_amount,
            price_x18: args.price_x18,
            agc_to_usdc: args.agc_to_usdc,
        });

        Ok(())
    }

    pub fn record_market_observation(ctx: Context<RecordSwap>, price_x18: u128) -> Result<()> {
        require!(
            !ctx.accounts.state.pause_flags.market_reporting_paused,
            AgcError::Paused
        );
        assert_market_reporter_or_admin(
            &ctx.accounts.state,
            ctx.accounts.authority.key(),
            ctx.accounts.keeper.to_account_info(),
        )?;
        require!(price_x18 > 0, AgcError::InvalidPrice);
        let now = current_timestamp()?;
        observe_mid_price(&mut ctx.accounts.state, price_x18, now)
    }

    pub fn settle_epoch(
        ctx: Context<SettleEpoch>,
        external_metrics: ExternalMetrics,
    ) -> Result<()> {
        require!(
            !ctx.accounts.state.pause_flags.settlement_paused,
            AgcError::Paused
        );
        assert_keeper_permission_or_admin(
            &ctx.accounts.state,
            ctx.accounts.authority.key(),
            ctx.accounts.keeper.to_account_info(),
            RequiredKeeperPermission::SettleEpoch,
        )?;
        require_keys_eq!(
            ctx.accounts.growth_programs_agc.key(),
            ctx.accounts.state.growth_programs_agc,
            AgcError::InvalidSettlementRecipient
        );
        require_keys_eq!(
            ctx.accounts.lp_agc.key(),
            ctx.accounts.state.lp_agc,
            AgcError::InvalidSettlementRecipient
        );
        require_keys_eq!(
            ctx.accounts.integrators_agc.key(),
            ctx.accounts.state.integrators_agc,
            AgcError::InvalidSettlementRecipient
        );

        let now = current_timestamp()?;
        {
            let state = &mut ctx.accounts.state;
            refresh_mint_window(state, now);
        }
        let state_snapshot = ctx.accounts.state.clone();
        validate_settlement_window(&state_snapshot, now)?;
        require!(
            state_snapshot.accumulator.epoch_id > state_snapshot.last_settled_epoch,
            AgcError::InvalidEpoch
        );

        let float_supply = circulating_float(
            ctx.accounts.agc_mint.supply,
            ctx.accounts.treasury_agc.amount,
            ctx.accounts.xagc_vault_agc.amount,
        );
        let treasury_quote_x18 = quote_to_x18(&state_snapshot, ctx.accounts.treasury_usdc.amount)?;

        let policy_state = PolicyState {
            anchor_price_x18: state_snapshot.anchor_price_x18,
            premium_persistence_epochs: state_snapshot.premium_persistence_epochs,
            last_gross_buy_quote_x18: state_snapshot.last_gross_buy_quote_x18,
            minted_today_acp: state_snapshot.minted_in_current_day,
            last_regime: state_snapshot.regime,
            recovery_cooldown_epochs_remaining: state_snapshot.recovery_cooldown_epochs_remaining,
            float_supply_acp: float_supply as u128,
            treasury_quote_x18,
            treasury_acp: ctx.accounts.treasury_agc.amount as u128,
            xagc_total_assets_acp: ctx.accounts.xagc_vault_agc.amount as u128,
        };
        let vault_flows = VaultFlows {
            xagc_deposits_acp: state_snapshot
                .xagc_gross_deposits_total
                .saturating_sub(state_snapshot.last_xagc_deposit_total),
            xagc_gross_redemptions_acp: state_snapshot
                .xagc_gross_redemptions_total
                .saturating_sub(state_snapshot.last_xagc_redemption_total),
        };
        let snapshot = preview_epoch_snapshot(&state_snapshot.accumulator, now)?;
        let mut result = evaluate_epoch(
            snapshot,
            external_metrics,
            policy_state,
            vault_flows,
            state_snapshot.policy_params,
            state_snapshot.agc_unit as u128,
        )?;
        result.mint_allocations =
            allocate_mint(result.mint_budget_acp, state_snapshot.mint_distribution);
        if !state_snapshot.growth_programs_enabled {
            result.mint_allocations.treasury_mint_acp = checked_add_u128(
                result.mint_allocations.treasury_mint_acp,
                result.mint_allocations.growth_programs_mint_acp,
                AgcError::MathOverflow,
            )?;
            result.mint_allocations.growth_programs_mint_acp = 0;
        }
        if result.mint_allocations.xagc_mint_acp > 0 && ctx.accounts.xagc_mint.supply == 0 {
            result.mint_allocations.treasury_mint_acp = checked_add_u128(
                result.mint_allocations.treasury_mint_acp,
                result.mint_allocations.xagc_mint_acp,
                AgcError::MathOverflow,
            )?;
            result.mint_allocations.xagc_mint_acp = 0;
        }

        {
            let state = &mut ctx.accounts.state;
            state.minted_in_current_day = checked_add_u128(
                state.minted_in_current_day,
                result.mint_budget_acp,
                AgcError::MathOverflow,
            )?;
        }

        mint_policy_allocations(&ctx, result.mint_allocations)?;

        let raw_buyback_budget =
            quote_from_x18(&ctx.accounts.state, result.buyback_budget_quote_x18)?;
        {
            let state = &mut ctx.accounts.state;
            persist_epoch_settlement(state, snapshot, result, raw_buyback_budget, now)?;
        }

        emit!(EpochSettled {
            epoch_id: snapshot.epoch_id,
            regime: result.regime,
            anchor_next_x18: result.anchor_next_x18,
            mint_budget_acp: result.mint_budget_acp,
            buyback_budget_quote_x18: result.buyback_budget_quote_x18,
        });

        Ok(())
    }

    pub fn reserve_treasury_buyback_usdc(
        ctx: Context<ReserveTreasuryBuybackUsdc>,
        amount: u64,
    ) -> Result<()> {
        require!(
            !ctx.accounts.state.pause_flags.buybacks_paused,
            AgcError::Paused
        );
        assert_keeper_permission_or_admin(
            &ctx.accounts.state,
            ctx.accounts.authority.key(),
            ctx.accounts.keeper.to_account_info(),
            RequiredKeeperPermission::ExecuteBuyback,
        )?;
        require!(amount > 0, AgcError::ZeroAmount);
        let state = &mut ctx.accounts.state;
        require!(
            state.buyback_usdc_escrow != Pubkey::default(),
            AgcError::BuybackEscrowNotConfigured
        );
        require_keys_eq!(
            ctx.accounts.buyback_usdc_destination.key(),
            state.buyback_usdc_escrow,
            AgcError::InvalidBuybackEscrow
        );
        let spend = amount.min(state.pending_treasury_buyback_usdc);
        require!(spend > 0, AgcError::NoPendingTreasuryBuyback);

        transfer_from_treasury(
            &ctx.accounts.treasury_usdc,
            &ctx.accounts.buyback_usdc_destination,
            &ctx.accounts.treasury_authority,
            &ctx.accounts.token_program,
            state.treasury_authority_bump,
            spend,
        )?;

        state.pending_treasury_buyback_usdc = state
            .pending_treasury_buyback_usdc
            .checked_sub(spend)
            .ok_or(AgcError::MathOverflow)?;
        state.buyback_execution_nonce = state
            .buyback_execution_nonce
            .checked_add(1)
            .ok_or(AgcError::MathOverflow)?;

        emit!(TreasuryBuybackUsdcReserved {
            nonce: state.buyback_execution_nonce,
            usdc_spent: spend,
            pending_treasury_buyback_usdc_after: state.pending_treasury_buyback_usdc,
        });

        Ok(())
    }

    pub fn burn_treasury_agc(ctx: Context<BurnTreasuryAgc>, amount: u64) -> Result<()> {
        require!(
            !ctx.accounts.state.pause_flags.treasury_burns_paused,
            AgcError::Paused
        );
        assert_keeper_permission_or_admin(
            &ctx.accounts.state,
            ctx.accounts.authority.key(),
            ctx.accounts.keeper.to_account_info(),
            RequiredKeeperPermission::BurnTreasury,
        )?;
        require!(amount > 0, AgcError::ZeroAmount);

        let state = &ctx.accounts.state;
        let signer: &[&[&[u8]]] = &[&[TREASURY_AUTHORITY_SEED, &[state.treasury_authority_bump]]];
        token::burn(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.key(),
                Burn {
                    mint: ctx.accounts.agc_mint.to_account_info(),
                    from: ctx.accounts.treasury_agc.to_account_info(),
                    authority: ctx.accounts.treasury_authority.to_account_info(),
                },
                signer,
            ),
            amount,
        )?;

        emit!(TreasuryAgcBurned { amount });

        Ok(())
    }
}

fn set_keeper_permissions_inner(
    ctx: Context<SetKeeper>,
    permissions: KeeperPermissions,
) -> Result<()> {
    let keeper = &mut ctx.accounts.keeper;
    keeper.authority = ctx.accounts.keeper_authority.key();
    keeper.permissions = permissions;
    keeper.bump = ctx.bumps.keeper;

    emit!(KeeperPermissionsUpdated {
        keeper: keeper.authority,
        permissions,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct InitializeProtocol<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = payer,
        seeds = [STATE_SEED],
        bump,
        space = 8 + ProtocolState::LEN
    )]
    pub state: Box<Account<'info, ProtocolState>>,
    #[account(mut)]
    pub agc_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub xagc_mint: Box<Account<'info, Mint>>,
    pub usdc_mint: Box<Account<'info, Mint>>,
    #[account(
        init,
        payer = payer,
        seeds = [TREASURY_AGC_SEED],
        bump,
        token::mint = agc_mint,
        token::authority = treasury_authority
    )]
    pub treasury_agc: Box<Account<'info, TokenAccount>>,
    #[account(
        init,
        payer = payer,
        seeds = [TREASURY_USDC_SEED],
        bump,
        token::mint = usdc_mint,
        token::authority = treasury_authority
    )]
    pub treasury_usdc: Box<Account<'info, TokenAccount>>,
    #[account(
        init,
        payer = payer,
        seeds = [XAGC_VAULT_AGC_SEED],
        bump,
        token::mint = agc_mint,
        token::authority = xagc_authority
    )]
    pub xagc_vault_agc: Box<Account<'info, TokenAccount>>,
    /// CHECK: PDA only used as SPL mint authority.
    #[account(seeds = [MINT_AUTHORITY_SEED], bump)]
    pub mint_authority: UncheckedAccount<'info>,
    /// CHECK: PDA only signs treasury token-account operations.
    #[account(seeds = [TREASURY_AUTHORITY_SEED], bump)]
    pub treasury_authority: UncheckedAccount<'info>,
    /// CHECK: PDA only signs xAGC vault token-account operations.
    #[account(seeds = [XAGC_AUTHORITY_SEED], bump)]
    pub xagc_authority: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct SetKeeper<'info> {
    #[account(mut, seeds = [STATE_SEED], bump = state.state_bump, has_one = admin)]
    pub state: Box<Account<'info, ProtocolState>>,
    #[account(mut)]
    pub admin: Signer<'info>,
    /// CHECK: Stored as the keeper authority key.
    pub keeper_authority: UncheckedAccount<'info>,
    #[account(
        init_if_needed,
        payer = admin,
        seeds = [KEEPER_SEED, keeper_authority.key().as_ref()],
        bump,
        space = 8 + Keeper::LEN
    )]
    pub keeper: Box<Account<'info, Keeper>>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetMarketAdapterAuthority<'info> {
    #[account(mut, seeds = [STATE_SEED], bump = state.state_bump, has_one = admin)]
    pub state: Box<Account<'info, ProtocolState>>,
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct SetBuybackUsdcEscrow<'info> {
    #[account(mut, seeds = [STATE_SEED], bump = state.state_bump, has_one = admin)]
    pub state: Box<Account<'info, ProtocolState>>,
    pub admin: Signer<'info>,
    #[account(
        constraint = buyback_usdc_escrow.mint == state.usdc_mint @ AgcError::InvalidTokenAccount
    )]
    pub buyback_usdc_escrow: Box<Account<'info, TokenAccount>>,
}

#[derive(Accounts)]
pub struct TransferAdmin<'info> {
    #[account(mut, seeds = [STATE_SEED], bump = state.state_bump, has_one = admin)]
    pub state: Box<Account<'info, ProtocolState>>,
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct AcceptAdmin<'info> {
    #[account(mut, seeds = [STATE_SEED], bump = state.state_bump)]
    pub state: Box<Account<'info, ProtocolState>>,
    pub pending_admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct SetPauseFlags<'info> {
    #[account(mut, seeds = [STATE_SEED], bump = state.state_bump, has_one = admin)]
    pub state: Box<Account<'info, ProtocolState>>,
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct SetPolicyParams<'info> {
    #[account(mut, seeds = [STATE_SEED], bump = state.state_bump, has_one = admin)]
    pub state: Box<Account<'info, ProtocolState>>,
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct SetMintDistribution<'info> {
    #[account(mut, seeds = [STATE_SEED], bump = state.state_bump, has_one = admin)]
    pub state: Box<Account<'info, ProtocolState>>,
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct SetSettlementRecipients<'info> {
    #[account(mut, seeds = [STATE_SEED], bump = state.state_bump, has_one = admin)]
    pub state: Box<Account<'info, ProtocolState>>,
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct SetGrowthProgramsEnabled<'info> {
    #[account(mut, seeds = [STATE_SEED], bump = state.state_bump, has_one = admin)]
    pub state: Box<Account<'info, ProtocolState>>,
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct SetExitFeeBps<'info> {
    #[account(mut, seeds = [STATE_SEED], bump = state.state_bump, has_one = admin)]
    pub state: Box<Account<'info, ProtocolState>>,
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct DepositXagc<'info> {
    #[account(mut, seeds = [STATE_SEED], bump = state.state_bump)]
    pub state: Box<Account<'info, ProtocolState>>,
    pub depositor: Signer<'info>,
    #[account(
        mut,
        constraint = depositor_agc.owner == depositor.key() @ AgcError::InvalidTokenAccount,
        constraint = depositor_agc.mint == state.agc_mint @ AgcError::InvalidTokenAccount
    )]
    pub depositor_agc: Box<Account<'info, TokenAccount>>,
    #[account(mut, address = state.xagc_vault_agc)]
    pub xagc_vault_agc: Box<Account<'info, TokenAccount>>,
    #[account(mut, address = state.xagc_mint)]
    pub xagc_mint: Box<Account<'info, Mint>>,
    #[account(mut, constraint = receiver_xagc.mint == state.xagc_mint @ AgcError::InvalidTokenAccount)]
    pub receiver_xagc: Box<Account<'info, TokenAccount>>,
    /// CHECK: PDA only used as SPL mint authority.
    #[account(seeds = [MINT_AUTHORITY_SEED], bump = state.mint_authority_bump)]
    pub mint_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct RedeemXagc<'info> {
    #[account(mut, seeds = [STATE_SEED], bump = state.state_bump)]
    pub state: Box<Account<'info, ProtocolState>>,
    pub owner: Signer<'info>,
    #[account(
        mut,
        constraint = owner_xagc.owner == owner.key() @ AgcError::InvalidTokenAccount,
        constraint = owner_xagc.mint == state.xagc_mint @ AgcError::InvalidTokenAccount
    )]
    pub owner_xagc: Box<Account<'info, TokenAccount>>,
    #[account(mut, address = state.xagc_mint)]
    pub xagc_mint: Box<Account<'info, Mint>>,
    #[account(mut, address = state.xagc_vault_agc)]
    pub xagc_vault_agc: Box<Account<'info, TokenAccount>>,
    #[account(mut, address = state.treasury_agc)]
    pub treasury_agc: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = receiver_agc.mint == state.agc_mint @ AgcError::InvalidTokenAccount)]
    pub receiver_agc: Box<Account<'info, TokenAccount>>,
    /// CHECK: PDA only signs xAGC vault token-account operations.
    #[account(seeds = [XAGC_AUTHORITY_SEED], bump = state.xagc_authority_bump)]
    pub xagc_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct RecordSwap<'info> {
    #[account(mut, seeds = [STATE_SEED], bump = state.state_bump)]
    pub state: Box<Account<'info, ProtocolState>>,
    pub authority: Signer<'info>,
    /// CHECK: Deserialized manually only when authority is not admin.
    pub keeper: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct SettleEpoch<'info> {
    #[account(mut, seeds = [STATE_SEED], bump = state.state_bump)]
    pub state: Box<Account<'info, ProtocolState>>,
    pub authority: Signer<'info>,
    /// CHECK: Deserialized manually only when authority is not admin.
    pub keeper: UncheckedAccount<'info>,
    #[account(mut, address = state.agc_mint)]
    pub agc_mint: Box<Account<'info, Mint>>,
    #[account(address = state.xagc_mint)]
    pub xagc_mint: Box<Account<'info, Mint>>,
    #[account(mut, address = state.xagc_vault_agc)]
    pub xagc_vault_agc: Box<Account<'info, TokenAccount>>,
    #[account(mut, address = state.treasury_agc)]
    pub treasury_agc: Box<Account<'info, TokenAccount>>,
    #[account(address = state.treasury_usdc)]
    pub treasury_usdc: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = growth_programs_agc.mint == state.agc_mint @ AgcError::InvalidTokenAccount)]
    pub growth_programs_agc: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = lp_agc.mint == state.agc_mint @ AgcError::InvalidTokenAccount)]
    pub lp_agc: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = integrators_agc.mint == state.agc_mint @ AgcError::InvalidTokenAccount)]
    pub integrators_agc: Box<Account<'info, TokenAccount>>,
    /// CHECK: PDA only used as SPL mint authority.
    #[account(seeds = [MINT_AUTHORITY_SEED], bump = state.mint_authority_bump)]
    pub mint_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ReserveTreasuryBuybackUsdc<'info> {
    #[account(mut, seeds = [STATE_SEED], bump = state.state_bump)]
    pub state: Box<Account<'info, ProtocolState>>,
    pub authority: Signer<'info>,
    /// CHECK: Deserialized manually only when authority is not admin.
    pub keeper: UncheckedAccount<'info>,
    #[account(mut, address = state.treasury_usdc)]
    pub treasury_usdc: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = buyback_usdc_destination.mint == state.usdc_mint @ AgcError::InvalidTokenAccount)]
    pub buyback_usdc_destination: Box<Account<'info, TokenAccount>>,
    /// CHECK: PDA only signs treasury token-account operations.
    #[account(seeds = [TREASURY_AUTHORITY_SEED], bump = state.treasury_authority_bump)]
    pub treasury_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct BurnTreasuryAgc<'info> {
    #[account(seeds = [STATE_SEED], bump = state.state_bump)]
    pub state: Box<Account<'info, ProtocolState>>,
    pub authority: Signer<'info>,
    /// CHECK: Deserialized manually only when authority is not admin.
    pub keeper: UncheckedAccount<'info>,
    #[account(mut, address = state.agc_mint)]
    pub agc_mint: Box<Account<'info, Mint>>,
    #[account(mut, address = state.treasury_agc)]
    pub treasury_agc: Box<Account<'info, TokenAccount>>,
    /// CHECK: PDA only signs treasury token-account operations.
    #[account(seeds = [TREASURY_AUTHORITY_SEED], bump = state.treasury_authority_bump)]
    pub treasury_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct ProtocolState {
    pub admin: Pubkey,
    pub pending_admin: Pubkey,
    pub agc_mint: Pubkey,
    pub xagc_mint: Pubkey,
    pub usdc_mint: Pubkey,
    pub treasury_agc: Pubkey,
    pub treasury_usdc: Pubkey,
    pub xagc_vault_agc: Pubkey,
    pub growth_programs_agc: Pubkey,
    pub lp_agc: Pubkey,
    pub integrators_agc: Pubkey,
    pub buyback_usdc_escrow: Pubkey,
    pub market_adapter_authority: Pubkey,
    pub state_bump: u8,
    pub mint_authority_bump: u8,
    pub treasury_authority_bump: u8,
    pub xagc_authority_bump: u8,
    pub treasury_agc_bump: u8,
    pub treasury_usdc_bump: u8,
    pub xagc_vault_agc_bump: u8,
    pub agc_decimals: u8,
    pub xagc_decimals: u8,
    pub usdc_decimals: u8,
    pub agc_unit: u64,
    pub quote_scale: u128,
    pub exit_fee_bps: u16,
    pub growth_programs_enabled: bool,
    pub pause_flags: PauseFlags,
    pub policy_params: PolicyParams,
    pub mint_distribution: MintDistribution,
    pub regime: Regime,
    pub anchor_price_x18: u128,
    pub premium_persistence_epochs: u128,
    pub last_gross_buy_quote_x18: u128,
    pub last_coverage_bps: u128,
    pub last_exit_pressure_bps: u128,
    pub last_volatility_bps: u128,
    pub last_premium_bps: u128,
    pub last_locked_share_bps: u128,
    pub last_lock_flow_bps: u128,
    pub last_settled_epoch: u64,
    pub last_settlement_timestamp: u64,
    pub recovery_cooldown_epochs_remaining: u64,
    pub mint_window_day: u64,
    pub minted_in_current_day: u128,
    pub pending_treasury_buyback_usdc: u64,
    pub xagc_gross_deposits_total: u128,
    pub xagc_gross_redemptions_total: u128,
    pub xagc_unaccounted_assets: u64,
    pub last_xagc_deposit_total: u128,
    pub last_xagc_redemption_total: u128,
    pub buyback_execution_nonce: u64,
    pub accumulator: EpochAccumulator,
    pub last_epoch_result: EpochResult,
}

impl ProtocolState {
    pub const LEN: usize = 8192;
}

#[account]
pub struct Keeper {
    pub authority: Pubkey,
    pub permissions: KeeperPermissions,
    pub bump: u8,
}

impl Keeper {
    pub const LEN: usize = 32 + KeeperPermissions::LEN + 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct KeeperPermissions {
    pub market_reporter: bool,
    pub epoch_settler: bool,
    pub buyback_executor: bool,
    pub treasury_burner: bool,
}

impl KeeperPermissions {
    pub const LEN: usize = 4;

    pub fn all() -> Self {
        Self {
            market_reporter: true,
            epoch_settler: true,
            buyback_executor: true,
            treasury_burner: true,
        }
    }

    fn allows(self, required: RequiredKeeperPermission) -> bool {
        match required {
            RequiredKeeperPermission::ReportMarket => self.market_reporter,
            RequiredKeeperPermission::SettleEpoch => self.epoch_settler,
            RequiredKeeperPermission::ExecuteBuyback => self.buyback_executor,
            RequiredKeeperPermission::BurnTreasury => self.treasury_burner,
        }
    }
}

#[derive(Clone, Copy)]
enum RequiredKeeperPermission {
    ReportMarket,
    SettleEpoch,
    ExecuteBuyback,
    BurnTreasury,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct PauseFlags {
    pub xagc_deposits_paused: bool,
    pub xagc_redemptions_paused: bool,
    pub market_reporting_paused: bool,
    pub settlement_paused: bool,
    pub buybacks_paused: bool,
    pub treasury_burns_paused: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Regime {
    #[default]
    Neutral,
    Expansion,
    Defense,
    Recovery,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct InitializeProtocolArgs {
    pub initial_anchor_price_x18: u128,
    pub policy_params: PolicyParams,
    pub mint_distribution: MintDistribution,
    pub settlement_recipients: SettlementRecipients,
    pub exit_fee_bps: u16,
    pub growth_programs_enabled: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct SettlementRecipients {
    pub growth_programs_agc: Pubkey,
    pub lp_agc: Pubkey,
    pub integrators_agc: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct MintDistribution {
    pub xagc_bps: u16,
    pub growth_programs_bps: u16,
    pub lp_bps: u16,
    pub integrators_bps: u16,
    pub treasury_bps: u16,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct MintAllocation {
    pub xagc_mint_acp: u128,
    pub growth_programs_mint_acp: u128,
    pub lp_mint_acp: u128,
    pub integrators_mint_acp: u128,
    pub treasury_mint_acp: u128,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct PolicyParams {
    pub normal_band_bps: u16,
    pub stressed_band_bps: u16,
    pub anchor_ema_bps: u16,
    pub max_anchor_crawl_bps: u16,
    pub min_premium_bps: u16,
    pub premium_persistence_required: u16,
    pub min_gross_buy_floor_bps: u16,
    pub min_locked_share_bps: u16,
    pub target_gross_buy_bps: u16,
    pub target_net_buy_bps: u16,
    pub target_lock_flow_bps: u16,
    pub target_buy_growth_bps: u16,
    pub target_locked_share_bps: u16,
    pub expansion_reserve_coverage_bps: u16,
    pub target_reserve_coverage_bps: u16,
    pub neutral_reserve_coverage_bps: u16,
    pub defense_reserve_coverage_bps: u16,
    pub hard_defense_reserve_coverage_bps: u16,
    pub max_expansion_volatility_bps: u16,
    pub defense_volatility_bps: u16,
    pub max_expansion_exit_pressure_bps: u16,
    pub defense_exit_pressure_bps: u16,
    pub expansion_kappa_bps: u16,
    pub max_mint_per_epoch_bps: u16,
    pub max_mint_per_day_bps: u16,
    pub buyback_kappa_bps: u16,
    pub mild_defense_spend_bps: u16,
    pub severe_defense_spend_bps: u16,
    pub severe_stress_threshold_bps: u16,
    pub recovery_cooldown_epochs: u16,
    pub policy_epoch_duration: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct EpochAccumulator {
    pub epoch_id: u64,
    pub started_at: u64,
    pub updated_at: u64,
    pub last_observed_at: u64,
    pub observation_count: u64,
    pub gross_buy_volume_quote_x18: u128,
    pub gross_sell_volume_quote_x18: u128,
    pub total_volume_quote_x18: u128,
    pub last_mid_price_x18: u128,
    pub cumulative_mid_price_time_x18: u128,
    pub cumulative_abs_mid_price_change_bps: u128,
    pub total_hook_fees_quote_x18: u128,
    pub total_hook_fees_agc: u128,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct EpochSnapshot {
    pub epoch_id: u64,
    pub started_at: u64,
    pub ended_at: u64,
    pub gross_buy_volume_quote_x18: u128,
    pub gross_sell_volume_quote_x18: u128,
    pub total_volume_quote_x18: u128,
    pub short_twap_price_x18: u128,
    pub realized_volatility_bps: u128,
    pub total_hook_fees_quote_x18: u128,
    pub total_hook_fees_agc: u128,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct ExternalMetrics {
    pub depth_to_target_slippage_quote_x18: u128,
}

#[derive(Clone, Copy, Default)]
pub struct PolicyState {
    pub anchor_price_x18: u128,
    pub premium_persistence_epochs: u128,
    pub last_gross_buy_quote_x18: u128,
    pub minted_today_acp: u128,
    pub last_regime: Regime,
    pub recovery_cooldown_epochs_remaining: u64,
    pub float_supply_acp: u128,
    pub treasury_quote_x18: u128,
    pub treasury_acp: u128,
    pub xagc_total_assets_acp: u128,
}

#[derive(Clone, Copy, Default)]
pub struct VaultFlows {
    pub xagc_deposits_acp: u128,
    pub xagc_gross_redemptions_acp: u128,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct EpochResult {
    pub epoch_id: u64,
    pub regime: Regime,
    pub anchor_price_x18: u128,
    pub anchor_next_x18: u128,
    pub normal_floor_x18: u128,
    pub stressed_floor_x18: u128,
    pub price_twap_x18: u128,
    pub premium_bps: u128,
    pub premium_persistence_epochs: u128,
    pub credit_outstanding_quote_x18: u128,
    pub gross_buy_floor_bps: u128,
    pub net_buy_pressure_bps: u128,
    pub buy_growth_bps: u128,
    pub exit_pressure_bps: u128,
    pub reserve_coverage_bps: u128,
    pub locked_share_bps: u128,
    pub lock_flow_bps: u128,
    pub demand_score_bps: u128,
    pub health_score_bps: u128,
    pub mint_rate_bps: u128,
    pub mint_budget_acp: u128,
    pub buyback_budget_quote_x18: u128,
    pub stress_score_bps: u128,
    pub gross_buy_quote_x18: u128,
    pub gross_sell_quote_x18: u128,
    pub total_volume_quote_x18: u128,
    pub depth_to_target_slippage_quote_x18: u128,
    pub realized_volatility_bps: u128,
    pub xagc_deposits_acp: u128,
    pub xagc_gross_redemptions_acp: u128,
    pub treasury_quote_x18: u128,
    pub treasury_acp: u128,
    pub xagc_total_assets_acp: u128,
    pub mint_allocations: MintAllocation,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct RecordSwapArgs {
    pub agc_amount: u64,
    pub usdc_amount: u64,
    pub price_x18: u128,
    pub agc_to_usdc: bool,
    pub hook_fee_usdc: u64,
    pub hook_fee_agc: u64,
}

#[event]
pub struct ProtocolInitialized {
    pub admin: Pubkey,
    pub agc_mint: Pubkey,
    pub xagc_mint: Pubkey,
    pub usdc_mint: Pubkey,
    pub initial_anchor_price_x18: u128,
}

#[event]
pub struct KeeperPermissionsUpdated {
    pub keeper: Pubkey,
    pub permissions: KeeperPermissions,
}

#[event]
pub struct MarketAdapterAuthorityUpdated {
    pub authority: Pubkey,
}

#[event]
pub struct BuybackUsdcEscrowUpdated {
    pub escrow: Pubkey,
}

#[event]
pub struct AdminTransferStarted {
    pub current_admin: Pubkey,
    pub pending_admin: Pubkey,
}

#[event]
pub struct AdminTransferred {
    pub previous_admin: Pubkey,
    pub new_admin: Pubkey,
}

#[event]
pub struct PauseFlagsUpdated {
    pub pause_flags: PauseFlags,
}

#[event]
pub struct PolicyParametersUpdated {
    pub normal_band_bps: u16,
    pub stressed_band_bps: u16,
    pub policy_epoch_duration: u64,
}

#[event]
pub struct MintDistributionUpdated {
    pub distribution: MintDistribution,
}

#[event]
pub struct SettlementRecipientsUpdated {
    pub recipients: SettlementRecipients,
}

#[event]
pub struct GrowthProgramsEnabledUpdated {
    pub enabled: bool,
}

#[event]
pub struct ExitFeeUpdated {
    pub exit_fee_bps: u16,
}

#[event]
pub struct XagcDeposited {
    pub caller: Pubkey,
    pub receiver_xagc: Pubkey,
    pub assets: u64,
    pub shares: u64,
}

#[event]
pub struct XagcRedeemed {
    pub caller: Pubkey,
    pub receiver_agc: Pubkey,
    pub shares: u64,
    pub gross_assets: u64,
    pub fee_assets: u64,
    pub net_assets: u64,
}

#[event]
pub struct SwapRecorded {
    pub epoch_id: u64,
    pub agc_amount: u64,
    pub usdc_amount: u64,
    pub price_x18: u128,
    pub agc_to_usdc: bool,
}

#[event]
pub struct EpochSettled {
    pub epoch_id: u64,
    pub regime: Regime,
    pub anchor_next_x18: u128,
    pub mint_budget_acp: u128,
    pub buyback_budget_quote_x18: u128,
}

#[event]
pub struct TreasuryBuybackUsdcReserved {
    pub nonce: u64,
    pub usdc_spent: u64,
    pub pending_treasury_buyback_usdc_after: u64,
}

#[event]
pub struct TreasuryAgcBurned {
    pub amount: u64,
}

#[error_code]
pub enum AgcError {
    #[msg("The signer is not authorized for this instruction.")]
    Unauthorized,
    #[msg("The configured SPL token mint authority is not the AGC program PDA.")]
    InvalidMintAuthority,
    #[msg("The AGC-controlled mint must not have an external freeze authority.")]
    InvalidFreezeAuthority,
    #[msg("The provided token account does not match the protocol configuration.")]
    InvalidTokenAccount,
    #[msg("The mint decimals are unsupported.")]
    UnsupportedDecimalConfig,
    #[msg("The mint distribution must total 10_000 bps and include xAGC.")]
    InvalidMintDistribution,
    #[msg("The fee must be below 10_000 bps.")]
    InvalidFee,
    #[msg("The policy parameters are internally inconsistent.")]
    InvalidPolicyParams,
    #[msg("The amount must be non-zero.")]
    ZeroAmount,
    #[msg("The xAGC token account does not have enough shares.")]
    InsufficientShares,
    #[msg("The market price must be non-zero.")]
    InvalidPrice,
    #[msg("The epoch cannot be settled yet.")]
    EpochTooSoon,
    #[msg("The epoch id has already been settled.")]
    InvalidEpoch,
    #[msg("A settlement recipient account does not match protocol state.")]
    InvalidSettlementRecipient,
    #[msg("There is no queued treasury buyback budget.")]
    NoPendingTreasuryBuyback,
    #[msg("The protocol buyback escrow has not been configured.")]
    BuybackEscrowNotConfigured,
    #[msg("The provided buyback escrow is not the configured escrow.")]
    InvalidBuybackEscrow,
    #[msg("The protocol is paused for this instruction.")]
    Paused,
    #[msg("The requested admin is invalid.")]
    InvalidAdmin,
    #[msg("Arithmetic overflow or underflow.")]
    MathOverflow,
    #[msg("A u128 policy amount does not fit into a u64 SPL token amount.")]
    AmountTooLarge,
    #[msg("Clock returned a negative timestamp.")]
    InvalidClock,
}

fn validate_mint_authority(mint: &Account<Mint>, authority: Pubkey) -> Result<()> {
    require!(
        mint.mint_authority == COption::Some(authority),
        AgcError::InvalidMintAuthority
    );
    Ok(())
}

fn validate_no_freeze_authority(mint: &Account<Mint>) -> Result<()> {
    require!(
        mint.freeze_authority == COption::None,
        AgcError::InvalidFreezeAuthority
    );
    Ok(())
}

fn validate_distribution(distribution: MintDistribution) -> Result<()> {
    let total = distribution.xagc_bps as u32
        + distribution.growth_programs_bps as u32
        + distribution.lp_bps as u32
        + distribution.integrators_bps as u32
        + distribution.treasury_bps as u32;
    require!(
        total == BPS as u32 && distribution.xagc_bps > 0,
        AgcError::InvalidMintDistribution
    );
    Ok(())
}

fn validate_policy_params(params: PolicyParams) -> Result<()> {
    require!(
        params.policy_epoch_duration > 0,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.normal_band_bps <= BPS as u16,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.stressed_band_bps <= BPS as u16,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.normal_band_bps <= params.stressed_band_bps,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.anchor_ema_bps <= BPS as u16,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.max_anchor_crawl_bps <= BPS as u16,
        AgcError::InvalidPolicyParams
    );
    require!(params.min_premium_bps > 0, AgcError::InvalidPolicyParams);
    require!(
        params.target_gross_buy_bps > 0,
        AgcError::InvalidPolicyParams
    );
    require!(params.target_net_buy_bps > 0, AgcError::InvalidPolicyParams);
    require!(
        params.target_lock_flow_bps > 0,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.target_buy_growth_bps > 0,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.target_locked_share_bps > 0,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.expansion_reserve_coverage_bps <= params.target_reserve_coverage_bps,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.hard_defense_reserve_coverage_bps <= params.defense_reserve_coverage_bps,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.defense_reserve_coverage_bps <= params.neutral_reserve_coverage_bps,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.neutral_reserve_coverage_bps <= params.expansion_reserve_coverage_bps,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.max_expansion_volatility_bps > 0,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.defense_volatility_bps > params.max_expansion_volatility_bps,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.max_expansion_exit_pressure_bps > 0,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.max_expansion_exit_pressure_bps <= BPS as u16,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.defense_exit_pressure_bps <= BPS as u16,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.defense_exit_pressure_bps > params.max_expansion_exit_pressure_bps,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.expansion_kappa_bps <= BPS as u16,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.max_mint_per_epoch_bps <= BPS as u16,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.max_mint_per_day_bps <= BPS as u16,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.buyback_kappa_bps <= BPS as u16,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.mild_defense_spend_bps <= params.severe_defense_spend_bps,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.mild_defense_spend_bps <= BPS as u16,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.severe_defense_spend_bps <= BPS as u16,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.severe_stress_threshold_bps > 0 && params.severe_stress_threshold_bps <= BPS as u16,
        AgcError::InvalidPolicyParams
    );
    Ok(())
}

fn assert_market_reporter_or_admin(
    state: &ProtocolState,
    authority_key: Pubkey,
    keeper_info: AccountInfo,
) -> Result<()> {
    if authority_key == state.admin {
        return Ok(());
    }
    if state.market_adapter_authority != Pubkey::default()
        && authority_key == state.market_adapter_authority
    {
        return Ok(());
    }
    assert_keeper_permission_or_admin(
        state,
        authority_key,
        keeper_info,
        RequiredKeeperPermission::ReportMarket,
    )
}

fn assert_keeper_permission_or_admin(
    state: &ProtocolState,
    authority_key: Pubkey,
    keeper_info: AccountInfo,
    required: RequiredKeeperPermission,
) -> Result<()> {
    if authority_key == state.admin {
        return Ok(());
    }

    require_keys_eq!(*keeper_info.owner, crate::ID, AgcError::Unauthorized);
    let data = keeper_info.try_borrow_data()?;
    let mut data_slice: &[u8] = &data;
    let keeper = Keeper::try_deserialize(&mut data_slice)?;
    require!(keeper.permissions.allows(required), AgcError::Unauthorized);
    require_keys_eq!(keeper.authority, authority_key, AgcError::Unauthorized);
    Ok(())
}

fn current_timestamp() -> Result<u64> {
    let timestamp = Clock::get()?.unix_timestamp;
    require!(timestamp >= 0, AgcError::InvalidClock);
    Ok(timestamp as u64)
}

fn pow10_u64(exp: u8) -> Result<u64> {
    let mut value = 1_u64;
    for _ in 0..exp {
        value = value.checked_mul(10).ok_or(AgcError::MathOverflow)?;
    }
    Ok(value)
}

fn pow10_u128(exp: u8) -> Result<u128> {
    let mut value = 1_u128;
    for _ in 0..exp {
        value = value.checked_mul(10).ok_or(AgcError::MathOverflow)?;
    }
    Ok(value)
}

fn checked_mul_u128(lhs: u128, rhs: u128) -> Result<u128> {
    lhs.checked_mul(rhs).ok_or(error!(AgcError::MathOverflow))
}

fn checked_add_u128(lhs: u128, rhs: u128, err: AgcError) -> Result<u128> {
    lhs.checked_add(rhs).ok_or(error!(err))
}

fn checked_div_u128(lhs: u128, rhs: u128) -> Result<u128> {
    require!(rhs > 0, AgcError::MathOverflow);
    Ok(lhs / rhs)
}

fn mul_div(lhs: u128, rhs: u128, denominator: u128) -> Result<u128> {
    if denominator == 0 {
        return Ok(0);
    }
    checked_div_u128(checked_mul_u128(lhs, rhs)?, denominator)
}

fn safe_div(numerator: u128, denominator: u128) -> Result<u128> {
    if denominator == 0 {
        return Ok(0);
    }
    Ok(numerator / denominator)
}

fn positive_delta(lhs: u128, rhs: u128) -> u128 {
    lhs.saturating_sub(rhs)
}

fn min_u128(lhs: u128, rhs: u128) -> u128 {
    lhs.min(rhs)
}

fn max_u128(lhs: u128, rhs: u128) -> u128 {
    lhs.max(rhs)
}

fn quote_to_x18(state: &ProtocolState, raw_usdc: u64) -> Result<u128> {
    checked_mul_u128(raw_usdc as u128, state.quote_scale)
}

fn quote_from_x18(state: &ProtocolState, quote_x18: u128) -> Result<u64> {
    u64::try_from(quote_x18 / state.quote_scale).map_err(|_| error!(AgcError::AmountTooLarge))
}

fn circulating_float(total_supply: u64, treasury_agc: u64, xagc_assets: u64) -> u64 {
    total_supply.saturating_sub(treasury_agc.saturating_add(xagc_assets))
}

fn accounted_assets(total_assets: u64, unaccounted_assets: u64) -> u64 {
    total_assets.saturating_sub(unaccounted_assets)
}

fn convert_to_shares(
    assets: u64,
    share_supply: u64,
    total_assets: u64,
    unaccounted_assets: u64,
) -> Result<u64> {
    if share_supply == 0 {
        return Ok(assets);
    }

    let assets_before = accounted_assets(total_assets, unaccounted_assets);
    require!(assets_before > 0, AgcError::ZeroAmount);
    u64::try_from(mul_div(
        assets as u128,
        share_supply as u128,
        assets_before as u128,
    )?)
    .map_err(|_| error!(AgcError::AmountTooLarge))
}

fn convert_to_assets(
    shares: u64,
    share_supply: u64,
    total_assets: u64,
    unaccounted_assets: u64,
) -> Result<u64> {
    if share_supply == 0 {
        return Ok(shares);
    }

    let assets_before = accounted_assets(total_assets, unaccounted_assets);
    require!(assets_before > 0, AgcError::ZeroAmount);
    u64::try_from(mul_div(
        shares as u128,
        assets_before as u128,
        share_supply as u128,
    )?)
    .map_err(|_| error!(AgcError::AmountTooLarge))
}

fn observe_mid_price(
    state: &mut ProtocolState,
    current_mid_price_x18: u128,
    now: u64,
) -> Result<()> {
    let acc = &mut state.accumulator;
    if acc.last_observed_at == 0 {
        acc.updated_at = now;
        acc.last_observed_at = now;
        acc.last_mid_price_x18 = current_mid_price_x18;
        if acc.observation_count == 0 {
            acc.observation_count = 1;
        }
        return Ok(());
    }

    if now > acc.last_observed_at && acc.last_mid_price_x18 > 0 {
        let elapsed = now - acc.last_observed_at;
        acc.cumulative_mid_price_time_x18 = checked_add_u128(
            acc.cumulative_mid_price_time_x18,
            checked_mul_u128(acc.last_mid_price_x18, elapsed as u128)?,
            AgcError::MathOverflow,
        )?;
        let price_change_bps = if current_mid_price_x18 > acc.last_mid_price_x18 {
            checked_div_u128(
                checked_mul_u128(current_mid_price_x18 - acc.last_mid_price_x18, BPS)?,
                acc.last_mid_price_x18,
            )?
        } else {
            checked_div_u128(
                checked_mul_u128(acc.last_mid_price_x18 - current_mid_price_x18, BPS)?,
                acc.last_mid_price_x18,
            )?
        };
        acc.cumulative_abs_mid_price_change_bps = checked_add_u128(
            acc.cumulative_abs_mid_price_change_bps,
            price_change_bps,
            AgcError::MathOverflow,
        )?;
        acc.observation_count = acc
            .observation_count
            .checked_add(1)
            .ok_or(AgcError::MathOverflow)?;
        acc.last_observed_at = now;
    }

    acc.updated_at = now;
    acc.last_mid_price_x18 = current_mid_price_x18;
    Ok(())
}

fn volatility_bps(acc: &EpochAccumulator) -> u128 {
    if acc.observation_count <= 1 {
        return 0;
    }
    acc.cumulative_abs_mid_price_change_bps / (acc.observation_count - 1) as u128
}

fn preview_epoch_snapshot(acc: &EpochAccumulator, now: u64) -> Result<EpochSnapshot> {
    let mut cumulative_mid_price_time_x18 = acc.cumulative_mid_price_time_x18;
    if now > acc.last_observed_at && acc.last_mid_price_x18 > 0 {
        cumulative_mid_price_time_x18 = checked_add_u128(
            cumulative_mid_price_time_x18,
            checked_mul_u128(acc.last_mid_price_x18, (now - acc.last_observed_at) as u128)?,
            AgcError::MathOverflow,
        )?;
    }

    let epoch_elapsed = now.saturating_sub(acc.started_at);
    let short_twap_price_x18 = if epoch_elapsed == 0 {
        acc.last_mid_price_x18
    } else if cumulative_mid_price_time_x18 == 0 && acc.observation_count == 0 {
        acc.last_mid_price_x18
    } else {
        cumulative_mid_price_time_x18 / epoch_elapsed as u128
    };

    Ok(EpochSnapshot {
        epoch_id: acc.epoch_id,
        started_at: acc.started_at,
        ended_at: now,
        gross_buy_volume_quote_x18: acc.gross_buy_volume_quote_x18,
        gross_sell_volume_quote_x18: acc.gross_sell_volume_quote_x18,
        total_volume_quote_x18: acc.total_volume_quote_x18,
        short_twap_price_x18,
        realized_volatility_bps: volatility_bps(acc),
        total_hook_fees_quote_x18: acc.total_hook_fees_quote_x18,
        total_hook_fees_agc: acc.total_hook_fees_agc,
    })
}

fn compute_anchor_next(
    anchor_price_x18: u128,
    price_twap_x18: u128,
    anchor_ema_bps: u16,
    max_anchor_crawl_bps: u16,
) -> Result<u128> {
    let ema = checked_div_u128(
        checked_add_u128(
            checked_mul_u128(anchor_price_x18, BPS - anchor_ema_bps as u128)?,
            checked_mul_u128(price_twap_x18, anchor_ema_bps as u128)?,
            AgcError::MathOverflow,
        )?,
        BPS,
    )?;
    let min_anchor = checked_div_u128(
        checked_mul_u128(anchor_price_x18, BPS - max_anchor_crawl_bps as u128)?,
        BPS,
    )?;
    let max_anchor = checked_div_u128(
        checked_mul_u128(anchor_price_x18, BPS + max_anchor_crawl_bps as u128)?,
        BPS,
    )?;

    Ok(ema.clamp(min_anchor, max_anchor))
}

fn evaluate_epoch(
    snapshot: EpochSnapshot,
    external_metrics: ExternalMetrics,
    state: PolicyState,
    flows: VaultFlows,
    policy_params: PolicyParams,
    agc_unit: u128,
) -> Result<EpochResult> {
    let price_twap_x18 = snapshot.short_twap_price_x18;
    let gross_buy_quote_x18 = snapshot.gross_buy_volume_quote_x18;
    let gross_sell_quote_x18 = snapshot.gross_sell_volume_quote_x18;
    let total_volume_quote_x18 = snapshot.total_volume_quote_x18;
    let xagc_net_deposits_acp =
        flows.xagc_deposits_acp as i128 - flows.xagc_gross_redemptions_acp as i128;

    let credit_outstanding_quote_x18 =
        mul_div(state.float_supply_acp, state.anchor_price_x18, agc_unit)?;
    let gross_buy_floor_bps = safe_div(
        checked_mul_u128(gross_buy_quote_x18, BPS)?,
        credit_outstanding_quote_x18,
    )?;
    let net_buy_quote_x18 = gross_buy_quote_x18.saturating_sub(gross_sell_quote_x18);
    let net_buy_pressure_bps = safe_div(
        checked_mul_u128(net_buy_quote_x18, BPS)?,
        credit_outstanding_quote_x18,
    )?;
    let buy_growth_bps = if state.last_gross_buy_quote_x18 == 0 {
        0
    } else {
        safe_div(
            checked_mul_u128(
                positive_delta(gross_buy_quote_x18, state.last_gross_buy_quote_x18),
                BPS,
            )?,
            state.last_gross_buy_quote_x18,
        )?
    };
    let exit_pressure_bps = safe_div(
        checked_mul_u128(gross_sell_quote_x18, BPS)?,
        total_volume_quote_x18,
    )?;
    let reserve_coverage_bps = safe_div(
        checked_mul_u128(external_metrics.depth_to_target_slippage_quote_x18, BPS)?,
        credit_outstanding_quote_x18,
    )?;
    let locked_share_bps = safe_div(
        checked_mul_u128(state.xagc_total_assets_acp, BPS)?,
        state.float_supply_acp,
    )?;
    let lock_flow_bps = if xagc_net_deposits_acp <= 0 {
        0
    } else {
        safe_div(
            checked_mul_u128(xagc_net_deposits_acp as u128, BPS)?,
            state.float_supply_acp,
        )?
    };
    let premium_bps = if price_twap_x18 > state.anchor_price_x18 && state.anchor_price_x18 > 0 {
        checked_div_u128(
            checked_mul_u128(price_twap_x18 - state.anchor_price_x18, BPS)?,
            state.anchor_price_x18,
        )?
    } else {
        0
    };
    let premium_persistence_epochs = if premium_bps >= policy_params.min_premium_bps as u128 {
        state
            .premium_persistence_epochs
            .checked_add(1)
            .ok_or(AgcError::MathOverflow)?
    } else {
        0
    };

    let normal_floor_x18 = checked_div_u128(
        checked_mul_u128(
            state.anchor_price_x18,
            BPS - policy_params.normal_band_bps as u128,
        )?,
        BPS,
    )?;
    let stressed_floor_x18 = checked_div_u128(
        checked_mul_u128(
            state.anchor_price_x18,
            BPS - policy_params.stressed_band_bps as u128,
        )?,
        BPS,
    )?;
    let anchor_next_x18 = compute_anchor_next(
        state.anchor_price_x18,
        price_twap_x18,
        policy_params.anchor_ema_bps,
        policy_params.max_anchor_crawl_bps,
    )?;

    let in_defense = price_twap_x18 < stressed_floor_x18
        || reserve_coverage_bps < policy_params.defense_reserve_coverage_bps as u128
        || snapshot.realized_volatility_bps >= policy_params.defense_volatility_bps as u128
        || exit_pressure_bps >= policy_params.defense_exit_pressure_bps as u128;

    let can_expand = premium_bps >= policy_params.min_premium_bps as u128
        && premium_persistence_epochs >= policy_params.premium_persistence_required as u128
        && gross_buy_floor_bps >= policy_params.min_gross_buy_floor_bps as u128
        && net_buy_pressure_bps > 0
        && lock_flow_bps > 0
        && locked_share_bps >= policy_params.min_locked_share_bps as u128
        && reserve_coverage_bps >= policy_params.expansion_reserve_coverage_bps as u128
        && snapshot.realized_volatility_bps <= policy_params.max_expansion_volatility_bps as u128
        && exit_pressure_bps <= policy_params.max_expansion_exit_pressure_bps as u128
        && buy_growth_bps > 0;

    let in_recovery = !in_defense
        && state.recovery_cooldown_epochs_remaining > 0
        && (state.last_regime == Regime::Defense || state.last_regime == Regime::Recovery);

    let next_regime = if in_defense {
        Regime::Defense
    } else if in_recovery {
        Regime::Recovery
    } else if can_expand {
        Regime::Expansion
    } else {
        Regime::Neutral
    };

    let mut demand_score_bps = 0;
    let mut health_score_bps = 0;
    let mut mint_rate_bps = 0;
    let mut mint_budget_acp = 0;

    if next_regime == Regime::Expansion {
        let premium_score_bps = min_u128(
            safe_div(
                checked_mul_u128(
                    positive_delta(premium_bps, policy_params.min_premium_bps as u128),
                    BPS,
                )?,
                policy_params.min_premium_bps as u128,
            )?,
            BPS,
        );
        let buy_score_bps = min_u128(
            safe_div(
                checked_mul_u128(gross_buy_floor_bps, BPS)?,
                policy_params.target_gross_buy_bps as u128,
            )?,
            BPS,
        );
        let net_buy_score_bps = min_u128(
            safe_div(
                checked_mul_u128(net_buy_pressure_bps, BPS)?,
                policy_params.target_net_buy_bps as u128,
            )?,
            BPS,
        );
        let lock_flow_score_bps = min_u128(
            safe_div(
                checked_mul_u128(lock_flow_bps, BPS)?,
                policy_params.target_lock_flow_bps as u128,
            )?,
            BPS,
        );
        let buy_growth_score_bps = min_u128(
            safe_div(
                checked_mul_u128(buy_growth_bps, BPS)?,
                policy_params.target_buy_growth_bps as u128,
            )?,
            BPS,
        );

        demand_score_bps = min_u128(
            premium_score_bps,
            min_u128(
                buy_score_bps,
                min_u128(
                    net_buy_score_bps,
                    min_u128(lock_flow_score_bps, buy_growth_score_bps),
                ),
            ),
        );

        let reserve_health_bps = if reserve_coverage_bps
            <= policy_params.expansion_reserve_coverage_bps as u128
        {
            0
        } else {
            min_u128(
                safe_div(
                    checked_mul_u128(
                        reserve_coverage_bps - policy_params.expansion_reserve_coverage_bps as u128,
                        BPS,
                    )?,
                    (policy_params.target_reserve_coverage_bps
                        - policy_params.expansion_reserve_coverage_bps) as u128,
                )?,
                BPS,
            )
        };
        let volatility_health_bps = if snapshot.realized_volatility_bps
            >= policy_params.max_expansion_volatility_bps as u128
        {
            0
        } else {
            checked_div_u128(
                checked_mul_u128(
                    policy_params.max_expansion_volatility_bps as u128
                        - snapshot.realized_volatility_bps,
                    BPS,
                )?,
                policy_params.max_expansion_volatility_bps as u128,
            )?
        };
        let exit_health_bps =
            if exit_pressure_bps >= policy_params.max_expansion_exit_pressure_bps as u128 {
                0
            } else {
                checked_div_u128(
                    checked_mul_u128(
                        policy_params.max_expansion_exit_pressure_bps as u128 - exit_pressure_bps,
                        BPS,
                    )?,
                    policy_params.max_expansion_exit_pressure_bps as u128,
                )?
            };
        let locked_share_health_bps = min_u128(
            safe_div(
                checked_mul_u128(locked_share_bps, BPS)?,
                policy_params.target_locked_share_bps as u128,
            )?,
            BPS,
        );

        health_score_bps = min_u128(
            reserve_health_bps,
            min_u128(
                volatility_health_bps,
                min_u128(exit_health_bps, locked_share_health_bps),
            ),
        );

        let raw_mint_rate_bps = checked_div_u128(
            checked_mul_u128(
                checked_div_u128(
                    checked_mul_u128(policy_params.expansion_kappa_bps as u128, demand_score_bps)?,
                    BPS,
                )?,
                health_score_bps,
            )?,
            BPS,
        )?;
        mint_rate_bps = min_u128(
            raw_mint_rate_bps,
            policy_params.max_mint_per_epoch_bps as u128,
        );

        let remaining_daily_mint_acp = positive_delta(
            checked_div_u128(
                checked_mul_u128(
                    state.float_supply_acp,
                    policy_params.max_mint_per_day_bps as u128,
                )?,
                BPS,
            )?,
            state.minted_today_acp,
        );
        mint_budget_acp = min_u128(
            checked_div_u128(
                checked_mul_u128(state.float_supply_acp, mint_rate_bps)?,
                BPS,
            )?,
            remaining_daily_mint_acp,
        );
    }

    let price_stress_bps = if price_twap_x18 < stressed_floor_x18 && state.anchor_price_x18 > 0 {
        checked_div_u128(
            checked_mul_u128(stressed_floor_x18 - price_twap_x18, BPS)?,
            state.anchor_price_x18,
        )?
    } else {
        0
    };
    let coverage_stress_bps = positive_delta(
        policy_params.defense_reserve_coverage_bps as u128,
        reserve_coverage_bps,
    );
    let exit_stress_bps = positive_delta(
        exit_pressure_bps,
        policy_params.defense_exit_pressure_bps as u128,
    );
    let volatility_stress_bps = positive_delta(
        snapshot.realized_volatility_bps,
        policy_params.defense_volatility_bps as u128,
    );
    let mut stress_score_bps = max_u128(
        price_stress_bps,
        max_u128(
            coverage_stress_bps,
            max_u128(exit_stress_bps, volatility_stress_bps),
        ),
    );
    if reserve_coverage_bps < policy_params.hard_defense_reserve_coverage_bps as u128 {
        stress_score_bps = max_u128(
            stress_score_bps,
            policy_params.severe_stress_threshold_bps as u128,
        );
    }

    let mut buyback_budget_quote_x18 = 0;
    if next_regime == Regime::Defense {
        let buyback_cap_bps =
            if stress_score_bps >= policy_params.severe_stress_threshold_bps as u128 {
                policy_params.severe_defense_spend_bps
            } else {
                policy_params.mild_defense_spend_bps
            };
        let buyback_spend_rate_bps = min_u128(
            checked_div_u128(
                checked_mul_u128(policy_params.buyback_kappa_bps as u128, stress_score_bps)?,
                BPS,
            )?,
            buyback_cap_bps as u128,
        );
        buyback_budget_quote_x18 = checked_div_u128(
            checked_mul_u128(state.treasury_quote_x18, buyback_spend_rate_bps)?,
            BPS,
        )?;
    }

    Ok(EpochResult {
        epoch_id: snapshot.epoch_id,
        regime: next_regime,
        anchor_price_x18: state.anchor_price_x18,
        anchor_next_x18,
        normal_floor_x18,
        stressed_floor_x18,
        price_twap_x18,
        premium_bps,
        premium_persistence_epochs,
        credit_outstanding_quote_x18,
        gross_buy_floor_bps,
        net_buy_pressure_bps,
        buy_growth_bps,
        exit_pressure_bps,
        reserve_coverage_bps,
        locked_share_bps,
        lock_flow_bps,
        demand_score_bps,
        health_score_bps,
        mint_rate_bps,
        mint_budget_acp,
        buyback_budget_quote_x18,
        stress_score_bps,
        gross_buy_quote_x18,
        gross_sell_quote_x18,
        total_volume_quote_x18,
        depth_to_target_slippage_quote_x18: external_metrics.depth_to_target_slippage_quote_x18,
        realized_volatility_bps: snapshot.realized_volatility_bps,
        xagc_deposits_acp: flows.xagc_deposits_acp,
        xagc_gross_redemptions_acp: flows.xagc_gross_redemptions_acp,
        treasury_quote_x18: state.treasury_quote_x18,
        treasury_acp: state.treasury_acp,
        xagc_total_assets_acp: state.xagc_total_assets_acp,
        mint_allocations: MintAllocation::default(),
    })
}

fn allocate_mint(mint_budget_acp: u128, distribution: MintDistribution) -> MintAllocation {
    let xagc_mint_acp = mint_budget_acp * distribution.xagc_bps as u128 / BPS;
    let growth_programs_mint_acp = mint_budget_acp * distribution.growth_programs_bps as u128 / BPS;
    let lp_mint_acp = mint_budget_acp * distribution.lp_bps as u128 / BPS;
    let integrators_mint_acp = mint_budget_acp * distribution.integrators_bps as u128 / BPS;
    let treasury_mint_acp = mint_budget_acp
        .saturating_sub(xagc_mint_acp)
        .saturating_sub(growth_programs_mint_acp)
        .saturating_sub(lp_mint_acp)
        .saturating_sub(integrators_mint_acp);

    MintAllocation {
        xagc_mint_acp,
        growth_programs_mint_acp,
        lp_mint_acp,
        integrators_mint_acp,
        treasury_mint_acp,
    }
}

fn validate_settlement_window(state: &ProtocolState, now: u64) -> Result<()> {
    let next_allowed = state
        .accumulator
        .started_at
        .checked_add(state.policy_params.policy_epoch_duration)
        .ok_or(AgcError::MathOverflow)?;
    require!(now >= next_allowed, AgcError::EpochTooSoon);
    Ok(())
}

fn refresh_mint_window(state: &mut ProtocolState, now: u64) {
    let current_day = now / SECONDS_PER_DAY;
    if state.mint_window_day != current_day {
        state.mint_window_day = current_day;
        state.minted_in_current_day = 0;
    }
}

fn persist_epoch_settlement(
    state: &mut ProtocolState,
    snapshot: EpochSnapshot,
    result: EpochResult,
    raw_buyback_budget: u64,
    now: u64,
) -> Result<()> {
    let next_epoch_id = snapshot
        .epoch_id
        .checked_add(1)
        .ok_or(AgcError::MathOverflow)?;
    state.anchor_price_x18 = result.anchor_next_x18;
    state.premium_persistence_epochs = result.premium_persistence_epochs;
    state.last_gross_buy_quote_x18 = result.gross_buy_quote_x18;
    state.regime = result.regime;

    state.recovery_cooldown_epochs_remaining = match result.regime {
        Regime::Defense => state.policy_params.recovery_cooldown_epochs as u64,
        Regime::Recovery => state.recovery_cooldown_epochs_remaining.saturating_sub(1),
        _ => 0,
    };

    if raw_buyback_budget > 0 {
        state.pending_treasury_buyback_usdc = state
            .pending_treasury_buyback_usdc
            .checked_add(raw_buyback_budget)
            .ok_or(AgcError::MathOverflow)?;
    }

    state.last_settled_epoch = snapshot.epoch_id;
    state.last_settlement_timestamp = now;
    state.last_coverage_bps = result.reserve_coverage_bps;
    state.last_exit_pressure_bps = result.exit_pressure_bps;
    state.last_volatility_bps = result.realized_volatility_bps;
    state.last_premium_bps = result.premium_bps;
    state.last_locked_share_bps = result.locked_share_bps;
    state.last_lock_flow_bps = result.lock_flow_bps;
    state.last_xagc_deposit_total = state.xagc_gross_deposits_total;
    state.last_xagc_redemption_total = state.xagc_gross_redemptions_total;
    state.last_epoch_result = result;

    let current_mid_price_x18 = state.accumulator.last_mid_price_x18;
    state.accumulator = EpochAccumulator {
        epoch_id: next_epoch_id,
        started_at: now,
        updated_at: now,
        last_observed_at: now,
        observation_count: if current_mid_price_x18 > 0 { 1 } else { 0 },
        gross_buy_volume_quote_x18: 0,
        gross_sell_volume_quote_x18: 0,
        total_volume_quote_x18: 0,
        last_mid_price_x18: current_mid_price_x18,
        cumulative_mid_price_time_x18: 0,
        cumulative_abs_mid_price_change_bps: 0,
        total_hook_fees_quote_x18: 0,
        total_hook_fees_agc: 0,
    };

    Ok(())
}

fn mint_policy_allocations(ctx: &Context<SettleEpoch>, allocation: MintAllocation) -> Result<()> {
    let state = &ctx.accounts.state;
    mint_amount(
        allocation.xagc_mint_acp,
        &ctx.accounts.agc_mint,
        &ctx.accounts.xagc_vault_agc,
        &ctx.accounts.mint_authority,
        &ctx.accounts.token_program,
        state.mint_authority_bump,
    )?;
    mint_amount(
        allocation.growth_programs_mint_acp,
        &ctx.accounts.agc_mint,
        &ctx.accounts.growth_programs_agc,
        &ctx.accounts.mint_authority,
        &ctx.accounts.token_program,
        state.mint_authority_bump,
    )?;
    mint_amount(
        allocation.lp_mint_acp,
        &ctx.accounts.agc_mint,
        &ctx.accounts.lp_agc,
        &ctx.accounts.mint_authority,
        &ctx.accounts.token_program,
        state.mint_authority_bump,
    )?;
    mint_amount(
        allocation.integrators_mint_acp,
        &ctx.accounts.agc_mint,
        &ctx.accounts.integrators_agc,
        &ctx.accounts.mint_authority,
        &ctx.accounts.token_program,
        state.mint_authority_bump,
    )?;
    mint_amount(
        allocation.treasury_mint_acp,
        &ctx.accounts.agc_mint,
        &ctx.accounts.treasury_agc,
        &ctx.accounts.mint_authority,
        &ctx.accounts.token_program,
        state.mint_authority_bump,
    )
}

fn mint_amount<'info>(
    amount: u128,
    mint: &Account<'info, Mint>,
    destination: &Account<'info, TokenAccount>,
    authority: &UncheckedAccount<'info>,
    token_program: &Program<'info, Token>,
    bump: u8,
) -> Result<()> {
    if amount == 0 {
        return Ok(());
    }
    let amount_u64 = u64::try_from(amount).map_err(|_| error!(AgcError::AmountTooLarge))?;
    mint_with_pda(
        mint,
        destination,
        authority,
        token_program,
        bump,
        amount_u64,
    )
}

fn mint_with_pda<'info>(
    mint: &Account<'info, Mint>,
    destination: &Account<'info, TokenAccount>,
    authority: &UncheckedAccount<'info>,
    token_program: &Program<'info, Token>,
    bump: u8,
    amount: u64,
) -> Result<()> {
    let signer: &[&[&[u8]]] = &[&[MINT_AUTHORITY_SEED, &[bump]]];
    token::mint_to(
        CpiContext::new_with_signer(
            token_program.key(),
            MintTo {
                mint: mint.to_account_info(),
                to: destination.to_account_info(),
                authority: authority.to_account_info(),
            },
            signer,
        ),
        amount,
    )
}

fn transfer_from_xagc_vault<'info>(
    source: &Account<'info, TokenAccount>,
    destination: &Account<'info, TokenAccount>,
    authority: &UncheckedAccount<'info>,
    token_program: &Program<'info, Token>,
    bump: u8,
    amount: u64,
) -> Result<()> {
    let signer: &[&[&[u8]]] = &[&[XAGC_AUTHORITY_SEED, &[bump]]];
    token::transfer(
        CpiContext::new_with_signer(
            token_program.key(),
            Transfer {
                from: source.to_account_info(),
                to: destination.to_account_info(),
                authority: authority.to_account_info(),
            },
            signer,
        ),
        amount,
    )
}

fn transfer_from_treasury<'info>(
    source: &Account<'info, TokenAccount>,
    destination: &Account<'info, TokenAccount>,
    authority: &UncheckedAccount<'info>,
    token_program: &Program<'info, Token>,
    bump: u8,
    amount: u64,
) -> Result<()> {
    let signer: &[&[&[u8]]] = &[&[TREASURY_AUTHORITY_SEED, &[bump]]];
    token::transfer(
        CpiContext::new_with_signer(
            token_program.key(),
            Transfer {
                from: source.to_account_info(),
                to: destination.to_account_info(),
                authority: authority.to_account_info(),
            },
            signer,
        ),
        amount,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const PRICE_SCALE: u128 = 1_000_000_000_000_000_000;

    fn distribution() -> MintDistribution {
        MintDistribution {
            xagc_bps: 3_000,
            growth_programs_bps: 2_000,
            lp_bps: 2_000,
            integrators_bps: 1_000,
            treasury_bps: 2_000,
        }
    }

    fn params() -> PolicyParams {
        PolicyParams {
            normal_band_bps: 300,
            stressed_band_bps: 700,
            anchor_ema_bps: 500,
            max_anchor_crawl_bps: 100,
            min_premium_bps: 100,
            premium_persistence_required: 2,
            min_gross_buy_floor_bps: 50,
            min_locked_share_bps: 1_000,
            target_gross_buy_bps: 500,
            target_net_buy_bps: 250,
            target_lock_flow_bps: 100,
            target_buy_growth_bps: 500,
            target_locked_share_bps: 3_000,
            expansion_reserve_coverage_bps: 3_000,
            target_reserve_coverage_bps: 8_000,
            neutral_reserve_coverage_bps: 2_000,
            defense_reserve_coverage_bps: 1_500,
            hard_defense_reserve_coverage_bps: 800,
            max_expansion_volatility_bps: 300,
            defense_volatility_bps: 1_000,
            max_expansion_exit_pressure_bps: 3_000,
            defense_exit_pressure_bps: 7_000,
            expansion_kappa_bps: 1_000,
            max_mint_per_epoch_bps: 100,
            max_mint_per_day_bps: 250,
            buyback_kappa_bps: 5_000,
            mild_defense_spend_bps: 500,
            severe_defense_spend_bps: 1_500,
            severe_stress_threshold_bps: 1_000,
            recovery_cooldown_epochs: 2,
            policy_epoch_duration: 3_600,
        }
    }

    fn test_state() -> ProtocolState {
        ProtocolState {
            admin: Pubkey::default(),
            pending_admin: Pubkey::default(),
            agc_mint: Pubkey::default(),
            xagc_mint: Pubkey::default(),
            usdc_mint: Pubkey::default(),
            treasury_agc: Pubkey::default(),
            treasury_usdc: Pubkey::default(),
            xagc_vault_agc: Pubkey::default(),
            growth_programs_agc: Pubkey::default(),
            lp_agc: Pubkey::default(),
            integrators_agc: Pubkey::default(),
            buyback_usdc_escrow: Pubkey::default(),
            market_adapter_authority: Pubkey::default(),
            state_bump: 0,
            mint_authority_bump: 0,
            treasury_authority_bump: 0,
            xagc_authority_bump: 0,
            treasury_agc_bump: 0,
            treasury_usdc_bump: 0,
            xagc_vault_agc_bump: 0,
            agc_decimals: 9,
            xagc_decimals: 9,
            usdc_decimals: 6,
            agc_unit: 1_000_000_000,
            quote_scale: 1_000_000_000_000,
            exit_fee_bps: 100,
            growth_programs_enabled: true,
            pause_flags: PauseFlags::default(),
            policy_params: params(),
            mint_distribution: distribution(),
            regime: Regime::Neutral,
            anchor_price_x18: PRICE_SCALE,
            premium_persistence_epochs: 0,
            last_gross_buy_quote_x18: 0,
            last_coverage_bps: 0,
            last_exit_pressure_bps: 0,
            last_volatility_bps: 0,
            last_premium_bps: 0,
            last_locked_share_bps: 0,
            last_lock_flow_bps: 0,
            last_settled_epoch: 0,
            last_settlement_timestamp: 0,
            recovery_cooldown_epochs_remaining: 0,
            mint_window_day: 0,
            minted_in_current_day: 0,
            pending_treasury_buyback_usdc: 0,
            xagc_gross_deposits_total: 0,
            xagc_gross_redemptions_total: 0,
            xagc_unaccounted_assets: 0,
            last_xagc_deposit_total: 0,
            last_xagc_redemption_total: 0,
            buyback_execution_nonce: 0,
            accumulator: EpochAccumulator {
                epoch_id: 1,
                started_at: 1_000,
                updated_at: 1_000,
                last_observed_at: 1_000,
                observation_count: 1,
                gross_buy_volume_quote_x18: 0,
                gross_sell_volume_quote_x18: 0,
                total_volume_quote_x18: 0,
                last_mid_price_x18: PRICE_SCALE,
                cumulative_mid_price_time_x18: 0,
                cumulative_abs_mid_price_change_bps: 0,
                total_hook_fees_quote_x18: 0,
                total_hook_fees_agc: 0,
            },
            last_epoch_result: EpochResult::default(),
        }
    }

    #[test]
    fn anchor_crawl_is_clamped() {
        let next = compute_anchor_next(PRICE_SCALE, PRICE_SCALE * 2, 5_000, 100).unwrap();
        assert_eq!(next, PRICE_SCALE * 101 / 100);

        let next_down = compute_anchor_next(PRICE_SCALE, PRICE_SCALE / 2, 5_000, 100).unwrap();
        assert_eq!(next_down, PRICE_SCALE * 99 / 100);
    }

    #[test]
    fn expansion_epoch_mints_budget() {
        let snapshot = EpochSnapshot {
            epoch_id: 1,
            started_at: 0,
            ended_at: 3_600,
            gross_buy_volume_quote_x18: 100_000 * PRICE_SCALE,
            gross_sell_volume_quote_x18: 10_000 * PRICE_SCALE,
            total_volume_quote_x18: 110_000 * PRICE_SCALE,
            short_twap_price_x18: PRICE_SCALE * 104 / 100,
            realized_volatility_bps: 50,
            total_hook_fees_quote_x18: 0,
            total_hook_fees_agc: 0,
        };
        let state = PolicyState {
            anchor_price_x18: PRICE_SCALE,
            premium_persistence_epochs: 1,
            last_gross_buy_quote_x18: 50_000 * PRICE_SCALE,
            minted_today_acp: 0,
            last_regime: Regime::Neutral,
            recovery_cooldown_epochs_remaining: 0,
            float_supply_acp: 1_000_000_000_000_000,
            treasury_quote_x18: 200_000 * PRICE_SCALE,
            treasury_acp: 0,
            xagc_total_assets_acp: 250_000_000_000_000,
        };
        let flows = VaultFlows {
            xagc_deposits_acp: 20_000_000_000_000,
            xagc_gross_redemptions_acp: 0,
        };

        let result = evaluate_epoch(
            snapshot,
            ExternalMetrics {
                depth_to_target_slippage_quote_x18: 600_000 * PRICE_SCALE,
            },
            state,
            flows,
            params(),
            1_000_000_000,
        )
        .unwrap();

        assert_eq!(result.regime, Regime::Expansion);
        assert!(result.mint_budget_acp > 0);
    }

    #[test]
    fn defense_epoch_queues_buyback() {
        let snapshot = EpochSnapshot {
            epoch_id: 1,
            started_at: 0,
            ended_at: 3_600,
            gross_buy_volume_quote_x18: 10_000 * PRICE_SCALE,
            gross_sell_volume_quote_x18: 90_000 * PRICE_SCALE,
            total_volume_quote_x18: 100_000 * PRICE_SCALE,
            short_twap_price_x18: PRICE_SCALE * 90 / 100,
            realized_volatility_bps: 50,
            total_hook_fees_quote_x18: 0,
            total_hook_fees_agc: 0,
        };
        let state = PolicyState {
            anchor_price_x18: PRICE_SCALE,
            premium_persistence_epochs: 0,
            last_gross_buy_quote_x18: 20_000 * PRICE_SCALE,
            minted_today_acp: 0,
            last_regime: Regime::Neutral,
            recovery_cooldown_epochs_remaining: 0,
            float_supply_acp: 1_000_000_000_000_000,
            treasury_quote_x18: 200_000 * PRICE_SCALE,
            treasury_acp: 0,
            xagc_total_assets_acp: 250_000_000_000_000,
        };

        let result = evaluate_epoch(
            snapshot,
            ExternalMetrics {
                depth_to_target_slippage_quote_x18: 100_000 * PRICE_SCALE,
            },
            state,
            VaultFlows::default(),
            params(),
            1_000_000_000,
        )
        .unwrap();

        assert_eq!(result.regime, Regime::Defense);
        assert!(result.buyback_budget_quote_x18 > 0);
        assert_eq!(result.mint_budget_acp, 0);
    }

    #[test]
    fn xagc_share_math_tracks_external_mints() {
        let first_shares = convert_to_shares(1_000, 0, 0, 0).unwrap();
        assert_eq!(first_shares, 1_000);

        let second_shares = convert_to_shares(500, 1_000, 2_000, 0).unwrap();
        assert_eq!(second_shares, 250);

        let assets = convert_to_assets(250, 1_250, 2_500, 0).unwrap();
        assert_eq!(assets, 500);
    }

    #[test]
    fn invalid_policy_params_are_rejected() {
        let mut zero_duration = params();
        zero_duration.policy_epoch_duration = 0;
        assert!(validate_policy_params(zero_duration).is_err());

        let mut invalid_band = params();
        invalid_band.normal_band_bps = 4_000;
        invalid_band.stressed_band_bps = 3_000;
        assert!(validate_policy_params(invalid_band).is_err());

        let mut invalid_reserve_targets = params();
        invalid_reserve_targets.expansion_reserve_coverage_bps = 4_000;
        invalid_reserve_targets.target_reserve_coverage_bps = 3_000;
        assert!(validate_policy_params(invalid_reserve_targets).is_err());

        let mut invalid_mint_cap = params();
        invalid_mint_cap.max_mint_per_day_bps = 10_001;
        assert!(validate_policy_params(invalid_mint_cap).is_err());

        let mut invalid_exit_thresholds = params();
        invalid_exit_thresholds.defense_exit_pressure_bps =
            invalid_exit_thresholds.max_expansion_exit_pressure_bps;
        assert!(validate_policy_params(invalid_exit_thresholds).is_err());

        let mut invalid_volatility_thresholds = params();
        invalid_volatility_thresholds.defense_volatility_bps =
            invalid_volatility_thresholds.max_expansion_volatility_bps;
        assert!(validate_policy_params(invalid_volatility_thresholds).is_err());
    }

    #[test]
    fn keeper_permissions_are_role_scoped() {
        let permissions = KeeperPermissions {
            market_reporter: true,
            epoch_settler: false,
            buyback_executor: true,
            treasury_burner: false,
        };

        assert!(permissions.allows(RequiredKeeperPermission::ReportMarket));
        assert!(!permissions.allows(RequiredKeeperPermission::SettleEpoch));
        assert!(permissions.allows(RequiredKeeperPermission::ExecuteBuyback));
        assert!(!permissions.allows(RequiredKeeperPermission::BurnTreasury));

        let all_permissions = KeeperPermissions::all();
        assert!(all_permissions.allows(RequiredKeeperPermission::ReportMarket));
        assert!(all_permissions.allows(RequiredKeeperPermission::SettleEpoch));
        assert!(all_permissions.allows(RequiredKeeperPermission::ExecuteBuyback));
        assert!(all_permissions.allows(RequiredKeeperPermission::BurnTreasury));
    }

    #[test]
    fn initial_epoch_requires_full_duration_before_settlement() {
        let state = test_state();
        assert!(validate_settlement_window(&state, 4_599).is_err());
        assert!(validate_settlement_window(&state, 4_600).is_ok());
    }

    #[test]
    fn refresh_mint_window_resets_across_day_boundary() {
        let mut state = test_state();
        state.mint_window_day = 10;
        state.minted_in_current_day = 123_456;

        refresh_mint_window(&mut state, 10 * SECONDS_PER_DAY + 42);
        assert_eq!(state.mint_window_day, 10);
        assert_eq!(state.minted_in_current_day, 123_456);

        refresh_mint_window(&mut state, 11 * SECONDS_PER_DAY);
        assert_eq!(state.mint_window_day, 11);
        assert_eq!(state.minted_in_current_day, 0);
    }

    #[test]
    fn persist_epoch_settlement_rolls_state_forward() {
        let mut state = test_state();
        state.pending_treasury_buyback_usdc = 75;
        state.xagc_gross_deposits_total = 900;
        state.xagc_gross_redemptions_total = 125;
        state.accumulator.epoch_id = 7;
        state.accumulator.last_mid_price_x18 = PRICE_SCALE * 103 / 100;

        let snapshot = EpochSnapshot {
            epoch_id: 7,
            started_at: 1_000,
            ended_at: 4_600,
            gross_buy_volume_quote_x18: 55_000 * PRICE_SCALE,
            gross_sell_volume_quote_x18: 10_000 * PRICE_SCALE,
            total_volume_quote_x18: 65_000 * PRICE_SCALE,
            short_twap_price_x18: PRICE_SCALE * 102 / 100,
            realized_volatility_bps: 80,
            total_hook_fees_quote_x18: 0,
            total_hook_fees_agc: 0,
        };
        let result = EpochResult {
            epoch_id: 7,
            regime: Regime::Defense,
            anchor_price_x18: PRICE_SCALE,
            anchor_next_x18: PRICE_SCALE * 101 / 100,
            premium_bps: 200,
            premium_persistence_epochs: 3,
            gross_buy_quote_x18: 55_000 * PRICE_SCALE,
            reserve_coverage_bps: 1_200,
            exit_pressure_bps: 1_500,
            realized_volatility_bps: 80,
            locked_share_bps: 2_800,
            lock_flow_bps: 150,
            ..EpochResult::default()
        };

        persist_epoch_settlement(&mut state, snapshot, result, 25, 4_600).unwrap();

        assert_eq!(state.anchor_price_x18, PRICE_SCALE * 101 / 100);
        assert_eq!(state.premium_persistence_epochs, 3);
        assert_eq!(state.last_gross_buy_quote_x18, 55_000 * PRICE_SCALE);
        assert_eq!(state.regime, Regime::Defense);
        assert_eq!(
            state.recovery_cooldown_epochs_remaining,
            params().recovery_cooldown_epochs as u64
        );
        assert_eq!(state.pending_treasury_buyback_usdc, 100);
        assert_eq!(state.last_settled_epoch, 7);
        assert_eq!(state.last_settlement_timestamp, 4_600);
        assert_eq!(state.last_xagc_deposit_total, 900);
        assert_eq!(state.last_xagc_redemption_total, 125);
        assert_eq!(state.accumulator.epoch_id, 8);
        assert_eq!(state.accumulator.started_at, 4_600);
        assert_eq!(
            state.accumulator.last_mid_price_x18,
            PRICE_SCALE * 103 / 100
        );
        assert_eq!(state.accumulator.observation_count, 1);
        assert_eq!(state.accumulator.total_volume_quote_x18, 0);
    }

    #[test]
    fn recovery_cooldown_counts_down_when_stress_clears() {
        let snapshot = EpochSnapshot {
            epoch_id: 2,
            started_at: 3_600,
            ended_at: 7_200,
            gross_buy_volume_quote_x18: 40_000 * PRICE_SCALE,
            gross_sell_volume_quote_x18: 10_000 * PRICE_SCALE,
            total_volume_quote_x18: 50_000 * PRICE_SCALE,
            short_twap_price_x18: PRICE_SCALE,
            realized_volatility_bps: 20,
            total_hook_fees_quote_x18: 0,
            total_hook_fees_agc: 0,
        };
        let state = PolicyState {
            anchor_price_x18: PRICE_SCALE,
            premium_persistence_epochs: 0,
            last_gross_buy_quote_x18: 40_000 * PRICE_SCALE,
            minted_today_acp: 0,
            last_regime: Regime::Defense,
            recovery_cooldown_epochs_remaining: 1,
            float_supply_acp: 1_000_000_000_000_000,
            treasury_quote_x18: 400_000 * PRICE_SCALE,
            treasury_acp: 0,
            xagc_total_assets_acp: 250_000_000_000_000,
        };

        let result = evaluate_epoch(
            snapshot,
            ExternalMetrics {
                depth_to_target_slippage_quote_x18: 500_000 * PRICE_SCALE,
            },
            state,
            VaultFlows::default(),
            params(),
            1_000_000_000,
        )
        .unwrap();

        assert_eq!(result.regime, Regime::Recovery);
        assert_eq!(result.mint_budget_acp, 0);
        assert_eq!(result.buyback_budget_quote_x18, 0);
    }

    #[test]
    fn daily_mint_cap_blocks_additional_expansion_budget() {
        let snapshot = EpochSnapshot {
            epoch_id: 3,
            started_at: 7_200,
            ended_at: 10_800,
            gross_buy_volume_quote_x18: 100_000 * PRICE_SCALE,
            gross_sell_volume_quote_x18: 10_000 * PRICE_SCALE,
            total_volume_quote_x18: 110_000 * PRICE_SCALE,
            short_twap_price_x18: PRICE_SCALE * 104 / 100,
            realized_volatility_bps: 50,
            total_hook_fees_quote_x18: 0,
            total_hook_fees_agc: 0,
        };
        let float_supply_acp = 1_000_000_000_000_000_u128;
        let daily_cap_acp = float_supply_acp * params().max_mint_per_day_bps as u128 / BPS;
        let state = PolicyState {
            anchor_price_x18: PRICE_SCALE,
            premium_persistence_epochs: 1,
            last_gross_buy_quote_x18: 50_000 * PRICE_SCALE,
            minted_today_acp: daily_cap_acp,
            last_regime: Regime::Neutral,
            recovery_cooldown_epochs_remaining: 0,
            float_supply_acp,
            treasury_quote_x18: 200_000 * PRICE_SCALE,
            treasury_acp: 0,
            xagc_total_assets_acp: 250_000_000_000_000,
        };
        let flows = VaultFlows {
            xagc_deposits_acp: 20_000_000_000_000,
            xagc_gross_redemptions_acp: 0,
        };

        let result = evaluate_epoch(
            snapshot,
            ExternalMetrics {
                depth_to_target_slippage_quote_x18: 600_000 * PRICE_SCALE,
            },
            state,
            flows,
            params(),
            1_000_000_000,
        )
        .unwrap();

        assert_eq!(result.regime, Regime::Expansion);
        assert_eq!(result.mint_budget_acp, 0);
    }
}
