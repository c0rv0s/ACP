#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

declare_id!("6GCCBywkt8WwNoaroggSwnMkd8ggWvLNBzpWnFXWWR6n");

const MARKET_SEED: &[u8] = b"market";
const FEE_VAULT_AUTHORITY_SEED: &[u8] = b"fee-vault-authority";
const FEE_VAULT_USDC_SEED: &[u8] = b"fee-vault-usdc";
const BORROWER_SEED: &[u8] = b"borrower";
const POOL_SEED: &[u8] = b"liquidity-pool";
const POOL_AUTHORITY_SEED: &[u8] = b"pool-authority";
const POOL_USDC_SEED: &[u8] = b"pool-usdc";
const APPLICATION_SEED: &[u8] = b"credit-application";
const CREDIT_LINE_SEED: &[u8] = b"credit-line";
const ATTESTATION_SEED: &[u8] = b"attestation";

const BPS_DENOMINATOR: u64 = 10_000;

#[program]
pub mod agc_credit_control {
    use super::*;

    pub fn initialize_market(ctx: Context<InitializeMarket>, args: InitializeMarketArgs) -> Result<()> {
        require!(args.max_total_credit_usdc > 0, CreditMarketError::InvalidLimit);
        require!(args.max_single_borrower_usdc > 0, CreditMarketError::InvalidLimit);
        require!(
            args.max_single_borrower_usdc <= args.max_total_credit_usdc,
            CreditMarketError::InvalidLimit
        );
        require!(args.platform_fee_bps <= 1_000, CreditMarketError::InvalidBps);

        let market = &mut ctx.accounts.market_config;
        market.admin = ctx.accounts.admin.key();
        market.risk_admin = args.risk_admin;
        market.emergency_admin = args.emergency_admin;
        market.usdc_mint = ctx.accounts.usdc_mint.key();
        market.fee_vault = ctx.accounts.fee_vault.key();
        market.fee_vault_authority = ctx.accounts.fee_vault_authority.key();
        market.max_total_credit_usdc = args.max_total_credit_usdc;
        market.max_single_borrower_usdc = args.max_single_borrower_usdc;
        market.total_liquidity_usdc = 0;
        market.total_credit_limit_usdc = 0;
        market.total_outstanding_usdc = 0;
        market.total_platform_fees_usdc = 0;
        market.platform_fee_bps = args.platform_fee_bps;
        market.paused = false;
        market.bump = ctx.bumps.market_config;
        market.fee_vault_authority_bump = ctx.bumps.fee_vault_authority;
        market.fee_vault_bump = ctx.bumps.fee_vault;

        emit!(MarketInitialized {
            market: market.key(),
            admin: market.admin,
            usdc_mint: market.usdc_mint,
            platform_fee_bps: market.platform_fee_bps,
        });

        Ok(())
    }

    pub fn set_market_pause(ctx: Context<MarketAuthority>, paused: bool) -> Result<()> {
        assert_admin_or_emergency(&ctx.accounts.market_config, ctx.accounts.authority.key())?;
        ctx.accounts.market_config.paused = paused;
        emit!(MarketPauseUpdated { paused });
        Ok(())
    }

    pub fn register_borrower(ctx: Context<RegisterBorrower>, args: RegisterBorrowerArgs) -> Result<()> {
        require!(!ctx.accounts.market_config.paused, CreditMarketError::MarketPaused);
        require!(args.trust_score <= 1000, CreditMarketError::InvalidScore);

        let borrower = &mut ctx.accounts.borrower;
        borrower.operator = ctx.accounts.operator.key();
        borrower.primary_wallet = args.primary_wallet;
        borrower.metadata_hash = args.metadata_hash;
        borrower.borrower_type = args.borrower_type;
        borrower.verification_status = args.verification_status;
        borrower.status = BorrowerStatus::Active;
        borrower.trust_score = args.trust_score;
        borrower.total_credit_limit_usdc = 0;
        borrower.total_outstanding_usdc = 0;
        borrower.repayment_count = 0;
        borrower.delinquency_count = 0;
        borrower.created_at = current_timestamp()?;
        borrower.bump = ctx.bumps.borrower;

        emit!(BorrowerRegistered {
            borrower: borrower.key(),
            operator: borrower.operator,
            trust_score: borrower.trust_score,
        });

        Ok(())
    }

    pub fn update_borrower_verification(
        ctx: Context<UpdateBorrowerVerification>,
        args: UpdateBorrowerVerificationArgs,
    ) -> Result<()> {
        assert_risk_authority(&ctx.accounts.market_config, ctx.accounts.authority.key())?;
        require!(args.trust_score <= 1000, CreditMarketError::InvalidScore);

        let borrower = &mut ctx.accounts.borrower;
        borrower.metadata_hash = args.metadata_hash;
        borrower.verification_status = args.verification_status;
        borrower.trust_score = args.trust_score;

        emit!(BorrowerVerificationUpdated {
            borrower: borrower.key(),
            verification_status: borrower.verification_status,
            trust_score: borrower.trust_score,
        });

        Ok(())
    }

    pub fn create_liquidity_pool(ctx: Context<CreateLiquidityPool>, args: CreateLiquidityPoolArgs) -> Result<()> {
        require!(!ctx.accounts.market_config.paused, CreditMarketError::MarketPaused);
        require!(args.max_single_line_usdc > 0, CreditMarketError::InvalidLimit);
        require!(args.min_trust_score <= 1000, CreditMarketError::InvalidScore);
        require!(args.max_apr_bps > 0, CreditMarketError::InvalidBps);

        let pool = &mut ctx.accounts.liquidity_pool;
        pool.market = ctx.accounts.market_config.key();
        pool.manager = ctx.accounts.manager.key();
        pool.usdc_mint = ctx.accounts.market_config.usdc_mint;
        pool.usdc_vault = ctx.accounts.usdc_vault.key();
        pool.vault_authority = ctx.accounts.vault_authority.key();
        pool.metadata_hash = args.metadata_hash;
        pool.policy_hash = args.policy_hash;
        pool.status = PoolStatus::Active;
        pool.approval_mode = args.approval_mode;
        pool.committed_capital_usdc = 0;
        pool.available_capital_usdc = 0;
        pool.committed_to_lines_usdc = 0;
        pool.principal_drawn_usdc = 0;
        pool.principal_repaid_usdc = 0;
        pool.interest_repaid_usdc = 0;
        pool.platform_fees_paid_usdc = 0;
        pool.max_single_line_usdc = args.max_single_line_usdc;
        pool.min_trust_score = args.min_trust_score;
        pool.max_apr_bps = args.max_apr_bps;
        pool.auto_approve_under_usdc = args.auto_approve_under_usdc;
        pool.created_at = current_timestamp()?;
        pool.bump = ctx.bumps.liquidity_pool;
        pool.authority_bump = ctx.bumps.vault_authority;
        pool.usdc_vault_bump = ctx.bumps.usdc_vault;

        emit!(LiquidityPoolCreated {
            liquidity_pool: pool.key(),
            manager: pool.manager,
            approval_mode: pool.approval_mode,
            max_single_line_usdc: pool.max_single_line_usdc,
        });

        Ok(())
    }

