#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

declare_id!("6GCCBywkt8WwNoaroggSwnMkd8ggWvLNBzpWnFXWWR6n");

const PROTOCOL_SEED: &[u8] = b"protocol";
const BORROWER_SEED: &[u8] = b"borrower";
const AGENT_SEED: &[u8] = b"agent";
const WORKFLOW_SEED: &[u8] = b"workflow";
const MERCHANT_SEED: &[u8] = b"merchant";
const POLICY_SEED: &[u8] = b"policy";
const UNDERWRITER_VAULT_SEED: &[u8] = b"underwriter-vault";
const UNDERWRITER_VAULT_AUTHORITY_SEED: &[u8] = b"underwriter-vault-authority";
const UNDERWRITER_VAULT_USDC_SEED: &[u8] = b"underwriter-vault-usdc";
const CREDIT_LINE_SEED: &[u8] = b"credit-line";
const SPEND_AUTH_SEED: &[u8] = b"spend-auth";
const SCORE_SEED: &[u8] = b"score";

const BPS_DENOMINATOR: u64 = 10_000;
const DAY_SECONDS: u64 = 86_400;
const WEEK_SECONDS: u64 = 604_800;
const MAX_LIST_ITEMS: usize = 8;

#[program]
pub mod agc_credit_control {
    use super::*;

    pub fn initialize_protocol(
        ctx: Context<InitializeProtocol>,
        args: InitializeProtocolArgs,
    ) -> Result<()> {
        require!(args.max_total_live_capital_at_risk_usdc > 0, AgcCreditError::InvalidLimit);
        require!(args.max_single_borrower_limit_usdc > 0, AgcCreditError::InvalidLimit);
        require!(args.max_daily_total_spend_usdc > 0, AgcCreditError::InvalidLimit);
        require!(
            args.max_single_borrower_limit_usdc <= args.max_total_live_capital_at_risk_usdc,
            AgcCreditError::InvalidLimit
        );

        let config = &mut ctx.accounts.protocol_config;
        config.admin = ctx.accounts.admin.key();
        config.risk_admin = args.risk_admin;
        config.emergency_admin = args.emergency_admin;
        config.payment_router = args.payment_router;
        config.usdc_mint = ctx.accounts.usdc_mint.key();
        config.max_total_live_capital_at_risk_usdc = args.max_total_live_capital_at_risk_usdc;
        config.max_single_borrower_limit_usdc = args.max_single_borrower_limit_usdc;
        config.max_unsecured_exposure_per_borrower_usdc =
            args.max_unsecured_exposure_per_borrower_usdc;
        config.max_daily_total_spend_usdc = args.max_daily_total_spend_usdc;
        config.max_loss_budget_usdc = args.max_loss_budget_usdc;
        config.total_live_capital_at_risk_usdc = 0;
        config.total_daily_spend_usdc = 0;
        config.total_daily_spend_window_started_at = current_timestamp()?;
        config.total_defaulted_usdc = 0;
        config.paused = false;
        config.bump = ctx.bumps.protocol_config;

        emit!(ProtocolInitialized {
            admin: config.admin,
            risk_admin: config.risk_admin,
            payment_router: config.payment_router,
            usdc_mint: config.usdc_mint,
        });

        Ok(())
    }

    pub fn set_protocol_pause(ctx: Context<AdminOnly>, paused: bool) -> Result<()> {
        assert_admin(&ctx.accounts.protocol_config, ctx.accounts.authority.key())?;
        ctx.accounts.protocol_config.paused = paused;
        emit!(ProtocolPauseUpdated { paused });
        Ok(())
    }

    pub fn register_borrower(
        ctx: Context<RegisterBorrower>,
        args: RegisterBorrowerArgs,
    ) -> Result<()> {
        require!(!ctx.accounts.protocol_config.paused, AgcCreditError::ProtocolPaused);

        let borrower = &mut ctx.accounts.borrower;
        borrower.operator = ctx.accounts.operator.key();
        borrower.primary_wallet = args.primary_wallet;
        borrower.metadata_hash = args.metadata_hash;
        borrower.borrower_type = args.borrower_type;
        borrower.verification_status = args.verification_status;
        borrower.status = BorrowerStatus::Active;
        borrower.total_principal_limit_usdc = 0;
        borrower.total_outstanding_usdc = 0;
        borrower.created_at = current_timestamp()?;
        borrower.bump = ctx.bumps.borrower;

        emit!(BorrowerRegistered {
            borrower: borrower.key(),
            operator: borrower.operator,
            verification_status: borrower.verification_status,
        });

        Ok(())
    }

    pub fn register_agent(ctx: Context<RegisterAgent>, args: RegisterAgentArgs) -> Result<()> {
        require!(!ctx.accounts.protocol_config.paused, AgcCreditError::ProtocolPaused);
        require_keys_eq!(
            ctx.accounts.borrower.operator,
            ctx.accounts.operator.key(),
            AgcCreditError::Unauthorized
        );
        require!(
            ctx.accounts.borrower.status == BorrowerStatus::Active,
            AgcCreditError::BorrowerNotActive
        );

        let agent = &mut ctx.accounts.agent;
        agent.borrower = ctx.accounts.borrower.key();
        agent.operator = ctx.accounts.operator.key();
        agent.wallet = args.wallet;
        agent.metadata_hash = args.metadata_hash;
        agent.framework = args.framework;
        agent.status = AgentStatus::Active;
        agent.created_at = current_timestamp()?;
        agent.last_active_at = agent.created_at;
        agent.bump = ctx.bumps.agent;

        emit!(AgentRegistered {
            borrower: agent.borrower,
            agent: agent.key(),
            wallet: agent.wallet,
        });

        Ok(())
    }

    pub fn create_workflow(
        ctx: Context<CreateWorkflow>,
        args: CreateWorkflowArgs,
    ) -> Result<()> {
        require!(!ctx.accounts.protocol_config.paused, AgcCreditError::ProtocolPaused);
        require_keys_eq!(
            ctx.accounts.agent.operator,
            ctx.accounts.operator.key(),
            AgcCreditError::Unauthorized
        );
        require!(ctx.accounts.agent.status == AgentStatus::Active, AgcCreditError::AgentNotActive);

        let workflow = &mut ctx.accounts.workflow;
        workflow.agent = ctx.accounts.agent.key();
        workflow.borrower = ctx.accounts.agent.borrower;
        workflow.workflow_seed = args.workflow_seed;
        workflow.metadata_hash = args.metadata_hash;
        workflow.status = WorkflowStatus::Active;
        workflow.policy = Pubkey::default();
        workflow.created_at = current_timestamp()?;
        workflow.bump = ctx.bumps.workflow;

        emit!(WorkflowCreated {
            workflow: workflow.key(),
            agent: workflow.agent,
            borrower: workflow.borrower,
        });

        Ok(())
    }

    pub fn register_merchant(
        ctx: Context<RegisterMerchant>,
        args: RegisterMerchantArgs,
    ) -> Result<()> {
        assert_risk_authority(&ctx.accounts.protocol_config, ctx.accounts.authority.key())?;
        require!(args.category != 0, AgcCreditError::InvalidCategory);

        let merchant = &mut ctx.accounts.merchant;
        merchant.merchant_id_hash = args.merchant_id_hash;
        merchant.metadata_hash = args.metadata_hash;
        merchant.category = args.category;
        merchant.status = args.status;
        merchant.adapter = args.adapter;
        merchant.created_at = current_timestamp()?;
        merchant.bump = ctx.bumps.merchant;

        emit!(MerchantRegistered {
            merchant: merchant.key(),
            category: merchant.category,
            status: merchant.status,
        });

        Ok(())
    }

    pub fn create_spend_policy(
        ctx: Context<CreateSpendPolicy>,
        args: CreateSpendPolicyArgs,
    ) -> Result<()> {
        require!(!ctx.accounts.protocol_config.paused, AgcCreditError::ProtocolPaused);
        require_keys_eq!(
            ctx.accounts.workflow.borrower,
            ctx.accounts.borrower.key(),
            AgcCreditError::InvalidAccount
        );
        require_keys_eq!(
            ctx.accounts.borrower.operator,
            ctx.accounts.operator.key(),
            AgcCreditError::Unauthorized
        );
        validate_policy_args(&args)?;

        let now = current_timestamp()?;
        let policy = &mut ctx.accounts.policy;
        policy.workflow = ctx.accounts.workflow.key();
        policy.borrower = ctx.accounts.borrower.key();
        policy.version = 1;
        policy.metadata_hash = args.metadata_hash;
        policy.max_per_transaction_usdc = args.max_per_transaction_usdc;
        policy.max_daily_spend_usdc = args.max_daily_spend_usdc;
        policy.max_weekly_spend_usdc = args.max_weekly_spend_usdc;
        policy.allowed_merchants = args.allowed_merchants;
        policy.allowed_merchant_count = args.allowed_merchant_count;
        policy.blocked_merchants = args.blocked_merchants;
        policy.blocked_merchant_count = args.blocked_merchant_count;
        policy.allowed_categories = args.allowed_categories;
        policy.allowed_category_count = args.allowed_category_count;
        policy.human_approval_threshold_usdc = args.human_approval_threshold_usdc;
        policy.revenue_sweep_bps = args.revenue_sweep_bps;
        policy.min_available_limit_after_spend_usdc = args.min_available_limit_after_spend_usdc;
        policy.cooldown_after_policy_violation_seconds =
            args.cooldown_after_policy_violation_seconds;
        policy.daily_spend_usdc = 0;
        policy.weekly_spend_usdc = 0;
        policy.daily_window_started_at = now;
        policy.weekly_window_started_at = now;
        policy.violation_count = 0;
        policy.last_violation_at = 0;
        policy.status = PolicyStatus::Active;
        policy.valid_from = now;
        policy.valid_to = 0;
        policy.bump = ctx.bumps.policy;

        ctx.accounts.workflow.policy = policy.key();

        emit!(PolicyCreated {
            policy: policy.key(),
            workflow: policy.workflow,
            version: policy.version,
            revenue_sweep_bps: policy.revenue_sweep_bps,
        });

        Ok(())
    }

