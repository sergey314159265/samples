use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    transfer_checked,
    Mint,
    TokenAccount,
    TokenInterface,
    TransferChecked,
};

use crate::{
    constants::{
        CONTRIBUTE_SEED,
        PRESALE_SEED,
    },
    error::PresaleError,
    state::{
        contribution::ContributionState,
        presale::{
            PresaleState,
            PresaleType,
        },
    },
    utils::validate_presale_pda,
};
#[event]
pub struct TokensClaimed {
    pub claimer: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[derive(Accounts)]
pub struct ClaimTokens<'info> {
    /// CHECK: Presale account (PDA)
    #[account(mut)]
    pub presale: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [CONTRIBUTE_SEED, presale.key().as_ref(), user.key().as_ref()],
        bump
    )]
    pub contribution: Account<'info, ContributionState>,

    #[account(
        mut,
        associated_token::mint = token,
        associated_token::authority = presale,
        associated_token::token_program = token_program
    )]
    pub token_vault_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = token,
        associated_token::authority = user,
        associated_token::token_program = token_program
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub token: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn claim_tokens(ctx: Context<ClaimTokens>) -> Result<()> {
    let presale = &mut PresaleState::deserialize_data(
        &ctx.accounts.presale,
        &ctx.accounts.user,
        &ctx.accounts.system_program,
    )?;
    let contribution = &mut ctx.accounts.contribution;

    let presale_bump = validate_presale_pda(
        presale,
        ctx.accounts.presale.key(),
        ctx.accounts.token.key(),
    )?;

    require!(
        presale.token == ctx.accounts.token.key(),
        PresaleError::Invalid
    );
    require!(presale.presale_ended, PresaleError::PresaleNotFinalized);
    require!(!presale.presale_canceled, PresaleError::PresaleCanceled);
    require!(!presale.presale_refund, PresaleError::PresaleRefund);
    require!(contribution.amount > 0, PresaleError::NoTokensToClaim);
    require!(
        contribution.contributor == ctx.accounts.user.key(),
        PresaleError::Invalid
    );

    let token_key = ctx.accounts.token.key();

    let signer: &[&[&[u8]]] = &[&[
        PRESALE_SEED,
        token_key.as_ref(),
        presale.identifier.as_ref(),
        &[presale_bump],
    ]];

    if presale.presale_type == PresaleType::HardCapped {
        let tokens_to_be_claimed = contribution.tokens_purchased;

        contribution.amount = 0;
        contribution.tokens_purchased = 0;

        transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.token_vault_account.to_account_info(),
                    mint: ctx.accounts.token.to_account_info(),
                    to: ctx.accounts.user_token_account.to_account_info(),
                    authority: ctx.accounts.presale.to_account_info(),
                },
                signer,
            ),
            tokens_to_be_claimed,
            ctx.accounts.token.decimals,
        )?;

        emit!(TokensClaimed {
            claimer: ctx.accounts.user.key(),
            amount: tokens_to_be_claimed,
            timestamp: Clock::get().unwrap().unix_timestamp
        });
    } else if presale.presale_type == PresaleType::FairLaunch {
        let tokens_to_be_claimed = (contribution.amount as u128)
            .checked_mul(10000)
            .and_then(|f| f.checked_mul(presale.total_tokens_sold as u128))
            .and_then(|f| f.checked_div(presale.total_raised as u128))
            .and_then(|f| f.checked_div(10000))
            .and_then(|f| u64::try_from(f).ok())
            .ok_or(PresaleError::ArithmeticOverflow)?;

        transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.token_vault_account.to_account_info(),
                    mint: ctx.accounts.token.to_account_info(),
                    to: ctx.accounts.user_token_account.to_account_info(),
                    authority: ctx.accounts.presale.to_account_info(),
                },
                signer,
            ),
            tokens_to_be_claimed,
            ctx.accounts.token.decimals,
        )?;

        emit!(TokensClaimed {
            claimer: ctx.accounts.user.key(),
            amount: tokens_to_be_claimed,
            timestamp: Clock::get().unwrap().unix_timestamp
        });
    }

    presale.serialize_data(&ctx.accounts.presale)?;

    Ok(())
}
