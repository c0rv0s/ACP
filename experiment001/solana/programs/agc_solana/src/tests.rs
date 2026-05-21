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
            pyth_receiver_program: Pubkey::default(),
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
            oracle_source: OracleSource::Manual,
            oracle_feed: Pubkey::new_unique(),
            pyth_price_feed_id: [0; 32],
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
            oracle_source: asset.oracle_source,
            pyth_price_feed_id: asset.pyth_price_feed_id,
            price_quote_x18: PRICE_SCALE,
            confidence_bps: 25,
            updated_at,
            publish_time: updated_at,
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

    fn buyback_config() -> BuybackCampaignConfig {
        BuybackCampaignConfig {
            total_usdc: 1_000_000,
            min_total_agc_out: 1_900_000_000,
            max_slice_usdc: 100_000,
            slice_interval_seconds: 300,
            start_after: 1_000,
            expires_at: 10_000,
            adapter_usdc_account: Pubkey::new_unique(),
        }
    }

    fn buyback_campaign() -> BuybackCampaign {
        BuybackCampaign {
            campaign_id: 7,
            status: BuybackCampaignStatus::Active,
            total_usdc: 1_000_000,
            remaining_usdc: 1_000_000,
            spent_usdc: 0,
            min_total_agc_out: 1_900_000_000,
            agc_burned: 0,
            max_slice_usdc: 100_000,
            slice_interval_seconds: 300,
            started_at: 1_000,
            expires_at: 10_000,
            last_slice_at: 0,
            slice_count: 0,
            adapter_usdc_account: Pubkey::new_unique(),
            usdc_escrow: Pubkey::new_unique(),
            agc_vault: Pubkey::new_unique(),
            bump: 0,
            authority_bump: 0,
            usdc_escrow_bump: 0,
            agc_vault_bump: 0,
            reserved: [0; 128],
        }
    }

    fn pyth_update_bytes(feed_id: [u8; 32], price: i64, conf: u64, publish_time: i64) -> Vec<u8> {
        let mut data = pyth_price_update_v2_discriminator().to_vec();
        let update = PythPriceUpdateV2AccountData {
            write_authority: Pubkey::new_unique(),
            verification_level: PythVerificationLevel::Full,
            price_message: PythPriceFeedMessage {
                feed_id,
                price,
                conf,
                exponent: -8,
                publish_time,
                prev_publish_time: publish_time - 1,
                ema_price: price,
                ema_conf: conf,
            },
            posted_slot: 10,
        };
        update.serialize(&mut data).unwrap();
        data
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
            oracle_source: OracleSource::Manual,
            oracle_feed: Pubkey::new_unique(),
            pyth_price_feed_id: [0; 32],
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
        assert!(validate_collateral_asset_config(config, Pubkey::default()).is_ok());

        config.reserve_weight_bps = 10_001;
        assert!(validate_collateral_asset_config(config, Pubkey::default()).is_err());

        config.reserve_weight_bps = 6_000;
        config.collateral_factor_bps = 7_000;
        assert!(validate_collateral_asset_config(config, Pubkey::default()).is_err());

        config.collateral_factor_bps = 5_000;
        config.oracle_feed = Pubkey::default();
        assert!(validate_collateral_asset_config(config, Pubkey::default()).is_err());
    }

    #[test]
    fn pyth_collateral_config_requires_receiver_and_feed_id() {
        let mut feed_id = [0_u8; 32];
        feed_id[31] = 1;
        let receiver = Pubkey::new_unique();
        let mut config = CollateralAssetConfig {
            oracle_source: OracleSource::Pyth,
            oracle_feed: Pubkey::default(),
            pyth_price_feed_id: feed_id,
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

        assert!(validate_collateral_asset_config(config, receiver).is_ok());
        assert!(validate_collateral_asset_config(config, Pubkey::default()).is_err());

        config.pyth_price_feed_id = [0; 32];
        assert!(validate_collateral_asset_config(config, receiver).is_err());
    }

    #[test]
    fn pyth_price_conversion_and_confidence_are_bounded() {
        let btc_price = pyth_price_to_quote_x18(6_500_000_000_000, -8).unwrap();
        assert_eq!(btc_price, 65_000 * PRICE_SCALE);
        assert_eq!(
            pyth_confidence_bps(6_500_000_000_000, 6_500_000_000).unwrap(),
            10
        );

        assert!(pyth_price_to_quote_x18(0, -8).is_err());
        assert!(pyth_price_to_quote_x18(1, -19).is_err());
        assert!(pyth_confidence_bps(100, 101).is_err());
    }

    #[test]
    fn pyth_price_update_decoder_checks_discriminator_and_feed() {
        let mut feed_id = [0_u8; 32];
        feed_id[0] = 9;
        let data = pyth_update_bytes(feed_id, 6_500_000_000_000, 10_000_000, 2_000);
        let update = decode_pyth_price_update(&data).unwrap();
        assert_eq!(update.verification_level, PythVerificationLevel::Full);
        assert_eq!(update.price_message.feed_id, feed_id);

        let mut bad_data = data;
        bad_data[0] ^= 1;
        assert!(decode_pyth_price_update(&bad_data).is_err());
    }

    #[test]
    fn buyback_campaign_configs_are_guarded() {
        let mut config = buyback_config();
        assert!(validate_buyback_campaign_config(config, 2_000_000, 500).is_ok());

        config.total_usdc = 0;
        assert!(validate_buyback_campaign_config(config, 2_000_000, 500).is_err());

        config = buyback_config();
        config.total_usdc = 3_000_000;
        assert!(validate_buyback_campaign_config(config, 2_000_000, 500).is_err());

        config = buyback_config();
        config.max_slice_usdc = config.total_usdc + 1;
        assert!(validate_buyback_campaign_config(config, 2_000_000, 500).is_err());

        config = buyback_config();
        config.expires_at = config.start_after;
        assert!(validate_buyback_campaign_config(config, 2_000_000, 500).is_err());
    }

    #[test]
    fn buyback_slices_enforce_twap_cadence_and_burn_output() {
        let mut campaign = buyback_campaign();
        let first_slice = BuybackSliceArgs {
            usdc_amount: 100_000,
            agc_amount_to_burn: 200_000_000,
            min_agc_out: 190_000_000,
            deadline: 1_500,
        };
        assert!(validate_buyback_slice(&campaign, first_slice, 1_200).is_ok());

        let too_large = BuybackSliceArgs {
            usdc_amount: 100_001,
            ..first_slice
        };
        assert!(validate_buyback_slice(&campaign, too_large, 1_200).is_err());

        let weak_output = BuybackSliceArgs {
            agc_amount_to_burn: 100_000_000,
            min_agc_out: 190_000_000,
            ..first_slice
        };
        assert!(validate_buyback_slice(&campaign, weak_output, 1_200).is_err());

        campaign.slice_count = 1;
        campaign.last_slice_at = 1_200;
        assert!(validate_buyback_slice(&campaign, first_slice, 1_400).is_err());
        assert!(validate_buyback_slice(&campaign, first_slice, 1_500).is_ok());

        let expired = BuybackSliceArgs {
            deadline: 1_499,
            ..first_slice
        };
        assert!(validate_buyback_slice(&campaign, expired, 1_500).is_err());
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
    fn admin_accept_migrates_roles_still_held_by_previous_admin() {
        let previous_admin = Pubkey::new_unique();
        let new_admin = Pubkey::new_unique();
        let mut state = test_state();
        state.admin = previous_admin;
        state.pending_admin = new_admin;
        state.risk_admin = previous_admin;
        state.emergency_admin = previous_admin;

        let returned_previous = accept_admin_inner(&mut state, new_admin).unwrap();

        assert_eq!(returned_previous, previous_admin);
        assert_eq!(state.admin, new_admin);
        assert_eq!(state.pending_admin, Pubkey::default());
        assert_eq!(state.risk_admin, new_admin);
        assert_eq!(state.emergency_admin, new_admin);
    }

    #[test]
    fn admin_accept_preserves_explicit_separate_roles() {
        let previous_admin = Pubkey::new_unique();
        let new_admin = Pubkey::new_unique();
        let risk_admin = Pubkey::new_unique();
        let emergency_admin = Pubkey::new_unique();
        let mut state = test_state();
        state.admin = previous_admin;
        state.pending_admin = new_admin;
        state.risk_admin = risk_admin;
        state.emergency_admin = emergency_admin;

        accept_admin_inner(&mut state, new_admin).unwrap();

        assert_eq!(state.admin, new_admin);
        assert_eq!(state.risk_admin, risk_admin);
        assert_eq!(state.emergency_admin, emergency_admin);
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
