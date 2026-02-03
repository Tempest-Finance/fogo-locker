use anchor_spl::memo::Memo;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::util::{transfer_to_user2, MemoTransferContext, TRANSFER_MEMO_CLAIM_VESTING};
use crate::*;

#[event_cpi]
#[derive(Accounts)]
pub struct ClaimWithSessionCtx<'info> {
    #[account(
        mut,
        has_one = token_mint,
        constraint = escrow.load()?.cancelled_at == 0 @ LockerError::AlreadyCancelled
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

    /// CHECK: Session or recipient - validated to match escrow.recipient
    pub signer_or_session: AccountInfo<'info>,

    #[account(
        mut,
        constraint = recipient_token.key() != escrow_token.key() @ LockerError::InvalidRecipientTokenAccount,
        constraint = recipient_token.mint == token_mint.key() @ LockerError::InvalidRecipientTokenAccount
    )]
    pub recipient_token: Box<InterfaceAccount<'info, TokenAccount>>,

    pub memo_program: Program<'info, Memo>,
    pub token_program: Interface<'info, TokenInterface>,
}

pub fn handle_claim_with_session(
    ctx: Context<ClaimWithSessionCtx>,
    max_amount: u64,
) -> Result<()> {
    use fogo_sessions_sdk::session::Session;

    let user_pubkey = Session::extract_user_from_signer_or_session(
        &ctx.accounts.signer_or_session,
        ctx.program_id,
    )
    .map_err(|_| LockerError::InvalidSession)?;

    {
        let escrow = ctx.accounts.escrow.load()?;
        require!(
            escrow.recipient == user_pubkey,
            LockerError::NotPermitToDoThisAction
        );
    }

    require!(
        ctx.accounts.recipient_token.owner == user_pubkey,
        LockerError::InvalidTokenOwner
    );

    let mut escrow = ctx.accounts.escrow.load_mut()?;
    let amount = escrow.claim(max_amount)?;
    drop(escrow);

    transfer_to_user2(
        &ctx.accounts.escrow,
        &ctx.accounts.token_mint,
        &ctx.accounts.escrow_token.to_account_info(),
        &ctx.accounts.recipient_token,
        &ctx.accounts.token_program,
        Some(MemoTransferContext {
            memo_program: &ctx.accounts.memo_program,
            memo: TRANSFER_MEMO_CLAIM_VESTING.as_bytes(),
        }),
        amount,
        None,
    )?;

    let current_ts = Clock::get()?.unix_timestamp as u64;
    emit_cpi!(EventClaim {
        amount,
        current_ts,
        escrow: ctx.accounts.escrow.key(),
    });
    Ok(())
}