    pub fn fund_liquidity_pool(ctx: Context<FundLiquidityPool>, amount_usdc: u64) -> Result<()> {
        require!(amount_usdc > 0, CreditMarketError::InvalidAmount);
        require!(!ctx.accounts.market_config.paused, CreditMarketError::MarketPaused);
        require!(ctx.accounts.liquidity_pool.status == PoolStatus::Active, CreditMarketError::PoolNotActive);

        token::transfer(ctx.accounts.fund_transfer_ctx(), amount_usdc)?;

        let pool = &mut ctx.accounts.liquidity_pool;
        pool.committed_capital_usdc = checked_add(pool.committed_capital_usdc, amount_usdc)?;
        pool.available_capital_usdc = checked_add(pool.available_capital_usdc, amount_usdc)?;
        ctx.accounts.market_config.total_liquidity_usdc =
            checked_add(ctx.accounts.market_config.total_liquidity_usdc, amount_usdc)?;

        emit!(LiquidityPoolFunded {
            liquidity_pool: pool.key(),
            amount_usdc,
            available_capital_usdc: pool.available_capital_usdc,
        });

        Ok(())
    }

    pub fn set_pool_policy(ctx: Context<SetPoolPolicy>, args: SetPoolPolicyArgs) -> Result<()> {
        require!(args.min_trust_score <= 1000, CreditMarketError::InvalidScore);
        require!(args.max_single_line_usdc > 0, CreditMarketError::InvalidLimit);
        require!(args.max_apr_bps > 0, CreditMarketError::InvalidBps);

        let pool = &mut ctx.accounts.liquidity_pool;
        pool.policy_hash = args.policy_hash;
        pool.approval_mode = args.approval_mode;
        pool.max_single_line_usdc = args.max_single_line_usdc;
        pool.min_trust_score = args.min_trust_score;
        pool.max_apr_bps = args.max_apr_bps;
        pool.auto_approve_under_usdc = args.auto_approve_under_usdc;

        emit!(PoolPolicyUpdated {
            liquidity_pool: pool.key(),
            approval_mode: pool.approval_mode,
            min_trust_score: pool.min_trust_score,
            max_apr_bps: pool.max_apr_bps,
        });

        Ok(())
    }

    pub fn submit_credit_application(
        ctx: Context<SubmitCreditApplication>,
        args: SubmitCreditApplicationArgs,
    ) -> Result<()> {
        require!(!ctx.accounts.market_config.paused, CreditMarketError::MarketPaused);
        require!(ctx.accounts.borrower.status == BorrowerStatus::Active, CreditMarketError::BorrowerNotActive);
        require!(ctx.accounts.liquidity_pool.status == PoolStatus::Active, CreditMarketError::PoolNotActive);
        require!(args.requested_limit_usdc > 0, CreditMarketError::InvalidLimit);
        require!(args.proposed_apr_bps > 0, CreditMarketError::InvalidBps);

        let application = &mut ctx.accounts.credit_application;
        application.borrower = ctx.accounts.borrower.key();
        application.liquidity_pool = ctx.accounts.liquidity_pool.key();
        application.application_seed = args.application_seed;
        application.metadata_hash = args.metadata_hash;
        application.requested_limit_usdc = args.requested_limit_usdc;
        application.proposed_apr_bps = args.proposed_apr_bps;
        application.collateralization_bps = args.collateralization_bps;
        application.trust_score_snapshot = ctx.accounts.borrower.trust_score;
        application.status = ApplicationStatus::Pending;
        application.approval_mode = ctx.accounts.liquidity_pool.approval_mode;
        application.created_at = current_timestamp()?;
        application.reviewed_at = 0;
        application.reviewer = Pubkey::default();
        application.bump = ctx.bumps.credit_application;

        emit!(CreditApplicationSubmitted {
            application: application.key(),
            borrower: application.borrower,
            liquidity_pool: application.liquidity_pool,
            requested_limit_usdc: application.requested_limit_usdc,
        });

        Ok(())
    }

