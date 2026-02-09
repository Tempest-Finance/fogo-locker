use anchor_spl::token::spl_token;
use anchor_spl::token_2022::spl_token_2022;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::util::{
    calculate_transfer_fee_included_amount, parse_remaining_accounts,
    token::transfer_to_escrow_with_session, validate_mint, AccountsType, ParsedRemainingAccounts,
};
use crate::TokenProgramFlag::{UseSplToken, UseToken2022};
use crate::*;

#[event_cpi]
#[derive(Accounts)]
pub struct CreateVestingEscrowWithSessionCtx<'info> {
    pub base: Signer<'info>,

    #[account(
        init,
        seeds = [b"escrow".as_ref(), base.key().as_ref()],
        bump,
        payer = payer,
        space = 8 + VestingEscrow::INIT_SPACE
    )]
    pub escrow: AccountLoader<'info, VestingEscrow>,

    pub token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = escrow,
        associated_token::token_program = token_program
    )]
    pub escrow_token: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK: Session or direct signer - must be a signer
    #[account(signer)]
    pub signer_or_session: AccountInfo<'info>,

    #[account(
        mut,
        constraint = sender_token.mint == token_mint.key() @ LockerError::InvalidEscrowTokenAddress
    )]
    pub sender_token: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK: recipient
    pub recipient: UncheckedAccount<'info>,

    /// CHECK: Program signer PDA
    pub program_signer: AccountInfo<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn handle_create_vesting_escrow_with_session<'c: 'info, 'info>(
    ctx: Context<'_, '_, 'c, 'info, CreateVestingEscrowWithSessionCtx<'info>>,
    params: &CreateVestingEscrowParameters,
    remaining_accounts_info: Option<RemainingAccountsInfo>,
) -> Result<()> {
    use fogo_sessions_sdk::session::{is_session, Session};
    use fogo_sessions_sdk::token::PROGRAM_SIGNER_SEED;

    require!(
        is_session(&ctx.accounts.signer_or_session),
        LockerError::InvalidSession
    );

    validate_mint(&ctx.accounts.token_mint, true)?;

    let token_mint_info = ctx.accounts.token_mint.to_account_info();
    let token_program_flag = match *token_mint_info.owner {
        spl_token::ID => Ok(UseSplToken),
        spl_token_2022::ID => Ok(UseToken2022),
        _ => Err(LockerError::IncorrectTokenProgramId),
    }?;

    let user_pubkey = Session::extract_user_from_signer_or_session(
        &ctx.accounts.signer_or_session,
        ctx.program_id,
    )
    .map_err(|_| LockerError::InvalidSession)?;

    require!(
        ctx.accounts.sender_token.owner == user_pubkey,
        LockerError::InvalidTokenOwner
    );

    require!(
        ctx.accounts.recipient.key() == user_pubkey,
        LockerError::InvalidSession
    );

    let (expected_signer, bump) =
        Pubkey::find_program_address(&[PROGRAM_SIGNER_SEED], ctx.program_id);

    require!(
        expected_signer == ctx.accounts.program_signer.key(),
        LockerError::InvalidSession
    );

    params.init_escrow(
        &ctx.accounts.escrow,
        ctx.accounts.recipient.key(),
        ctx.accounts.sender_token.mint,
        user_pubkey,
        ctx.accounts.base.key(),
        ctx.bumps.escrow,
        token_program_flag.into(),
    )?;

    let mut remaining_accounts = ctx.remaining_accounts;
    let parsed_transfer_hook_accounts = match remaining_accounts_info {
        Some(info) => parse_remaining_accounts(
            &mut remaining_accounts,
            &info.slices,
            &[AccountsType::TransferHookEscrow],
        )?,
        None => ParsedRemainingAccounts::default(),
    };

    transfer_to_escrow_with_session(
        &ctx.accounts.signer_or_session,
        &ctx.accounts.program_signer,
        bump,
        &ctx.accounts.token_mint,
        &ctx.accounts.sender_token,
        &ctx.accounts.escrow_token.to_account_info(),
        &ctx.accounts.token_program,
        calculate_transfer_fee_included_amount(
            params.get_total_deposit_amount()?,
            &ctx.accounts.token_mint,
        )?,
        parsed_transfer_hook_accounts.transfer_hook_escrow,
    )?;

    let &CreateVestingEscrowParameters {
        vesting_start_time,
        cliff_time,
        frequency,
        cliff_unlock_amount,
        amount_per_period,
        number_of_period,
        update_recipient_mode,
        cancel_mode,
    } = params;
    emit_cpi!(EventCreateVestingEscrow {
        vesting_start_time,
        cliff_time,
        frequency,
        cliff_unlock_amount,
        amount_per_period,
        number_of_period,
        recipient: ctx.accounts.recipient.key(),
        escrow: ctx.accounts.escrow.key(),
        update_recipient_mode,
        cancel_mode,
    });
    Ok(())
}