    pub fn create_underwriter_vault(
        ctx: Context<CreateUnderwriterVault>,
        args: CreateUnderwriterVaultArgs,
    ) -> Result<()> {
        require!(!ctx.accounts.protocol_config.paused, AgcCreditError::ProtocolPaused);
        require!(
            args.max_single_line_usdc > 0,
            AgcCreditError::InvalidLimit
        );

        let vault = &mut ctx.accounts.underwriter_vault;
        vault.underwriter = ctx.accounts.underwriter.key();
        vault.usdc_mint = ctx.accounts.protocol_config.usdc_mint;
        vault.usdc_vault = ctx.accounts.usdc_vault.key();
        vault.metadata_hash = args.metadata_hash;
        vault.status = UnderwriterVaultStatus::Active;
        vault.committed_capital_usdc = 0;
        vault.available_capital_usdc = 0;
        vault.committed_to_lines_usdc = 0;
        vault.principal_drawn_usdc = 0;
        vault.principal_repaid_usdc = 0;
        vault.interest_earned_usdc = 0;
        vault.loss_realized_usdc = 0;
        vault.max_single_line_usdc = args.max_single_line_usdc;
        vault.bump = ctx.bumps.underwriter_vault;
        vault.authority_bump = ctx.bumps.vault_authority;
        vault.usdc_vault_bump = ctx.bumps.usdc_vault;

        emit!(UnderwriterVaultCreated {
            underwriter_vault: vault.key(),
            underwriter: vault.underwriter,
            usdc_vault: vault.usdc_vault,
        });

        Ok(())
    }

    pub fn fund_underwriter_vault(
        ctx: Context<FundUnderwriterVault>,
        amount_usdc: u64,
    ) -> Result<()> {
        require!(amount_usdc > 0, AgcCreditError::InvalidAmount);
        require!(
            ctx.accounts.underwriter_vault.status == UnderwriterVaultStatus::Active,
            AgcCreditError::UnderwriterVaultNotActive
        );
        require_keys_eq!(
            ctx.accounts.underwriter_vault.underwriter,
            ctx.accounts.underwriter.key(),
            AgcCreditError::Unauthorized
        );

        token::transfer(ctx.accounts.fund_transfer_ctx(), amount_usdc)?;

        let vault = &mut ctx.accounts.underwriter_vault;
        vault.committed_capital_usdc = vault
            .committed_capital_usdc
            .checked_add(amount_usdc)
            .ok_or(AgcCreditError::MathOverflow)?;
        vault.available_capital_usdc = vault
            .available_capital_usdc
            .checked_add(amount_usdc)
            .ok_or(AgcCreditError::MathOverflow)?;

        emit!(UnderwriterVaultFunded {
            underwriter_vault: vault.key(),
            amount_usdc,
            available_capital_usdc: vault.available_capital_usdc,
        });

        Ok(())
    }

    pub fn approve_credit_line(
        ctx: Context<ApproveCreditLine>,
        args: ApproveCreditLineArgs,
    ) -> Result<()> {
        assert_risk_authority(&ctx.accounts.protocol_config, ctx.accounts.authority.key())?;
        require!(!ctx.accounts.protocol_config.paused, AgcCreditError::ProtocolPaused);
        require!(args.principal_limit_usdc > 0, AgcCreditError::InvalidLimit);
        require!(args.tenor_seconds > 0, AgcCreditError::InvalidTenor);
        require!(args.grace_period_seconds > 0, AgcCreditError::InvalidTenor);
        require!(args.risk_grade != RiskGrade::D, AgcCreditError::RiskGradeNotLive);
        require!(
            ctx.accounts.borrower.status == BorrowerStatus::Active,
            AgcCreditError::BorrowerNotActive
        );
        require!(
            ctx.accounts.workflow.status == WorkflowStatus::Active,
            AgcCreditError::WorkflowNotActive
        );
        require!(
            ctx.accounts.policy.status == PolicyStatus::Active,
            AgcCreditError::PolicyNotActive
        );
        require_keys_eq!(
            ctx.accounts.workflow.borrower,
            ctx.accounts.borrower.key(),
            AgcCreditError::InvalidAccount
        );
        require_keys_eq!(
            ctx.accounts.workflow.policy,
            ctx.accounts.policy.key(),
            AgcCreditError::InvalidAccount
        );
        require!(
            ctx.accounts.underwriter_vault.status == UnderwriterVaultStatus::Active,
            AgcCreditError::UnderwriterVaultNotActive
        );
        require!(
            args.principal_limit_usdc <= ctx.accounts.underwriter_vault.max_single_line_usdc,
            AgcCreditError::UnderwriterLimitExceeded
        );
        require!(
            args.principal_limit_usdc <= ctx.accounts.protocol_config.max_single_borrower_limit_usdc,
            AgcCreditError::GlobalLimitExceeded
        );
        require!(
            ctx.accounts.borrower.total_principal_limit_usdc
                .checked_add(args.principal_limit_usdc)
                .ok_or(AgcCreditError::MathOverflow)?
                <= ctx.accounts.protocol_config.max_single_borrower_limit_usdc,
            AgcCreditError::BorrowerLimitExceeded
        );
        require!(
            ctx.accounts.protocol_config.total_live_capital_at_risk_usdc
                .checked_add(args.principal_limit_usdc)
                .ok_or(AgcCreditError::MathOverflow)?
                <= ctx.accounts.protocol_config.max_total_live_capital_at_risk_usdc,
            AgcCreditError::GlobalLimitExceeded
        );
        require!(
            ctx.accounts.protocol_config.total_defaulted_usdc
                < ctx.accounts.protocol_config.max_loss_budget_usdc,
            AgcCreditError::GlobalLimitExceeded
        );
        require!(
            ctx.accounts.borrower.total_principal_limit_usdc
                .checked_add(args.principal_limit_usdc)
                .ok_or(AgcCreditError::MathOverflow)?
                <= ctx.accounts.protocol_config.max_unsecured_exposure_per_borrower_usdc,
            AgcCreditError::BorrowerLimitExceeded
        );
        require!(
            ctx.accounts.underwriter_vault.available_capital_usdc >= args.principal_limit_usdc,
            AgcCreditError::InsufficientUnderwriterCapital
        );

        let now = current_timestamp()?;
        let line = &mut ctx.accounts.credit_line;
        line.borrower = ctx.accounts.borrower.key();
        line.agent = ctx.accounts.workflow.agent;
        line.workflow = ctx.accounts.workflow.key();
        line.underwriter_vault = ctx.accounts.underwriter_vault.key();
        line.policy = ctx.accounts.policy.key();
        line.repayment_rule = args.repayment_rule;
        line.score_attestation = Pubkey::default();
        line.line_seed = args.line_seed;
        line.metadata_hash = args.metadata_hash;
        line.principal_limit_usdc = args.principal_limit_usdc;
        line.available_limit_usdc = args.principal_limit_usdc;
        line.reserved_spend_usdc = 0;
        line.principal_outstanding_usdc = 0;
        line.accrued_interest_usdc = 0;
        line.fees_due_usdc = calculate_bps(args.principal_limit_usdc, args.origination_fee_bps)?;
        line.apr_bps = args.apr_bps;
        line.origination_fee_bps = args.origination_fee_bps;
        line.tenor_seconds = args.tenor_seconds;
        line.maturity_at = now
            .checked_add(args.tenor_seconds)
            .ok_or(AgcCreditError::MathOverflow)?;
        line.grace_period_seconds = args.grace_period_seconds;
        line.status = CreditLineStatus::Approved;
        line.risk_grade = args.risk_grade;
        line.created_at = now;
        line.activated_at = 0;
        line.last_repayment_at = 0;
        line.defaulted_at = 0;
        line.bump = ctx.bumps.credit_line;

        let vault = &mut ctx.accounts.underwriter_vault;
        vault.available_capital_usdc = vault
            .available_capital_usdc
            .checked_sub(args.principal_limit_usdc)
            .ok_or(AgcCreditError::MathOverflow)?;
        vault.committed_to_lines_usdc = vault
            .committed_to_lines_usdc
            .checked_add(args.principal_limit_usdc)
            .ok_or(AgcCreditError::MathOverflow)?;

        let borrower = &mut ctx.accounts.borrower;
        borrower.total_principal_limit_usdc = borrower
            .total_principal_limit_usdc
            .checked_add(args.principal_limit_usdc)
            .ok_or(AgcCreditError::MathOverflow)?;

        ctx.accounts.protocol_config.total_live_capital_at_risk_usdc = ctx
            .accounts
            .protocol_config
            .total_live_capital_at_risk_usdc
            .checked_add(args.principal_limit_usdc)
            .ok_or(AgcCreditError::MathOverflow)?;

        emit!(CreditLineApproved {
            credit_line: line.key(),
            borrower: line.borrower,
            workflow: line.workflow,
            principal_limit_usdc: line.principal_limit_usdc,
            apr_bps: line.apr_bps,
            risk_grade: line.risk_grade,
        });

        Ok(())
    }

    pub fn activate_credit_line(ctx: Context<CreditLineRiskAction>) -> Result<()> {
        assert_risk_authority(&ctx.accounts.protocol_config, ctx.accounts.authority.key())?;
        require!(
            ctx.accounts.credit_line.status == CreditLineStatus::Approved,
            AgcCreditError::InvalidCreditLineStatus
        );
        let now = current_timestamp()?;
        ctx.accounts.credit_line.status = CreditLineStatus::Active;
        ctx.accounts.credit_line.activated_at = now;
        emit!(CreditLineActivated {
            credit_line: ctx.accounts.credit_line.key(),
            activated_at: now,
        });
        Ok(())
    }

