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
const COLLATERAL_ASSET_SEED: &[u8] = b"collateral-asset";
const COLLATERAL_ORACLE_SEED: &[u8] = b"collateral-oracle";
const CREDIT_FACILITY_SEED: &[u8] = b"credit-facility";
const CREDIT_FACILITY_AUTHORITY_SEED: &[u8] = b"credit-facility-authority";
const CREDIT_COLLATERAL_VAULT_SEED: &[u8] = b"credit-collateral-vault";
const UNDERWRITER_VAULT_SEED: &[u8] = b"underwriter-vault";
const UNDERWRITER_POSITION_SEED: &[u8] = b"underwriter-position";
const CREDIT_LINE_SEED: &[u8] = b"credit-line";

const BPS: u128 = 10_000;
const SECONDS_PER_DAY: u64 = 86_400;
const SECONDS_PER_YEAR: u128 = 31_536_000;

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
        state.risk_admin = ctx.accounts.admin.key();
        state.emergency_admin = ctx.accounts.admin.key();
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
        state.protocol_version = 2;
        state.credit_facility_count = 0;
        state.credit_principal_outstanding_agc = 0;
        state.credit_drawn_agc = 0;
        state.credit_repaid_agc = 0;
        state.credit_interest_paid_agc = 0;
        state.credit_defaulted_agc = 0;
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
        assert_emergency_authority_or_admin(&ctx.accounts.state, ctx.accounts.authority.key())?;
        ctx.accounts.state.pause_flags = pause_flags;
        emit!(PauseFlagsUpdated { pause_flags });
        Ok(())
    }

    pub fn set_governance_authorities(
        ctx: Context<SetGovernanceAuthorities>,
        authorities: GovernanceAuthorities,
    ) -> Result<()> {
        validate_governance_authorities(authorities)?;
        let state = &mut ctx.accounts.state;
        state.risk_admin = authorities.risk_admin;
        state.emergency_admin = authorities.emergency_admin;
        emit!(GovernanceAuthoritiesUpdated { authorities });
        Ok(())
    }

    pub fn set_policy_params(ctx: Context<SetPolicyParams>, params: PolicyParams) -> Result<()> {
        assert_risk_authority_or_admin(&ctx.accounts.state, ctx.accounts.authority.key())?;
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
        assert_risk_authority_or_admin(&ctx.accounts.state, ctx.accounts.authority.key())?;
        validate_distribution(distribution)?;
        ctx.accounts.state.mint_distribution = distribution;
        emit!(MintDistributionUpdated { distribution });
        Ok(())
    }

    pub fn set_settlement_recipients(
        ctx: Context<SetSettlementRecipients>,
        recipients: SettlementRecipients,
    ) -> Result<()> {
        assert_risk_authority_or_admin(&ctx.accounts.state, ctx.accounts.authority.key())?;
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
        assert_risk_authority_or_admin(&ctx.accounts.state, ctx.accounts.authority.key())?;
        ctx.accounts.state.growth_programs_enabled = enabled;
        emit!(GrowthProgramsEnabledUpdated { enabled });
        Ok(())
    }

    pub fn set_exit_fee_bps(ctx: Context<SetExitFeeBps>, exit_fee_bps: u16) -> Result<()> {
        assert_risk_authority_or_admin(&ctx.accounts.state, ctx.accounts.authority.key())?;
        require!(exit_fee_bps < BPS as u16, AgcError::InvalidFee);
        ctx.accounts.state.exit_fee_bps = exit_fee_bps;
        emit!(ExitFeeUpdated { exit_fee_bps });
        Ok(())
    }

    pub fn set_collateral_asset(
        ctx: Context<SetCollateralAsset>,
        config: CollateralAssetConfig,
    ) -> Result<()> {
        require!(
            !ctx.accounts.state.pause_flags.collateral_updates_paused || !config.enabled,
            AgcError::Paused
        );
        assert_risk_authority_or_admin(&ctx.accounts.state, ctx.accounts.authority.key())?;
        validate_collateral_asset_config(config)?;

        let collateral_asset = &mut ctx.accounts.collateral_asset;
        collateral_asset.mint = ctx.accounts.mint.key();
        collateral_asset.mint_decimals = ctx.accounts.mint.decimals;
        collateral_asset.oracle_feed = config.oracle_feed;
        collateral_asset.reserve_token_account = config.reserve_token_account;
        collateral_asset.asset_class = config.asset_class;
        collateral_asset.reserve_weight_bps = config.reserve_weight_bps;
        collateral_asset.collateral_factor_bps = config.collateral_factor_bps;
        collateral_asset.liquidation_threshold_bps = config.liquidation_threshold_bps;
        collateral_asset.max_concentration_bps = config.max_concentration_bps;
        collateral_asset.max_oracle_staleness_seconds = config.max_oracle_staleness_seconds;
        collateral_asset.max_oracle_confidence_bps = config.max_oracle_confidence_bps;
        collateral_asset.enabled = config.enabled;
        collateral_asset.bump = ctx.bumps.collateral_asset;

        emit!(CollateralAssetUpdated {
            mint: collateral_asset.mint,
            config,
        });

        Ok(())
    }

    pub fn set_collateral_oracle_price(
        ctx: Context<SetCollateralOraclePrice>,
        price: CollateralOraclePriceInput,
    ) -> Result<()> {
        require!(
            !ctx.accounts.state.pause_flags.collateral_updates_paused,
            AgcError::Paused
        );
        assert_oracle_reporter_or_admin(
            &ctx.accounts.state,
            ctx.accounts.authority.key(),
            ctx.accounts.keeper.to_account_info(),
        )?;
        require!(price.price_quote_x18 > 0, AgcError::InvalidPrice);
        require!(
            price.confidence_bps <= ctx.accounts.collateral_asset.max_oracle_confidence_bps,
            AgcError::InvalidOraclePrice
        );

        let oracle = &mut ctx.accounts.collateral_oracle;
        oracle.mint = ctx.accounts.mint.key();
        oracle.oracle_feed = ctx.accounts.collateral_asset.oracle_feed;
        oracle.price_quote_x18 = price.price_quote_x18;
        oracle.confidence_bps = price.confidence_bps;
        oracle.updated_at = current_timestamp()?;
        oracle.bump = ctx.bumps.collateral_oracle;

        emit!(CollateralOraclePriceUpdated {
            mint: oracle.mint,
            price_quote_x18: oracle.price_quote_x18,
            confidence_bps: oracle.confidence_bps,
            updated_at: oracle.updated_at,
        });

        Ok(())
    }

    pub fn initialize_credit_facility(
        ctx: Context<InitializeCreditFacility>,
        facility_id: u64,
        config: CreditFacilityConfig,
    ) -> Result<()> {
        require!(
            !ctx.accounts
                .state
                .pause_flags
                .credit_facility_updates_paused,
            AgcError::Paused
        );
        assert_risk_authority_or_admin(&ctx.accounts.state, ctx.accounts.authority.key())?;
        require!(
            ctx.accounts.collateral_asset.enabled,
            AgcError::CollateralDisabled
        );
        require_keys_eq!(
            ctx.accounts.collateral_asset.mint,
            ctx.accounts.collateral_mint.key(),
            AgcError::InvalidCollateralAssetConfig
        );
        validate_credit_facility_config(config, ctx.accounts.collateral_asset.asset_class)?;

        let facility = &mut ctx.accounts.facility;
        facility.facility_id = facility_id;
        facility.collateral_mint = ctx.accounts.collateral_mint.key();
        facility.collateral_asset = ctx.accounts.collateral_asset.key();
        facility.collateral_vault = ctx.accounts.collateral_vault.key();
        facility.underwriter_vault_agc = ctx.accounts.underwriter_vault_agc.key();
        facility.collateral_decimals = ctx.accounts.collateral_mint.decimals;
        facility.config = config;
        facility.status = CreditFacilityStatus::Active;
        facility.bump = ctx.bumps.facility;
        facility.authority_bump = ctx.bumps.facility_authority;
        facility.collateral_vault_bump = ctx.bumps.collateral_vault;
        facility.underwriter_vault_bump = ctx.bumps.underwriter_vault_agc;
        facility.created_at = current_timestamp()?;

        let state = &mut ctx.accounts.state;
        state.credit_facility_count = state
            .credit_facility_count
            .checked_add(1)
            .ok_or(AgcError::MathOverflow)?;

        emit!(CreditFacilityInitialized {
            facility: facility.key(),
            facility_id,
            collateral_mint: facility.collateral_mint,
            config,
        });

        Ok(())
    }

    pub fn set_credit_facility_config(
        ctx: Context<SetCreditFacilityConfig>,
        config: CreditFacilityConfig,
    ) -> Result<()> {
        require!(
            !ctx.accounts
                .state
                .pause_flags
                .credit_facility_updates_paused,
            AgcError::Paused
        );
        assert_risk_authority_or_admin(&ctx.accounts.state, ctx.accounts.authority.key())?;
        validate_credit_facility_config(config, ctx.accounts.collateral_asset.asset_class)?;

        ctx.accounts.facility.config = config;
        ctx.accounts.facility.status = if config.enabled {
            CreditFacilityStatus::Active
        } else {
            CreditFacilityStatus::Disabled
        };

        emit!(CreditFacilityConfigUpdated {
            facility: ctx.accounts.facility.key(),
            config,
        });

        Ok(())
    }

    pub fn deposit_underwriter_agc(ctx: Context<DepositUnderwriterAgc>, amount: u64) -> Result<()> {
        require!(
            !ctx.accounts.state.pause_flags.underwriter_deposits_paused,
            AgcError::Paused
        );
        require!(amount > 0, AgcError::ZeroAmount);
        require_facility_active(&ctx.accounts.facility)?;

        let shares = convert_to_shares(
            amount,
            ctx.accounts.facility.underwriter_total_shares,
            ctx.accounts.underwriter_vault_agc.amount,
            0,
        )?;
        require!(shares > 0, AgcError::ZeroAmount);

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.key(),
                Transfer {
                    from: ctx.accounts.underwriter_agc.to_account_info(),
                    to: ctx.accounts.underwriter_vault_agc.to_account_info(),
                    authority: ctx.accounts.underwriter.to_account_info(),
                },
            ),
            amount,
        )?;

        let facility = &mut ctx.accounts.facility;
        facility.underwriter_total_shares = facility
            .underwriter_total_shares
            .checked_add(shares)
            .ok_or(AgcError::MathOverflow)?;
        facility.total_underwriter_deposits_agc = checked_add_u128(
            facility.total_underwriter_deposits_agc,
            amount as u128,
            AgcError::MathOverflow,
        )?;

        let position = &mut ctx.accounts.underwriter_position;
        position.facility = facility.key();
        position.underwriter = ctx.accounts.underwriter.key();
        position.shares = position
            .shares
            .checked_add(shares)
            .ok_or(AgcError::MathOverflow)?;
        position.deposited_agc = checked_add_u128(
            position.deposited_agc,
            amount as u128,
            AgcError::MathOverflow,
        )?;
        position.bump = ctx.bumps.underwriter_position;

        emit!(UnderwriterAgcDeposited {
            facility: facility.key(),
            underwriter: position.underwriter,
            amount,
            shares,
        });

        Ok(())
    }

    pub fn withdraw_underwriter_agc(
        ctx: Context<WithdrawUnderwriterAgc>,
        shares: u64,
    ) -> Result<()> {
        require!(
            !ctx.accounts
                .state
                .pause_flags
                .underwriter_withdrawals_paused,
            AgcError::Paused
        );
        require!(shares > 0, AgcError::ZeroAmount);
        require!(
            ctx.accounts.underwriter_position.shares >= shares,
            AgcError::InsufficientShares
        );

        let assets = convert_to_assets(
            shares,
            ctx.accounts.facility.underwriter_total_shares,
            ctx.accounts.underwriter_vault_agc.amount,
            0,
        )?;
        require!(assets > 0, AgcError::ZeroAmount);

        let remaining_underwriter_assets = ctx
            .accounts
            .underwriter_vault_agc
            .amount
            .checked_sub(assets)
            .ok_or(AgcError::MathOverflow)?;
        validate_underwriter_reserve(&ctx.accounts.facility, remaining_underwriter_assets)?;

        transfer_from_credit_facility_vault(
            &ctx.accounts.facility,
            &ctx.accounts.underwriter_vault_agc,
            &ctx.accounts.underwriter_agc_destination,
            &ctx.accounts.facility_authority,
            &ctx.accounts.token_program,
            assets,
        )?;

        let facility = &mut ctx.accounts.facility;
        facility.underwriter_total_shares = facility
            .underwriter_total_shares
            .checked_sub(shares)
            .ok_or(AgcError::MathOverflow)?;
        facility.total_underwriter_withdrawals_agc = checked_add_u128(
            facility.total_underwriter_withdrawals_agc,
            assets as u128,
            AgcError::MathOverflow,
        )?;

        let position = &mut ctx.accounts.underwriter_position;
        position.shares = position
            .shares
            .checked_sub(shares)
            .ok_or(AgcError::MathOverflow)?;
        position.withdrawn_agc = checked_add_u128(
            position.withdrawn_agc,
            assets as u128,
            AgcError::MathOverflow,
        )?;

        emit!(UnderwriterAgcWithdrawn {
            facility: facility.key(),
            underwriter: position.underwriter,
            assets,
            shares,
        });

        Ok(())
    }

    pub fn open_credit_line(
        ctx: Context<OpenCreditLine>,
        line_id: u64,
        args: OpenCreditLineArgs,
    ) -> Result<()> {
        require!(
            !ctx.accounts.state.pause_flags.credit_line_updates_paused,
            AgcError::Paused
        );
        assert_risk_authority_or_admin(&ctx.accounts.state, ctx.accounts.authority.key())?;
        require_facility_active(&ctx.accounts.facility)?;
        validate_open_credit_line_args(args, &ctx.accounts.facility)?;

        let now = current_timestamp()?;
        require!(
            args.maturity_timestamp > now,
            AgcError::InvalidCreditLineConfig
        );

        let credit_line = &mut ctx.accounts.credit_line;
        credit_line.facility = ctx.accounts.facility.key();
        credit_line.borrower = ctx.accounts.borrower.key();
        credit_line.line_id = line_id;
        credit_line.credit_limit_agc = args.credit_limit_agc;
        credit_line.maturity_timestamp = args.maturity_timestamp;
        credit_line.status = CreditLineStatus::Active;
        credit_line.opened_at = now;
        credit_line.last_accrued_at = now;
        credit_line.bump = ctx.bumps.credit_line;

        ctx.accounts.facility.active_credit_lines = ctx
            .accounts
            .facility
            .active_credit_lines
            .checked_add(1)
            .ok_or(AgcError::MathOverflow)?;

        emit!(CreditLineOpened {
            facility: credit_line.facility,
            borrower: credit_line.borrower,
            line_id,
            credit_limit_agc: credit_line.credit_limit_agc,
            maturity_timestamp: credit_line.maturity_timestamp,
        });

        Ok(())
    }

    pub fn deposit_credit_collateral(
        ctx: Context<DepositCreditCollateral>,
        amount: u64,
    ) -> Result<()> {
        require!(
            !ctx.accounts.state.pause_flags.credit_line_updates_paused,
            AgcError::Paused
        );
        require!(amount > 0, AgcError::ZeroAmount);
        require_credit_line_active(&ctx.accounts.credit_line)?;

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.key(),
                Transfer {
                    from: ctx.accounts.borrower_collateral.to_account_info(),
                    to: ctx.accounts.collateral_vault.to_account_info(),
                    authority: ctx.accounts.borrower.to_account_info(),
                },
            ),
            amount,
        )?;

        ctx.accounts.credit_line.collateral_amount = ctx
            .accounts
            .credit_line
            .collateral_amount
            .checked_add(amount)
            .ok_or(AgcError::MathOverflow)?;
        ctx.accounts.facility.total_collateral_deposited = checked_add_u128(
            ctx.accounts.facility.total_collateral_deposited,
            amount as u128,
            AgcError::MathOverflow,
        )?;

        emit!(CreditCollateralDeposited {
            facility: ctx.accounts.facility.key(),
            borrower: ctx.accounts.borrower.key(),
            line: ctx.accounts.credit_line.key(),
            amount,
        });

        Ok(())
    }

    pub fn withdraw_credit_collateral(
        ctx: Context<WithdrawCreditCollateral>,
        amount: u64,
    ) -> Result<()> {
        require!(
            !ctx.accounts.state.pause_flags.credit_line_updates_paused,
            AgcError::Paused
        );
        require!(amount > 0, AgcError::ZeroAmount);
        require_credit_line_allows_collateral_withdrawal(&ctx.accounts.credit_line)?;

        let remaining_collateral = ctx
            .accounts
            .credit_line
            .collateral_amount
            .checked_sub(amount)
            .ok_or(AgcError::InsufficientCollateral)?;

        if ctx.accounts.credit_line.status == CreditLineStatus::Active {
            let now = current_timestamp()?;
            accrue_facility_line_interest(
                &mut ctx.accounts.credit_line,
                &mut ctx.accounts.facility,
                now,
            )?;
            if collateral_withdrawal_needs_health_check(&ctx.accounts.credit_line)? {
                validate_oracle_fresh(
                    &ctx.accounts.collateral_asset,
                    &ctx.accounts.collateral_oracle,
                    now,
                )?;
                validate_credit_line_health(
                    &ctx.accounts.credit_line,
                    &ctx.accounts.facility,
                    &ctx.accounts.collateral_oracle,
                    remaining_collateral,
                    ctx.accounts.state.anchor_price_x18,
                    ctx.accounts.state.agc_unit as u128,
                    ctx.accounts.facility.config.min_collateral_health_bps,
                )?;
            }
        }

        transfer_from_credit_facility_vault(
            &ctx.accounts.facility,
            &ctx.accounts.collateral_vault,
            &ctx.accounts.borrower_collateral_destination,
            &ctx.accounts.facility_authority,
            &ctx.accounts.token_program,
            amount,
        )?;

        ctx.accounts.credit_line.collateral_amount = remaining_collateral;

        emit!(CreditCollateralWithdrawn {
            facility: ctx.accounts.facility.key(),
            borrower: ctx.accounts.borrower.key(),
            line: ctx.accounts.credit_line.key(),
            amount,
        });

        Ok(())
    }

    pub fn draw_credit_line(ctx: Context<DrawCreditLine>, amount: u64) -> Result<()> {
        require!(
            !ctx.accounts.state.pause_flags.credit_draws_paused,
            AgcError::Paused
        );
        require!(amount > 0, AgcError::ZeroAmount);
        require_facility_active(&ctx.accounts.facility)?;
        require_credit_line_active(&ctx.accounts.credit_line)?;

        let now = current_timestamp()?;
        require!(
            now <= ctx.accounts.credit_line.maturity_timestamp,
            AgcError::CreditLineMatured
        );
        accrue_facility_line_interest(
            &mut ctx.accounts.credit_line,
            &mut ctx.accounts.facility,
            now,
        )?;
        validate_oracle_fresh(
            &ctx.accounts.collateral_asset,
            &ctx.accounts.collateral_oracle,
            now,
        )?;
        let accounted_underwriter_assets =
            accounted_underwriter_assets_agc(&ctx.accounts.facility)?;

        validate_credit_draw(
            &ctx.accounts.credit_line,
            &ctx.accounts.facility,
            &ctx.accounts.collateral_asset,
            &ctx.accounts.collateral_oracle,
            amount,
            accounted_underwriter_assets,
            ctx.accounts.state.anchor_price_x18,
            ctx.accounts.state.agc_unit as u128,
        )?;

        let fee = checked_div_u128(
            checked_mul_u128(
                amount as u128,
                ctx.accounts.facility.config.origination_fee_bps as u128,
            )?,
            BPS,
        )? as u64;
        let net_amount = amount.checked_sub(fee).ok_or(AgcError::MathOverflow)?;

        if net_amount > 0 {
            mint_with_pda(
                &ctx.accounts.agc_mint,
                &ctx.accounts.borrower_agc_destination,
                &ctx.accounts.mint_authority,
                &ctx.accounts.token_program,
                ctx.accounts.state.mint_authority_bump,
                net_amount,
            )?;
        }
        if fee > 0 {
            mint_with_pda(
                &ctx.accounts.agc_mint,
                &ctx.accounts.treasury_agc,
                &ctx.accounts.mint_authority,
                &ctx.accounts.token_program,
                ctx.accounts.state.mint_authority_bump,
                fee,
            )?;
        }

        ctx.accounts.credit_line.principal_debt_agc = ctx
            .accounts
            .credit_line
            .principal_debt_agc
            .checked_add(amount)
            .ok_or(AgcError::MathOverflow)?;
        ctx.accounts.facility.total_principal_debt_agc = ctx
            .accounts
            .facility
            .total_principal_debt_agc
            .checked_add(amount)
            .ok_or(AgcError::MathOverflow)?;
        ctx.accounts.facility.total_drawn_agc = checked_add_u128(
            ctx.accounts.facility.total_drawn_agc,
            amount as u128,
            AgcError::MathOverflow,
        )?;
        ctx.accounts.state.credit_principal_outstanding_agc = checked_add_u128(
            ctx.accounts.state.credit_principal_outstanding_agc,
            amount as u128,
            AgcError::MathOverflow,
        )?;
        ctx.accounts.state.credit_drawn_agc = checked_add_u128(
            ctx.accounts.state.credit_drawn_agc,
            amount as u128,
            AgcError::MathOverflow,
        )?;

        emit!(CreditLineDrawn {
            facility: ctx.accounts.facility.key(),
            borrower: ctx.accounts.borrower.key(),
            line: ctx.accounts.credit_line.key(),
            gross_amount: amount,
            net_amount,
            fee,
        });

        Ok(())
    }

    pub fn repay_credit_line(ctx: Context<RepayCreditLine>, amount: u64) -> Result<()> {
        require!(
            !ctx.accounts.state.pause_flags.credit_repayments_paused,
            AgcError::Paused
        );
        require!(amount > 0, AgcError::ZeroAmount);
        require_credit_line_open_for_repayment(&ctx.accounts.credit_line)?;

        accrue_facility_line_interest(
            &mut ctx.accounts.credit_line,
            &mut ctx.accounts.facility,
            current_timestamp()?,
        )?;

        let outstanding = credit_line_total_debt_agc(&ctx.accounts.credit_line)?;
        let repay_amount = amount.min(outstanding);
        require!(repay_amount > 0, AgcError::NoOutstandingDebt);

        let interest_paid = repay_amount.min(ctx.accounts.credit_line.accrued_interest_agc);
        let principal_paid = repay_amount
            .checked_sub(interest_paid)
            .ok_or(AgcError::MathOverflow)?;

        if interest_paid > 0 {
            token::transfer(
                CpiContext::new(
                    ctx.accounts.token_program.key(),
                    Transfer {
                        from: ctx.accounts.payer_agc.to_account_info(),
                        to: ctx.accounts.underwriter_vault_agc.to_account_info(),
                        authority: ctx.accounts.payer.to_account_info(),
                    },
                ),
                interest_paid,
            )?;
            ctx.accounts.credit_line.accrued_interest_agc = ctx
                .accounts
                .credit_line
                .accrued_interest_agc
                .checked_sub(interest_paid)
                .ok_or(AgcError::MathOverflow)?;
            ctx.accounts.facility.total_interest_paid_agc = checked_add_u128(
                ctx.accounts.facility.total_interest_paid_agc,
                interest_paid as u128,
                AgcError::MathOverflow,
            )?;
            ctx.accounts.state.credit_interest_paid_agc = checked_add_u128(
                ctx.accounts.state.credit_interest_paid_agc,
                interest_paid as u128,
                AgcError::MathOverflow,
            )?;
        }

        if principal_paid > 0 {
            token::burn(
                CpiContext::new(
                    ctx.accounts.token_program.key(),
                    Burn {
                        mint: ctx.accounts.agc_mint.to_account_info(),
                        from: ctx.accounts.payer_agc.to_account_info(),
                        authority: ctx.accounts.payer.to_account_info(),
                    },
                ),
                principal_paid,
            )?;
            ctx.accounts.credit_line.principal_debt_agc = ctx
                .accounts
                .credit_line
                .principal_debt_agc
                .checked_sub(principal_paid)
                .ok_or(AgcError::MathOverflow)?;
            ctx.accounts.facility.total_principal_debt_agc = ctx
                .accounts
                .facility
                .total_principal_debt_agc
                .checked_sub(principal_paid)
                .ok_or(AgcError::MathOverflow)?;
            ctx.accounts.facility.total_repaid_principal_agc = checked_add_u128(
                ctx.accounts.facility.total_repaid_principal_agc,
                principal_paid as u128,
                AgcError::MathOverflow,
            )?;
            ctx.accounts.state.credit_principal_outstanding_agc = ctx
                .accounts
                .state
                .credit_principal_outstanding_agc
                .checked_sub(principal_paid as u128)
                .ok_or(AgcError::MathOverflow)?;
            ctx.accounts.state.credit_repaid_agc = checked_add_u128(
                ctx.accounts.state.credit_repaid_agc,
                principal_paid as u128,
                AgcError::MathOverflow,
            )?;
        }

        if credit_line_total_debt_agc(&ctx.accounts.credit_line)? == 0 {
            ctx.accounts.credit_line.status = CreditLineStatus::Repaid;
            ctx.accounts.credit_line.closed_at = current_timestamp()?;
            ctx.accounts.facility.active_credit_lines =
                ctx.accounts.facility.active_credit_lines.saturating_sub(1);
        }

        emit!(CreditLineRepaid {
            facility: ctx.accounts.facility.key(),
            borrower: ctx.accounts.credit_line.borrower,
            line: ctx.accounts.credit_line.key(),
            principal_paid,
            interest_paid,
        });

        Ok(())
    }

    pub fn mark_credit_line_default(ctx: Context<MarkCreditLineDefault>) -> Result<()> {
        require!(
            !ctx.accounts.state.pause_flags.liquidations_paused,
            AgcError::Paused
        );
        assert_credit_operator_or_admin(
            &ctx.accounts.state,
            ctx.accounts.authority.key(),
            ctx.accounts.keeper.to_account_info(),
        )?;
        require_credit_line_open_for_repayment(&ctx.accounts.credit_line)?;

        let now = current_timestamp()?;
        accrue_facility_line_interest(
            &mut ctx.accounts.credit_line,
            &mut ctx.accounts.facility,
            now,
        )?;
        validate_credit_line_defaultable(
            &ctx.accounts.credit_line,
            &ctx.accounts.facility,
            &ctx.accounts.collateral_asset,
            &ctx.accounts.collateral_oracle,
            now,
            ctx.accounts.state.anchor_price_x18,
            ctx.accounts.state.agc_unit as u128,
        )?;

        let defaulted_debt = credit_line_total_debt_agc(&ctx.accounts.credit_line)?;
        let underwriter_loss = defaulted_debt.min(ctx.accounts.underwriter_vault_agc.amount);
        if underwriter_loss > 0 {
            burn_from_credit_facility_vault(
                &ctx.accounts.facility,
                &ctx.accounts.agc_mint,
                &ctx.accounts.underwriter_vault_agc,
                &ctx.accounts.facility_authority,
                &ctx.accounts.token_program,
                underwriter_loss,
            )?;
        }
        let uncovered_debt = defaulted_debt
            .checked_sub(underwriter_loss)
            .ok_or(AgcError::MathOverflow)?;

        ctx.accounts.credit_line.status = CreditLineStatus::Defaulted;
        ctx.accounts.credit_line.defaulted_at = now;
        ctx.accounts.credit_line.underwriter_loss_agc = underwriter_loss;
        ctx.accounts.credit_line.uncovered_default_agc = uncovered_debt;
        ctx.accounts.facility.total_principal_debt_agc = ctx
            .accounts
            .facility
            .total_principal_debt_agc
            .checked_sub(ctx.accounts.credit_line.principal_debt_agc)
            .ok_or(AgcError::MathOverflow)?;
        ctx.accounts.facility.total_defaulted_agc = checked_add_u128(
            ctx.accounts.facility.total_defaulted_agc,
            defaulted_debt as u128,
            AgcError::MathOverflow,
        )?;
        ctx.accounts.facility.total_underwriter_loss_agc = checked_add_u128(
            ctx.accounts.facility.total_underwriter_loss_agc,
            underwriter_loss as u128,
            AgcError::MathOverflow,
        )?;
        ctx.accounts.facility.active_credit_lines =
            ctx.accounts.facility.active_credit_lines.saturating_sub(1);
        ctx.accounts.state.credit_principal_outstanding_agc = ctx
            .accounts
            .state
            .credit_principal_outstanding_agc
            .checked_sub(ctx.accounts.credit_line.principal_debt_agc as u128)
            .ok_or(AgcError::MathOverflow)?;
        ctx.accounts.state.credit_defaulted_agc = checked_add_u128(
            ctx.accounts.state.credit_defaulted_agc,
            defaulted_debt as u128,
            AgcError::MathOverflow,
        )?;
        ctx.accounts.credit_line.principal_debt_agc = 0;
        ctx.accounts.credit_line.accrued_interest_agc = 0;

        emit!(CreditLineDefaulted {
            facility: ctx.accounts.facility.key(),
            borrower: ctx.accounts.credit_line.borrower,
            line: ctx.accounts.credit_line.key(),
            defaulted_debt,
            underwriter_loss,
            uncovered_debt,
        });

        Ok(())
    }

    pub fn seize_defaulted_collateral(
        ctx: Context<SeizeDefaultedCollateral>,
        amount: u64,
    ) -> Result<()> {
        require!(
            !ctx.accounts.state.pause_flags.liquidations_paused,
            AgcError::Paused
        );
        assert_credit_operator_or_admin(
            &ctx.accounts.state,
            ctx.accounts.authority.key(),
            ctx.accounts.keeper.to_account_info(),
        )?;
        require!(
            ctx.accounts.credit_line.status == CreditLineStatus::Defaulted,
            AgcError::CreditLineNotDefaulted
        );
        require!(amount > 0, AgcError::ZeroAmount);
        require!(
            ctx.accounts.credit_line.collateral_amount >= amount,
            AgcError::InsufficientCollateral
        );

        transfer_from_credit_facility_vault(
            &ctx.accounts.facility,
            &ctx.accounts.collateral_vault,
            &ctx.accounts.collateral_destination,
            &ctx.accounts.facility_authority,
            &ctx.accounts.token_program,
            amount,
        )?;

        ctx.accounts.credit_line.collateral_amount = ctx
            .accounts
            .credit_line
            .collateral_amount
            .checked_sub(amount)
            .ok_or(AgcError::MathOverflow)?;
        ctx.accounts.credit_line.collateral_seized = checked_add_u128(
            ctx.accounts.credit_line.collateral_seized,
            amount as u128,
            AgcError::MathOverflow,
        )?;
        ctx.accounts.facility.total_collateral_seized = checked_add_u128(
            ctx.accounts.facility.total_collateral_seized,
            amount as u128,
            AgcError::MathOverflow,
        )?;

        emit!(DefaultedCollateralSeized {
            facility: ctx.accounts.facility.key(),
            line: ctx.accounts.credit_line.key(),
            destination: ctx.accounts.collateral_destination.key(),
            amount,
        });

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
        if state_snapshot.pause_flags.credit_issuance_paused {
            result.mint_budget_acp = 0;
            result.mint_rate_bps = 0;
            result.mint_allocations = MintAllocation::default();
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
    #[account(mut, seeds = [STATE_SEED], bump = state.state_bump)]
    pub state: Box<Account<'info, ProtocolState>>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct SetGovernanceAuthorities<'info> {
    #[account(mut, seeds = [STATE_SEED], bump = state.state_bump, has_one = admin)]
    pub state: Box<Account<'info, ProtocolState>>,
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct SetPolicyParams<'info> {
    #[account(mut, seeds = [STATE_SEED], bump = state.state_bump)]
    pub state: Box<Account<'info, ProtocolState>>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct SetMintDistribution<'info> {
    #[account(mut, seeds = [STATE_SEED], bump = state.state_bump)]
    pub state: Box<Account<'info, ProtocolState>>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct SetSettlementRecipients<'info> {
    #[account(mut, seeds = [STATE_SEED], bump = state.state_bump)]
    pub state: Box<Account<'info, ProtocolState>>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct SetGrowthProgramsEnabled<'info> {
    #[account(mut, seeds = [STATE_SEED], bump = state.state_bump)]
    pub state: Box<Account<'info, ProtocolState>>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct SetExitFeeBps<'info> {
    #[account(mut, seeds = [STATE_SEED], bump = state.state_bump)]
    pub state: Box<Account<'info, ProtocolState>>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct SetCollateralAsset<'info> {
    #[account(seeds = [STATE_SEED], bump = state.state_bump)]
    pub state: Box<Account<'info, ProtocolState>>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub mint: Box<Account<'info, Mint>>,
    #[account(
        init_if_needed,
        payer = authority,
        seeds = [COLLATERAL_ASSET_SEED, mint.key().as_ref()],
        bump,
        space = 8 + CollateralAsset::LEN
    )]
    pub collateral_asset: Box<Account<'info, CollateralAsset>>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetCollateralOraclePrice<'info> {
    #[account(seeds = [STATE_SEED], bump = state.state_bump)]
    pub state: Box<Account<'info, ProtocolState>>,
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: Deserialized manually only when authority is not admin/risk admin.
    pub keeper: UncheckedAccount<'info>,
    pub mint: Box<Account<'info, Mint>>,
    #[account(
        seeds = [COLLATERAL_ASSET_SEED, mint.key().as_ref()],
        bump = collateral_asset.bump,
        constraint = collateral_asset.mint == mint.key() @ AgcError::InvalidCollateralAssetConfig
    )]
    pub collateral_asset: Box<Account<'info, CollateralAsset>>,
    #[account(
        init_if_needed,
        payer = authority,
        seeds = [COLLATERAL_ORACLE_SEED, mint.key().as_ref()],
        bump,
        space = 8 + CollateralOracle::LEN
    )]
    pub collateral_oracle: Box<Account<'info, CollateralOracle>>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(facility_id: u64)]
pub struct InitializeCreditFacility<'info> {
    #[account(mut, seeds = [STATE_SEED], bump = state.state_bump)]
    pub state: Box<Account<'info, ProtocolState>>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub collateral_mint: Box<Account<'info, Mint>>,
    #[account(
        seeds = [COLLATERAL_ASSET_SEED, collateral_mint.key().as_ref()],
        bump = collateral_asset.bump,
        constraint = collateral_asset.mint == collateral_mint.key() @ AgcError::InvalidCollateralAssetConfig
    )]
    pub collateral_asset: Box<Account<'info, CollateralAsset>>,
    #[account(
        init,
        payer = authority,
        seeds = [CREDIT_FACILITY_SEED, facility_id.to_le_bytes().as_ref()],
        bump,
        space = 8 + CreditFacility::LEN
    )]
    pub facility: Box<Account<'info, CreditFacility>>,
    /// CHECK: PDA only signs facility token-vault operations.
    #[account(seeds = [CREDIT_FACILITY_AUTHORITY_SEED, facility.key().as_ref()], bump)]
    pub facility_authority: UncheckedAccount<'info>,
    #[account(mut, address = state.agc_mint)]
    pub agc_mint: Box<Account<'info, Mint>>,
    #[account(
        init,
        payer = authority,
        seeds = [CREDIT_COLLATERAL_VAULT_SEED, facility.key().as_ref()],
        bump,
        token::mint = collateral_mint,
        token::authority = facility_authority
    )]
    pub collateral_vault: Box<Account<'info, TokenAccount>>,
    #[account(
        init,
        payer = authority,
        seeds = [UNDERWRITER_VAULT_SEED, facility.key().as_ref()],
        bump,
        token::mint = agc_mint,
        token::authority = facility_authority
    )]
    pub underwriter_vault_agc: Box<Account<'info, TokenAccount>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct SetCreditFacilityConfig<'info> {
    #[account(seeds = [STATE_SEED], bump = state.state_bump)]
    pub state: Box<Account<'info, ProtocolState>>,
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [CREDIT_FACILITY_SEED, facility.facility_id.to_le_bytes().as_ref()],
        bump = facility.bump
    )]
    pub facility: Box<Account<'info, CreditFacility>>,
    #[account(address = facility.collateral_asset)]
    pub collateral_asset: Box<Account<'info, CollateralAsset>>,
}