    pub fn approve_credit_application(
        ctx: Context<ApproveCreditApplication>,
        args: ApproveCreditApplicationArgs,
    ) -> Result<()> {
        assert_pool_manager_or_risk(
            &ctx.accounts.market_config,
            &ctx.accounts.liquidity_pool,
            ctx.accounts.authority.key(),
        )?;
        require!(!ctx.accounts.market_config.paused, CreditMarketError::MarketPaused);
        require!(ctx.accounts.credit_application.status == ApplicationStatus::Pending, CreditMarketError::InvalidApplicationStatus);
        require!(ctx.accounts.borrower.status == BorrowerStatus::Active, CreditMarketError::BorrowerNotActive);
        require!(ctx.accounts.liquidity_pool.status == PoolStatus::Active, CreditMarketError::PoolNotActive);
        require!(args.approved_limit_usdc > 0, CreditMarketError::InvalidLimit);
        require!(args.approved_limit_usdc <= ctx.accounts.credit_application.requested_limit_usdc, CreditMarketError::InvalidLimit);
        require!(args.approved_limit_usdc <= ctx.accounts.liquidity_pool.max_single_line_usdc, CreditMarketError::PoolLimitExceeded);
        require!(args.approved_limit_usdc <= ctx.accounts.liquidity_pool.available_capital_usdc, CreditMarketError::InsufficientLiquidity);
        require!(args.apr_bps <= ctx.accounts.liquidity_pool.max_apr_bps, CreditMarketError::InvalidBps);
        require!(ctx.accounts.borrower.trust_score >= ctx.accounts.liquidity_pool.min_trust_score, CreditMarketError::TrustScoreTooLow);
        require!(
            checked_add(ctx.accounts.borrower.total_credit_limit_usdc, args.approved_limit_usdc)?
                <= ctx.accounts.market_config.max_single_borrower_usdc,
            CreditMarketError::BorrowerLimitExceeded
        );
        require!(
            checked_add(ctx.accounts.market_config.total_credit_limit_usdc, args.approved_limit_usdc)?
                <= ctx.accounts.market_config.max_total_credit_usdc,
            CreditMarketError::MarketLimitExceeded
        );

        let now = current_timestamp()?;
        let collateralization_bps = ctx.accounts.credit_application.collateralization_bps;
        let application = &mut ctx.accounts.credit_application;
        application.status = ApplicationStatus::Approved;
        application.reviewed_at = now;
        application.reviewer = ctx.accounts.authority.key();

        let line = &mut ctx.accounts.credit_line;
        line.borrower = ctx.accounts.borrower.key();
        line.liquidity_pool = ctx.accounts.liquidity_pool.key();
        line.credit_application = application.key();
        line.borrower_wallet = ctx.accounts.borrower.primary_wallet;
        line.borrower_usdc = ctx.accounts.borrower_usdc.key();
        line.line_seed = args.line_seed;
        line.metadata_hash = args.metadata_hash;
        line.principal_limit_usdc = args.approved_limit_usdc;
        line.available_limit_usdc = args.approved_limit_usdc;
        line.principal_outstanding_usdc = 0;
        line.principal_repaid_usdc = 0;
        line.interest_paid_usdc = 0;
        line.platform_fees_paid_usdc = 0;
        line.apr_bps = args.apr_bps;
        line.platform_fee_bps = ctx.accounts.market_config.platform_fee_bps;
        line.collateralization_bps = collateralization_bps;
        line.status = CreditLineStatus::Active;
        line.risk_grade = args.risk_grade;
        line.maturity_at = now
            .checked_add(args.tenor_seconds)
            .ok_or(CreditMarketError::MathOverflow)?;
        line.created_at = now;
        line.last_draw_at = 0;
        line.last_repayment_at = 0;
        line.bump = ctx.bumps.credit_line;

        let pool = &mut ctx.accounts.liquidity_pool;
        pool.available_capital_usdc = checked_sub(pool.available_capital_usdc, args.approved_limit_usdc)?;
        pool.committed_to_lines_usdc = checked_add(pool.committed_to_lines_usdc, args.approved_limit_usdc)?;
        ctx.accounts.borrower.total_credit_limit_usdc =
            checked_add(ctx.accounts.borrower.total_credit_limit_usdc, args.approved_limit_usdc)?;
        ctx.accounts.market_config.total_credit_limit_usdc =
            checked_add(ctx.accounts.market_config.total_credit_limit_usdc, args.approved_limit_usdc)?;

        emit!(CreditLineDeployed {
            credit_line: line.key(),
            application: application.key(),
            borrower: line.borrower,
            liquidity_pool: line.liquidity_pool,
            principal_limit_usdc: line.principal_limit_usdc,
            apr_bps: line.apr_bps,
            risk_grade: line.risk_grade,
        });

        Ok(())
    }

    pub fn draw_credit(ctx: Context<DrawCredit>, amount_usdc: u64) -> Result<()> {
        require!(amount_usdc > 0, CreditMarketError::InvalidAmount);
        require!(!ctx.accounts.market_config.paused, CreditMarketError::MarketPaused);
        require!(
            matches!(ctx.accounts.credit_line.status, CreditLineStatus::Active | CreditLineStatus::Repaid),
            CreditMarketError::CreditLineNotActive
        );
        require!(ctx.accounts.credit_line.available_limit_usdc >= amount_usdc, CreditMarketError::InsufficientCredit);
        require_keys_eq!(ctx.accounts.credit_line.borrower, ctx.accounts.borrower.key(), CreditMarketError::InvalidAccount);
        require_keys_eq!(
            ctx.accounts.credit_line.liquidity_pool,
            ctx.accounts.liquidity_pool.key(),
            CreditMarketError::InvalidAccount
        );
        require_keys_eq!(ctx.accounts.credit_line.borrower_usdc, ctx.accounts.borrower_usdc.key(), CreditMarketError::InvalidTokenAccount);
        require_keys_eq!(ctx.accounts.liquidity_pool.usdc_vault, ctx.accounts.pool_usdc_vault.key(), CreditMarketError::InvalidTokenAccount);

        let pool_key = ctx.accounts.liquidity_pool.key();
        let authority_bump = ctx.accounts.liquidity_pool.authority_bump;
        let signer_seeds: &[&[&[u8]]] = &[&[
            POOL_AUTHORITY_SEED,
            pool_key.as_ref(),
            &[authority_bump],
        ]];

        token::transfer(ctx.accounts.draw_transfer_ctx().with_signer(signer_seeds), amount_usdc)?;

        let line = &mut ctx.accounts.credit_line;
        line.status = CreditLineStatus::Active;
        line.available_limit_usdc = checked_sub(line.available_limit_usdc, amount_usdc)?;
        line.principal_outstanding_usdc = checked_add(line.principal_outstanding_usdc, amount_usdc)?;
        line.last_draw_at = current_timestamp()?;
        ctx.accounts.borrower.total_outstanding_usdc =
            checked_add(ctx.accounts.borrower.total_outstanding_usdc, amount_usdc)?;
        ctx.accounts.liquidity_pool.principal_drawn_usdc =
            checked_add(ctx.accounts.liquidity_pool.principal_drawn_usdc, amount_usdc)?;
        ctx.accounts.market_config.total_outstanding_usdc =
            checked_add(ctx.accounts.market_config.total_outstanding_usdc, amount_usdc)?;

        emit!(CreditDrawn {
            credit_line: line.key(),
            borrower: line.borrower,
            amount_usdc,
            principal_outstanding_usdc: line.principal_outstanding_usdc,
        });

        Ok(())
    }