    pub fn reserve_spend(ctx: Context<ReserveSpend>, args: ReserveSpendArgs) -> Result<()> {
        assert_router_or_risk_authority(&ctx.accounts.protocol_config, ctx.accounts.router.key())?;
        require!(!ctx.accounts.protocol_config.paused, AgcCreditError::ProtocolPaused);
        require!(args.amount_usdc > 0, AgcCreditError::InvalidAmount);

        let now = current_timestamp()?;
        reset_policy_windows(&mut ctx.accounts.policy, now);
        reset_protocol_daily_window(&mut ctx.accounts.protocol_config, now);

        validate_spend_request(
            &ctx.accounts.protocol_config,
            &ctx.accounts.borrower,
            &ctx.accounts.credit_line,
            &ctx.accounts.policy,
            &ctx.accounts.merchant,
            ctx.accounts.merchant.key(),
            args.amount_usdc,
            now,
        )?;

        let authorization = &mut ctx.accounts.spend_authorization;
        authorization.credit_line = ctx.accounts.credit_line.key();
        authorization.workflow = ctx.accounts.credit_line.workflow;
        authorization.agent = ctx.accounts.credit_line.agent;
        authorization.merchant = ctx.accounts.merchant.key();
        authorization.spend_id = args.spend_id;
        authorization.amount_usdc = args.amount_usdc;
        authorization.purpose_hash = args.purpose_hash;
        authorization.policy_version = ctx.accounts.policy.version;
        authorization.status = SpendAuthorizationStatus::Reserved;
        authorization.reason_code = SpendDecisionCode::Approved;
        authorization.created_at = now;
        authorization.expires_at = now
            .checked_add(args.authorization_ttl_seconds)
            .ok_or(AgcCreditError::MathOverflow)?;
        authorization.bump = ctx.bumps.spend_authorization;

        let line = &mut ctx.accounts.credit_line;
        line.available_limit_usdc = line
            .available_limit_usdc
            .checked_sub(args.amount_usdc)
            .ok_or(AgcCreditError::MathOverflow)?;
        line.reserved_spend_usdc = line
            .reserved_spend_usdc
            .checked_add(args.amount_usdc)
            .ok_or(AgcCreditError::MathOverflow)?;

        let policy = &mut ctx.accounts.policy;
        policy.daily_spend_usdc = policy
            .daily_spend_usdc
            .checked_add(args.amount_usdc)
            .ok_or(AgcCreditError::MathOverflow)?;
        policy.weekly_spend_usdc = policy
            .weekly_spend_usdc
            .checked_add(args.amount_usdc)
            .ok_or(AgcCreditError::MathOverflow)?;

        ctx.accounts.protocol_config.total_daily_spend_usdc = ctx
            .accounts
            .protocol_config
            .total_daily_spend_usdc
            .checked_add(args.amount_usdc)
            .ok_or(AgcCreditError::MathOverflow)?;

        emit!(SpendApproved {
            spend_authorization: authorization.key(),
            credit_line: authorization.credit_line,
            merchant: authorization.merchant,
            amount_usdc: authorization.amount_usdc,
            policy_version: authorization.policy_version,
        });

        Ok(())
    }

    pub fn cancel_reserved_spend(ctx: Context<CancelReservedSpend>) -> Result<()> {
        assert_router_or_risk_authority(&ctx.accounts.protocol_config, ctx.accounts.router.key())?;
        require!(
            ctx.accounts.spend_authorization.status == SpendAuthorizationStatus::Reserved,
            AgcCreditError::SpendAlreadyFinalized
        );
        require_keys_eq!(
            ctx.accounts.spend_authorization.credit_line,
            ctx.accounts.credit_line.key(),
            AgcCreditError::InvalidAccount
        );

        let amount = ctx.accounts.spend_authorization.amount_usdc;
        ctx.accounts.credit_line.reserved_spend_usdc = ctx
            .accounts
            .credit_line
            .reserved_spend_usdc
            .checked_sub(amount)
            .ok_or(AgcCreditError::MathOverflow)?;
        ctx.accounts.credit_line.available_limit_usdc = ctx
            .accounts
            .credit_line
            .available_limit_usdc
            .checked_add(amount)
            .ok_or(AgcCreditError::MathOverflow)?;
        ctx.accounts.spend_authorization.status = SpendAuthorizationStatus::Canceled;

        emit!(SpendCanceled {
            spend_authorization: ctx.accounts.spend_authorization.key(),
            credit_line: ctx.accounts.credit_line.key(),
            amount_usdc: amount,
        });

        Ok(())
    }