#[derive(Accounts)]
pub struct DepositUnderwriterAgc<'info> {
    #[account(seeds = [STATE_SEED], bump = state.state_bump)]
    pub state: Box<Account<'info, ProtocolState>>,
    #[account(
        mut,
        seeds = [CREDIT_FACILITY_SEED, facility.facility_id.to_le_bytes().as_ref()],
        bump = facility.bump
    )]
    pub facility: Box<Account<'info, CreditFacility>>,
    #[account(mut)]
    pub underwriter: Signer<'info>,
    #[account(
        mut,
        constraint = underwriter_agc.owner == underwriter.key() @ AgcError::InvalidTokenAccount,
        constraint = underwriter_agc.mint == state.agc_mint @ AgcError::InvalidTokenAccount
    )]
    pub underwriter_agc: Box<Account<'info, TokenAccount>>,
    #[account(mut, address = facility.underwriter_vault_agc)]
    pub underwriter_vault_agc: Box<Account<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = underwriter,
        seeds = [UNDERWRITER_POSITION_SEED, facility.key().as_ref(), underwriter.key().as_ref()],
        bump,
        space = 8 + UnderwriterPosition::LEN
    )]
    pub underwriter_position: Box<Account<'info, UnderwriterPosition>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct WithdrawUnderwriterAgc<'info> {
    #[account(seeds = [STATE_SEED], bump = state.state_bump)]
    pub state: Box<Account<'info, ProtocolState>>,
    #[account(
        mut,
        seeds = [CREDIT_FACILITY_SEED, facility.facility_id.to_le_bytes().as_ref()],
        bump = facility.bump
    )]
    pub facility: Box<Account<'info, CreditFacility>>,
    pub underwriter: Signer<'info>,
    #[account(
        mut,
        seeds = [UNDERWRITER_POSITION_SEED, facility.key().as_ref(), underwriter.key().as_ref()],
        bump = underwriter_position.bump,
        constraint = underwriter_position.facility == facility.key() @ AgcError::Unauthorized,
        constraint = underwriter_position.underwriter == underwriter.key() @ AgcError::Unauthorized
    )]
    pub underwriter_position: Box<Account<'info, UnderwriterPosition>>,
    #[account(mut, address = facility.underwriter_vault_agc)]
    pub underwriter_vault_agc: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = underwriter_agc_destination.owner == underwriter.key() @ AgcError::InvalidTokenAccount,
        constraint = underwriter_agc_destination.mint == state.agc_mint @ AgcError::InvalidTokenAccount
    )]
    pub underwriter_agc_destination: Box<Account<'info, TokenAccount>>,
    /// CHECK: PDA only signs facility token-vault operations.
    #[account(seeds = [CREDIT_FACILITY_AUTHORITY_SEED, facility.key().as_ref()], bump = facility.authority_bump)]
    pub facility_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(line_id: u64)]