    pub fn repay_credit(ctx: Context<RepayCredit>, principal_repayment_usdc: u64) -> Result<()> {
        require!(principal_repayment_usdc > 0, CreditMarketError::InvalidAmount);
        require_keys_eq!(ctx.accounts.credit_line.borrower, ctx.accounts.borrower.key(), CreditMarketError::InvalidAccount);
        require_keys_eq!(
            ctx.accounts.credit_line.liquidity_pool,
            ctx.accounts.liquidity_pool.key(),
            CreditMarketError::InvalidAccount
        );
        require_keys_eq!(ctx.accounts.liquidity_pool.usdc_vault, ctx.accounts.pool_usdc_vault.key(), CreditMarketError::InvalidTokenAccount);
        require_keys_eq!(ctx.accounts.market_config.fee_vault, ctx.accounts.fee_vault.key(), CreditMarketError::InvalidTokenAccount);

        let repay_amount = principal_repayment_usdc.min(ctx.accounts.credit_line.principal_outstanding_usdc);
        require!(repay_amount > 0, CreditMarketError::NothingToRepay);
        let fee_amount = calculate_bps(repay_amount, ctx.accounts.market_config.platform_fee_bps)?;

        token::transfer(ctx.accounts.repay_to_pool_ctx(), repay_amount)?;
        if fee_amount > 0 {
            token::transfer(ctx.accounts.repay_fee_ctx(), fee_amount)?;
        }

        let line = &mut ctx.accounts.credit_line;
        line.principal_outstanding_usdc = checked_sub(line.principal_outstanding_usdc, repay_amount)?;
        line.available_limit_usdc = checked_add(line.available_limit_usdc, repay_amount)?;
        if line.available_limit_usdc > line.principal_limit_usdc {
            line.available_limit_usdc = line.principal_limit_usdc;
        }
        line.principal_repaid_usdc = checked_add(line.principal_repaid_usdc, repay_amount)?;
        line.platform_fees_paid_usdc = checked_add(line.platform_fees_paid_usdc, fee_amount)?;
        line.last_repayment_at = current_timestamp()?;
        if line.principal_outstanding_usdc == 0 {
            line.status = CreditLineStatus::Repaid;
        }

        ctx.accounts.borrower.total_outstanding_usdc =
            checked_sub(ctx.accounts.borrower.total_outstanding_usdc, repay_amount)?;
        ctx.accounts.borrower.repayment_count = ctx
            .accounts
            .borrower
            .repayment_count
            .checked_add(1)
            .ok_or(CreditMarketError::MathOverflow)?;
        ctx.accounts.liquidity_pool.principal_repaid_usdc =
            checked_add(ctx.accounts.liquidity_pool.principal_repaid_usdc, repay_amount)?;
        ctx.accounts.liquidity_pool.platform_fees_paid_usdc =
            checked_add(ctx.accounts.liquidity_pool.platform_fees_paid_usdc, fee_amount)?;
        ctx.accounts.market_config.total_outstanding_usdc =
            checked_sub(ctx.accounts.market_config.total_outstanding_usdc, repay_amount)?;
        ctx.accounts.market_config.total_platform_fees_usdc =
            checked_add(ctx.accounts.market_config.total_platform_fees_usdc, fee_amount)?;

        emit!(CreditRepaid {
            credit_line: line.key(),
            borrower: line.borrower,
            principal_repayment_usdc: repay_amount,
            platform_fee_usdc: fee_amount,
            remaining_outstanding_usdc: line.principal_outstanding_usdc,
        });

        Ok(())
    }

    pub fn update_credit_attestation(ctx: Context<UpdateCreditAttestation>, args: UpdateCreditAttestationArgs) -> Result<()> {
        assert_risk_authority(&ctx.accounts.market_config, ctx.accounts.authority.key())?;
        require!(args.score <= 1000, CreditMarketError::InvalidScore);
        require!(args.trust_score <= 1000, CreditMarketError::InvalidScore);
        require!(args.confidence_bps <= BPS_DENOMINATOR as u16, CreditMarketError::InvalidBps);

        let attestation = &mut ctx.accounts.credit_attestation;
        attestation.borrower = ctx.accounts.borrower.key();
        attestation.credit_line = ctx.accounts.credit_line.key();
        attestation.score_hash = args.score_hash;
        attestation.score = args.score;
        attestation.trust_score = args.trust_score;
        attestation.risk_grade = args.risk_grade;
        attestation.recommended_limit_usdc = args.recommended_limit_usdc;
        attestation.recommended_apr_bps = args.recommended_apr_bps;
        attestation.pd_estimate_bps = args.pd_estimate_bps;
        attestation.lgd_estimate_bps = args.lgd_estimate_bps;
        attestation.confidence_bps = args.confidence_bps;
        attestation.features_hash = args.features_hash;
        attestation.created_at = current_timestamp()?;
        attestation.bump = ctx.bumps.credit_attestation;

        ctx.accounts.borrower.trust_score = args.trust_score;
        ctx.accounts.credit_line.risk_grade = args.risk_grade;

        emit!(CreditAttestationUpdated {
            credit_attestation: attestation.key(),
            credit_line: attestation.credit_line,
            borrower: attestation.borrower,
            trust_score: attestation.trust_score,
            risk_grade: attestation.risk_grade,
        });

        Ok(())
    }

    pub fn suspend_credit_line(ctx: Context<CreditLineRiskAction>) -> Result<()> {
        assert_pool_manager_or_risk(
            &ctx.accounts.market_config,
            &ctx.accounts.liquidity_pool,
            ctx.accounts.authority.key(),
        )?;
        require!(ctx.accounts.credit_line.status == CreditLineStatus::Active, CreditMarketError::InvalidCreditLineStatus);
        ctx.accounts.credit_line.status = CreditLineStatus::Suspended;
        emit!(CreditLineSuspended {
            credit_line: ctx.accounts.credit_line.key(),
        });
        Ok(())
    }

    pub fn resume_credit_line(ctx: Context<CreditLineRiskAction>) -> Result<()> {
        assert_pool_manager_or_risk(
            &ctx.accounts.market_config,
            &ctx.accounts.liquidity_pool,
            ctx.accounts.authority.key(),
        )?;
        require!(ctx.accounts.credit_line.status == CreditLineStatus::Suspended, CreditMarketError::InvalidCreditLineStatus);
        ctx.accounts.credit_line.status = CreditLineStatus::Active;
        emit!(CreditLineResumed {
            credit_line: ctx.accounts.credit_line.key(),
        });
        Ok(())
    }

    pub fn mark_default(ctx: Context<CreditLineRiskAction>) -> Result<()> {
        assert_pool_manager_or_risk(
            &ctx.accounts.market_config,
            &ctx.accounts.liquidity_pool,
            ctx.accounts.authority.key(),
        )?;
        require!(ctx.accounts.credit_line.principal_outstanding_usdc > 0, CreditMarketError::NothingToRepay);
        ctx.accounts.credit_line.status = CreditLineStatus::Defaulted;
        ctx.accounts.credit_line.available_limit_usdc = 0;
        ctx.accounts.borrower.delinquency_count = ctx
            .accounts
            .borrower
            .delinquency_count
            .checked_add(1)
            .ok_or(CreditMarketError::MathOverflow)?;
        emit!(CreditLineDefaulted {
            credit_line: ctx.accounts.credit_line.key(),
            loss_usdc: ctx.accounts.credit_line.principal_outstanding_usdc,
        });
        Ok(())
    }

