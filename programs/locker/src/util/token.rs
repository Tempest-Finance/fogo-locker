use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer};
use anchor_spl::token_interface::{Mint, TokenAccount as TokenAccount2022, TokenInterface};

use crate::VestingEscrow;

pub fn transfer_to_escrow<'info>(
    sender: &Signer<'info>,
    sender_token: &Account<'info, TokenAccount>,
    escrow_token: &Account<'info, TokenAccount>,
    token_program: &Program<'info, Token>,
    amount: u64,
) -> Result<()> {
    anchor_spl::token::transfer(
        CpiContext::new(
            token_program.to_account_info(),
            Transfer {
                from: sender_token.to_account_info(),
                to: escrow_token.to_account_info(),
                authority: sender.to_account_info(),
            },
        ),
        amount,
    )?;

    Ok(())
}

pub fn transfer_to_user<'info>(
    escrow: &AccountLoader<'info, VestingEscrow>,
    escrow_token: &Account<'info, TokenAccount>,
    recipient_token: &Account<'info, TokenAccount>,
    token_program: &Program<'info, Token>,
    amount: u64,
) -> Result<()> {
    let escrow_state = escrow.load()?;
    let escrow_seeds = escrow_seeds!(escrow_state);

    anchor_spl::token::transfer(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            Transfer {
                from: escrow_token.to_account_info(),
                to: recipient_token.to_account_info(),
                authority: escrow.to_account_info(),
            },
            &[&escrow_seeds[..]],
        ),
        amount,
    )?;
    Ok(())
}

pub fn transfer_to_escrow_with_session<'c: 'info, 'info>(
    signer_or_session: &AccountInfo<'info>,
    program_signer: &AccountInfo<'info>,
    program_signer_bump: u8,
    token_mint: &InterfaceAccount<'info, Mint>,
    sender_token: &InterfaceAccount<'info, TokenAccount2022>,
    escrow_token: &InterfaceAccount<'info, TokenAccount2022>,
    token_program: &Interface<'info, TokenInterface>,
    amount: u64,
) -> Result<()> {
    use anchor_lang::solana_program;
    use fogo_sessions_sdk::token::PROGRAM_SIGNER_SEED;

    let instruction = fogo_sessions_sdk::token::instruction::transfer_checked(
        token_program.key,
        &sender_token.key(),
        &token_mint.key(),
        &escrow_token.key(),
        signer_or_session.key,
        Some(program_signer.key),
        amount,
        token_mint.decimals,
    )?;

    let account_infos = vec![
        sender_token.to_account_info(),
        token_mint.to_account_info(),
        escrow_token.to_account_info(),
        signer_or_session.clone(),
        program_signer.clone(),
    ];

    let signer_seeds: &[&[u8]] = &[PROGRAM_SIGNER_SEED, &[program_signer_bump]];
    solana_program::program::invoke_signed(&instruction, &account_infos, &[signer_seeds])?;

    Ok(())
}