pub struct OpenCreditLine<'info> {
    #[account(seeds = [STATE_SEED], bump = state.state_bump)]
    pub state: Box<Account<'info, ProtocolState>>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [CREDIT_FACILITY_SEED, facility.facility_id.to_le_bytes().as_ref()],
        bump = facility.bump
    )]
    pub facility: Box<Account<'info, CreditFacility>>,
    /// CHECK: Stored as the approved borrower key.
    pub borrower: UncheckedAccount<'info>,
    #[account(
        init,
        payer = authority,
        seeds = [CREDIT_LINE_SEED, facility.key().as_ref(), borrower.key().as_ref(), line_id.to_le_bytes().as_ref()],
        bump,
        space = 8 + CreditLine::LEN
    )]
    pub credit_line: Box<Account<'info, CreditLine>>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DepositCreditCollateral<'info> {
    #[account(seeds = [STATE_SEED], bump = state.state_bump)]
    pub state: Box<Account<'info, ProtocolState>>,
    #[account(
        mut,
        seeds = [CREDIT_FACILITY_SEED, facility.facility_id.to_le_bytes().as_ref()],
        bump = facility.bump
    )]
    pub facility: Box<Account<'info, CreditFacility>>,
    #[account(
        mut,
        seeds = [CREDIT_LINE_SEED, facility.key().as_ref(), credit_line.borrower.as_ref(), credit_line.line_id.to_le_bytes().as_ref()],
        bump = credit_line.bump,
        constraint = credit_line.facility == facility.key() @ AgcError::Unauthorized,
        constraint = credit_line.borrower == borrower.key() @ AgcError::Unauthorized
    )]
    pub credit_line: Box<Account<'info, CreditLine>>,
    pub borrower: Signer<'info>,
    #[account(
        mut,
        constraint = borrower_collateral.owner == borrower.key() @ AgcError::InvalidTokenAccount,
        constraint = borrower_collateral.mint == facility.collateral_mint @ AgcError::InvalidTokenAccount
    )]
    pub borrower_collateral: Box<Account<'info, TokenAccount>>,
    #[account(mut, address = facility.collateral_vault)]
    pub collateral_vault: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct WithdrawCreditCollateral<'info> {
    #[account(seeds = [STATE_SEED], bump = state.state_bump)]
    pub state: Box<Account<'info, ProtocolState>>,
    #[account(
        mut,
        seeds = [CREDIT_FACILITY_SEED, facility.facility_id.to_le_bytes().as_ref()],
        bump = facility.bump
    )]
    pub facility: Box<Account<'info, CreditFacility>>,
    #[account(address = facility.collateral_asset)]
    pub collateral_asset: Box<Account<'info, CollateralAsset>>,
    #[account(
        seeds = [COLLATERAL_ORACLE_SEED, facility.collateral_mint.as_ref()],
        bump = collateral_oracle.bump,
        constraint = collateral_oracle.mint == facility.collateral_mint @ AgcError::InvalidOraclePrice
    )]
    pub collateral_oracle: Box<Account<'info, CollateralOracle>>,
    #[account(
        mut,
        seeds = [CREDIT_LINE_SEED, facility.key().as_ref(), credit_line.borrower.as_ref(), credit_line.line_id.to_le_bytes().as_ref()],
        bump = credit_line.bump,
        constraint = credit_line.facility == facility.key() @ AgcError::Unauthorized,
        constraint = credit_line.borrower == borrower.key() @ AgcError::Unauthorized
    )]
    pub credit_line: Box<Account<'info, CreditLine>>,
    pub borrower: Signer<'info>,
    #[account(mut, address = facility.collateral_vault)]
    pub collateral_vault: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = borrower_collateral_destination.owner == borrower.key() @ AgcError::InvalidTokenAccount,
        constraint = borrower_collateral_destination.mint == facility.collateral_mint @ AgcError::InvalidTokenAccount
    )]
    pub borrower_collateral_destination: Box<Account<'info, TokenAccount>>,
    /// CHECK: PDA only signs facility token-vault operations.
    #[account(seeds = [CREDIT_FACILITY_AUTHORITY_SEED, facility.key().as_ref()], bump = facility.authority_bump)]
    pub facility_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct DrawCreditLine<'info> {
    #[account(mut, seeds = [STATE_SEED], bump = state.state_bump)]
    pub state: Box<Account<'info, ProtocolState>>,
    #[account(
        mut,
        seeds = [CREDIT_FACILITY_SEED, facility.facility_id.to_le_bytes().as_ref()],
        bump = facility.bump
    )]
    pub facility: Box<Account<'info, CreditFacility>>,
    #[account(address = facility.collateral_asset)]
    pub collateral_asset: Box<Account<'info, CollateralAsset>>,
    #[account(
        seeds = [COLLATERAL_ORACLE_SEED, facility.collateral_mint.as_ref()],
        bump = collateral_oracle.bump,
        constraint = collateral_oracle.mint == facility.collateral_mint @ AgcError::InvalidOraclePrice
    )]
    pub collateral_oracle: Box<Account<'info, CollateralOracle>>,
    #[account(
        mut,
        seeds = [CREDIT_LINE_SEED, facility.key().as_ref(), credit_line.borrower.as_ref(), credit_line.line_id.to_le_bytes().as_ref()],
        bump = credit_line.bump,
        constraint = credit_line.facility == facility.key() @ AgcError::Unauthorized,
        constraint = credit_line.borrower == borrower.key() @ AgcError::Unauthorized
    )]
    pub credit_line: Box<Account<'info, CreditLine>>,
    pub borrower: Signer<'info>,
    #[account(mut, address = state.agc_mint)]
    pub agc_mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        constraint = borrower_agc_destination.owner == borrower.key() @ AgcError::InvalidTokenAccount,
        constraint = borrower_agc_destination.mint == state.agc_mint @ AgcError::InvalidTokenAccount
    )]
    pub borrower_agc_destination: Box<Account<'info, TokenAccount>>,
    #[account(mut, address = state.treasury_agc)]
    pub treasury_agc: Box<Account<'info, TokenAccount>>,
    /// CHECK: PDA only used as SPL mint authority.
    #[account(seeds = [MINT_AUTHORITY_SEED], bump = state.mint_authority_bump)]
    pub mint_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct RepayCreditLine<'info> {
    #[account(mut, seeds = [STATE_SEED], bump = state.state_bump)]
    pub state: Box<Account<'info, ProtocolState>>,
    #[account(
        mut,
        seeds = [CREDIT_FACILITY_SEED, facility.facility_id.to_le_bytes().as_ref()],
        bump = facility.bump
    )]
    pub facility: Box<Account<'info, CreditFacility>>,
    #[account(
        mut,
        seeds = [CREDIT_LINE_SEED, facility.key().as_ref(), credit_line.borrower.as_ref(), credit_line.line_id.to_le_bytes().as_ref()],
        bump = credit_line.bump,
        constraint = credit_line.facility == facility.key() @ AgcError::Unauthorized
    )]
    pub credit_line: Box<Account<'info, CreditLine>>,
    pub payer: Signer<'info>,
    #[account(
        mut,
        constraint = payer_agc.owner == payer.key() @ AgcError::InvalidTokenAccount,
        constraint = payer_agc.mint == state.agc_mint @ AgcError::InvalidTokenAccount
    )]
    pub payer_agc: Box<Account<'info, TokenAccount>>,
    #[account(mut, address = facility.underwriter_vault_agc)]
    pub underwriter_vault_agc: Box<Account<'info, TokenAccount>>,
    #[account(mut, address = state.agc_mint)]
    pub agc_mint: Box<Account<'info, Mint>>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct MarkCreditLineDefault<'info> {
    #[account(mut, seeds = [STATE_SEED], bump = state.state_bump)]
    pub state: Box<Account<'info, ProtocolState>>,
    pub authority: Signer<'info>,
    /// CHECK: Deserialized manually only when authority is not admin/risk admin.
    pub keeper: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [CREDIT_FACILITY_SEED, facility.facility_id.to_le_bytes().as_ref()],
        bump = facility.bump
    )]
    pub facility: Box<Account<'info, CreditFacility>>,
    #[account(address = facility.collateral_asset)]
    pub collateral_asset: Box<Account<'info, CollateralAsset>>,
    #[account(
        seeds = [COLLATERAL_ORACLE_SEED, facility.collateral_mint.as_ref()],
        bump = collateral_oracle.bump,
        constraint = collateral_oracle.mint == facility.collateral_mint @ AgcError::InvalidOraclePrice
    )]
    pub collateral_oracle: Box<Account<'info, CollateralOracle>>,
    #[account(
        mut,
        seeds = [CREDIT_LINE_SEED, facility.key().as_ref(), credit_line.borrower.as_ref(), credit_line.line_id.to_le_bytes().as_ref()],
        bump = credit_line.bump,
        constraint = credit_line.facility == facility.key() @ AgcError::Unauthorized
    )]
    pub credit_line: Box<Account<'info, CreditLine>>,
    #[account(mut, address = state.agc_mint)]
    pub agc_mint: Box<Account<'info, Mint>>,
    #[account(mut, address = facility.underwriter_vault_agc)]
    pub underwriter_vault_agc: Box<Account<'info, TokenAccount>>,
    /// CHECK: PDA only signs facility token-vault operations.
    #[account(seeds = [CREDIT_FACILITY_AUTHORITY_SEED, facility.key().as_ref()], bump = facility.authority_bump)]
    pub facility_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct SeizeDefaultedCollateral<'info> {
    #[account(seeds = [STATE_SEED], bump = state.state_bump)]
    pub state: Box<Account<'info, ProtocolState>>,
    pub authority: Signer<'info>,
    /// CHECK: Deserialized manually only when authority is not admin/risk admin.
    pub keeper: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [CREDIT_FACILITY_SEED, facility.facility_id.to_le_bytes().as_ref()],
        bump = facility.bump
    )]
    pub facility: Box<Account<'info, CreditFacility>>,
    #[account(address = facility.collateral_asset)]
    pub collateral_asset: Box<Account<'info, CollateralAsset>>,
    #[account(
        mut,
        seeds = [CREDIT_LINE_SEED, facility.key().as_ref(), credit_line.borrower.as_ref(), credit_line.line_id.to_le_bytes().as_ref()],
        bump = credit_line.bump,
        constraint = credit_line.facility == facility.key() @ AgcError::Unauthorized
    )]
    pub credit_line: Box<Account<'info, CreditLine>>,
    #[account(mut, address = facility.collateral_vault)]
    pub collateral_vault: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        address = collateral_asset.reserve_token_account,
        constraint = collateral_destination.mint == facility.collateral_mint @ AgcError::InvalidTokenAccount
    )]
    pub collateral_destination: Box<Account<'info, TokenAccount>>,
    /// CHECK: PDA only signs facility token-vault operations.
    #[account(seeds = [CREDIT_FACILITY_AUTHORITY_SEED, facility.key().as_ref()], bump = facility.authority_bump)]
    pub facility_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
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
    pub risk_admin: Pubkey,
    pub emergency_admin: Pubkey,
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
    pub last_stable_cash_coverage_bps: u128,
    pub last_liquidity_depth_coverage_bps: u128,
    pub last_reserve_concentration_bps: u128,
    pub last_oracle_confidence_bps: u128,
    pub last_stale_oracle_count: u16,
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
    pub protocol_version: u16,
    pub credit_facility_count: u64,
    pub credit_principal_outstanding_agc: u128,
    pub credit_drawn_agc: u128,
    pub credit_repaid_agc: u128,
    pub credit_interest_paid_agc: u128,
    pub credit_defaulted_agc: u128,
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
    pub oracle_reporter: bool,
    pub epoch_settler: bool,
    pub buyback_executor: bool,
    pub treasury_burner: bool,
    pub credit_operator: bool,
}