    pub fn close_credit_line(ctx: Context<CloseCreditLine>) -> Result<()> {
        assert_pool_manager_or_risk(
            &ctx.accounts.market_config,
            &ctx.accounts.liquidity_pool,
            ctx.accounts.authority.key(),
        )?;
        require!(ctx.accounts.credit_line.principal_outstanding_usdc == 0, CreditMarketError::OutstandingBalance);
        require!(
            matches!(
                ctx.accounts.credit_line.status,
                CreditLineStatus::Active | CreditLineStatus::Repaid | CreditLineStatus::Suspended
            ),
            CreditMarketError::InvalidCreditLineStatus
        );

        let limit = ctx.accounts.credit_line.principal_limit_usdc;
        ctx.accounts.credit_line.status = CreditLineStatus::Closed;
        ctx.accounts.liquidity_pool.available_capital_usdc =
            checked_add(ctx.accounts.liquidity_pool.available_capital_usdc, limit)?;
        ctx.accounts.liquidity_pool.committed_to_lines_usdc =
            checked_sub(ctx.accounts.liquidity_pool.committed_to_lines_usdc, limit)?;
        ctx.accounts.borrower.total_credit_limit_usdc =
            checked_sub(ctx.accounts.borrower.total_credit_limit_usdc, limit)?;
        ctx.accounts.market_config.total_credit_limit_usdc =
            checked_sub(ctx.accounts.market_config.total_credit_limit_usdc, limit)?;

        emit!(CreditLineClosed {
            credit_line: ctx.accounts.credit_line.key(),
            liquidity_pool: ctx.accounts.liquidity_pool.key(),
        });

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeMarket<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + MarketConfig::LEN,
        seeds = [MARKET_SEED],
        bump
    )]
    pub market_config: Box<Account<'info, MarketConfig>>,
    /// CHECK: PDA authority for the fee vault.
    #[account(seeds = [FEE_VAULT_AUTHORITY_SEED], bump)]
    pub fee_vault_authority: UncheckedAccount<'info>,
    #[account(
        init,
        payer = admin,
        token::mint = usdc_mint,
        token::authority = fee_vault_authority,
        seeds = [FEE_VAULT_USDC_SEED],
        bump
    )]
    pub fee_vault: Account<'info, TokenAccount>,
    pub usdc_mint: Account<'info, Mint>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct MarketAuthority<'info> {
    #[account(mut, seeds = [MARKET_SEED], bump = market_config.bump)]
    pub market_config: Box<Account<'info, MarketConfig>>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct RegisterBorrower<'info> {
    #[account(seeds = [MARKET_SEED], bump = market_config.bump)]
    pub market_config: Account<'info, MarketConfig>,
    #[account(
        init,
        payer = operator,
        space = 8 + BorrowerProfile::LEN,
        seeds = [BORROWER_SEED, operator.key().as_ref()],
        bump
    )]
    pub borrower: Box<Account<'info, BorrowerProfile>>,
    #[account(mut)]
    pub operator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateBorrowerVerification<'info> {
    #[account(seeds = [MARKET_SEED], bump = market_config.bump)]
    pub market_config: Account<'info, MarketConfig>,
    #[account(mut)]
    pub borrower: Box<Account<'info, BorrowerProfile>>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct CreateLiquidityPool<'info> {
    #[account(mut, seeds = [MARKET_SEED], bump = market_config.bump)]
    pub market_config: Box<Account<'info, MarketConfig>>,
    #[account(
        init,
        payer = manager,
        space = 8 + LiquidityPool::LEN,
        seeds = [POOL_SEED, manager.key().as_ref()],
        bump
    )]
    pub liquidity_pool: Box<Account<'info, LiquidityPool>>,
    /// CHECK: PDA authority for this pool vault.
    #[account(seeds = [POOL_AUTHORITY_SEED, liquidity_pool.key().as_ref()], bump)]
    pub vault_authority: UncheckedAccount<'info>,
    #[account(
        init,
        payer = manager,
        token::mint = usdc_mint,
        token::authority = vault_authority,
        seeds = [POOL_USDC_SEED, liquidity_pool.key().as_ref()],
        bump
    )]
    pub usdc_vault: Account<'info, TokenAccount>,
    #[account(address = market_config.usdc_mint)]
    pub usdc_mint: Account<'info, Mint>,
    #[account(mut)]
    pub manager: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct FundLiquidityPool<'info> {
    #[account(mut, seeds = [MARKET_SEED], bump = market_config.bump)]
    pub market_config: Box<Account<'info, MarketConfig>>,
    #[account(mut, has_one = manager, constraint = liquidity_pool.market == market_config.key())]
    pub liquidity_pool: Box<Account<'info, LiquidityPool>>,
    #[account(mut, constraint = manager_usdc.mint == market_config.usdc_mint)]
    pub manager_usdc: Account<'info, TokenAccount>,
    #[account(mut, address = liquidity_pool.usdc_vault)]
    pub pool_usdc_vault: Account<'info, TokenAccount>,
    pub manager: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

impl<'info> FundLiquidityPool<'info> {
    fn fund_transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.key(),
            Transfer {
                from: self.manager_usdc.to_account_info(),
                to: self.pool_usdc_vault.to_account_info(),
                authority: self.manager.to_account_info(),
            },
        )
    }
}

#[derive(Accounts)]
pub struct SetPoolPolicy<'info> {
    #[account(seeds = [MARKET_SEED], bump = market_config.bump)]
    pub market_config: Account<'info, MarketConfig>,
    #[account(mut, has_one = manager, constraint = liquidity_pool.market == market_config.key())]
    pub liquidity_pool: Box<Account<'info, LiquidityPool>>,
    pub manager: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(args: SubmitCreditApplicationArgs)]
pub struct SubmitCreditApplication<'info> {
    #[account(seeds = [MARKET_SEED], bump = market_config.bump)]
    pub market_config: Account<'info, MarketConfig>,
    #[account(has_one = operator)]
    pub borrower: Box<Account<'info, BorrowerProfile>>,
    #[account(constraint = liquidity_pool.market == market_config.key())]
    pub liquidity_pool: Box<Account<'info, LiquidityPool>>,
    #[account(
        init,
        payer = operator,
        space = 8 + CreditApplication::LEN,
        seeds = [APPLICATION_SEED, borrower.key().as_ref(), liquidity_pool.key().as_ref(), args.application_seed.as_ref()],
        bump
    )]
    pub credit_application: Box<Account<'info, CreditApplication>>,
    #[account(mut)]
    pub operator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(args: ApproveCreditApplicationArgs)]
