use anchor_lang::prelude::*;

use crate::{constants::*, state::Counter};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + Counter::INIT_SPACE,
        seeds = [COUNTER_SEED, authority.key().as_ref()],
        bump
    )]
    pub counter: Account<'info, Counter>,

    pub system_program: Program<'info, System>,
}

pub fn handle_initialize(ctx: Context<Initialize>) -> Result<()> {
    ctx.accounts.counter.count = 0;
    ctx.accounts.counter.authority = ctx.accounts.authority.key();

    msg!("Counter initialized for authority");
    Ok(())
}
