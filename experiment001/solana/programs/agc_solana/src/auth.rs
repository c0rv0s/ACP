use anchor_lang::prelude::*;

use crate::{AgcError, Keeper, ProtocolState, RequiredKeeperPermission};

pub fn assert_market_reporter_or_admin(
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

pub fn assert_oracle_reporter_or_admin(
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

pub fn assert_credit_operator_or_admin(
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

pub fn assert_risk_authority_or_admin(state: &ProtocolState, authority_key: Pubkey) -> Result<()> {
    require!(
        authority_key == state.admin || authority_key == state.risk_admin,
        AgcError::Unauthorized
    );
    Ok(())
}

pub fn assert_emergency_authority_or_admin(state: &ProtocolState, authority_key: Pubkey) -> Result<()> {
    require!(
        authority_key == state.admin || authority_key == state.emergency_admin,
        AgcError::Unauthorized
    );
    Ok(())
}

pub fn assert_keeper_permission_or_admin(
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