pub struct ApproveCreditApplication<'info> {
    #[account(mut, seeds = [MARKET_SEED], bump = market_config.bump)]
    pub market_config: Box<Account<'info, MarketConfig>>,
    #[account(mut)]
    pub borrower: Box<Account<'info, BorrowerProfile>>,
    #[account(mut, constraint = liquidity_pool.market == market_config.key())]
    pub liquidity_pool: Box<Account<'info, LiquidityPool>>,
    #[account(
        mut,
        constraint = credit_application.borrower == borrower.key(),
        constraint = credit_application.liquidity_pool == liquidity_pool.key()
    )]
    pub credit_application: Box<Account<'info, CreditApplication>>,
    #[account(
        init,
        payer = authority,
        space = 8 + CreditLine::LEN,
        seeds = [CREDIT_LINE_SEED, credit_application.key().as_ref(), args.line_seed.as_ref()],
        bump
    )]
    pub credit_line: Box<Account<'info, CreditLine>>,
    #[account(
        constraint = borrower_usdc.mint == market_config.usdc_mint,
        constraint = borrower_usdc.owner == borrower.primary_wallet
    )]
    pub borrower_usdc: Account<'info, TokenAccount>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DrawCredit<'info> {
    #[account(mut, seeds = [MARKET_SEED], bump = market_config.bump)]
    pub market_config: Box<Account<'info, MarketConfig>>,
    #[account(mut, has_one = operator)]
    pub borrower: Box<Account<'info, BorrowerProfile>>,
    #[account(mut, constraint = liquidity_pool.market == market_config.key())]
    pub liquidity_pool: Box<Account<'info, LiquidityPool>>,
    #[account(mut)]
    pub credit_line: Box<Account<'info, CreditLine>>,
    /// CHECK: PDA authority for this pool vault.
    #[account(seeds = [POOL_AUTHORITY_SEED, liquidity_pool.key().as_ref()], bump = liquidity_pool.authority_bump)]
    pub vault_authority: UncheckedAccount<'info>,
    #[account(mut, address = liquidity_pool.usdc_vault)]
    pub pool_usdc_vault: Account<'info, TokenAccount>,
    #[account(mut, address = credit_line.borrower_usdc)]
    pub borrower_usdc: Account<'info, TokenAccount>,
    pub operator: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

impl<'info> DrawCredit<'info> {
    fn draw_transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.key(),
            Transfer {
                from: self.pool_usdc_vault.to_account_info(),
                to: self.borrower_usdc.to_account_info(),
                authority: self.vault_authority.to_account_info(),
            },
        )
    }
}

#[derive(Accounts)]
pub struct RepayCredit<'info> {
    #[account(mut, seeds = [MARKET_SEED], bump = market_config.bump)]
    pub market_config: Box<Account<'info, MarketConfig>>,
    #[account(mut)]
    pub borrower: Box<Account<'info, BorrowerProfile>>,
    #[account(mut, constraint = liquidity_pool.market == market_config.key())]
    pub liquidity_pool: Box<Account<'info, LiquidityPool>>,
    #[account(mut)]
    pub credit_line: Box<Account<'info, CreditLine>>,
    #[account(mut, constraint = payer_usdc.mint == market_config.usdc_mint)]
    pub payer_usdc: Account<'info, TokenAccount>,
    #[account(mut, address = liquidity_pool.usdc_vault)]
    pub pool_usdc_vault: Account<'info, TokenAccount>,
    #[account(mut, address = market_config.fee_vault)]
    pub fee_vault: Account<'info, TokenAccount>,
    pub payer: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

impl<'info> RepayCredit<'info> {
    fn repay_to_pool_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.key(),
            Transfer {
                from: self.payer_usdc.to_account_info(),
                to: self.pool_usdc_vault.to_account_info(),
                authority: self.payer.to_account_info(),
            },
        )
    }

    fn repay_fee_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.key(),
            Transfer {
                from: self.payer_usdc.to_account_info(),
                to: self.fee_vault.to_account_info(),
                authority: self.payer.to_account_info(),
            },
        )
    }
}

#[derive(Accounts)]
#[instruction(args: UpdateCreditAttestationArgs)]
pub struct UpdateCreditAttestation<'info> {
    #[account(seeds = [MARKET_SEED], bump = market_config.bump)]
    pub market_config: Account<'info, MarketConfig>,
    #[account(mut)]
    pub borrower: Box<Account<'info, BorrowerProfile>>,
    #[account(mut, constraint = credit_line.borrower == borrower.key())]
    pub credit_line: Box<Account<'info, CreditLine>>,
    #[account(
        init,
        payer = authority,
        space = 8 + CreditAttestation::LEN,
        seeds = [ATTESTATION_SEED, borrower.key().as_ref(), args.score_hash.as_ref()],
        bump
    )]
    pub credit_attestation: Box<Account<'info, CreditAttestation>>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreditLineRiskAction<'info> {
    #[account(seeds = [MARKET_SEED], bump = market_config.bump)]
    pub market_config: Account<'info, MarketConfig>,
    #[account(mut)]
    pub borrower: Box<Account<'info, BorrowerProfile>>,
    #[account(constraint = liquidity_pool.market == market_config.key())]
    pub liquidity_pool: Box<Account<'info, LiquidityPool>>,
    #[account(
        mut,
        constraint = credit_line.borrower == borrower.key(),
        constraint = credit_line.liquidity_pool == liquidity_pool.key()
    )]
    pub credit_line: Box<Account<'info, CreditLine>>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct CloseCreditLine<'info> {
    #[account(mut, seeds = [MARKET_SEED], bump = market_config.bump)]
    pub market_config: Box<Account<'info, MarketConfig>>,
    #[account(mut)]
    pub borrower: Box<Account<'info, BorrowerProfile>>,
    #[account(mut, constraint = liquidity_pool.market == market_config.key())]
    pub liquidity_pool: Box<Account<'info, LiquidityPool>>,
    #[account(
        mut,
        constraint = credit_line.borrower == borrower.key(),
        constraint = credit_line.liquidity_pool == liquidity_pool.key()
    )]
    pub credit_line: Box<Account<'info, CreditLine>>,
    pub authority: Signer<'info>,
}

#[account]
pub struct MarketConfig {
    pub admin: Pubkey,
    pub risk_admin: Pubkey,
    pub emergency_admin: Pubkey,
    pub usdc_mint: Pubkey,
    pub fee_vault: Pubkey,
    pub fee_vault_authority: Pubkey,
    pub max_total_credit_usdc: u64,
    pub max_single_borrower_usdc: u64,
    pub total_liquidity_usdc: u64,
    pub total_credit_limit_usdc: u64,
    pub total_outstanding_usdc: u64,
    pub total_platform_fees_usdc: u64,
    pub platform_fee_bps: u16,
    pub paused: bool,
    pub bump: u8,
    pub fee_vault_authority_bump: u8,
    pub fee_vault_bump: u8,
}