    pub fn settle_spend(ctx: Context<SettleSpend>) -> Result<()> {
        assert_router_or_risk_authority(&ctx.accounts.protocol_config, ctx.accounts.router.key())?;
        require!(!ctx.accounts.protocol_config.paused, AgcCreditError::ProtocolPaused);
        require!(
            ctx.accounts.credit_line.status == CreditLineStatus::Active,
            AgcCreditError::CreditLineNotActive
        );
        require!(
            ctx.accounts.spend_authorization.status == SpendAuthorizationStatus::Reserved,
            AgcCreditError::SpendAlreadyFinalized
        );
        require!(
            current_timestamp()? <= ctx.accounts.spend_authorization.expires_at,
            AgcCreditError::SpendAuthorizationExpired
        );
        require_keys_eq!(
            ctx.accounts.spend_authorization.credit_line,
            ctx.accounts.credit_line.key(),
            AgcCreditError::InvalidAccount
        );
        require_keys_eq!(
            ctx.accounts.credit_line.borrower,
            ctx.accounts.borrower.key(),
            AgcCreditError::InvalidAccount
        );
        require_keys_eq!(
            ctx.accounts.credit_line.underwriter_vault,
            ctx.accounts.underwriter_vault.key(),
            AgcCreditError::InvalidAccount
        );
        require_keys_eq!(
            ctx.accounts.underwriter_vault.usdc_vault,
            ctx.accounts.underwriter_usdc_vault.key(),
            AgcCreditError::InvalidTokenAccount
        );

        let amount = ctx.accounts.spend_authorization.amount_usdc;
        let underwriter_vault_key = ctx.accounts.underwriter_vault.key();
        let authority_bump = ctx.accounts.underwriter_vault.authority_bump;
        let signer_seeds: &[&[&[u8]]] = &[&[
            UNDERWRITER_VAULT_AUTHORITY_SEED,
            underwriter_vault_key.as_ref(),
            &[authority_bump],
        ]];
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.key(),
                Transfer {
                    from: ctx.accounts.underwriter_usdc_vault.to_account_info(),
                    to: ctx.accounts.merchant_usdc.to_account_info(),
                    authority: ctx.accounts.vault_authority.to_account_info(),
                },
                signer_seeds,
            ),
            amount,
        )?;

        let line = &mut ctx.accounts.credit_line;
        line.reserved_spend_usdc = line
            .reserved_spend_usdc
            .checked_sub(amount)
            .ok_or(AgcCreditError::MathOverflow)?;
        line.principal_outstanding_usdc = line
            .principal_outstanding_usdc
            .checked_add(amount)
            .ok_or(AgcCreditError::MathOverflow)?;

        ctx.accounts.borrower.total_outstanding_usdc = ctx
            .accounts
            .borrower
            .total_outstanding_usdc
            .checked_add(amount)
            .ok_or(AgcCreditError::MathOverflow)?;
        ctx.accounts.underwriter_vault.principal_drawn_usdc = ctx
            .accounts
            .underwriter_vault
            .principal_drawn_usdc
            .checked_add(amount)
            .ok_or(AgcCreditError::MathOverflow)?;
        ctx.accounts.spend_authorization.status = SpendAuthorizationStatus::Settled;

        emit!(PaymentSettled {
            spend_authorization: ctx.accounts.spend_authorization.key(),
            credit_line: ctx.accounts.credit_line.key(),
            merchant: ctx.accounts.spend_authorization.merchant,
            merchant_token_account: ctx.accounts.merchant_usdc.key(),
            amount_usdc: amount,
        });

        Ok(())
    }

    pub fn record_revenue_and_sweep(
        ctx: Context<RecordRevenueAndSweep>,
        amount_usdc: u64,
        source: RevenueSource,
        source_hash: [u8; 32],
    ) -> Result<()> {
        require!(amount_usdc > 0, AgcCreditError::InvalidAmount);
        require_keys_eq!(
            ctx.accounts.credit_line.policy,
            ctx.accounts.policy.key(),
            AgcCreditError::InvalidAccount
        );
        require_keys_eq!(
            ctx.accounts.credit_line.borrower,
            ctx.accounts.borrower.key(),
            AgcCreditError::InvalidAccount
        );
        require_keys_eq!(
            ctx.accounts.credit_line.underwriter_vault,
            ctx.accounts.underwriter_vault.key(),
            AgcCreditError::InvalidAccount
        );
        require_keys_eq!(
            ctx.accounts.underwriter_vault.usdc_vault,
            ctx.accounts.underwriter_usdc_vault.key(),
            AgcCreditError::InvalidTokenAccount
        );

        let outstanding_before = ctx.accounts.credit_line.outstanding_balance_usdc();
        let sweep_usdc = calculate_sweep(amount_usdc, ctx.accounts.policy.revenue_sweep_bps, outstanding_before)?;
        let borrower_receives_usdc = amount_usdc
            .checked_sub(sweep_usdc)
            .ok_or(AgcCreditError::MathOverflow)?;

        if sweep_usdc > 0 {
            token::transfer(ctx.accounts.revenue_to_vault_ctx(), sweep_usdc)?;
            apply_repayment_to_line(
                &mut ctx.accounts.credit_line,
                &mut ctx.accounts.borrower,
                &mut ctx.accounts.underwriter_vault,
                sweep_usdc,
                RepaymentSource::AutoSweep,
            )?;
        }

        if borrower_receives_usdc > 0 {
            token::transfer(ctx.accounts.revenue_to_borrower_ctx(), borrower_receives_usdc)?;
        }

        emit!(RevenueReceived {
            credit_line: ctx.accounts.credit_line.key(),
            agent: ctx.accounts.credit_line.agent,
            workflow: ctx.accounts.credit_line.workflow,
            amount_usdc,
            source,
            source_hash,
        });
        if sweep_usdc > 0 {
            emit!(RepaymentSwept {
                credit_line: ctx.accounts.credit_line.key(),
                amount_usdc: sweep_usdc,
                borrower_receives_usdc,
                remaining_balance_usdc: ctx.accounts.credit_line.outstanding_balance_usdc(),
            });
        }

        Ok(())
    }

    pub fn manual_repay(ctx: Context<ManualRepay>, amount_usdc: u64) -> Result<()> {
        require!(amount_usdc > 0, AgcCreditError::InvalidAmount);
        require_keys_eq!(
            ctx.accounts.credit_line.underwriter_vault,
            ctx.accounts.underwriter_vault.key(),
            AgcCreditError::InvalidAccount
        );
        require_keys_eq!(
            ctx.accounts.credit_line.borrower,
            ctx.accounts.borrower.key(),
            AgcCreditError::InvalidAccount
        );
        require_keys_eq!(
            ctx.accounts.underwriter_vault.usdc_vault,
            ctx.accounts.underwriter_usdc_vault.key(),
            AgcCreditError::InvalidTokenAccount
        );

        let repay_amount = amount_usdc.min(ctx.accounts.credit_line.outstanding_balance_usdc());
        require!(repay_amount > 0, AgcCreditError::NothingToRepay);
        token::transfer(ctx.accounts.payer_to_vault_ctx(), repay_amount)?;
        apply_repayment_to_line(
            &mut ctx.accounts.credit_line,
            &mut ctx.accounts.borrower,
            &mut ctx.accounts.underwriter_vault,
            repay_amount,
            RepaymentSource::Manual,
        )?;

        emit!(ManualRepayment {
            credit_line: ctx.accounts.credit_line.key(),
            payer: ctx.accounts.payer.key(),
            amount_usdc: repay_amount,
            remaining_balance_usdc: ctx.accounts.credit_line.outstanding_balance_usdc(),
        });

        Ok(())
    }

    pub fn update_score_attestation(
        ctx: Context<UpdateScoreAttestation>,
        args: UpdateScoreAttestationArgs,
    ) -> Result<()> {
        assert_risk_authority(&ctx.accounts.protocol_config, ctx.accounts.authority.key())?;
        require!(args.score <= 1000, AgcCreditError::InvalidScore);
        require!(args.confidence_bps <= BPS_DENOMINATOR as u16, AgcCreditError::InvalidBps);

        let score = &mut ctx.accounts.score_attestation;
        score.borrower = ctx.accounts.credit_line.borrower;
        score.agent = ctx.accounts.credit_line.agent;
        score.workflow = ctx.accounts.credit_line.workflow;
        score.credit_line = ctx.accounts.credit_line.key();
        score.score_version_hash = args.score_version_hash;
        score.score = args.score;
        score.risk_grade = args.risk_grade;
        score.recommended_limit_usdc = args.recommended_limit_usdc;
        score.recommended_apr_bps = args.recommended_apr_bps;
        score.pd_estimate_bps = args.pd_estimate_bps;
        score.lgd_estimate_bps = args.lgd_estimate_bps;
        score.confidence_bps = args.confidence_bps;
        score.features_hash = args.features_hash;
        score.created_at = current_timestamp()?;
        score.bump = ctx.bumps.score_attestation;

        ctx.accounts.credit_line.score_attestation = score.key();
        ctx.accounts.credit_line.risk_grade = args.risk_grade;

        emit!(ScoreUpdated {
            score_attestation: score.key(),
            credit_line: score.credit_line,
            score: score.score,
            risk_grade: score.risk_grade,
            recommended_limit_usdc: score.recommended_limit_usdc,
        });

        Ok(())
    }

    pub fn suspend_credit_line(ctx: Context<CreditLineRiskAction>) -> Result<()> {
        assert_risk_authority(&ctx.accounts.protocol_config, ctx.accounts.authority.key())?;
        require!(
            ctx.accounts.credit_line.status == CreditLineStatus::Active,
            AgcCreditError::InvalidCreditLineStatus
        );
        ctx.accounts.credit_line.status = CreditLineStatus::Suspended;
        emit!(CreditLineSuspended {
            credit_line: ctx.accounts.credit_line.key(),
        });
        Ok(())
    }

    pub fn resume_credit_line(ctx: Context<CreditLineRiskAction>) -> Result<()> {
        assert_risk_authority(&ctx.accounts.protocol_config, ctx.accounts.authority.key())?;
        require!(
            ctx.accounts.credit_line.status == CreditLineStatus::Suspended,
            AgcCreditError::InvalidCreditLineStatus
        );
        ctx.accounts.credit_line.status = CreditLineStatus::Active;
        emit!(CreditLineResumed {
            credit_line: ctx.accounts.credit_line.key(),
        });
        Ok(())
    }

    pub fn mark_delinquent(ctx: Context<CreditLineRiskAction>) -> Result<()> {
        assert_risk_authority(&ctx.accounts.protocol_config, ctx.accounts.authority.key())?;
        require!(
            ctx.accounts.credit_line.outstanding_balance_usdc() > 0,
            AgcCreditError::NothingToRepay
        );
        require!(
            current_timestamp()? > ctx.accounts.credit_line.maturity_at,
            AgcCreditError::LineNotMatured
        );
        ctx.accounts.credit_line.status = CreditLineStatus::Delinquent;
        emit!(CreditLineDelinquent {
            credit_line: ctx.accounts.credit_line.key(),
            outstanding_usdc: ctx.accounts.credit_line.outstanding_balance_usdc(),
        });
        Ok(())
    }

    pub fn mark_default(ctx: Context<MarkDefault>) -> Result<()> {
        assert_risk_authority(&ctx.accounts.protocol_config, ctx.accounts.authority.key())?;
        require!(
            ctx.accounts.credit_line.outstanding_balance_usdc() > 0,
            AgcCreditError::NothingToRepay
        );
        require_keys_eq!(
            ctx.accounts.credit_line.underwriter_vault,
            ctx.accounts.underwriter_vault.key(),
            AgcCreditError::InvalidAccount
        );
        let default_allowed_at = ctx
            .accounts
            .credit_line
            .maturity_at
            .checked_add(ctx.accounts.credit_line.grace_period_seconds)
            .ok_or(AgcCreditError::MathOverflow)?;
        require!(
            current_timestamp()? > default_allowed_at || ctx.accounts.credit_line.status == CreditLineStatus::Suspended,
            AgcCreditError::GracePeriodActive
        );

        let loss_usdc = ctx.accounts.credit_line.outstanding_balance_usdc();
        ctx.accounts.credit_line.status = CreditLineStatus::Defaulted;
        ctx.accounts.credit_line.available_limit_usdc = 0;
        ctx.accounts.credit_line.reserved_spend_usdc = 0;
        ctx.accounts.credit_line.defaulted_at = current_timestamp()?;

        ctx.accounts.underwriter_vault.loss_realized_usdc = ctx
            .accounts
            .underwriter_vault
            .loss_realized_usdc
            .checked_add(loss_usdc)
            .ok_or(AgcCreditError::MathOverflow)?;
        ctx.accounts.protocol_config.total_defaulted_usdc = ctx
            .accounts
            .protocol_config
            .total_defaulted_usdc
            .checked_add(loss_usdc)
            .ok_or(AgcCreditError::MathOverflow)?;
        if ctx.accounts.protocol_config.total_defaulted_usdc >= ctx.accounts.protocol_config.max_loss_budget_usdc {
            ctx.accounts.protocol_config.paused = true;
        }

        emit!(CreditLineDefaulted {
            credit_line: ctx.accounts.credit_line.key(),
            loss_usdc,
            underwriter_vault: ctx.accounts.underwriter_vault.key(),
        });

        Ok(())
    }

    pub fn close_repaid_credit_line(ctx: Context<CloseRepaidCreditLine>) -> Result<()> {
        assert_risk_authority(&ctx.accounts.protocol_config, ctx.accounts.authority.key())?;
        require!(
            ctx.accounts.credit_line.outstanding_balance_usdc() == 0
                && ctx.accounts.credit_line.reserved_spend_usdc == 0,
            AgcCreditError::OutstandingBalance
        );
        require!(
            matches!(
                ctx.accounts.credit_line.status,
                CreditLineStatus::Active
                    | CreditLineStatus::Suspended
                    | CreditLineStatus::Approved
                    | CreditLineStatus::Delinquent
                    | CreditLineStatus::Repaid
            ),
            AgcCreditError::InvalidCreditLineStatus
        );
        require_keys_eq!(
            ctx.accounts.credit_line.borrower,
            ctx.accounts.borrower.key(),
            AgcCreditError::InvalidAccount
        );
        require_keys_eq!(
            ctx.accounts.credit_line.underwriter_vault,
            ctx.accounts.underwriter_vault.key(),
            AgcCreditError::InvalidAccount
        );

        let committed_limit = ctx.accounts.credit_line.principal_limit_usdc;
        ctx.accounts.credit_line.status = CreditLineStatus::Closed;
        ctx.accounts.underwriter_vault.available_capital_usdc = ctx
            .accounts
            .underwriter_vault
            .available_capital_usdc
            .checked_add(committed_limit)
            .ok_or(AgcCreditError::MathOverflow)?;
        ctx.accounts.underwriter_vault.committed_to_lines_usdc = ctx
            .accounts
            .underwriter_vault
            .committed_to_lines_usdc
            .checked_sub(committed_limit)
            .ok_or(AgcCreditError::MathOverflow)?;
        ctx.accounts.borrower.total_principal_limit_usdc = ctx
            .accounts
            .borrower
            .total_principal_limit_usdc
            .checked_sub(committed_limit)
            .ok_or(AgcCreditError::MathOverflow)?;
        ctx.accounts.protocol_config.total_live_capital_at_risk_usdc = ctx
            .accounts
            .protocol_config
            .total_live_capital_at_risk_usdc
            .checked_sub(committed_limit)
            .ok_or(AgcCreditError::MathOverflow)?;

        emit!(CreditLineClosed {
            credit_line: ctx.accounts.credit_line.key(),
            underwriter_vault: ctx.accounts.underwriter_vault.key(),
        });

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeProtocol<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + ProtocolConfig::LEN,
        seeds = [PROTOCOL_SEED],
        bump
    )]
    pub protocol_config: Box<Account<'info, ProtocolConfig>>,
    pub usdc_mint: Account<'info, Mint>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AdminOnly<'info> {
    #[account(mut, seeds = [PROTOCOL_SEED], bump = protocol_config.bump)]
    pub protocol_config: Box<Account<'info, ProtocolConfig>>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct RegisterBorrower<'info> {
    #[account(seeds = [PROTOCOL_SEED], bump = protocol_config.bump)]
    pub protocol_config: Account<'info, ProtocolConfig>,
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
#[instruction(args: RegisterAgentArgs)]
pub struct RegisterAgent<'info> {
    #[account(seeds = [PROTOCOL_SEED], bump = protocol_config.bump)]
    pub protocol_config: Account<'info, ProtocolConfig>,
    #[account(mut, has_one = operator)]
    pub borrower: Account<'info, BorrowerProfile>,
    #[account(
        init,
        payer = operator,
        space = 8 + AgentProfile::LEN,
        seeds = [AGENT_SEED, borrower.key().as_ref(), args.wallet.as_ref()],
        bump
    )]
    pub agent: Account<'info, AgentProfile>,
    #[account(mut)]
    pub operator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(args: CreateWorkflowArgs)]
