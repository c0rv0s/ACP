use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, MintTo, Token, TokenAccount, Transfer};

use crate::{
    AgcError, CreditFacility, BUYBACK_CAMPAIGN_AUTHORITY_SEED, CREDIT_FACILITY_AUTHORITY_SEED,
    MINT_AUTHORITY_SEED, TREASURY_AUTHORITY_SEED, XAGC_AUTHORITY_SEED,
};

pub fn mint_amount<'info>(
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

pub fn mint_with_pda<'info>(
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

pub fn transfer_from_xagc_vault<'info>(
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

pub fn transfer_from_treasury<'info>(
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

pub fn transfer_from_buyback_campaign_vault<'info>(
    campaign_id: u64,
    authority_bump: u8,
    source: &Account<'info, TokenAccount>,
    destination: &Account<'info, TokenAccount>,
    authority: &UncheckedAccount<'info>,
    token_program: &Program<'info, Token>,
    amount: u64,
) -> Result<()> {
    let campaign_id_bytes = campaign_id.to_le_bytes();
    let bump = [authority_bump];
    let signer_seeds: &[&[u8]] = &[
        BUYBACK_CAMPAIGN_AUTHORITY_SEED,
        campaign_id_bytes.as_ref(),
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

pub fn burn_from_buyback_campaign_vault<'info>(
    campaign_id: u64,
    authority_bump: u8,
    mint: &Account<'info, Mint>,
    source: &Account<'info, TokenAccount>,
    authority: &UncheckedAccount<'info>,
    token_program: &Program<'info, Token>,
    amount: u64,
) -> Result<()> {
    let campaign_id_bytes = campaign_id.to_le_bytes();
    let bump = [authority_bump];
    let signer_seeds: &[&[u8]] = &[
        BUYBACK_CAMPAIGN_AUTHORITY_SEED,
        campaign_id_bytes.as_ref(),
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

pub fn transfer_from_credit_facility_vault<'info>(
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

pub fn burn_from_credit_facility_vault<'info>(
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
