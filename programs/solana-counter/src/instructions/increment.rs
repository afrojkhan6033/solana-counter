use anchor_lang::prelude::*;

use crate::{constants::*, error::ErrorCode, state::Counter};

#[derive(Accounts)]
pub struct Increment<'info> {
    #[account(
        mut,
        seeds = [COUNTER_SEED, authority.key().as_ref()],
        bump,
        has_one = authority @ ErrorCode::Unauthorized
    )]
    pub counter: Account<'info, Counter>,

    pub authority: Signer<'info>,
}

pub fn handle_increment(ctx: Context<Increment>) -> Result<()> {
    require!(
        ctx.accounts.counter.count < MAX_COUNT,
        ErrorCode::CounterOverflow
    );

    ctx.accounts.counter.count += 1;

    msg!(
        "Counter incremented to {}",
        ctx.accounts.counter.count
    );

    Ok(())
}
