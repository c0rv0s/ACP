#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount, Transfer};

mod auth;
mod cpi;
mod credit;
mod events;
mod math;
mod policy;
mod pyth;
mod state;
mod validation;

pub use events::*;
pub use state::*;

use auth::*;
use cpi::*;
use credit::*;
use math::*;
use policy::*;
use pyth::*;
use validation::*;

#[cfg(test)]
mod tests;

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
const BUYBACK_CAMPAIGN_SEED: &[u8] = b"buyback-campaign";
const BUYBACK_CAMPAIGN_AUTHORITY_SEED: &[u8] = b"buyback-campaign-authority";
const BUYBACK_CAMPAIGN_USDC_ESCROW_SEED: &[u8] = b"buyback-campaign-usdc";
const BUYBACK_CAMPAIGN_AGC_VAULT_SEED: &[u8] = b"buyback-campaign-agc";

const BPS: u128 = 10_000;
const SECONDS_PER_DAY: u64 = 86_400;
const SECONDS_PER_YEAR: u128 = 31_536_000;
const PYTH_PRICE_UPDATE_V2_DISCRIMINATOR: [u8; 8] =
    [0x22, 0xf1, 0x23, 0x63, 0x9d, 0x7e, 0xf4, 0xcd];

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
        state.pyth_receiver_program = Pubkey::default();
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

    pub fn set_pyth_receiver_program(
        ctx: Context<SetPythReceiverProgram>,
        receiver_program: Pubkey,
    ) -> Result<()> {
        require!(
            receiver_program != Pubkey::default(),
            AgcError::InvalidOraclePrice
        );
        ctx.accounts.state.pyth_receiver_program = receiver_program;
        emit!(PythReceiverProgramUpdated { receiver_program });
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
        let previous_admin =
            accept_admin_inner(&mut ctx.accounts.state, ctx.accounts.pending_admin.key())?;
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
        validate_collateral_asset_config(config, ctx.accounts.state.pyth_receiver_program)?;

        let collateral_asset = &mut ctx.accounts.collateral_asset;
        collateral_asset.mint = ctx.accounts.mint.key();
        collateral_asset.mint_decimals = ctx.accounts.mint.decimals;
        collateral_asset.oracle_source = config.oracle_source;
        collateral_asset.oracle_feed = match config.oracle_source {
            OracleSource::Manual => config.oracle_feed,
            OracleSource::Pyth => ctx.accounts.state.pyth_receiver_program,
        };
        collateral_asset.pyth_price_feed_id = config.pyth_price_feed_id;
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
        require!(
            ctx.accounts.collateral_asset.oracle_source == OracleSource::Manual,
            AgcError::InvalidOracleSource
        );
        require!(price.price_quote_x18 > 0, AgcError::InvalidPrice);
        require!(
            price.confidence_bps <= ctx.accounts.collateral_asset.max_oracle_confidence_bps,
            AgcError::InvalidOraclePrice
        );

        let oracle = &mut ctx.accounts.collateral_oracle;
        oracle.mint = ctx.accounts.mint.key();
        oracle.oracle_feed = ctx.accounts.collateral_asset.oracle_feed;
        oracle.oracle_source = OracleSource::Manual;
        oracle.pyth_price_feed_id = [0; 32];
        oracle.price_quote_x18 = price.price_quote_x18;
        oracle.confidence_bps = price.confidence_bps;
        oracle.updated_at = current_timestamp()?;
        oracle.publish_time = oracle.updated_at;
        oracle.bump = ctx.bumps.collateral_oracle;

        emit!(CollateralOraclePriceUpdated {
            mint: oracle.mint,
            price_quote_x18: oracle.price_quote_x18,
            confidence_bps: oracle.confidence_bps,
            updated_at: oracle.updated_at,
        });

        Ok(())
    }

    pub fn refresh_collateral_oracle_from_pyth(
        ctx: Context<RefreshCollateralOracleFromPyth>,
    ) -> Result<()> {
        require!(
            !ctx.accounts.state.pause_flags.collateral_updates_paused,
            AgcError::Paused
        );
        require!(
            ctx.accounts.collateral_asset.oracle_source == OracleSource::Pyth,
            AgcError::InvalidOracleSource
        );

        let now = current_timestamp()?;
        let price = read_verified_pyth_price(
            ctx.accounts.price_update.to_account_info(),
            ctx.accounts.state.pyth_receiver_program,
            ctx.accounts.collateral_asset.pyth_price_feed_id,
            ctx.accounts.collateral_asset.max_oracle_staleness_seconds,
            now,
        )?;
        let price_quote_x18 = pyth_price_to_quote_x18(price.price, price.exponent)?;
        let confidence_bps = pyth_confidence_bps(price.price, price.conf)?;
        require!(
            confidence_bps <= ctx.accounts.collateral_asset.max_oracle_confidence_bps,
            AgcError::InvalidOraclePrice
        );

        let publish_time =
            u64::try_from(price.publish_time).map_err(|_| error!(AgcError::InvalidOraclePrice))?;
        let oracle = &mut ctx.accounts.collateral_oracle;
        if oracle.mint != Pubkey::default() {
            require!(
                publish_time >= oracle.publish_time,
                AgcError::InvalidOraclePrice
            );
        }
        oracle.mint = ctx.accounts.mint.key();
        oracle.oracle_feed = ctx.accounts.collateral_asset.oracle_feed;
        oracle.oracle_source = OracleSource::Pyth;
        oracle.pyth_price_feed_id = ctx.accounts.collateral_asset.pyth_price_feed_id;
        oracle.price_quote_x18 = price_quote_x18;
        oracle.confidence_bps = confidence_bps;
        oracle.updated_at = publish_time;
        oracle.publish_time = publish_time;
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
        let needs_health_check =
            collateral_withdrawal_needs_health_check(&ctx.accounts.credit_line)?;

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
            if needs_health_check {
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

    pub fn start_buyback_campaign(
        ctx: Context<StartBuybackCampaign>,
        campaign_id: u64,
        config: BuybackCampaignConfig,
    ) -> Result<()> {
        require!(
            !ctx.accounts.state.pause_flags.buybacks_paused,
            AgcError::Paused
        );
        require_keys_eq!(
            ctx.accounts.treasury_usdc.key(),
            ctx.accounts.state.treasury_usdc,
            AgcError::InvalidTokenAccount
        );
        require_keys_eq!(
            ctx.accounts.usdc_mint.key(),
            ctx.accounts.state.usdc_mint,
            AgcError::InvalidTokenAccount
        );
        require_keys_eq!(
            ctx.accounts.agc_mint.key(),
            ctx.accounts.state.agc_mint,
            AgcError::InvalidTokenAccount
        );
        assert_keeper_permission_or_admin(
            &ctx.accounts.state,
            ctx.accounts.authority.key(),
            ctx.accounts.keeper.to_account_info(),
            RequiredKeeperPermission::ExecuteBuyback,
        )?;

        let now = current_timestamp()?;
        validate_buyback_campaign_config(
            config,
            ctx.accounts.state.pending_treasury_buyback_usdc,
            now,
        )?;

        transfer_from_treasury(
            &ctx.accounts.treasury_usdc,
            &ctx.accounts.campaign_usdc_escrow,
            &ctx.accounts.treasury_authority,
            &ctx.accounts.token_program,
            ctx.accounts.state.treasury_authority_bump,
            config.total_usdc,
        )?;

        let started_at = if config.start_after == 0 {
            now
        } else {
            config.start_after
        };
        let campaign = &mut ctx.accounts.campaign;
        campaign.campaign_id = campaign_id;
        campaign.status = BuybackCampaignStatus::Active;
        campaign.total_usdc = config.total_usdc;
        campaign.remaining_usdc = config.total_usdc;
        campaign.spent_usdc = 0;
        campaign.min_total_agc_out = config.min_total_agc_out;
        campaign.agc_burned = 0;
        campaign.max_slice_usdc = config.max_slice_usdc;
        campaign.slice_interval_seconds = config.slice_interval_seconds;
        campaign.started_at = started_at;
        campaign.expires_at = config.expires_at;
        campaign.last_slice_at = 0;
        campaign.slice_count = 0;
        campaign.adapter_usdc_account = config.adapter_usdc_account;
        campaign.usdc_escrow = ctx.accounts.campaign_usdc_escrow.key();
        campaign.agc_vault = ctx.accounts.campaign_agc_vault.key();
        campaign.bump = ctx.bumps.campaign;
        campaign.authority_bump = ctx.bumps.campaign_authority;
        campaign.usdc_escrow_bump = ctx.bumps.campaign_usdc_escrow;
        campaign.agc_vault_bump = ctx.bumps.campaign_agc_vault;

        let state = &mut ctx.accounts.state;
        state.pending_treasury_buyback_usdc = state
            .pending_treasury_buyback_usdc
            .checked_sub(config.total_usdc)
            .ok_or(AgcError::MathOverflow)?;
        state.buyback_execution_nonce = state
            .buyback_execution_nonce
            .checked_add(1)
            .ok_or(AgcError::MathOverflow)?;

        emit!(BuybackCampaignStarted {
            campaign: campaign.key(),
            campaign_id,
            total_usdc: config.total_usdc,
            min_total_agc_out: config.min_total_agc_out,
            adapter_usdc_account: config.adapter_usdc_account,
        });

        Ok(())
    }

    pub fn execute_buyback_twap_slice(
        ctx: Context<ExecuteBuybackTwapSlice>,
        args: BuybackSliceArgs,
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

        let now = current_timestamp()?;
        validate_buyback_slice(&ctx.accounts.campaign, args, now)?;
        require!(
            ctx.accounts.campaign_agc_vault.amount >= args.agc_amount_to_burn,
            AgcError::InsufficientBuybackOutput
        );

        let remaining_after = ctx
            .accounts
            .campaign
            .remaining_usdc
            .checked_sub(args.usdc_amount)
            .ok_or(AgcError::MathOverflow)?;
        let burned_after = ctx
            .accounts
            .campaign
            .agc_burned
            .checked_add(args.agc_amount_to_burn)
            .ok_or(AgcError::MathOverflow)?;
        if remaining_after == 0 {
            require!(
                burned_after >= ctx.accounts.campaign.min_total_agc_out,
                AgcError::InsufficientBuybackOutput
            );
        }

        burn_from_buyback_campaign_vault(
            ctx.accounts.campaign.campaign_id,
            ctx.accounts.campaign.authority_bump,
            &ctx.accounts.agc_mint,
            &ctx.accounts.campaign_agc_vault,
            &ctx.accounts.campaign_authority,
            &ctx.accounts.token_program,
            args.agc_amount_to_burn,
        )?;
        transfer_from_buyback_campaign_vault(
            ctx.accounts.campaign.campaign_id,
            ctx.accounts.campaign.authority_bump,
            &ctx.accounts.campaign_usdc_escrow,
            &ctx.accounts.adapter_usdc_destination,
            &ctx.accounts.campaign_authority,
            &ctx.accounts.token_program,
            args.usdc_amount,
        )?;

        let campaign = &mut ctx.accounts.campaign;
        campaign.remaining_usdc = remaining_after;
        campaign.spent_usdc = campaign
            .spent_usdc
            .checked_add(args.usdc_amount)
            .ok_or(AgcError::MathOverflow)?;
        campaign.agc_burned = burned_after;
        campaign.last_slice_at = now;
        campaign.slice_count = campaign
            .slice_count
            .checked_add(1)
            .ok_or(AgcError::MathOverflow)?;
        if campaign.remaining_usdc == 0 {
            campaign.status = BuybackCampaignStatus::Completed;
        }

        emit!(BuybackTwapSliceExecuted {
            campaign: campaign.key(),
            campaign_id: campaign.campaign_id,
            usdc_amount: args.usdc_amount,
            agc_burned: args.agc_amount_to_burn,
            remaining_usdc: campaign.remaining_usdc,
            total_agc_burned: campaign.agc_burned,
        });

        Ok(())
    }

    pub fn cancel_buyback_campaign(ctx: Context<CancelBuybackCampaign>) -> Result<()> {
        assert_emergency_authority_or_admin(&ctx.accounts.state, ctx.accounts.authority.key())?;
        require!(
            ctx.accounts.campaign.status == BuybackCampaignStatus::Active,
            AgcError::BuybackCampaignInactive
        );

        if ctx.accounts.campaign_agc_vault.amount > 0 {
            burn_from_buyback_campaign_vault(
                ctx.accounts.campaign.campaign_id,
                ctx.accounts.campaign.authority_bump,
                &ctx.accounts.agc_mint,
                &ctx.accounts.campaign_agc_vault,
                &ctx.accounts.campaign_authority,
                &ctx.accounts.token_program,
                ctx.accounts.campaign_agc_vault.amount,
            )?;
        }
        if ctx.accounts.campaign_usdc_escrow.amount > 0 {
            transfer_from_buyback_campaign_vault(
                ctx.accounts.campaign.campaign_id,
                ctx.accounts.campaign.authority_bump,
                &ctx.accounts.campaign_usdc_escrow,
                &ctx.accounts.treasury_usdc,
                &ctx.accounts.campaign_authority,
                &ctx.accounts.token_program,
                ctx.accounts.campaign_usdc_escrow.amount,
            )?;
        }

        let campaign = &mut ctx.accounts.campaign;
        campaign.status = BuybackCampaignStatus::Cancelled;
        campaign.remaining_usdc = 0;

        emit!(BuybackCampaignCancelled {
            campaign: campaign.key(),
            campaign_id: campaign.campaign_id,
        });

        Ok(())
    }

    pub fn reserve_treasury_buyback_usdc(
        _ctx: Context<ReserveTreasuryBuybackUsdc>,
        _amount: u64,
    ) -> Result<()> {
        err!(AgcError::DeprecatedBuybackPath)
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

fn accept_admin_inner(state: &mut ProtocolState, new_admin: Pubkey) -> Result<Pubkey> {
    require!(new_admin != Pubkey::default(), AgcError::InvalidAdmin);

    let previous_admin = state.admin;
    state.admin = new_admin;
    state.pending_admin = Pubkey::default();
    if state.risk_admin == previous_admin {
        state.risk_admin = new_admin;
    }
    if state.emergency_admin == previous_admin {
        state.emergency_admin = new_admin;
    }

    Ok(previous_admin)
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
pub struct SetPythReceiverProgram<'info> {
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
pub struct RefreshCollateralOracleFromPyth<'info> {
    #[account(seeds = [STATE_SEED], bump = state.state_bump)]
    pub state: Box<Account<'info, ProtocolState>>,
    #[account(mut)]
    pub authority: Signer<'info>,
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
    /// CHECK: Validated by owner, Anchor discriminator, feed id, verification level, and timestamp.
    pub price_update: UncheckedAccount<'info>,
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
#[instruction(campaign_id: u64)]
pub struct StartBuybackCampaign<'info> {
    #[account(mut, seeds = [STATE_SEED], bump = state.state_bump)]
    pub state: Box<Account<'info, ProtocolState>>,
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: Deserialized manually only when authority is not admin.
    pub keeper: UncheckedAccount<'info>,
    #[account(mut)]
    pub treasury_usdc: Box<Account<'info, TokenAccount>>,
    #[account(
        init,
        payer = authority,
        seeds = [BUYBACK_CAMPAIGN_SEED, campaign_id.to_le_bytes().as_ref()],
        bump,
        space = 8 + BuybackCampaign::LEN
    )]
    pub campaign: Box<Account<'info, BuybackCampaign>>,
    /// CHECK: PDA only signs campaign token-account operations.
    #[account(seeds = [BUYBACK_CAMPAIGN_AUTHORITY_SEED, campaign_id.to_le_bytes().as_ref()], bump)]
    pub campaign_authority: UncheckedAccount<'info>,
    #[account(
        init,
        payer = authority,
        seeds = [BUYBACK_CAMPAIGN_USDC_ESCROW_SEED, campaign_id.to_le_bytes().as_ref()],
        bump,
        token::mint = usdc_mint,
        token::authority = campaign_authority
    )]
    pub campaign_usdc_escrow: Box<Account<'info, TokenAccount>>,
    #[account(
        init,
        payer = authority,
        seeds = [BUYBACK_CAMPAIGN_AGC_VAULT_SEED, campaign_id.to_le_bytes().as_ref()],
        bump,
        token::mint = agc_mint,
        token::authority = campaign_authority
    )]
    pub campaign_agc_vault: Box<Account<'info, TokenAccount>>,
    pub usdc_mint: Box<Account<'info, Mint>>,
    pub agc_mint: Box<Account<'info, Mint>>,
    /// CHECK: PDA only signs treasury token-account operations.
    #[account(seeds = [TREASURY_AUTHORITY_SEED], bump = state.treasury_authority_bump)]
    pub treasury_authority: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ExecuteBuybackTwapSlice<'info> {
    #[account(mut, seeds = [STATE_SEED], bump = state.state_bump)]
    pub state: Box<Account<'info, ProtocolState>>,
    pub authority: Signer<'info>,
    /// CHECK: Deserialized manually only when authority is not admin.
    pub keeper: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [BUYBACK_CAMPAIGN_SEED, campaign.campaign_id.to_le_bytes().as_ref()],
        bump = campaign.bump
    )]
    pub campaign: Box<Account<'info, BuybackCampaign>>,
    #[account(
        mut,
        address = campaign.usdc_escrow,
        constraint = campaign_usdc_escrow.mint == state.usdc_mint @ AgcError::InvalidTokenAccount
    )]
    pub campaign_usdc_escrow: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        address = campaign.agc_vault,
        constraint = campaign_agc_vault.mint == state.agc_mint @ AgcError::InvalidTokenAccount
    )]
    pub campaign_agc_vault: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        address = campaign.adapter_usdc_account,
        constraint = adapter_usdc_destination.mint == state.usdc_mint @ AgcError::InvalidTokenAccount
    )]
    pub adapter_usdc_destination: Box<Account<'info, TokenAccount>>,
    #[account(mut, address = state.agc_mint)]
    pub agc_mint: Box<Account<'info, Mint>>,
    /// CHECK: PDA only signs campaign token-account operations.
    #[account(
        seeds = [BUYBACK_CAMPAIGN_AUTHORITY_SEED, campaign.campaign_id.to_le_bytes().as_ref()],
        bump = campaign.authority_bump
    )]
    pub campaign_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CancelBuybackCampaign<'info> {
    #[account(seeds = [STATE_SEED], bump = state.state_bump)]
    pub state: Box<Account<'info, ProtocolState>>,
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [BUYBACK_CAMPAIGN_SEED, campaign.campaign_id.to_le_bytes().as_ref()],
        bump = campaign.bump
    )]
    pub campaign: Box<Account<'info, BuybackCampaign>>,
    #[account(
        mut,
        address = campaign.usdc_escrow,
        constraint = campaign_usdc_escrow.mint == state.usdc_mint @ AgcError::InvalidTokenAccount
    )]
    pub campaign_usdc_escrow: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        address = campaign.agc_vault,
        constraint = campaign_agc_vault.mint == state.agc_mint @ AgcError::InvalidTokenAccount
    )]
    pub campaign_agc_vault: Box<Account<'info, TokenAccount>>,
    #[account(mut, address = state.treasury_usdc)]
    pub treasury_usdc: Box<Account<'info, TokenAccount>>,
    #[account(mut, address = state.agc_mint)]
    pub agc_mint: Box<Account<'info, Mint>>,
    /// CHECK: PDA only signs campaign token-account operations.
    #[account(
        seeds = [BUYBACK_CAMPAIGN_AUTHORITY_SEED, campaign.campaign_id.to_le_bytes().as_ref()],
        bump = campaign.authority_bump
    )]
    pub campaign_authority: UncheckedAccount<'info>,
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
    #[msg("The collateral oracle source is invalid for this instruction.")]
    InvalidOracleSource,
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
    #[msg("The legacy buyback transfer path is disabled; use buyback campaigns.")]
    DeprecatedBuybackPath,
    #[msg("The buyback campaign configuration is invalid.")]
    InvalidBuybackCampaignConfig,
    #[msg("The buyback campaign is not active.")]
    BuybackCampaignInactive,
    #[msg("The buyback campaign or slice is not ready.")]
    BuybackCampaignNotReady,
    #[msg("The buyback slice exceeds campaign limits.")]
    BuybackSliceTooLarge,
    #[msg("The buyback slice deadline has expired.")]
    BuybackDeadlineExpired,
    #[msg("The buyback slice did not deliver enough AGC to burn.")]
    InsufficientBuybackOutput,
    #[msg("Arithmetic overflow or underflow.")]
    MathOverflow,
    #[msg("A u128 policy amount does not fit into a u64 SPL token amount.")]
    AmountTooLarge,
    #[msg("Clock returned a negative timestamp.")]
    InvalidClock,
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