impl KeeperPermissions {
    pub const LEN: usize = 6;

    pub fn all() -> Self {
        Self {
            market_reporter: true,
            oracle_reporter: true,
            epoch_settler: true,
            buyback_executor: true,
            treasury_burner: true,
            credit_operator: true,
        }
    }

    fn allows(self, required: RequiredKeeperPermission) -> bool {
        match required {
            RequiredKeeperPermission::ReportMarket => self.market_reporter,
            RequiredKeeperPermission::ReportOracle => self.oracle_reporter,
            RequiredKeeperPermission::SettleEpoch => self.epoch_settler,
            RequiredKeeperPermission::ExecuteBuyback => self.buyback_executor,
            RequiredKeeperPermission::BurnTreasury => self.treasury_burner,
            RequiredKeeperPermission::OperateCredit => self.credit_operator,
        }
    }
}

#[derive(Clone, Copy)]
enum RequiredKeeperPermission {
    ReportMarket,
    ReportOracle,
    SettleEpoch,
    ExecuteBuyback,
    BurnTreasury,
    OperateCredit,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct PauseFlags {
    pub xagc_deposits_paused: bool,
    pub xagc_redemptions_paused: bool,
    pub market_reporting_paused: bool,
    pub settlement_paused: bool,
    pub credit_issuance_paused: bool,
    pub collateral_updates_paused: bool,
    pub buybacks_paused: bool,
    pub treasury_burns_paused: bool,
    pub credit_facility_updates_paused: bool,
    pub credit_line_updates_paused: bool,
    pub credit_draws_paused: bool,
    pub credit_repayments_paused: bool,
    pub underwriter_deposits_paused: bool,
    pub underwriter_withdrawals_paused: bool,
    pub liquidations_paused: bool,
}

#[account]
pub struct CollateralAsset {
    pub mint: Pubkey,
    pub mint_decimals: u8,
    pub oracle_feed: Pubkey,
    pub reserve_token_account: Pubkey,
    pub asset_class: AssetClass,
    pub reserve_weight_bps: u16,
    pub collateral_factor_bps: u16,
    pub liquidation_threshold_bps: u16,
    pub max_concentration_bps: u16,
    pub max_oracle_staleness_seconds: u64,
    pub max_oracle_confidence_bps: u16,
    pub enabled: bool,
    pub bump: u8,
}

impl CollateralAsset {
    pub const LEN: usize = 32 + 1 + 32 + 32 + 1 + 2 + 2 + 2 + 2 + 8 + 2 + 1 + 1 + 64;
}

#[account]
pub struct CollateralOracle {
    pub mint: Pubkey,
    pub oracle_feed: Pubkey,
    pub price_quote_x18: u128,
    pub confidence_bps: u16,
    pub updated_at: u64,
    pub bump: u8,
    pub reserved: [u8; 64],
}

impl CollateralOracle {
    pub const LEN: usize = 32 + 32 + 16 + 2 + 8 + 1 + 64;
}

#[account]
pub struct CreditFacility {
    pub facility_id: u64,
    pub collateral_mint: Pubkey,
    pub collateral_asset: Pubkey,
    pub collateral_vault: Pubkey,
    pub underwriter_vault_agc: Pubkey,
    pub collateral_decimals: u8,
    pub config: CreditFacilityConfig,
    pub status: CreditFacilityStatus,
    pub underwriter_total_shares: u64,
    pub total_principal_debt_agc: u64,
    pub total_underwriter_deposits_agc: u128,
    pub total_underwriter_withdrawals_agc: u128,
    pub total_drawn_agc: u128,
    pub total_repaid_principal_agc: u128,
    pub total_interest_accrued_agc: u128,
    pub total_interest_paid_agc: u128,
    pub total_defaulted_agc: u128,
    pub total_underwriter_loss_agc: u128,
    pub total_collateral_deposited: u128,
    pub total_collateral_seized: u128,
    pub active_credit_lines: u64,
    pub created_at: u64,
    pub bump: u8,
    pub authority_bump: u8,
    pub collateral_vault_bump: u8,
    pub underwriter_vault_bump: u8,
    pub reserved: [u8; 256],
}

impl CreditFacility {
    pub const LEN: usize = 1024;
}

#[account]
pub struct UnderwriterPosition {
    pub facility: Pubkey,
    pub underwriter: Pubkey,
    pub shares: u64,
    pub deposited_agc: u128,
    pub withdrawn_agc: u128,
    pub loss_agc: u128,
    pub bump: u8,
    pub reserved: [u8; 128],
}

impl UnderwriterPosition {
    pub const LEN: usize = 32 + 32 + 8 + 16 + 16 + 16 + 1 + 128;
}

#[account]
pub struct CreditLine {
    pub facility: Pubkey,
    pub borrower: Pubkey,
    pub line_id: u64,
    pub credit_limit_agc: u64,
    pub principal_debt_agc: u64,
    pub accrued_interest_agc: u64,
    pub collateral_amount: u64,
    pub maturity_timestamp: u64,
    pub opened_at: u64,
    pub last_accrued_at: u64,
    pub defaulted_at: u64,
    pub closed_at: u64,
    pub status: CreditLineStatus,
    pub underwriter_loss_agc: u64,
    pub uncovered_default_agc: u64,
    pub collateral_seized: u128,
    pub bump: u8,
    pub reserved: [u8; 128],
}

impl CreditLine {
    pub const LEN: usize =
        32 + 32 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 1 + 8 + 8 + 16 + 1 + 128;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum AssetClass {
    #[default]
    Stable,
    Btc,
    Rwa,
    Other,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CreditFacilityStatus {
    #[default]
    Uninitialized,
    Active,
    Disabled,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CreditLineStatus {
    #[default]
    Uninitialized,
    Active,
    Repaid,
    Defaulted,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct CollateralOraclePriceInput {
    pub price_quote_x18: u128,
    pub confidence_bps: u16,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct CreditFacilityConfig {
    pub max_total_debt_agc: u64,
    pub max_line_debt_agc: u64,
    pub min_collateral_health_bps: u16,
    pub liquidation_health_bps: u16,
    pub min_underwriter_reserve_bps: u16,
    pub interest_rate_bps: u16,
    pub origination_fee_bps: u16,
    pub default_grace_seconds: u64,
    pub isolated: bool,
    pub enabled: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct OpenCreditLineArgs {
    pub credit_limit_agc: u64,
    pub maturity_timestamp: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct CollateralAssetConfig {
    pub oracle_feed: Pubkey,
    pub reserve_token_account: Pubkey,
    pub asset_class: AssetClass,
    pub reserve_weight_bps: u16,
    pub collateral_factor_bps: u16,
    pub liquidation_threshold_bps: u16,
    pub max_concentration_bps: u16,
    pub max_oracle_staleness_seconds: u64,
    pub max_oracle_confidence_bps: u16,
    pub enabled: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct GovernanceAuthorities {
    pub risk_admin: Pubkey,
    pub emergency_admin: Pubkey,
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
    pub min_stable_cash_coverage_bps: u16,
    pub target_stable_cash_coverage_bps: u16,
    pub defense_stable_cash_coverage_bps: u16,
    pub min_liquidity_depth_coverage_bps: u16,
    pub target_liquidity_depth_coverage_bps: u16,
    pub max_reserve_concentration_bps: u16,
    pub max_oracle_confidence_bps: u16,
    pub max_stale_oracle_count: u16,
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
    pub stable_cash_reserve_quote_x18: u128,
    pub risk_weighted_reserve_quote_x18: u128,
    pub liquidity_depth_quote_x18: u128,
    pub largest_collateral_concentration_bps: u16,
    pub oracle_confidence_bps: u16,
    pub stale_oracle_count: u16,
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
    pub stable_cash_coverage_bps: u128,
    pub liquidity_depth_coverage_bps: u128,
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
    pub stable_cash_reserve_quote_x18: u128,
    pub risk_weighted_reserve_quote_x18: u128,
    pub liquidity_depth_quote_x18: u128,
    pub largest_collateral_concentration_bps: u16,
    pub oracle_confidence_bps: u16,
    pub stale_oracle_count: u16,
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
pub struct GovernanceAuthoritiesUpdated {
    pub authorities: GovernanceAuthorities,
}

#[event]
pub struct CollateralAssetUpdated {
    pub mint: Pubkey,
    pub config: CollateralAssetConfig,
}

#[event]
pub struct CollateralOraclePriceUpdated {
    pub mint: Pubkey,
    pub price_quote_x18: u128,
    pub confidence_bps: u16,
    pub updated_at: u64,
}

#[event]
pub struct CreditFacilityInitialized {
    pub facility: Pubkey,
    pub facility_id: u64,
    pub collateral_mint: Pubkey,
    pub config: CreditFacilityConfig,
}

#[event]
pub struct CreditFacilityConfigUpdated {
    pub facility: Pubkey,
    pub config: CreditFacilityConfig,
}

#[event]
pub struct UnderwriterAgcDeposited {
    pub facility: Pubkey,
    pub underwriter: Pubkey,
    pub amount: u64,
    pub shares: u64,
}

#[event]
pub struct UnderwriterAgcWithdrawn {
    pub facility: Pubkey,
    pub underwriter: Pubkey,
    pub assets: u64,
    pub shares: u64,
}

#[event]
pub struct CreditLineOpened {
    pub facility: Pubkey,
    pub borrower: Pubkey,
    pub line_id: u64,
    pub credit_limit_agc: u64,
    pub maturity_timestamp: u64,
}

#[event]
pub struct CreditCollateralDeposited {
    pub facility: Pubkey,
    pub borrower: Pubkey,
    pub line: Pubkey,
    pub amount: u64,
}

#[event]
pub struct CreditCollateralWithdrawn {
    pub facility: Pubkey,
    pub borrower: Pubkey,
    pub line: Pubkey,
    pub amount: u64,
}

#[event]
pub struct CreditLineDrawn {
    pub facility: Pubkey,
    pub borrower: Pubkey,
    pub line: Pubkey,
    pub gross_amount: u64,
    pub net_amount: u64,
    pub fee: u64,
}

#[event]
pub struct CreditLineRepaid {
    pub facility: Pubkey,
    pub borrower: Pubkey,
    pub line: Pubkey,
    pub principal_paid: u64,
    pub interest_paid: u64,
}

#[event]
pub struct CreditLineDefaulted {
    pub facility: Pubkey,
    pub borrower: Pubkey,
    pub line: Pubkey,
    pub defaulted_debt: u64,
    pub underwriter_loss: u64,
    pub uncovered_debt: u64,
}

#[event]
pub struct DefaultedCollateralSeized {
    pub facility: Pubkey,
    pub line: Pubkey,
    pub destination: Pubkey,
    pub amount: u64,
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
    #[msg("The requested governance authority is invalid.")]
    InvalidGovernanceAuthority,
    #[msg("The collateral asset configuration is invalid.")]
    InvalidCollateralAssetConfig,
    #[msg("The cached oracle price is invalid or stale.")]
    InvalidOraclePrice,
    #[msg("The collateral asset is disabled.")]
    CollateralDisabled,
    #[msg("The credit facility configuration is invalid.")]
    InvalidCreditFacilityConfig,
    #[msg("The credit line configuration is invalid.")]
    InvalidCreditLineConfig,
    #[msg("The credit facility is not active.")]
    CreditFacilityInactive,
    #[msg("The credit line is not active.")]
    CreditLineInactive,
    #[msg("The requested draw exceeds the credit line limit.")]
    CreditLimitExceeded,
    #[msg("The credit line does not have enough collateral.")]
    InsufficientCollateral,
    #[msg("The credit line would be undercollateralized.")]
    InsufficientCreditHealth,
    #[msg("The underwriter vault would fall below required reserves.")]
    InsufficientUnderwriterReserve,
    #[msg("The credit line has already matured.")]
    CreditLineMatured,
    #[msg("The credit line has no outstanding debt.")]
    NoOutstandingDebt,
    #[msg("The credit line is still healthy.")]
    CreditLineHealthy,
    #[msg("The credit line is not defaulted.")]
    CreditLineNotDefaulted,
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

fn validate_governance_authorities(authorities: GovernanceAuthorities) -> Result<()> {
    require!(
        authorities.risk_admin != Pubkey::default()
            && authorities.emergency_admin != Pubkey::default(),
        AgcError::InvalidGovernanceAuthority
    );
    Ok(())
}

fn validate_collateral_asset_config(config: CollateralAssetConfig) -> Result<()> {
    require!(
        config.reserve_weight_bps <= BPS as u16,
        AgcError::InvalidCollateralAssetConfig
    );
    require!(
        config.collateral_factor_bps <= config.liquidation_threshold_bps,
        AgcError::InvalidCollateralAssetConfig
    );
    require!(
        config.liquidation_threshold_bps <= BPS as u16,
        AgcError::InvalidCollateralAssetConfig
    );
    require!(
        config.max_concentration_bps > 0 && config.max_concentration_bps <= BPS as u16,
        AgcError::InvalidCollateralAssetConfig
    );
    require!(
        config.max_oracle_staleness_seconds > 0,
        AgcError::InvalidCollateralAssetConfig
    );
    require!(
        config.max_oracle_confidence_bps <= BPS as u16,
        AgcError::InvalidCollateralAssetConfig
    );
    if config.enabled {
        require!(
            config.oracle_feed != Pubkey::default(),
            AgcError::InvalidCollateralAssetConfig
        );
        require!(
            config.reserve_token_account != Pubkey::default(),
            AgcError::InvalidCollateralAssetConfig
        );
    }
    Ok(())
}

fn validate_credit_facility_config(
    config: CreditFacilityConfig,
    asset_class: AssetClass,
) -> Result<()> {
    if config.enabled {
        require!(
            config.max_total_debt_agc > 0 && config.max_line_debt_agc > 0,
            AgcError::InvalidCreditFacilityConfig
        );
        require!(
            config.min_underwriter_reserve_bps > 0,
            AgcError::InvalidCreditFacilityConfig
        );
    }
    require!(
        config.max_line_debt_agc <= config.max_total_debt_agc,
        AgcError::InvalidCreditFacilityConfig
    );
    require!(
        config.liquidation_health_bps >= BPS as u16,
        AgcError::InvalidCreditFacilityConfig
    );
    require!(
        config.min_collateral_health_bps >= config.liquidation_health_bps,
        AgcError::InvalidCreditFacilityConfig
    );
    require!(
        config.min_underwriter_reserve_bps <= BPS as u16,
        AgcError::InvalidCreditFacilityConfig
    );
    require!(
        config.interest_rate_bps <= BPS as u16 && config.origination_fee_bps < BPS as u16,
        AgcError::InvalidCreditFacilityConfig
    );
    require!(
        config.default_grace_seconds > 0,
        AgcError::InvalidCreditFacilityConfig
    );
    if asset_class == AssetClass::Rwa {
        require!(config.isolated, AgcError::InvalidCreditFacilityConfig);
    }
    Ok(())
}

fn validate_open_credit_line_args(
    args: OpenCreditLineArgs,
    facility: &CreditFacility,
) -> Result<()> {
    require!(
        args.credit_limit_agc > 0 && args.credit_limit_agc <= facility.config.max_line_debt_agc,
        AgcError::InvalidCreditLineConfig
    );
    Ok(())
}

fn require_facility_active(facility: &CreditFacility) -> Result<()> {
    require!(
        facility.status == CreditFacilityStatus::Active && facility.config.enabled,
        AgcError::CreditFacilityInactive
    );
    Ok(())
}

fn require_credit_line_active(credit_line: &CreditLine) -> Result<()> {
    require!(
        credit_line.status == CreditLineStatus::Active,
        AgcError::CreditLineInactive
    );
    Ok(())
}

fn require_credit_line_open_for_repayment(credit_line: &CreditLine) -> Result<()> {
    require_credit_line_active(credit_line)
}

fn require_credit_line_allows_collateral_withdrawal(credit_line: &CreditLine) -> Result<()> {
    require!(
        matches!(
            credit_line.status,
            CreditLineStatus::Active | CreditLineStatus::Repaid
        ),
        AgcError::CreditLineInactive
    );
    Ok(())
}

fn collateral_withdrawal_needs_health_check(credit_line: &CreditLine) -> Result<bool> {
    require_credit_line_allows_collateral_withdrawal(credit_line)?;
    if credit_line.status == CreditLineStatus::Repaid {
        require!(
            credit_line_total_debt_agc(credit_line)? == 0,
            AgcError::CreditLineInactive
        );
        return Ok(false);
    }
    Ok(credit_line_total_debt_agc(credit_line)? > 0)
}

fn validate_oracle_fresh(
    collateral_asset: &CollateralAsset,
    collateral_oracle: &CollateralOracle,
    now: u64,
) -> Result<()> {
    require_keys_eq!(
        collateral_oracle.mint,
        collateral_asset.mint,
        AgcError::InvalidOraclePrice
    );
    require_keys_eq!(
        collateral_oracle.oracle_feed,
        collateral_asset.oracle_feed,
        AgcError::InvalidOraclePrice
    );
    require!(
        collateral_oracle.price_quote_x18 > 0,
        AgcError::InvalidOraclePrice
    );
    require!(
        collateral_oracle.confidence_bps <= collateral_asset.max_oracle_confidence_bps,
        AgcError::InvalidOraclePrice
    );
    require!(
        now.saturating_sub(collateral_oracle.updated_at)
            <= collateral_asset.max_oracle_staleness_seconds,
        AgcError::InvalidOraclePrice
    );
    Ok(())
}

fn credit_line_past_default_grace(
    credit_line: &CreditLine,
    facility: &CreditFacility,
    now: u64,
) -> bool {
    now > credit_line
        .maturity_timestamp
        .saturating_add(facility.config.default_grace_seconds)
}

fn validate_credit_line_defaultable(
    credit_line: &CreditLine,
    facility: &CreditFacility,
    collateral_asset: &CollateralAsset,
    collateral_oracle: &CollateralOracle,
    now: u64,
    anchor_price_x18: u128,
    agc_unit: u128,
) -> Result<()> {
    if credit_line_past_default_grace(credit_line, facility, now) {
        return Ok(());
    }

    validate_oracle_fresh(collateral_asset, collateral_oracle, now)?;
    let health_bps = credit_line_health_bps(
        credit_line,
        facility,
        collateral_oracle,
        credit_line.collateral_amount,
        anchor_price_x18,
        agc_unit,
    )?;
    require!(
        health_bps < facility.config.liquidation_health_bps as u128,
        AgcError::CreditLineHealthy
    );
    Ok(())
}

fn credit_line_total_debt_agc(credit_line: &CreditLine) -> Result<u64> {
    credit_line
        .principal_debt_agc
        .checked_add(credit_line.accrued_interest_agc)
        .ok_or(error!(AgcError::MathOverflow))
}

fn collateral_value_quote_x18(
    collateral_amount: u64,
    collateral_decimals: u8,
    collateral_price_quote_x18: u128,
) -> Result<u128> {
    let collateral_unit = pow10_u128(collateral_decimals)?;
    mul_div(
        collateral_amount as u128,
        collateral_price_quote_x18,
        collateral_unit,
    )
}

fn agc_value_quote_x18(amount_agc: u64, anchor_price_x18: u128, agc_unit: u128) -> Result<u128> {
    mul_div(amount_agc as u128, anchor_price_x18, agc_unit)
}

fn credit_line_health_bps(
    credit_line: &CreditLine,
    facility: &CreditFacility,
    collateral_oracle: &CollateralOracle,
    collateral_amount: u64,
    anchor_price_x18: u128,
    agc_unit: u128,
) -> Result<u128> {
    let total_debt_agc = credit_line_total_debt_agc(credit_line)?;
    credit_health_bps_for_debt(
        facility,
        collateral_oracle,
        collateral_amount,
        total_debt_agc,
        anchor_price_x18,
        agc_unit,
    )
}

fn credit_health_bps_for_debt(
    facility: &CreditFacility,
    collateral_oracle: &CollateralOracle,
    collateral_amount: u64,
    total_debt_agc: u64,
    anchor_price_x18: u128,
    agc_unit: u128,
) -> Result<u128> {
    if total_debt_agc == 0 {
        return Ok(u128::MAX);
    }

    let collateral_value = collateral_value_quote_x18(
        collateral_amount,
        facility.collateral_decimals,
        collateral_oracle.price_quote_x18,
    )?;
    let debt_value = agc_value_quote_x18(total_debt_agc, anchor_price_x18, agc_unit)?;
    safe_div(checked_mul_u128(collateral_value, BPS)?, debt_value)
}

fn validate_credit_line_health(
    credit_line: &CreditLine,
    facility: &CreditFacility,
    collateral_oracle: &CollateralOracle,
    collateral_amount: u64,
    anchor_price_x18: u128,
    agc_unit: u128,
    min_health_bps: u16,
) -> Result<()> {
    let health_bps = credit_line_health_bps(
        credit_line,
        facility,
        collateral_oracle,
        collateral_amount,
        anchor_price_x18,
        agc_unit,
    )?;
    require!(
        health_bps >= min_health_bps as u128,
        AgcError::InsufficientCreditHealth
    );
    Ok(())
}

fn validate_credit_draw(
    credit_line: &CreditLine,
    facility: &CreditFacility,
    collateral_asset: &CollateralAsset,
    collateral_oracle: &CollateralOracle,
    draw_amount: u64,
    underwriter_vault_assets_agc: u64,
    anchor_price_x18: u128,
    agc_unit: u128,
) -> Result<()> {
    require!(collateral_asset.enabled, AgcError::CollateralDisabled);
    let new_principal_debt = credit_line
        .principal_debt_agc
        .checked_add(draw_amount)
        .ok_or(AgcError::MathOverflow)?;
    let new_total_debt = new_principal_debt
        .checked_add(credit_line.accrued_interest_agc)
        .ok_or(AgcError::MathOverflow)?;
    require!(
        new_total_debt <= credit_line.credit_limit_agc
            && new_total_debt <= facility.config.max_line_debt_agc,
        AgcError::CreditLimitExceeded
    );
    let facility_principal_after = facility
        .total_principal_debt_agc
        .checked_add(draw_amount)
        .ok_or(AgcError::MathOverflow)?;
    require!(
        facility_principal_after <= facility.config.max_total_debt_agc,
        AgcError::CreditLimitExceeded
    );

    let required_underwriter_assets = mul_div(
        facility_principal_after as u128,
        facility.config.min_underwriter_reserve_bps as u128,
        BPS,
    )?;
    require!(
        underwriter_vault_assets_agc as u128 >= required_underwriter_assets,
        AgcError::InsufficientUnderwriterReserve
    );

    let collateral_value = collateral_value_quote_x18(
        credit_line.collateral_amount,
        facility.collateral_decimals,
        collateral_oracle.price_quote_x18,
    )?;
    let borrowable_value = mul_div(
        collateral_value,
        collateral_asset.collateral_factor_bps as u128,
        BPS,
    )?;
    let debt_value = agc_value_quote_x18(new_total_debt, anchor_price_x18, agc_unit)?;
    require!(
        debt_value <= borrowable_value,
        AgcError::InsufficientCollateral
    );

    let health_bps = credit_health_bps_for_debt(
        facility,
        collateral_oracle,
        credit_line.collateral_amount,
        new_total_debt,
        anchor_price_x18,
        agc_unit,
    )?;
    require!(
        health_bps >= facility.config.min_collateral_health_bps as u128,
        AgcError::InsufficientCreditHealth
    );
    Ok(())
}

fn accounted_underwriter_assets_agc(facility: &CreditFacility) -> Result<u64> {
    let inflows = checked_add_u128(
        facility.total_underwriter_deposits_agc,
        facility.total_interest_paid_agc,
        AgcError::MathOverflow,
    )?;
    let outflows = checked_add_u128(
        facility.total_underwriter_withdrawals_agc,
        facility.total_underwriter_loss_agc,
        AgcError::MathOverflow,
    )?;
    let assets = inflows.saturating_sub(outflows);
    u64::try_from(assets).map_err(|_| error!(AgcError::AmountTooLarge))
}

fn validate_underwriter_reserve(
    facility: &CreditFacility,
    underwriter_vault_assets_agc: u64,
) -> Result<()> {
    let required_underwriter_assets = mul_div(
        facility.total_principal_debt_agc as u128,
        facility.config.min_underwriter_reserve_bps as u128,
        BPS,
    )?;
    require!(
        underwriter_vault_assets_agc as u128 >= required_underwriter_assets,
        AgcError::InsufficientUnderwriterReserve
    );
    Ok(())
}

fn accrue_facility_line_interest(
    credit_line: &mut CreditLine,
    facility: &mut CreditFacility,
    now: u64,
) -> Result<()> {
    if credit_line.status != CreditLineStatus::Active {
        return Ok(());
    }
    if now <= credit_line.last_accrued_at {
        return Ok(());
    }
    if credit_line.principal_debt_agc == 0 {
        credit_line.last_accrued_at = now;
        return Ok(());
    }

    let elapsed = now - credit_line.last_accrued_at;
    let annual_interest = checked_mul_u128(
        credit_line.principal_debt_agc as u128,
        facility.config.interest_rate_bps as u128,
    )?;
    let elapsed_interest = checked_div_u128(
        checked_mul_u128(annual_interest, elapsed as u128)?,
        checked_mul_u128(BPS, SECONDS_PER_YEAR)?,
    )?;
    let interest_u64 =
        u64::try_from(elapsed_interest).map_err(|_| error!(AgcError::AmountTooLarge))?;

    if interest_u64 > 0 {
        credit_line.accrued_interest_agc = credit_line
            .accrued_interest_agc
            .checked_add(interest_u64)
            .ok_or(AgcError::MathOverflow)?;
        facility.total_interest_accrued_agc = checked_add_u128(
            facility.total_interest_accrued_agc,
            interest_u64 as u128,
            AgcError::MathOverflow,
        )?;
    }
    credit_line.last_accrued_at = now;
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
        params.defense_stable_cash_coverage_bps <= params.min_stable_cash_coverage_bps,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.min_stable_cash_coverage_bps < params.target_stable_cash_coverage_bps,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.min_liquidity_depth_coverage_bps < params.target_liquidity_depth_coverage_bps,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.max_reserve_concentration_bps > 0
            && params.max_reserve_concentration_bps <= BPS as u16,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.max_oracle_confidence_bps <= BPS as u16,
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

fn assert_oracle_reporter_or_admin(
    state: &ProtocolState,
    authority_key: Pubkey,
    keeper_info: AccountInfo,
) -> Result<()> {
    if authority_key == state.admin || authority_key == state.risk_admin {
        return Ok(());
    }
    assert_keeper_permission_or_admin(
        state,
        authority_key,
        keeper_info,
        RequiredKeeperPermission::ReportOracle,
    )
}

fn assert_credit_operator_or_admin(
    state: &ProtocolState,
    authority_key: Pubkey,
    keeper_info: AccountInfo,
) -> Result<()> {
    if authority_key == state.admin || authority_key == state.risk_admin {
        return Ok(());
    }
    assert_keeper_permission_or_admin(
        state,
        authority_key,
        keeper_info,
        RequiredKeeperPermission::OperateCredit,
    )
}

fn assert_risk_authority_or_admin(state: &ProtocolState, authority_key: Pubkey) -> Result<()> {
    require!(
        authority_key == state.admin || authority_key == state.risk_admin,
        AgcError::Unauthorized
    );
    Ok(())
}

fn assert_emergency_authority_or_admin(state: &ProtocolState, authority_key: Pubkey) -> Result<()> {
    require!(
        authority_key == state.admin || authority_key == state.emergency_admin,
        AgcError::Unauthorized
    );
    Ok(())
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
    let stable_cash_reserve_quote_x18 = if external_metrics.stable_cash_reserve_quote_x18 > 0 {
        external_metrics.stable_cash_reserve_quote_x18
    } else {
        state.treasury_quote_x18
    };
    let risk_weighted_reserve_quote_x18 = if external_metrics.risk_weighted_reserve_quote_x18 > 0 {
        external_metrics.risk_weighted_reserve_quote_x18
    } else {
        stable_cash_reserve_quote_x18
    };
    let liquidity_depth_quote_x18 = if external_metrics.liquidity_depth_quote_x18 > 0 {
        external_metrics.liquidity_depth_quote_x18
    } else {
        external_metrics.depth_to_target_slippage_quote_x18
    };
    let reserve_coverage_bps = safe_div(
        checked_mul_u128(risk_weighted_reserve_quote_x18, BPS)?,
        credit_outstanding_quote_x18,
    )?;
    let stable_cash_coverage_bps = safe_div(
        checked_mul_u128(stable_cash_reserve_quote_x18, BPS)?,
        credit_outstanding_quote_x18,
    )?;
    let liquidity_depth_coverage_bps = safe_div(
        checked_mul_u128(liquidity_depth_quote_x18, BPS)?,
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

    let oracle_health_blocked = external_metrics.oracle_confidence_bps
        > policy_params.max_oracle_confidence_bps
        || external_metrics.stale_oracle_count > policy_params.max_stale_oracle_count;
    let concentration_blocked = external_metrics.largest_collateral_concentration_bps
        > policy_params.max_reserve_concentration_bps;

    let in_defense = price_twap_x18 < stressed_floor_x18
        || reserve_coverage_bps < policy_params.defense_reserve_coverage_bps as u128
        || stable_cash_coverage_bps < policy_params.defense_stable_cash_coverage_bps as u128
        || oracle_health_blocked
        || snapshot.realized_volatility_bps >= policy_params.defense_volatility_bps as u128
        || exit_pressure_bps >= policy_params.defense_exit_pressure_bps as u128;

    let can_expand = premium_bps >= policy_params.min_premium_bps as u128
        && premium_persistence_epochs >= policy_params.premium_persistence_required as u128
        && gross_buy_floor_bps >= policy_params.min_gross_buy_floor_bps as u128
        && net_buy_pressure_bps > 0
        && lock_flow_bps > 0
        && locked_share_bps >= policy_params.min_locked_share_bps as u128
        && reserve_coverage_bps >= policy_params.expansion_reserve_coverage_bps as u128
        && stable_cash_coverage_bps >= policy_params.min_stable_cash_coverage_bps as u128
        && liquidity_depth_coverage_bps >= policy_params.min_liquidity_depth_coverage_bps as u128
        && !concentration_blocked
        && !oracle_health_blocked
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
        let stable_cash_health_bps = if stable_cash_coverage_bps
            <= policy_params.min_stable_cash_coverage_bps as u128
        {
            0
        } else {
            min_u128(
                safe_div(
                    checked_mul_u128(
                        stable_cash_coverage_bps
                            - policy_params.min_stable_cash_coverage_bps as u128,
                        BPS,
                    )?,
                    (policy_params.target_stable_cash_coverage_bps
                        - policy_params.min_stable_cash_coverage_bps) as u128,
                )?,
                BPS,
            )
        };
        let liquidity_health_bps = if liquidity_depth_coverage_bps
            <= policy_params.min_liquidity_depth_coverage_bps as u128
        {
            0
        } else {
            min_u128(
                safe_div(
                    checked_mul_u128(
                        liquidity_depth_coverage_bps
                            - policy_params.min_liquidity_depth_coverage_bps as u128,
                        BPS,
                    )?,
                    (policy_params.target_liquidity_depth_coverage_bps
                        - policy_params.min_liquidity_depth_coverage_bps)
                        as u128,
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
                stable_cash_health_bps,
                min_u128(
                    liquidity_health_bps,
                    min_u128(
                        volatility_health_bps,
                        min_u128(exit_health_bps, locked_share_health_bps),
                    ),
                ),
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
    let stable_cash_stress_bps = positive_delta(
        policy_params.defense_stable_cash_coverage_bps as u128,
        stable_cash_coverage_bps,
    );
    let concentration_stress_bps = positive_delta(
        external_metrics.largest_collateral_concentration_bps as u128,
        policy_params.max_reserve_concentration_bps as u128,
    );
    let oracle_stress_bps = positive_delta(
        external_metrics.oracle_confidence_bps as u128,
        policy_params.max_oracle_confidence_bps as u128,
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
            max_u128(
                stable_cash_stress_bps,
                max_u128(
                    concentration_stress_bps,
                    max_u128(
                        oracle_stress_bps,
                        max_u128(exit_stress_bps, volatility_stress_bps),
                    ),
                ),
            ),
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
        stable_cash_coverage_bps,
        liquidity_depth_coverage_bps,
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
        depth_to_target_slippage_quote_x18: liquidity_depth_quote_x18,
        stable_cash_reserve_quote_x18,
        risk_weighted_reserve_quote_x18,
        liquidity_depth_quote_x18,
        largest_collateral_concentration_bps: external_metrics.largest_collateral_concentration_bps,
        oracle_confidence_bps: external_metrics.oracle_confidence_bps,
        stale_oracle_count: external_metrics.stale_oracle_count,
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
    state.last_stable_cash_coverage_bps = result.stable_cash_coverage_bps;
    state.last_liquidity_depth_coverage_bps = result.liquidity_depth_coverage_bps;
    state.last_reserve_concentration_bps = result.largest_collateral_concentration_bps as u128;
    state.last_oracle_confidence_bps = result.oracle_confidence_bps as u128;
    state.last_stale_oracle_count = result.stale_oracle_count;
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

fn transfer_from_credit_facility_vault<'info>(
    facility: &Account<'info, CreditFacility>,
    source: &Account<'info, TokenAccount>,
    destination: &Account<'info, TokenAccount>,
    authority: &UncheckedAccount<'info>,
    token_program: &Program<'info, Token>,
    amount: u64,
) -> Result<()> {
    let facility_key = facility.key();
    let bump = [facility.authority_bump];
    let signer_seeds: &[&[u8]] = &[
        CREDIT_FACILITY_AUTHORITY_SEED,
        facility_key.as_ref(),
        bump.as_ref(),
    ];
    let signer: &[&[&[u8]]] = &[signer_seeds];
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

fn burn_from_credit_facility_vault<'info>(
    facility: &Account<'info, CreditFacility>,
    mint: &Account<'info, Mint>,
    source: &Account<'info, TokenAccount>,
    authority: &UncheckedAccount<'info>,
    token_program: &Program<'info, Token>,
    amount: u64,
) -> Result<()> {
    let facility_key = facility.key();
    let bump = [facility.authority_bump];
    let signer_seeds: &[&[u8]] = &[
        CREDIT_FACILITY_AUTHORITY_SEED,
        facility_key.as_ref(),
        bump.as_ref(),
    ];
    let signer: &[&[&[u8]]] = &[signer_seeds];
    token::burn(
        CpiContext::new_with_signer(
            token_program.key(),
            Burn {
                mint: mint.to_account_info(),
                from: source.to_account_info(),
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
            min_stable_cash_coverage_bps: 1_200,
            target_stable_cash_coverage_bps: 2_500,
            defense_stable_cash_coverage_bps: 800,
            min_liquidity_depth_coverage_bps: 2_000,
            target_liquidity_depth_coverage_bps: 5_000,
            max_reserve_concentration_bps: 6_000,
            max_oracle_confidence_bps: 150,
            max_stale_oracle_count: 0,
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

    fn metrics(
        stable_cash: u128,
        risk_weighted_reserve: u128,
        liquidity_depth: u128,
    ) -> ExternalMetrics {
        ExternalMetrics {
            depth_to_target_slippage_quote_x18: liquidity_depth,
            stable_cash_reserve_quote_x18: stable_cash,
            risk_weighted_reserve_quote_x18: risk_weighted_reserve,
            liquidity_depth_quote_x18: liquidity_depth,
            largest_collateral_concentration_bps: 4_500,
            oracle_confidence_bps: 25,
            stale_oracle_count: 0,
        }
    }

    fn test_state() -> ProtocolState {
        ProtocolState {
            admin: Pubkey::default(),
            pending_admin: Pubkey::default(),
            risk_admin: Pubkey::default(),
            emergency_admin: Pubkey::default(),
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
            last_stable_cash_coverage_bps: 0,
            last_liquidity_depth_coverage_bps: 0,
            last_reserve_concentration_bps: 0,
            last_oracle_confidence_bps: 0,
            last_stale_oracle_count: 0,
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
            protocol_version: 2,
            credit_facility_count: 0,
            credit_principal_outstanding_agc: 0,
            credit_drawn_agc: 0,
            credit_repaid_agc: 0,
            credit_interest_paid_agc: 0,
            credit_defaulted_agc: 0,
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

    fn credit_facility_config() -> CreditFacilityConfig {
        CreditFacilityConfig {
            max_total_debt_agc: 1_000_000 * 1_000_000_000,
            max_line_debt_agc: 500_000 * 1_000_000_000,
            min_collateral_health_bps: 20_000,
            liquidation_health_bps: 14_000,
            min_underwriter_reserve_bps: 1_000,
            interest_rate_bps: 1_200,
            origination_fee_bps: 50,
            default_grace_seconds: SECONDS_PER_DAY,
            isolated: false,
            enabled: true,
        }
    }

    fn credit_collateral_asset() -> CollateralAsset {
        CollateralAsset {
            mint: Pubkey::new_unique(),
            mint_decimals: 9,
            oracle_feed: Pubkey::new_unique(),
            reserve_token_account: Pubkey::new_unique(),
            asset_class: AssetClass::Btc,
            reserve_weight_bps: 6_000,
            collateral_factor_bps: 5_000,
            liquidation_threshold_bps: 6_500,
            max_concentration_bps: 4_000,
            max_oracle_staleness_seconds: 120,
            max_oracle_confidence_bps: 100,
            enabled: true,
            bump: 0,
        }
    }

    fn credit_oracle(asset: &CollateralAsset, updated_at: u64) -> CollateralOracle {
        CollateralOracle {
            mint: asset.mint,
            oracle_feed: asset.oracle_feed,
            price_quote_x18: PRICE_SCALE,
            confidence_bps: 25,
            updated_at,
            bump: 0,
            reserved: [0; 64],
        }
    }

    fn credit_facility(asset: &CollateralAsset) -> CreditFacility {
        CreditFacility {
            facility_id: 1,
            collateral_mint: asset.mint,
            collateral_asset: Pubkey::new_unique(),
            collateral_vault: Pubkey::new_unique(),
            underwriter_vault_agc: Pubkey::new_unique(),
            collateral_decimals: asset.mint_decimals,
            config: credit_facility_config(),
            status: CreditFacilityStatus::Active,
            underwriter_total_shares: 0,
            total_principal_debt_agc: 0,
            total_underwriter_deposits_agc: 0,
            total_underwriter_withdrawals_agc: 0,
            total_drawn_agc: 0,
            total_repaid_principal_agc: 0,
            total_interest_accrued_agc: 0,
            total_interest_paid_agc: 0,
            total_defaulted_agc: 0,
            total_underwriter_loss_agc: 0,
            total_collateral_deposited: 0,
            total_collateral_seized: 0,
            active_credit_lines: 0,
            created_at: 0,
            bump: 0,
            authority_bump: 0,
            collateral_vault_bump: 0,
            underwriter_vault_bump: 0,
            reserved: [0; 256],
        }
    }

    fn credit_line(facility: Pubkey) -> CreditLine {
        CreditLine {
            facility,
            borrower: Pubkey::new_unique(),
            line_id: 1,
            credit_limit_agc: 500_000 * 1_000_000_000,
            principal_debt_agc: 0,
            accrued_interest_agc: 0,
            collateral_amount: 2_000 * 1_000_000_000,
            maturity_timestamp: 10 * SECONDS_PER_DAY,
            opened_at: 0,
            last_accrued_at: 0,
            defaulted_at: 0,
            closed_at: 0,
            status: CreditLineStatus::Active,
            underwriter_loss_agc: 0,
            uncovered_default_agc: 0,
            collateral_seized: 0,
            bump: 0,
            reserved: [0; 128],
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
            metrics(
                250_000 * PRICE_SCALE,
                650_000 * PRICE_SCALE,
                600_000 * PRICE_SCALE,
            ),
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
            metrics(
                90_000 * PRICE_SCALE,
                100_000 * PRICE_SCALE,
                100_000 * PRICE_SCALE,
            ),
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

        let mut invalid_stable_cash_targets = params();
        invalid_stable_cash_targets.min_stable_cash_coverage_bps =
            invalid_stable_cash_targets.target_stable_cash_coverage_bps;
        assert!(validate_policy_params(invalid_stable_cash_targets).is_err());
    }

    #[test]
    fn collateral_asset_configs_are_guarded() {
        let mut config = CollateralAssetConfig {
            oracle_feed: Pubkey::new_unique(),
            reserve_token_account: Pubkey::new_unique(),
            asset_class: AssetClass::Btc,
            reserve_weight_bps: 6_000,
            collateral_factor_bps: 5_000,
            liquidation_threshold_bps: 6_500,
            max_concentration_bps: 4_000,
            max_oracle_staleness_seconds: 60,
            max_oracle_confidence_bps: 100,
            enabled: true,
        };
        assert!(validate_collateral_asset_config(config).is_ok());

        config.reserve_weight_bps = 10_001;
        assert!(validate_collateral_asset_config(config).is_err());

        config.reserve_weight_bps = 6_000;
        config.collateral_factor_bps = 7_000;
        assert!(validate_collateral_asset_config(config).is_err());

        config.collateral_factor_bps = 5_000;
        config.oracle_feed = Pubkey::default();
        assert!(validate_collateral_asset_config(config).is_err());
    }

    #[test]
    fn credit_facility_configs_are_guarded() {
        let mut config = credit_facility_config();
        assert!(validate_credit_facility_config(config, AssetClass::Btc).is_ok());

        config.min_underwriter_reserve_bps = 0;
        assert!(validate_credit_facility_config(config, AssetClass::Btc).is_err());

        config = credit_facility_config();
        config.max_line_debt_agc = config.max_total_debt_agc + 1;
        assert!(validate_credit_facility_config(config, AssetClass::Btc).is_err());

        config = credit_facility_config();
        config.isolated = false;
        assert!(validate_credit_facility_config(config, AssetClass::Rwa).is_err());

        config.isolated = true;
        assert!(validate_credit_facility_config(config, AssetClass::Rwa).is_ok());
    }

    #[test]
    fn credit_draw_requires_collateral_and_underwriter_reserve() {
        let asset = credit_collateral_asset();
        let oracle = credit_oracle(&asset, 100);
        let facility = credit_facility(&asset);
        let line = credit_line(Pubkey::new_unique());

        assert!(validate_credit_draw(
            &line,
            &facility,
            &asset,
            &oracle,
            500 * 1_000_000_000,
            100 * 1_000_000_000,
            PRICE_SCALE,
            1_000_000_000,
        )
        .is_ok());

        assert!(validate_credit_draw(
            &line,
            &facility,
            &asset,
            &oracle,
            500 * 1_000_000_000,
            1,
            PRICE_SCALE,
            1_000_000_000,
        )
        .is_err());

        let mut thin_line = credit_line(Pubkey::new_unique());
        thin_line.collateral_amount = 100 * 1_000_000_000;
        assert!(validate_credit_draw(
            &thin_line,
            &facility,
            &asset,
            &oracle,
            500 * 1_000_000_000,
            100 * 1_000_000_000,
            PRICE_SCALE,
            1_000_000_000,
        )
        .is_err());
    }

    #[test]
    fn credit_draw_rejects_disabled_collateral_and_debt_cap_breaches() {
        let mut asset = credit_collateral_asset();
        let oracle = credit_oracle(&asset, 100);
        let mut facility = credit_facility(&asset);
        let mut line = credit_line(Pubkey::new_unique());

        asset.enabled = false;
        assert!(validate_credit_draw(
            &line,
            &facility,
            &asset,
            &oracle,
            1,
            100 * 1_000_000_000,
            PRICE_SCALE,
            1_000_000_000,
        )
        .is_err());

        asset.enabled = true;
        line.principal_debt_agc = line.credit_limit_agc - 1;
        assert!(validate_credit_draw(
            &line,
            &facility,
            &asset,
            &oracle,
            2,
            100 * 1_000_000_000,
            PRICE_SCALE,
            1_000_000_000,
        )
        .is_err());

        line = credit_line(Pubkey::new_unique());
        facility.total_principal_debt_agc = facility.config.max_total_debt_agc - 1;
        assert!(validate_credit_draw(
            &line,
            &facility,
            &asset,
            &oracle,
            2,
            100_000 * 1_000_000_000,
            PRICE_SCALE,
            1_000_000_000,
        )
        .is_err());
    }

    #[test]
    fn underwriter_withdrawal_cannot_breach_required_reserve() {
        let asset = credit_collateral_asset();
        let mut facility = credit_facility(&asset);
        facility.total_principal_debt_agc = 1_000 * 1_000_000_000;

        assert!(validate_underwriter_reserve(&facility, 100 * 1_000_000_000).is_ok());
        assert!(validate_underwriter_reserve(&facility, 100 * 1_000_000_000 - 1).is_err());
    }

    #[test]
    fn credit_interest_accrues_to_facility_accounting() {
        let asset = credit_collateral_asset();
        let mut facility = credit_facility(&asset);
        let mut line = credit_line(Pubkey::new_unique());
        line.principal_debt_agc = 1_000 * 1_000_000_000;
        line.last_accrued_at = 0;

        accrue_facility_line_interest(&mut line, &mut facility, SECONDS_PER_YEAR as u64).unwrap();

        assert_eq!(line.accrued_interest_agc, 120 * 1_000_000_000);
        assert_eq!(facility.total_interest_accrued_agc, 120 * 1_000_000_000);
        assert_eq!(line.last_accrued_at, SECONDS_PER_YEAR as u64);
    }

    #[test]
    fn credit_oracle_freshness_is_enforced() {
        let asset = credit_collateral_asset();
        let fresh_oracle = credit_oracle(&asset, 100);
        assert!(validate_oracle_fresh(&asset, &fresh_oracle, 200).is_ok());

        let stale_oracle = credit_oracle(&asset, 1);
        assert!(validate_oracle_fresh(&asset, &stale_oracle, 200).is_err());
    }

    #[test]
    fn repaid_credit_lines_can_withdraw_collateral_without_oracle_health_check() {
        let mut line = credit_line(Pubkey::new_unique());
        line.status = CreditLineStatus::Repaid;
        line.principal_debt_agc = 0;
        line.accrued_interest_agc = 0;
        assert!(require_credit_line_allows_collateral_withdrawal(&line).is_ok());
        assert!(!collateral_withdrawal_needs_health_check(&line).unwrap());

        line.status = CreditLineStatus::Active;
        assert!(!collateral_withdrawal_needs_health_check(&line).unwrap());

        line.principal_debt_agc = 1;
        assert!(collateral_withdrawal_needs_health_check(&line).unwrap());

        line.status = CreditLineStatus::Repaid;
        assert!(collateral_withdrawal_needs_health_check(&line).is_err());

        line.status = CreditLineStatus::Defaulted;
        line.principal_debt_agc = 0;
        assert!(require_credit_line_allows_collateral_withdrawal(&line).is_err());
    }

    #[test]
    fn matured_default_does_not_require_fresh_oracle() {
        let asset = credit_collateral_asset();
        let facility = credit_facility(&asset);
        let mut line = credit_line(Pubkey::new_unique());
        line.principal_debt_agc = 1_000 * 1_000_000_000;
        line.maturity_timestamp = 1_000;

        let now_after_grace = line
            .maturity_timestamp
            .saturating_add(facility.config.default_grace_seconds)
            .saturating_add(1);
        let stale_oracle = credit_oracle(&asset, 1);
        assert!(validate_oracle_fresh(&asset, &stale_oracle, now_after_grace).is_err());
        assert!(validate_credit_line_defaultable(
            &line,
            &facility,
            &asset,
            &stale_oracle,
            now_after_grace,
            PRICE_SCALE,
            1_000_000_000,
        )
        .is_ok());
    }

    #[test]
    fn immature_default_requires_bad_health_and_fresh_oracle() {
        let asset = credit_collateral_asset();
        let facility = credit_facility(&asset);
        let mut line = credit_line(Pubkey::new_unique());
        line.principal_debt_agc = 1_000 * 1_000_000_000;
        line.maturity_timestamp = 10 * SECONDS_PER_DAY;

        let stale_oracle = credit_oracle(&asset, 1);
        assert!(validate_credit_line_defaultable(
            &line,
            &facility,
            &asset,
            &stale_oracle,
            200,
            PRICE_SCALE,
            1_000_000_000,
        )
        .is_err());

        let fresh_oracle = credit_oracle(&asset, 200);
        assert!(validate_credit_line_defaultable(
            &line,
            &facility,
            &asset,
            &fresh_oracle,
            200,
            PRICE_SCALE,
            1_000_000_000,
        )
        .is_err());

        line.principal_debt_agc = 2_000 * 1_000_000_000;
        assert!(validate_credit_line_defaultable(
            &line,
            &facility,
            &asset,
            &fresh_oracle,
            200,
            PRICE_SCALE,
            1_000_000_000,
        )
        .is_ok());
    }

    #[test]
    fn stable_cash_or_oracle_breaks_prevent_expansion() {
        let snapshot = EpochSnapshot {
            epoch_id: 9,
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

        let weak_cash = evaluate_epoch(
            snapshot,
            metrics(
                20_000 * PRICE_SCALE,
                650_000 * PRICE_SCALE,
                600_000 * PRICE_SCALE,
            ),
            state,
            flows,
            params(),
            1_000_000_000,
        )
        .unwrap();
        assert_ne!(weak_cash.regime, Regime::Expansion);
        assert_eq!(weak_cash.mint_budget_acp, 0);

        let mut oracle_break = metrics(
            250_000 * PRICE_SCALE,
            650_000 * PRICE_SCALE,
            600_000 * PRICE_SCALE,
        );
        oracle_break.oracle_confidence_bps = params().max_oracle_confidence_bps + 1;
        let oracle_result = evaluate_epoch(
            snapshot,
            oracle_break,
            state,
            flows,
            params(),
            1_000_000_000,
        )
        .unwrap();
        assert_eq!(oracle_result.regime, Regime::Defense);
    }

    #[test]
    fn reserve_concentration_blocks_expansion_even_with_hot_demand() {
        let snapshot = EpochSnapshot {
            epoch_id: 10,
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
        let mut concentrated = metrics(
            250_000 * PRICE_SCALE,
            650_000 * PRICE_SCALE,
            600_000 * PRICE_SCALE,
        );
        concentrated.largest_collateral_concentration_bps =
            params().max_reserve_concentration_bps + 1;

        let result = evaluate_epoch(
            snapshot,
            concentrated,
            state,
            flows,
            params(),
            1_000_000_000,
        )
        .unwrap();

        assert_ne!(result.regime, Regime::Expansion);
        assert_eq!(result.mint_budget_acp, 0);
    }

    #[test]
    fn keeper_permissions_are_role_scoped() {
        let permissions = KeeperPermissions {
            market_reporter: true,
            oracle_reporter: false,
            epoch_settler: false,
            buyback_executor: true,
            treasury_burner: false,
            credit_operator: false,
        };

        assert!(permissions.allows(RequiredKeeperPermission::ReportMarket));
        assert!(!permissions.allows(RequiredKeeperPermission::ReportOracle));
        assert!(!permissions.allows(RequiredKeeperPermission::SettleEpoch));
        assert!(permissions.allows(RequiredKeeperPermission::ExecuteBuyback));
        assert!(!permissions.allows(RequiredKeeperPermission::BurnTreasury));
        assert!(!permissions.allows(RequiredKeeperPermission::OperateCredit));

        let all_permissions = KeeperPermissions::all();
        assert!(all_permissions.allows(RequiredKeeperPermission::ReportMarket));
        assert!(all_permissions.allows(RequiredKeeperPermission::ReportOracle));
        assert!(all_permissions.allows(RequiredKeeperPermission::SettleEpoch));
        assert!(all_permissions.allows(RequiredKeeperPermission::ExecuteBuyback));
        assert!(all_permissions.allows(RequiredKeeperPermission::BurnTreasury));
        assert!(all_permissions.allows(RequiredKeeperPermission::OperateCredit));
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
            metrics(
                250_000 * PRICE_SCALE,
                500_000 * PRICE_SCALE,
                500_000 * PRICE_SCALE,
            ),
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
            metrics(
                250_000 * PRICE_SCALE,
                650_000 * PRICE_SCALE,
                600_000 * PRICE_SCALE,
            ),
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