impl MarketConfig {
    pub const LEN: usize = 256;
}

#[account]
pub struct LiquidityPool {
    pub market: Pubkey,
    pub manager: Pubkey,
    pub usdc_mint: Pubkey,
    pub usdc_vault: Pubkey,
    pub vault_authority: Pubkey,
    pub metadata_hash: [u8; 32],
    pub policy_hash: [u8; 32],
    pub status: PoolStatus,
    pub approval_mode: ApprovalMode,
    pub committed_capital_usdc: u64,
    pub available_capital_usdc: u64,
    pub committed_to_lines_usdc: u64,
    pub principal_drawn_usdc: u64,
    pub principal_repaid_usdc: u64,
    pub interest_repaid_usdc: u64,
    pub platform_fees_paid_usdc: u64,
    pub max_single_line_usdc: u64,
    pub min_trust_score: u16,
    pub max_apr_bps: u16,
    pub auto_approve_under_usdc: u64,
    pub created_at: i64,
    pub bump: u8,
    pub authority_bump: u8,
    pub usdc_vault_bump: u8,
}

impl LiquidityPool {
    pub const LEN: usize = 336;
}

#[account]
pub struct BorrowerProfile {
    pub operator: Pubkey,
    pub primary_wallet: Pubkey,
    pub metadata_hash: [u8; 32],
    pub borrower_type: BorrowerType,
    pub verification_status: VerificationStatus,
    pub status: BorrowerStatus,
    pub trust_score: u16,
    pub total_credit_limit_usdc: u64,
    pub total_outstanding_usdc: u64,
    pub repayment_count: u32,
    pub delinquency_count: u32,
    pub created_at: i64,
    pub bump: u8,
}

impl BorrowerProfile {
    pub const LEN: usize = 160;
}

#[account]
pub struct CreditApplication {
    pub borrower: Pubkey,
    pub liquidity_pool: Pubkey,
    pub application_seed: [u8; 16],
    pub metadata_hash: [u8; 32],
    pub requested_limit_usdc: u64,
    pub proposed_apr_bps: u16,
    pub collateralization_bps: u16,
    pub trust_score_snapshot: u16,
    pub status: ApplicationStatus,
    pub approval_mode: ApprovalMode,
    pub created_at: i64,
    pub reviewed_at: i64,
    pub reviewer: Pubkey,
    pub bump: u8,
}

impl CreditApplication {
    pub const LEN: usize = 208;
}

#[account]
pub struct CreditLine {
    pub borrower: Pubkey,
    pub liquidity_pool: Pubkey,
    pub credit_application: Pubkey,
    pub borrower_wallet: Pubkey,
    pub borrower_usdc: Pubkey,
    pub line_seed: [u8; 16],
    pub metadata_hash: [u8; 32],
    pub principal_limit_usdc: u64,
    pub available_limit_usdc: u64,
    pub principal_outstanding_usdc: u64,
    pub principal_repaid_usdc: u64,
    pub interest_paid_usdc: u64,
    pub platform_fees_paid_usdc: u64,
    pub apr_bps: u16,
    pub platform_fee_bps: u16,
    pub collateralization_bps: u16,
    pub status: CreditLineStatus,
    pub risk_grade: RiskGrade,
    pub maturity_at: i64,
    pub created_at: i64,
    pub last_draw_at: i64,
    pub last_repayment_at: i64,
    pub bump: u8,
}

impl CreditLine {
    pub const LEN: usize = 336;
}

#[account]
pub struct CreditAttestation {
    pub borrower: Pubkey,
    pub credit_line: Pubkey,
    pub score_hash: [u8; 32],
    pub score: u16,
    pub trust_score: u16,
    pub risk_grade: RiskGrade,
    pub recommended_limit_usdc: u64,
    pub recommended_apr_bps: u16,
    pub pd_estimate_bps: u16,
    pub lgd_estimate_bps: u16,
    pub confidence_bps: u16,
    pub features_hash: [u8; 32],
    pub created_at: i64,
    pub bump: u8,
}