pub struct CreateWorkflow<'info> {
    #[account(seeds = [PROTOCOL_SEED], bump = protocol_config.bump)]
    pub protocol_config: Account<'info, ProtocolConfig>,
    #[account(has_one = operator)]
    pub agent: Account<'info, AgentProfile>,
    #[account(
        init,
        payer = operator,
        space = 8 + WorkflowProfile::LEN,
        seeds = [WORKFLOW_SEED, agent.key().as_ref(), args.workflow_seed.as_ref()],
        bump
    )]
    pub workflow: Account<'info, WorkflowProfile>,
    #[account(mut)]
    pub operator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(args: RegisterMerchantArgs)]
pub struct RegisterMerchant<'info> {
    #[account(seeds = [PROTOCOL_SEED], bump = protocol_config.bump)]
    pub protocol_config: Account<'info, ProtocolConfig>,
    #[account(
        init,
        payer = authority,
        space = 8 + Merchant::LEN,
        seeds = [MERCHANT_SEED, args.merchant_id_hash.as_ref()],
        bump
    )]
    pub merchant: Account<'info, Merchant>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateSpendPolicy<'info> {
    #[account(seeds = [PROTOCOL_SEED], bump = protocol_config.bump)]
    pub protocol_config: Account<'info, ProtocolConfig>,
    #[account(mut)]
    pub borrower: Account<'info, BorrowerProfile>,
    #[account(mut)]
    pub workflow: Account<'info, WorkflowProfile>,
    #[account(
        init,
        payer = operator,
        space = 8 + SpendPolicy::LEN,
        seeds = [POLICY_SEED, workflow.key().as_ref()],
        bump
    )]
    pub policy: Account<'info, SpendPolicy>,
    #[account(mut)]
    pub operator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateUnderwriterVault<'info> {
    #[account(seeds = [PROTOCOL_SEED], bump = protocol_config.bump)]
    pub protocol_config: Account<'info, ProtocolConfig>,
    #[account(
        init,
        payer = underwriter,
        space = 8 + UnderwriterVault::LEN,
        seeds = [UNDERWRITER_VAULT_SEED, underwriter.key().as_ref()],
        bump
    )]
    pub underwriter_vault: Account<'info, UnderwriterVault>,
    /// CHECK: PDA authority for the USDC token vault.
    #[account(
        seeds = [UNDERWRITER_VAULT_AUTHORITY_SEED, underwriter_vault.key().as_ref()],
        bump
    )]
    pub vault_authority: UncheckedAccount<'info>,
    #[account(
        init,
        payer = underwriter,
        token::mint = usdc_mint,
        token::authority = vault_authority,
        seeds = [UNDERWRITER_VAULT_USDC_SEED, underwriter_vault.key().as_ref()],
        bump
    )]
    pub usdc_vault: Account<'info, TokenAccount>,
    #[account(address = protocol_config.usdc_mint)]
    pub usdc_mint: Account<'info, Mint>,
    #[account(mut)]
    pub underwriter: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct FundUnderwriterVault<'info> {
    #[account(seeds = [PROTOCOL_SEED], bump = protocol_config.bump)]
    pub protocol_config: Account<'info, ProtocolConfig>,
    #[account(mut, has_one = underwriter)]
    pub underwriter_vault: Account<'info, UnderwriterVault>,
    #[account(mut, constraint = underwriter_usdc.mint == protocol_config.usdc_mint)]
    pub underwriter_usdc: Account<'info, TokenAccount>,
    #[account(mut, address = underwriter_vault.usdc_vault)]
    pub vault_usdc: Account<'info, TokenAccount>,
    pub underwriter: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

impl<'info> FundUnderwriterVault<'info> {
    fn fund_transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.key(),
            Transfer {
                from: self.underwriter_usdc.to_account_info(),
                to: self.vault_usdc.to_account_info(),
                authority: self.underwriter.to_account_info(),
            },
        )
    }
}

#[derive(Accounts)]
#[instruction(args: ApproveCreditLineArgs)]
pub struct ApproveCreditLine<'info> {
    #[account(mut, seeds = [PROTOCOL_SEED], bump = protocol_config.bump)]
    pub protocol_config: Account<'info, ProtocolConfig>,
    #[account(mut)]
    pub borrower: Account<'info, BorrowerProfile>,
    pub workflow: Box<Account<'info, WorkflowProfile>>,
    pub policy: Box<Account<'info, SpendPolicy>>,
    #[account(mut)]
    pub underwriter_vault: Box<Account<'info, UnderwriterVault>>,
    #[account(
        init,
        payer = authority,
        space = 8 + CreditLine::LEN,
        seeds = [
            CREDIT_LINE_SEED,
            workflow.key().as_ref(),
            underwriter_vault.key().as_ref(),
            args.line_seed.as_ref()
        ],
        bump
    )]
    pub credit_line: Box<Account<'info, CreditLine>>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreditLineRiskAction<'info> {
    #[account(seeds = [PROTOCOL_SEED], bump = protocol_config.bump)]
    pub protocol_config: Box<Account<'info, ProtocolConfig>>,
    #[account(mut)]
    pub credit_line: Box<Account<'info, CreditLine>>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(args: ReserveSpendArgs)]
pub struct ReserveSpend<'info> {
    #[account(mut, seeds = [PROTOCOL_SEED], bump = protocol_config.bump)]
    pub protocol_config: Box<Account<'info, ProtocolConfig>>,
    pub borrower: Box<Account<'info, BorrowerProfile>>,
    #[account(mut, has_one = borrower, has_one = policy)]
    pub credit_line: Box<Account<'info, CreditLine>>,
    #[account(mut)]
    pub policy: Box<Account<'info, SpendPolicy>>,
    pub merchant: Box<Account<'info, Merchant>>,
    #[account(
        init,
        payer = router,
        space = 8 + SpendAuthorization::LEN,
        seeds = [SPEND_AUTH_SEED, credit_line.key().as_ref(), args.spend_id.as_ref()],
        bump
    )]
    pub spend_authorization: Box<Account<'info, SpendAuthorization>>,
    #[account(mut)]
    pub router: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CancelReservedSpend<'info> {
    #[account(seeds = [PROTOCOL_SEED], bump = protocol_config.bump)]
    pub protocol_config: Box<Account<'info, ProtocolConfig>>,
    #[account(mut)]
    pub credit_line: Box<Account<'info, CreditLine>>,
    #[account(mut)]
    pub spend_authorization: Box<Account<'info, SpendAuthorization>>,
    pub router: Signer<'info>,
}

#[derive(Accounts)]
pub struct SettleSpend<'info> {
    #[account(seeds = [PROTOCOL_SEED], bump = protocol_config.bump)]
    pub protocol_config: Box<Account<'info, ProtocolConfig>>,
    #[account(mut)]
    pub borrower: Box<Account<'info, BorrowerProfile>>,
    #[account(mut)]
    pub credit_line: Box<Account<'info, CreditLine>>,
    #[account(mut)]
    pub spend_authorization: Box<Account<'info, SpendAuthorization>>,
    #[account(mut)]
    pub underwriter_vault: Box<Account<'info, UnderwriterVault>>,
    /// CHECK: PDA authority for the USDC token vault.
    #[account(
        seeds = [UNDERWRITER_VAULT_AUTHORITY_SEED, underwriter_vault.key().as_ref()],
        bump = underwriter_vault.authority_bump
    )]
    pub vault_authority: UncheckedAccount<'info>,
    #[account(mut, address = underwriter_vault.usdc_vault)]
    pub underwriter_usdc_vault: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = merchant_usdc.mint == protocol_config.usdc_mint)]
    pub merchant_usdc: Box<Account<'info, TokenAccount>>,
    pub router: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct RecordRevenueAndSweep<'info> {
    #[account(seeds = [PROTOCOL_SEED], bump = protocol_config.bump)]
    pub protocol_config: Box<Account<'info, ProtocolConfig>>,
    #[account(mut)]
    pub borrower: Box<Account<'info, BorrowerProfile>>,
    #[account(mut)]
    pub credit_line: Box<Account<'info, CreditLine>>,
    pub policy: Box<Account<'info, SpendPolicy>>,
    #[account(mut)]
    pub underwriter_vault: Box<Account<'info, UnderwriterVault>>,
    #[account(mut, constraint = revenue_source_usdc.mint == protocol_config.usdc_mint)]
    pub revenue_source_usdc: Box<Account<'info, TokenAccount>>,
    #[account(mut, address = underwriter_vault.usdc_vault)]
    pub underwriter_usdc_vault: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = borrower_receivable_usdc.mint == protocol_config.usdc_mint)]
    pub borrower_receivable_usdc: Box<Account<'info, TokenAccount>>,
    pub revenue_payer: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

impl<'info> RecordRevenueAndSweep<'info> {
    fn revenue_to_vault_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.key(),
            Transfer {
                from: self.revenue_source_usdc.to_account_info(),
                to: self.underwriter_usdc_vault.to_account_info(),
                authority: self.revenue_payer.to_account_info(),
            },
        )
    }

    fn revenue_to_borrower_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.key(),
            Transfer {
                from: self.revenue_source_usdc.to_account_info(),
                to: self.borrower_receivable_usdc.to_account_info(),
                authority: self.revenue_payer.to_account_info(),
            },
        )
    }
}

#[derive(Accounts)]
pub struct ManualRepay<'info> {
    #[account(seeds = [PROTOCOL_SEED], bump = protocol_config.bump)]
    pub protocol_config: Box<Account<'info, ProtocolConfig>>,
    #[account(mut)]
    pub borrower: Box<Account<'info, BorrowerProfile>>,
    #[account(mut)]
    pub credit_line: Box<Account<'info, CreditLine>>,
    #[account(mut)]
    pub underwriter_vault: Box<Account<'info, UnderwriterVault>>,
    #[account(mut, constraint = payer_usdc.mint == protocol_config.usdc_mint)]
    pub payer_usdc: Box<Account<'info, TokenAccount>>,
    #[account(mut, address = underwriter_vault.usdc_vault)]
    pub underwriter_usdc_vault: Box<Account<'info, TokenAccount>>,
    pub payer: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

impl<'info> ManualRepay<'info> {
    fn payer_to_vault_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.key(),
            Transfer {
                from: self.payer_usdc.to_account_info(),
                to: self.underwriter_usdc_vault.to_account_info(),
                authority: self.payer.to_account_info(),
            },
        )
    }
}

