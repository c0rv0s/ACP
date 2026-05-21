use anchor_lang::prelude::*;

use crate::math::{checked_add_u128, checked_div_u128, checked_mul_u128, pow10_u128};
use crate::{AgcError, BPS, PYTH_PRICE_UPDATE_V2_DISCRIMINATOR};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PythVerificationLevel {
    Partial { num_signatures: u8 },
    Full,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PythPriceFeedMessage {
    pub feed_id: [u8; 32],
    pub price: i64,
    pub conf: u64,
    pub exponent: i32,
    pub publish_time: i64,
    pub prev_publish_time: i64,
    pub ema_price: i64,
    pub ema_conf: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PythPriceUpdateV2AccountData {
    pub write_authority: Pubkey,
    pub verification_level: PythVerificationLevel,
    pub price_message: PythPriceFeedMessage,
    pub posted_slot: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PythPrice {
    pub price: i64,
    pub conf: u64,
    pub exponent: i32,
    pub publish_time: i64,
}

pub fn read_verified_pyth_price(
    price_update: AccountInfo,
    expected_receiver_program: Pubkey,
    expected_feed_id: [u8; 32],
    maximum_age: u64,
    now: u64,
) -> Result<PythPrice> {
    require!(
        expected_receiver_program != Pubkey::default(),
        AgcError::InvalidOraclePrice
    );
    require_keys_eq!(
        *price_update.owner,
        expected_receiver_program,
        AgcError::InvalidOraclePrice
    );

    let data = price_update.try_borrow_data()?;
    let update = decode_pyth_price_update(&data)?;
    require!(
        update.verification_level == PythVerificationLevel::Full,
        AgcError::InvalidOraclePrice
    );
    require!(
        update.price_message.feed_id == expected_feed_id,
        AgcError::InvalidOraclePrice
    );
    require!(update.price_message.price > 0, AgcError::InvalidPrice);
    require!(
        update.price_message.publish_time >= 0,
        AgcError::InvalidOraclePrice
    );
    let publish_time = update.price_message.publish_time as u64;
    require!(publish_time <= now, AgcError::InvalidOraclePrice);
    require!(
        now.saturating_sub(publish_time) <= maximum_age,
        AgcError::InvalidOraclePrice
    );

    Ok(PythPrice {
        price: update.price_message.price,
        conf: update.price_message.conf,
        exponent: update.price_message.exponent,
        publish_time: update.price_message.publish_time,
    })
}

pub fn decode_pyth_price_update(data: &[u8]) -> Result<PythPriceUpdateV2AccountData> {
    require!(data.len() >= 8, AgcError::InvalidOraclePrice);
    let expected_discriminator = pyth_price_update_v2_discriminator();
    require!(
        data[..8] == expected_discriminator[..],
        AgcError::InvalidOraclePrice
    );
    PythPriceUpdateV2AccountData::deserialize(&mut &data[8..])
        .map_err(|_| error!(AgcError::InvalidOraclePrice))
}

pub fn pyth_price_update_v2_discriminator() -> [u8; 8] {
    PYTH_PRICE_UPDATE_V2_DISCRIMINATOR
}

pub fn pyth_price_to_quote_x18(price: i64, exponent: i32) -> Result<u128> {
    require!(price > 0, AgcError::InvalidPrice);
    let price = price as u128;
    let x18_exponent = exponent.checked_add(18).ok_or(AgcError::MathOverflow)?;
    let scaled = if x18_exponent >= 0 {
        let multiplier =
            pow10_u128(u8::try_from(x18_exponent).map_err(|_| error!(AgcError::MathOverflow))?)?;
        checked_mul_u128(price, multiplier)?
    } else {
        let divisor =
            pow10_u128(u8::try_from(-x18_exponent).map_err(|_| error!(AgcError::MathOverflow))?)?;
        checked_div_u128(price, divisor)?
    };
    require!(scaled > 0, AgcError::InvalidPrice);
    Ok(scaled)
}

pub fn pyth_confidence_bps(price: i64, conf: u64) -> Result<u16> {
    require!(price > 0, AgcError::InvalidPrice);
    let price = price as u128;
    let scaled_conf = checked_mul_u128(conf as u128, BPS)?;
    let confidence_bps = checked_div_u128(
        checked_add_u128(scaled_conf, price.saturating_sub(1), AgcError::MathOverflow)?,
        price,
    )?;
    require!(confidence_bps <= BPS, AgcError::InvalidOraclePrice);
    u16::try_from(confidence_bps).map_err(|_| error!(AgcError::AmountTooLarge))
}
