use anchor_lang::prelude::*;

use crate::AgcError;

pub fn current_timestamp() -> Result<u64> {
    let timestamp = Clock::get()?.unix_timestamp;
    require!(timestamp >= 0, AgcError::InvalidClock);
    Ok(timestamp as u64)
}

pub fn pow10_u64(exp: u8) -> Result<u64> {
    let mut value = 1_u64;
    for _ in 0..exp {
        value = value.checked_mul(10).ok_or(AgcError::MathOverflow)?;
    }
    Ok(value)
}

pub fn pow10_u128(exp: u8) -> Result<u128> {
    let mut value = 1_u128;
    for _ in 0..exp {
        value = value.checked_mul(10).ok_or(AgcError::MathOverflow)?;
    }
    Ok(value)
}

pub fn checked_mul_u128(lhs: u128, rhs: u128) -> Result<u128> {
    lhs.checked_mul(rhs).ok_or(error!(AgcError::MathOverflow))
}

pub fn checked_add_u128(lhs: u128, rhs: u128, err: AgcError) -> Result<u128> {
    lhs.checked_add(rhs).ok_or(error!(err))
}

pub fn checked_div_u128(lhs: u128, rhs: u128) -> Result<u128> {
    require!(rhs > 0, AgcError::MathOverflow);
    Ok(lhs / rhs)
}

pub fn mul_div(lhs: u128, rhs: u128, denominator: u128) -> Result<u128> {
    if denominator == 0 {
        return Ok(0);
    }
    checked_div_u128(checked_mul_u128(lhs, rhs)?, denominator)
}

pub fn safe_div(numerator: u128, denominator: u128) -> Result<u128> {
    if denominator == 0 {
        return Ok(0);
    }
    Ok(numerator / denominator)
}

pub fn positive_delta(lhs: u128, rhs: u128) -> u128 {
    lhs.saturating_sub(rhs)
}

pub fn min_u128(lhs: u128, rhs: u128) -> u128 {
    lhs.min(rhs)
}

pub fn max_u128(lhs: u128, rhs: u128) -> u128 {
    lhs.max(rhs)
}