impl CreditAttestation {
    pub const LEN: usize = 192;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum BorrowerType {
    Business,
    Neobank,
    Protocol,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum VerificationStatus {
    Unverified,
    Documents,
    ZkVerified,
    ManuallyVerified,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum BorrowerStatus {
    Active,
    Suspended,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum PoolStatus {
    Active,
    Paused,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalMode {
    Manual,
    Rules,
    Agentic,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum ApplicationStatus {
    Pending,
    Approved,
    Rejected,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum CreditLineStatus {
    Active,
    Repaid,
    Suspended,
    Defaulted,
    Closed,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum RiskGrade {
    A,
    B,
    C,
    D,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeMarketArgs {
    pub risk_admin: Pubkey,
    pub emergency_admin: Pubkey,
    pub platform_fee_bps: u16,
    pub max_total_credit_usdc: u64,
    pub max_single_borrower_usdc: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct RegisterBorrowerArgs {
    pub primary_wallet: Pubkey,
    pub metadata_hash: [u8; 32],
    pub borrower_type: BorrowerType,
    pub verification_status: VerificationStatus,
    pub trust_score: u16,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct UpdateBorrowerVerificationArgs {
    pub metadata_hash: [u8; 32],
    pub verification_status: VerificationStatus,
    pub trust_score: u16,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CreateLiquidityPoolArgs {
    pub metadata_hash: [u8; 32],
    pub policy_hash: [u8; 32],
    pub approval_mode: ApprovalMode,
    pub max_single_line_usdc: u64,
    pub min_trust_score: u16,
    pub max_apr_bps: u16,
    pub auto_approve_under_usdc: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct SetPoolPolicyArgs {
    pub policy_hash: [u8; 32],
    pub approval_mode: ApprovalMode,
    pub max_single_line_usdc: u64,
    pub min_trust_score: u16,
    pub max_apr_bps: u16,
    pub auto_approve_under_usdc: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct SubmitCreditApplicationArgs {
    pub application_seed: [u8; 16],
    pub metadata_hash: [u8; 32],
    pub requested_limit_usdc: u64,
    pub proposed_apr_bps: u16,
    pub collateralization_bps: u16,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ApproveCreditApplicationArgs {
    pub line_seed: [u8; 16],
    pub metadata_hash: [u8; 32],
    pub approved_limit_usdc: u64,
    pub apr_bps: u16,
    pub tenor_seconds: i64,
    pub risk_grade: RiskGrade,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct UpdateCreditAttestationArgs {
    pub score_hash: [u8; 32],
    pub score: u16,
    pub trust_score: u16,
    pub risk_grade: RiskGrade,
    pub recommended_limit_usdc: u64,
    pub recommended_apr_bps: u16,
    pub pd_estimate_bps: u16,
    pub lgd_estimate_bps: u16,
    pub confidence_bps: u16,
    pub features_hash: [u8; 32],
}

#[event]
pub struct MarketInitialized {
    pub market: Pubkey,
    pub admin: Pubkey,
    pub usdc_mint: Pubkey,
    pub platform_fee_bps: u16,
}

#[event]
pub struct MarketPauseUpdated {
    pub paused: bool,
}

#[event]
pub struct BorrowerRegistered {
    pub borrower: Pubkey,
    pub operator: Pubkey,
    pub trust_score: u16,
}

#[event]
pub struct BorrowerVerificationUpdated {
    pub borrower: Pubkey,
    pub verification_status: VerificationStatus,
    pub trust_score: u16,
}

#[event]
pub struct LiquidityPoolCreated {
    pub liquidity_pool: Pubkey,
    pub manager: Pubkey,
    pub approval_mode: ApprovalMode,
    pub max_single_line_usdc: u64,
}

#[event]
pub struct LiquidityPoolFunded {
    pub liquidity_pool: Pubkey,
    pub amount_usdc: u64,
    pub available_capital_usdc: u64,
}

#[event]
pub struct PoolPolicyUpdated {
    pub liquidity_pool: Pubkey,
    pub approval_mode: ApprovalMode,
    pub min_trust_score: u16,
    pub max_apr_bps: u16,
}

#[event]
pub struct CreditApplicationSubmitted {
    pub application: Pubkey,
    pub borrower: Pubkey,
    pub liquidity_pool: Pubkey,
    pub requested_limit_usdc: u64,
}

#[event]
pub struct CreditLineDeployed {
    pub credit_line: Pubkey,
    pub application: Pubkey,
    pub borrower: Pubkey,
    pub liquidity_pool: Pubkey,
    pub principal_limit_usdc: u64,
    pub apr_bps: u16,
    pub risk_grade: RiskGrade,
}

#[event]
pub struct CreditDrawn {
    pub credit_line: Pubkey,
    pub borrower: Pubkey,
    pub amount_usdc: u64,
    pub principal_outstanding_usdc: u64,
}

#[event]
pub struct CreditRepaid {
    pub credit_line: Pubkey,
    pub borrower: Pubkey,
    pub principal_repayment_usdc: u64,
    pub platform_fee_usdc: u64,
    pub remaining_outstanding_usdc: u64,
}

#[event]
pub struct CreditAttestationUpdated {
    pub credit_attestation: Pubkey,
    pub credit_line: Pubkey,
    pub borrower: Pubkey,
    pub trust_score: u16,
    pub risk_grade: RiskGrade,
}

#[event]
pub struct CreditLineSuspended {
    pub credit_line: Pubkey,
}

#[event]
pub struct CreditLineResumed {
    pub credit_line: Pubkey,
}

#[event]
pub struct CreditLineDefaulted {
    pub credit_line: Pubkey,
    pub loss_usdc: u64,
}

#[event]
pub struct CreditLineClosed {
    pub credit_line: Pubkey,
    pub liquidity_pool: Pubkey,
}

fn current_timestamp() -> Result<i64> {
    Ok(Clock::get()?.unix_timestamp)
}

fn calculate_bps(amount: u64, bps: u16) -> Result<u64> {
    amount
        .checked_mul(bps as u64)
        .ok_or(CreditMarketError::MathOverflow)?
        .checked_div(BPS_DENOMINATOR)
        .ok_or(CreditMarketError::MathOverflow.into())
}

fn checked_add(left: u64, right: u64) -> Result<u64> {
    left.checked_add(right).ok_or(CreditMarketError::MathOverflow.into())
}

fn checked_sub(left: u64, right: u64) -> Result<u64> {
    left.checked_sub(right).ok_or(CreditMarketError::MathOverflow.into())
}

fn assert_admin_or_emergency(market: &MarketConfig, authority: Pubkey) -> Result<()> {
    require!(
        authority == market.admin || authority == market.emergency_admin,
        CreditMarketError::Unauthorized
    );
    Ok(())
}

fn assert_risk_authority(market: &MarketConfig, authority: Pubkey) -> Result<()> {
    require!(
        authority == market.admin || authority == market.risk_admin,
        CreditMarketError::Unauthorized
    );
    Ok(())
}

fn assert_pool_manager_or_risk(market: &MarketConfig, pool: &LiquidityPool, authority: Pubkey) -> Result<()> {
    require!(
        authority == pool.manager || authority == market.admin || authority == market.risk_admin,
        CreditMarketError::Unauthorized
    );
    Ok(())
}

#[error_code]
pub enum CreditMarketError {
    #[msg("Market is paused.")]
    MarketPaused,
    #[msg("Unauthorized signer.")]
    Unauthorized,
    #[msg("Invalid amount.")]
    InvalidAmount,
    #[msg("Invalid limit.")]
    InvalidLimit,
    #[msg("Invalid basis-point value.")]
    InvalidBps,
    #[msg("Invalid score.")]
    InvalidScore,
    #[msg("Math overflow.")]
    MathOverflow,
    #[msg("Borrower is not active.")]
    BorrowerNotActive,
    #[msg("Liquidity pool is not active.")]
    PoolNotActive,
    #[msg("Liquidity pool limit exceeded.")]
    PoolLimitExceeded,
    #[msg("Borrower limit exceeded.")]
    BorrowerLimitExceeded,
    #[msg("Market limit exceeded.")]
    MarketLimitExceeded,
    #[msg("Insufficient liquidity.")]
    InsufficientLiquidity,
    #[msg("Trust score is below the pool policy minimum.")]
    TrustScoreTooLow,
    #[msg("Invalid application status.")]
    InvalidApplicationStatus,
    #[msg("Invalid credit line status.")]
    InvalidCreditLineStatus,
    #[msg("Credit line is not active.")]
    CreditLineNotActive,
    #[msg("Insufficient available credit.")]
    InsufficientCredit,
    #[msg("Nothing to repay.")]
    NothingToRepay,
    #[msg("Outstanding balance remains.")]
    OutstandingBalance,
    #[msg("Invalid account.")]
    InvalidAccount,
    #[msg("Invalid token account.")]
    InvalidTokenAccount,
}