#[derive(Accounts)]
#[instruction(args: UpdateScoreAttestationArgs)]
pub struct UpdateScoreAttestation<'info> {
    #[account(seeds = [PROTOCOL_SEED], bump = protocol_config.bump)]
    pub protocol_config: Box<Account<'info, ProtocolConfig>>,
    #[account(mut)]
    pub credit_line: Box<Account<'info, CreditLine>>,
    #[account(
        init,
        payer = authority,
        space = 8 + ScoreAttestation::LEN,
        seeds = [SCORE_SEED, credit_line.key().as_ref(), args.score_version_hash.as_ref()],
        bump
    )]
    pub score_attestation: Box<Account<'info, ScoreAttestation>>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MarkDefault<'info> {
    #[account(mut, seeds = [PROTOCOL_SEED], bump = protocol_config.bump)]
    pub protocol_config: Box<Account<'info, ProtocolConfig>>,
    #[account(mut)]
    pub credit_line: Box<Account<'info, CreditLine>>,
    #[account(mut)]
    pub underwriter_vault: Box<Account<'info, UnderwriterVault>>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct CloseRepaidCreditLine<'info> {
    #[account(mut, seeds = [PROTOCOL_SEED], bump = protocol_config.bump)]
    pub protocol_config: Box<Account<'info, ProtocolConfig>>,
    #[account(mut)]
    pub borrower: Box<Account<'info, BorrowerProfile>>,
    #[account(mut)]
    pub credit_line: Box<Account<'info, CreditLine>>,
    #[account(mut)]
    pub underwriter_vault: Box<Account<'info, UnderwriterVault>>,
    pub authority: Signer<'info>,
}

#[account]
pub struct ProtocolConfig {
    pub admin: Pubkey,
    pub risk_admin: Pubkey,
    pub emergency_admin: Pubkey,
    pub payment_router: Pubkey,
    pub usdc_mint: Pubkey,
    pub max_total_live_capital_at_risk_usdc: u64,
    pub max_single_borrower_limit_usdc: u64,
    pub max_unsecured_exposure_per_borrower_usdc: u64,
    pub max_daily_total_spend_usdc: u64,
    pub max_loss_budget_usdc: u64,
    pub total_live_capital_at_risk_usdc: u64,
    pub total_daily_spend_usdc: u64,
    pub total_daily_spend_window_started_at: u64,
    pub total_defaulted_usdc: u64,
    pub paused: bool,
    pub bump: u8,
}

impl ProtocolConfig {
    pub const LEN: usize = 32 * 5 + 8 * 9 + 1 + 1 + 64;
}

#[account]
pub struct BorrowerProfile {
    pub operator: Pubkey,
    pub primary_wallet: Pubkey,
    pub metadata_hash: [u8; 32],
    pub borrower_type: BorrowerType,
    pub verification_status: VerificationStatus,
    pub status: BorrowerStatus,
    pub total_principal_limit_usdc: u64,
    pub total_outstanding_usdc: u64,
    pub created_at: u64,
    pub bump: u8,
}

impl BorrowerProfile {
    pub const LEN: usize = 32 + 32 + 32 + 1 + 1 + 1 + 8 + 8 + 8 + 1 + 64;
}

#[account]
pub struct AgentProfile {
    pub borrower: Pubkey,
    pub operator: Pubkey,
    pub wallet: Pubkey,
    pub metadata_hash: [u8; 32],
    pub framework: AgentFramework,
    pub status: AgentStatus,
    pub created_at: u64,
    pub last_active_at: u64,
    pub bump: u8,
}

impl AgentProfile {
    pub const LEN: usize = 32 * 4 + 1 + 1 + 8 + 8 + 1 + 64;
}

#[account]
pub struct WorkflowProfile {
    pub borrower: Pubkey,
    pub agent: Pubkey,
    pub workflow_seed: [u8; 16],
    pub metadata_hash: [u8; 32],
    pub policy: Pubkey,
    pub status: WorkflowStatus,
    pub created_at: u64,
    pub bump: u8,
}

impl WorkflowProfile {
    pub const LEN: usize = 32 + 32 + 16 + 32 + 32 + 1 + 8 + 1 + 64;
}

#[account]
pub struct Merchant {
    pub merchant_id_hash: [u8; 32],
    pub metadata_hash: [u8; 32],
    pub category: u16,
    pub status: MerchantStatus,
    pub adapter: MerchantAdapter,
    pub created_at: u64,
    pub bump: u8,
}

impl Merchant {
    pub const LEN: usize = 32 + 32 + 2 + 1 + 1 + 8 + 1 + 64;
}

#[account]
pub struct SpendPolicy {
    pub workflow: Pubkey,
    pub borrower: Pubkey,
    pub version: u32,
    pub metadata_hash: [u8; 32],
    pub max_per_transaction_usdc: u64,
    pub max_daily_spend_usdc: u64,
    pub max_weekly_spend_usdc: u64,
    pub allowed_merchants: [Pubkey; MAX_LIST_ITEMS],
    pub allowed_merchant_count: u8,
    pub blocked_merchants: [Pubkey; MAX_LIST_ITEMS],
    pub blocked_merchant_count: u8,
    pub allowed_categories: [u16; MAX_LIST_ITEMS],
    pub allowed_category_count: u8,
    pub human_approval_threshold_usdc: u64,
    pub revenue_sweep_bps: u16,
    pub min_available_limit_after_spend_usdc: u64,
    pub cooldown_after_policy_violation_seconds: u64,
    pub daily_spend_usdc: u64,
    pub weekly_spend_usdc: u64,
    pub daily_window_started_at: u64,
    pub weekly_window_started_at: u64,
    pub violation_count: u64,
    pub last_violation_at: u64,
    pub status: PolicyStatus,
    pub valid_from: u64,
    pub valid_to: u64,
    pub bump: u8,
}

impl SpendPolicy {
    pub const LEN: usize = 32 + 32 + 4 + 32 + 8 + 8 + 8 + (32 * MAX_LIST_ITEMS) + 1
        + (32 * MAX_LIST_ITEMS) + 1 + (2 * MAX_LIST_ITEMS) + 1 + 8 + 2 + 8 + 8
        + 8 + 8 + 8 + 8 + 8 + 8 + 1 + 8 + 8 + 1 + 128;
}

#[account]
pub struct UnderwriterVault {
    pub underwriter: Pubkey,
    pub usdc_mint: Pubkey,
    pub usdc_vault: Pubkey,
    pub metadata_hash: [u8; 32],
    pub status: UnderwriterVaultStatus,
    pub committed_capital_usdc: u64,
    pub available_capital_usdc: u64,
    pub committed_to_lines_usdc: u64,
    pub principal_drawn_usdc: u64,
    pub principal_repaid_usdc: u64,
    pub interest_earned_usdc: u64,
    pub loss_realized_usdc: u64,
    pub max_single_line_usdc: u64,
    pub bump: u8,
    pub authority_bump: u8,
    pub usdc_vault_bump: u8,
}

impl UnderwriterVault {
    pub const LEN: usize = 32 + 32 + 32 + 32 + 1 + (8 * 8) + 3 + 64;
}

#[account]
pub struct CreditLine {
    pub borrower: Pubkey,
    pub agent: Pubkey,
    pub workflow: Pubkey,
    pub underwriter_vault: Pubkey,
    pub policy: Pubkey,
    pub repayment_rule: Pubkey,
    pub score_attestation: Pubkey,
    pub line_seed: [u8; 16],
    pub metadata_hash: [u8; 32],
    pub principal_limit_usdc: u64,
    pub available_limit_usdc: u64,
    pub reserved_spend_usdc: u64,
    pub principal_outstanding_usdc: u64,
    pub accrued_interest_usdc: u64,
    pub fees_due_usdc: u64,
    pub apr_bps: u16,
    pub origination_fee_bps: u16,
    pub tenor_seconds: u64,
    pub maturity_at: u64,
    pub grace_period_seconds: u64,
    pub status: CreditLineStatus,
    pub risk_grade: RiskGrade,
    pub created_at: u64,
    pub activated_at: u64,
    pub last_repayment_at: u64,
    pub defaulted_at: u64,
    pub bump: u8,
}

impl CreditLine {
    pub const LEN: usize = (32 * 7) + 16 + 32 + (8 * 13) + 2 + 2 + 1 + 1 + 1 + 128;

    pub fn outstanding_balance_usdc(&self) -> u64 {
        self.principal_outstanding_usdc
            .saturating_add(self.accrued_interest_usdc)
            .saturating_add(self.fees_due_usdc)
    }
}

#[account]
pub struct SpendAuthorization {
    pub credit_line: Pubkey,
    pub workflow: Pubkey,
    pub agent: Pubkey,
    pub merchant: Pubkey,
    pub spend_id: [u8; 16],
    pub amount_usdc: u64,
    pub purpose_hash: [u8; 32],
    pub policy_version: u32,
    pub status: SpendAuthorizationStatus,
    pub reason_code: SpendDecisionCode,
    pub created_at: u64,
    pub expires_at: u64,
    pub bump: u8,
}

impl SpendAuthorization {
    pub const LEN: usize = (32 * 4) + 16 + 8 + 32 + 4 + 1 + 1 + 8 + 8 + 1 + 64;
}

#[account]
pub struct ScoreAttestation {
    pub borrower: Pubkey,
    pub agent: Pubkey,
    pub workflow: Pubkey,
    pub credit_line: Pubkey,
    pub score_version_hash: [u8; 32],
    pub score: u16,
    pub risk_grade: RiskGrade,
    pub recommended_limit_usdc: u64,
    pub recommended_apr_bps: u16,
    pub pd_estimate_bps: u16,
    pub lgd_estimate_bps: u16,
    pub confidence_bps: u16,
    pub features_hash: [u8; 32],
    pub created_at: u64,
    pub bump: u8,
}

impl ScoreAttestation {
    pub const LEN: usize = (32 * 4) + 32 + 2 + 1 + 8 + 2 + 2 + 2 + 2 + 32 + 8 + 1 + 64;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct InitializeProtocolArgs {
    pub risk_admin: Pubkey,
    pub emergency_admin: Pubkey,
    pub payment_router: Pubkey,
    pub max_total_live_capital_at_risk_usdc: u64,
    pub max_single_borrower_limit_usdc: u64,
    pub max_unsecured_exposure_per_borrower_usdc: u64,
    pub max_daily_total_spend_usdc: u64,
    pub max_loss_budget_usdc: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct RegisterBorrowerArgs {
    pub primary_wallet: Pubkey,
    pub metadata_hash: [u8; 32],
    pub borrower_type: BorrowerType,
    pub verification_status: VerificationStatus,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct RegisterAgentArgs {
    pub wallet: Pubkey,
    pub metadata_hash: [u8; 32],
    pub framework: AgentFramework,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct CreateWorkflowArgs {
    pub workflow_seed: [u8; 16],
    pub metadata_hash: [u8; 32],
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct RegisterMerchantArgs {
    pub merchant_id_hash: [u8; 32],
    pub metadata_hash: [u8; 32],
    pub category: u16,
    pub status: MerchantStatus,
    pub adapter: MerchantAdapter,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct CreateSpendPolicyArgs {
    pub metadata_hash: [u8; 32],
    pub max_per_transaction_usdc: u64,
    pub max_daily_spend_usdc: u64,
    pub max_weekly_spend_usdc: u64,
    pub allowed_merchants: [Pubkey; MAX_LIST_ITEMS],
    pub allowed_merchant_count: u8,
    pub blocked_merchants: [Pubkey; MAX_LIST_ITEMS],
    pub blocked_merchant_count: u8,
    pub allowed_categories: [u16; MAX_LIST_ITEMS],
    pub allowed_category_count: u8,
    pub human_approval_threshold_usdc: u64,
    pub revenue_sweep_bps: u16,
    pub min_available_limit_after_spend_usdc: u64,
    pub cooldown_after_policy_violation_seconds: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct CreateUnderwriterVaultArgs {
    pub metadata_hash: [u8; 32],
    pub max_single_line_usdc: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct ApproveCreditLineArgs {
    pub line_seed: [u8; 16],
    pub metadata_hash: [u8; 32],
    pub principal_limit_usdc: u64,
    pub apr_bps: u16,
    pub origination_fee_bps: u16,
    pub tenor_seconds: u64,
    pub grace_period_seconds: u64,
    pub risk_grade: RiskGrade,
    pub repayment_rule: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct ReserveSpendArgs {
    pub spend_id: [u8; 16],
    pub amount_usdc: u64,
    pub purpose_hash: [u8; 32],
    pub authorization_ttl_seconds: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct UpdateScoreAttestationArgs {
    pub score_version_hash: [u8; 32],
    pub score: u16,
    pub risk_grade: RiskGrade,
    pub recommended_limit_usdc: u64,
    pub recommended_apr_bps: u16,
    pub pd_estimate_bps: u16,
    pub lgd_estimate_bps: u16,
    pub confidence_bps: u16,
    pub features_hash: [u8; 32],
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum BorrowerType {
    Individual,
    Company,
    PseudonymousSandbox,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum VerificationStatus {
    None,
    Email,
    Wallet,
    Kyc,
    Kyb,
    Manual,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum BorrowerStatus {
    Active,
    Suspended,
    Blocked,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum AgentFramework {
    OpenAi,
    Langchain,
    Agentkit,
    Custom,
    Other,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum AgentStatus {
    Pending,
    Active,
    Suspended,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowStatus {
    Draft,
    Approved,
    Active,
    Suspended,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum MerchantStatus {
    Active,
    Blocked,
    Suspended,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum MerchantAdapter {
    Mock,
    X402,
    Manual,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum PolicyStatus {
    Active,
    Suspended,
    Expired,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum UnderwriterVaultStatus {
    Active,
    Paused,
    Closed,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum CreditLineStatus {
    Draft,
    Approved,
    Active,
    Suspended,
    Matured,
    GracePeriod,
    Delinquent,
    Defaulted,
    Repaid,
    Closed,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum RiskGrade {
    A,
    B,
    C,
    D,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum SpendAuthorizationStatus {
    Reserved,
    Settled,
    Canceled,
    Expired,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum SpendDecisionCode {
    Approved,
    ManualApprovalRequired,
    CreditLineNotActive,
    LineMatured,
    InsufficientAvailableLimit,
    PerTransactionLimitExceeded,
    DailyLimitExceeded,
    WeeklyLimitExceeded,
    MerchantNotAllowed,
    MerchantBlocked,
    CategoryNotAllowed,
    RevenueSweepRequired,
    ProtocolDailyLimitExceeded,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum RevenueSource {
    Manual,
    Wallet,
    X402,
    StripeAttested,
    TestMerchant,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum RepaymentSource {
    Manual,
    AutoSweep,
    Escrow,
}

#[event]
pub struct ProtocolInitialized {
    pub admin: Pubkey,
    pub risk_admin: Pubkey,
    pub payment_router: Pubkey,
    pub usdc_mint: Pubkey,
}

#[event]
pub struct ProtocolPauseUpdated {
    pub paused: bool,
}

#[event]
pub struct BorrowerRegistered {
    pub borrower: Pubkey,
    pub operator: Pubkey,
    pub verification_status: VerificationStatus,
}

#[event]
pub struct AgentRegistered {
    pub borrower: Pubkey,
    pub agent: Pubkey,
    pub wallet: Pubkey,
}

#[event]
pub struct WorkflowCreated {
    pub workflow: Pubkey,
    pub agent: Pubkey,
    pub borrower: Pubkey,
}

#[event]
pub struct MerchantRegistered {
    pub merchant: Pubkey,
    pub category: u16,
    pub status: MerchantStatus,
}

#[event]
pub struct PolicyCreated {
    pub policy: Pubkey,
    pub workflow: Pubkey,
    pub version: u32,
    pub revenue_sweep_bps: u16,
}

#[event]
pub struct UnderwriterVaultCreated {
    pub underwriter_vault: Pubkey,
    pub underwriter: Pubkey,
    pub usdc_vault: Pubkey,
}

#[event]
pub struct UnderwriterVaultFunded {
    pub underwriter_vault: Pubkey,
    pub amount_usdc: u64,
    pub available_capital_usdc: u64,
}

#[event]
pub struct CreditLineApproved {
    pub credit_line: Pubkey,
    pub borrower: Pubkey,
    pub workflow: Pubkey,
    pub principal_limit_usdc: u64,
    pub apr_bps: u16,
    pub risk_grade: RiskGrade,
}

#[event]
pub struct CreditLineActivated {
    pub credit_line: Pubkey,
    pub activated_at: u64,
}

#[event]
pub struct SpendApproved {
    pub spend_authorization: Pubkey,
    pub credit_line: Pubkey,
    pub merchant: Pubkey,
    pub amount_usdc: u64,
    pub policy_version: u32,
}

#[event]
pub struct SpendCanceled {
    pub spend_authorization: Pubkey,
    pub credit_line: Pubkey,
    pub amount_usdc: u64,
}

#[event]
pub struct PaymentSettled {
    pub spend_authorization: Pubkey,
    pub credit_line: Pubkey,
    pub merchant: Pubkey,
    pub merchant_token_account: Pubkey,
    pub amount_usdc: u64,
}

#[event]
pub struct RevenueReceived {
    pub credit_line: Pubkey,
    pub agent: Pubkey,
    pub workflow: Pubkey,
    pub amount_usdc: u64,
    pub source: RevenueSource,
    pub source_hash: [u8; 32],
}

#[event]
pub struct RepaymentSwept {
    pub credit_line: Pubkey,
    pub amount_usdc: u64,
    pub borrower_receives_usdc: u64,
    pub remaining_balance_usdc: u64,
}

#[event]
pub struct ManualRepayment {
    pub credit_line: Pubkey,
    pub payer: Pubkey,
    pub amount_usdc: u64,
    pub remaining_balance_usdc: u64,
}

#[event]
pub struct ScoreUpdated {
    pub score_attestation: Pubkey,
    pub credit_line: Pubkey,
    pub score: u16,
    pub risk_grade: RiskGrade,
    pub recommended_limit_usdc: u64,
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
pub struct CreditLineDelinquent {
    pub credit_line: Pubkey,
    pub outstanding_usdc: u64,
}

#[event]
pub struct CreditLineDefaulted {
    pub credit_line: Pubkey,
    pub loss_usdc: u64,
    pub underwriter_vault: Pubkey,
}

#[event]
pub struct CreditLineClosed {
    pub credit_line: Pubkey,
    pub underwriter_vault: Pubkey,
}

#[error_code]
pub enum AgcCreditError {
    #[msg("The protocol is paused")]
    ProtocolPaused,
    #[msg("Unauthorized authority")]
    Unauthorized,
    #[msg("Invalid account")]
    InvalidAccount,
    #[msg("Invalid token account")]
    InvalidTokenAccount,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Invalid limit")]
    InvalidLimit,
    #[msg("Invalid tenor")]
    InvalidTenor,
    #[msg("Invalid basis points")]
    InvalidBps,
    #[msg("Invalid score")]
    InvalidScore,
    #[msg("Invalid category")]
    InvalidCategory,
    #[msg("List count exceeds fixed capacity")]
    ListCountTooLarge,
    #[msg("Policy requires at least one allowlisted merchant")]
    EmptyMerchantAllowlist,
    #[msg("Policy requires a revenue sweep")]
    RevenueSweepRequired,
    #[msg("Borrower is not active")]
    BorrowerNotActive,
    #[msg("Agent is not active")]
    AgentNotActive,
    #[msg("Workflow is not active")]
    WorkflowNotActive,
    #[msg("Policy is not active")]
    PolicyNotActive,
    #[msg("Underwriter vault is not active")]
    UnderwriterVaultNotActive,
    #[msg("Insufficient underwriter capital")]
    InsufficientUnderwriterCapital,
    #[msg("Underwriter line limit exceeded")]
    UnderwriterLimitExceeded,
    #[msg("Borrower limit exceeded")]
    BorrowerLimitExceeded,
    #[msg("Global experiment limit exceeded")]
    GlobalLimitExceeded,
    #[msg("D-grade credit is simulation-only")]
    RiskGradeNotLive,
    #[msg("Credit line status does not allow this action")]
    InvalidCreditLineStatus,
    #[msg("Credit line is not active")]
    CreditLineNotActive,
    #[msg("Line has matured")]
    LineMatured,
    #[msg("Line has not matured")]
    LineNotMatured,
    #[msg("Grace period is still active")]
    GracePeriodActive,
    #[msg("Insufficient available line limit")]
    InsufficientAvailableLimit,
    #[msg("Per-transaction limit exceeded")]
    PerTransactionLimitExceeded,
    #[msg("Daily spend limit exceeded")]
    DailyLimitExceeded,
    #[msg("Weekly spend limit exceeded")]
    WeeklyLimitExceeded,
    #[msg("Human approval required")]
    HumanApprovalRequired,
    #[msg("Merchant is blocked")]
    MerchantBlocked,
    #[msg("Merchant is not allowlisted")]
    MerchantNotAllowed,
    #[msg("Category is not allowed")]
    CategoryNotAllowed,
    #[msg("Protocol daily spend limit exceeded")]
    ProtocolDailyLimitExceeded,
    #[msg("Spend authorization already finalized")]
    SpendAlreadyFinalized,
    #[msg("Spend authorization expired")]
    SpendAuthorizationExpired,
    #[msg("Nothing to repay")]
    NothingToRepay,
    #[msg("Credit line still has outstanding balance")]
    OutstandingBalance,
    #[msg("Math overflow")]
    MathOverflow,
}

fn current_timestamp() -> Result<u64> {
    let ts = Clock::get()?.unix_timestamp;
    require!(ts >= 0, AgcCreditError::MathOverflow);
    Ok(ts as u64)
}

fn assert_admin(config: &ProtocolConfig, authority: Pubkey) -> Result<()> {
    require_keys_eq!(config.admin, authority, AgcCreditError::Unauthorized);
    Ok(())
}

fn assert_risk_authority(config: &ProtocolConfig, authority: Pubkey) -> Result<()> {
    require!(
        authority == config.admin || authority == config.risk_admin,
        AgcCreditError::Unauthorized
    );
    Ok(())
}

fn assert_router_or_risk_authority(config: &ProtocolConfig, authority: Pubkey) -> Result<()> {
    require!(
        authority == config.payment_router || authority == config.admin || authority == config.risk_admin,
        AgcCreditError::Unauthorized
    );
    Ok(())
}

fn validate_policy_args(args: &CreateSpendPolicyArgs) -> Result<()> {
    require!(
        args.allowed_merchant_count as usize <= MAX_LIST_ITEMS
            && args.blocked_merchant_count as usize <= MAX_LIST_ITEMS
            && args.allowed_category_count as usize <= MAX_LIST_ITEMS,
        AgcCreditError::ListCountTooLarge
    );
    require!(args.allowed_merchant_count > 0, AgcCreditError::EmptyMerchantAllowlist);
    require!(args.max_per_transaction_usdc > 0, AgcCreditError::InvalidLimit);
    require!(args.max_daily_spend_usdc > 0, AgcCreditError::InvalidLimit);
    require!(args.max_weekly_spend_usdc >= args.max_daily_spend_usdc, AgcCreditError::InvalidLimit);
    require!(
        args.max_per_transaction_usdc <= args.max_daily_spend_usdc,
        AgcCreditError::InvalidLimit
    );
    require!(
        args.human_approval_threshold_usdc <= args.max_per_transaction_usdc,
        AgcCreditError::InvalidLimit
    );
    require!(
        args.revenue_sweep_bps > 0 && args.revenue_sweep_bps <= BPS_DENOMINATOR as u16,
        AgcCreditError::RevenueSweepRequired
    );
    Ok(())
}

fn validate_spend_request(
    config: &ProtocolConfig,
    borrower: &BorrowerProfile,
    line: &CreditLine,
    policy: &SpendPolicy,
    merchant: &Merchant,
    merchant_key: Pubkey,
    amount_usdc: u64,
    now: u64,
) -> Result<()> {
    require!(borrower.status == BorrowerStatus::Active, AgcCreditError::BorrowerNotActive);
    require!(line.status == CreditLineStatus::Active, AgcCreditError::CreditLineNotActive);
    require!(policy.status == PolicyStatus::Active, AgcCreditError::PolicyNotActive);
    require!(now <= line.maturity_at, AgcCreditError::LineMatured);
    require!(line.available_limit_usdc >= amount_usdc, AgcCreditError::InsufficientAvailableLimit);
    require!(
        line.available_limit_usdc
            .checked_sub(amount_usdc)
            .ok_or(AgcCreditError::MathOverflow)?
            >= policy.min_available_limit_after_spend_usdc,
        AgcCreditError::InsufficientAvailableLimit
    );
    require!(
        amount_usdc <= policy.max_per_transaction_usdc,
        AgcCreditError::PerTransactionLimitExceeded
    );
    require!(
        amount_usdc <= policy.human_approval_threshold_usdc,
        AgcCreditError::HumanApprovalRequired
    );
    require!(
        policy.daily_spend_usdc
            .checked_add(amount_usdc)
            .ok_or(AgcCreditError::MathOverflow)?
            <= policy.max_daily_spend_usdc,
        AgcCreditError::DailyLimitExceeded
    );
    require!(
        policy.weekly_spend_usdc
            .checked_add(amount_usdc)
            .ok_or(AgcCreditError::MathOverflow)?
            <= policy.max_weekly_spend_usdc,
        AgcCreditError::WeeklyLimitExceeded
    );
    require!(
        config
            .total_daily_spend_usdc
            .checked_add(amount_usdc)
            .ok_or(AgcCreditError::MathOverflow)?
            <= config.max_daily_total_spend_usdc,
        AgcCreditError::ProtocolDailyLimitExceeded
    );
    require!(policy.revenue_sweep_bps > 0, AgcCreditError::RevenueSweepRequired);
    require!(merchant.status != MerchantStatus::Blocked, AgcCreditError::MerchantBlocked);
    require!(
        list_contains_pubkey(&policy.allowed_merchants, policy.allowed_merchant_count, &merchant_key),
        AgcCreditError::MerchantNotAllowed
    );
    require!(
        !list_contains_pubkey(&policy.blocked_merchants, policy.blocked_merchant_count, &merchant_key),
        AgcCreditError::MerchantBlocked
    );
    require!(
        list_contains_u16(&policy.allowed_categories, policy.allowed_category_count, merchant.category),
        AgcCreditError::CategoryNotAllowed
    );
    Ok(())
}

fn reset_policy_windows(policy: &mut SpendPolicy, now: u64) {
    if now.saturating_sub(policy.daily_window_started_at) >= DAY_SECONDS {
        policy.daily_spend_usdc = 0;
        policy.daily_window_started_at = now;
    }
    if now.saturating_sub(policy.weekly_window_started_at) >= WEEK_SECONDS {
        policy.weekly_spend_usdc = 0;
        policy.weekly_window_started_at = now;
    }
}

fn reset_protocol_daily_window(config: &mut ProtocolConfig, now: u64) {
    if now.saturating_sub(config.total_daily_spend_window_started_at) >= DAY_SECONDS {
        config.total_daily_spend_usdc = 0;
        config.total_daily_spend_window_started_at = now;
    }
}

fn apply_repayment_to_line(
    line: &mut CreditLine,
    borrower: &mut BorrowerProfile,
    underwriter_vault: &mut UnderwriterVault,
    amount_usdc: u64,
    source: RepaymentSource,
) -> Result<()> {
    let mut remaining = amount_usdc;

    let fee_paid = remaining.min(line.fees_due_usdc);
    line.fees_due_usdc = line
        .fees_due_usdc
        .checked_sub(fee_paid)
        .ok_or(AgcCreditError::MathOverflow)?;
    remaining = remaining
        .checked_sub(fee_paid)
        .ok_or(AgcCreditError::MathOverflow)?;

    let interest_paid = remaining.min(line.accrued_interest_usdc);
    line.accrued_interest_usdc = line
        .accrued_interest_usdc
        .checked_sub(interest_paid)
        .ok_or(AgcCreditError::MathOverflow)?;
    remaining = remaining
        .checked_sub(interest_paid)
        .ok_or(AgcCreditError::MathOverflow)?;

    let principal_paid = remaining.min(line.principal_outstanding_usdc);
    line.principal_outstanding_usdc = line
        .principal_outstanding_usdc
        .checked_sub(principal_paid)
        .ok_or(AgcCreditError::MathOverflow)?;
    line.available_limit_usdc = line
        .available_limit_usdc
        .checked_add(principal_paid)
        .ok_or(AgcCreditError::MathOverflow)?
        .min(line.principal_limit_usdc);
    borrower.total_outstanding_usdc = borrower
        .total_outstanding_usdc
        .checked_sub(principal_paid)
        .ok_or(AgcCreditError::MathOverflow)?;
    underwriter_vault.principal_repaid_usdc = underwriter_vault
        .principal_repaid_usdc
        .checked_add(principal_paid)
        .ok_or(AgcCreditError::MathOverflow)?;
    underwriter_vault.interest_earned_usdc = underwriter_vault
        .interest_earned_usdc
        .checked_add(interest_paid)
        .ok_or(AgcCreditError::MathOverflow)?;
    line.last_repayment_at = current_timestamp()?;

    if line.outstanding_balance_usdc() == 0 {
        line.status = CreditLineStatus::Repaid;
    } else if source == RepaymentSource::AutoSweep && line.status == CreditLineStatus::Delinquent {
        line.status = CreditLineStatus::Active;
    }

    Ok(())
}

fn calculate_bps(amount: u64, bps: u16) -> Result<u64> {
    Ok(((amount as u128)
        .checked_mul(bps as u128)
        .ok_or(AgcCreditError::MathOverflow)?
        / BPS_DENOMINATOR as u128) as u64)
}

fn calculate_sweep(amount: u64, sweep_bps: u16, outstanding: u64) -> Result<u64> {
    let raw = calculate_bps(amount, sweep_bps)?;
    Ok(raw.min(outstanding))
}

fn list_contains_pubkey(list: &[Pubkey; MAX_LIST_ITEMS], count: u8, value: &Pubkey) -> bool {
    list.iter().take(count as usize).any(|item| item == value)
}

fn list_contains_u16(list: &[u16; MAX_LIST_ITEMS], count: u8, value: u16) -> bool {
    list.iter().take(count as usize).any(|item| *item == value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sweep_is_capped_by_outstanding_balance() {
        assert_eq!(calculate_sweep(10_000, 3_000, 2_000).unwrap(), 2_000);
        assert_eq!(calculate_sweep(10_000, 3_000, 9_000).unwrap(), 3_000);
    }

    #[test]
    fn list_checks_only_counted_items() {
        let key = Pubkey::new_unique();
        let ignored = Pubkey::new_unique();
        let mut keys = [Pubkey::default(); MAX_LIST_ITEMS];
        keys[0] = key;
        keys[1] = ignored;
        assert!(list_contains_pubkey(&keys, 1, &key));
        assert!(!list_contains_pubkey(&keys, 1, &ignored));
    }
}
