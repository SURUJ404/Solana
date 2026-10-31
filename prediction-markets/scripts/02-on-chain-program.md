# On-Chain Program

On Solana, a program is a set of instructions that read and write accounts. The account layout is therefore the program's public interface: each instruction receives accounts, validates them, and mutates their state.

This implementation uses two account types: `Market` for global market state and `UserPosition` for a single participant's exposure. There are no order books, price histories, or off-chain references.

## Account Model

### Market

```rust
pub struct Market {
    pub creator: Pubkey,        // 32 bytes - who can resolve
    pub market_id: u64,         // 8 bytes - unique ID
    pub question: String,       // variable - "Will X happen?"
    pub resolution_time: i64,   // 8 bytes - betting deadline
    pub yes_pool: u64,          // 8 bytes - total YES lamports
    pub no_pool: u64,           // 8 bytes - total NO lamports
    pub resolved: bool,         // 1 byte - is outcome set?
    pub outcome: Option<bool>,  // 2 bytes - None, Some(true), Some(false)
    pub bump: u8,               // 1 byte - PDA bump seed
}
```

Field descriptions:

- `creator` is the authority allowed to resolve the market. It is also part of the PDA seeds, so one creator can operate multiple markets without address collisions.
- `market_id` is a per-creator identifier. The PDA seed is `b"market" + creator + market_id`, so two creators can both use `market_id = 1` and still receive different addresses. Clients can derive a market address from the creator and ID without a registry.
- `question` is capped at 200 characters by `MAX_QUESTION_LEN`. A `String` includes a 4-byte length prefix, so the maximum length determines the account's rent-exempt size.
- `resolution_time` is an `i64` Unix timestamp. It rejects markets created in the past and closes betting after the deadline. `Clock::get()?.unix_timestamp` provides the on-chain time.
- `yes_pool` and `no_pool` are running totals of lamports on each side. The implied probability is the ratio of the two pools. No pricing curve is involved.
- `resolved` prevents double resolution. `outcome` is an `Option<bool>` representing three states: `None` (unresolved), `Some(true)` (YES wins), and `Some(false)` (NO wins).
- `bump` stores the PDA bump so later instructions can re-derive the address without receiving the bump from the client.

### UserPosition

```rust
pub struct UserPosition {
    pub market: Pubkey,      // which market this position belongs to
    pub user: Pubkey,        // owner of the position
    pub yes_amount: u64,     // lamports bet on YES
    pub no_amount: u64,      // lamports bet on NO
    pub claimed: bool,       // has the payout been claimed?
    pub bump: u8,            // PDA bump seed
}
```

Field descriptions:

- `market` and `user` bind the position to a specific market and owner. They also support constraints such as `user_position.user == user.key()`.
- `yes_amount` and `no_amount` are cumulative totals. Both may be non-zero if a participant hedges or changes sides over time. Positions are not netted; only the winning side pays out.
- `claimed` is a one-way flag. Once a payout is withdrawn, further claims are rejected. This makes claims idempotent.
- `bump` allows deterministic re-derivation of the position PDA.

### Space and Rent

Both accounts derive space with `#[derive(InitSpace)]` and `#[max_len]`.

- `Market` uses the fixed fields, the 4-byte string length prefix, and the 200-byte maximum question.
- `UserPosition` uses two public keys, two `u64` totals, a boolean, and a bump.

Accounts are allocated with `8 + Market::INIT_SPACE` and `8 + UserPosition::INIT_SPACE`. The leading 8 bytes are the Anchor discriminator.

## Instructions

### `create_market`

Creates a market PDA, stores its metadata, and initializes both pools to zero.

The `CreateMarket` accounts use:

- `init`
- `payer = creator`
- `space = 8 + Market::INIT_SPACE`
- Seeds `b"market"`, the creator public key, and the `market_id` bytes

The seeds produce a deterministic address. Creating the same creator and `market_id` twice fails because the account already exists.

The instruction validates the question length and resolution time before changing state:

```rust
pub fn create_market(
    ctx: Context<CreateMarket>,
    market_id: u64,
    question: String,
    resolution_time: i64,
) -> Result<()> {
    require!(question.len() <= MAX_QUESTION_LEN, MarketError::Overflow);

    let clock = Clock::get()?;
    require!(
        resolution_time > clock.unix_timestamp,
        MarketError::ResolutionTimeInPast
    );

    let market = &mut ctx.accounts.market;
    market.creator = ctx.accounts.creator.key();
    market.market_id = market_id;
    market.question = question;
    market.resolution_time = resolution_time;
    market.yes_pool = 0;
    market.no_pool = 0;
    market.resolved = false;
    market.outcome = None;
    market.bump = ctx.bumps.market;

    Ok(())
}
```

The handler writes every field explicitly. The bump is taken from `ctx.bumps.market`, so later instructions can validate the PDA without a client-supplied bump.

`market_id` is stored on the account even though it is part of the seeds. This lets the UI display it and lets `claim_winnings` re-derive the PDA without an extra argument. `resolution_time` is stored as `i64` to match the clock sysvar and avoid casts.

A production implementation would likely add a minimum duration, question-content validation, or a creation fee to limit spam. This implementation uses the two validation guards above.

### `place_bet`

Adds a position on YES or NO. The handler validates the input, transfers lamports into the market PDA, and updates the market and position accounts.

Validation occurs before any transfer:

- The amount must be greater than zero.
- The current time must be strictly less than `resolution_time`.

The lamport transfer uses the system program:

```rust
pub fn place_bet(
    ctx: Context<PlaceBet>,
    amount: u64,
    bet_yes: bool,
) -> Result<()> {
    require!(amount > 0, MarketError::InvalidBetAmount);

    let clock = Clock::get()?;
    let market = &ctx.accounts.market;
    require!(
        clock.unix_timestamp < market.resolution_time,
        MarketError::BettingClosed
    );

    transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user.to_account_info(),
                to: ctx.accounts.market.to_account_info(),
            },
        ),
        amount,
    )?;

    // Update pools
    let market = &mut ctx.accounts.market;
    if bet_yes {
        market.yes_pool = market.yes_pool.checked_add(amount)
            .ok_or(MarketError::Overflow)?;
    } else {
        market.no_pool = market.no_pool.checked_add(amount)
            .ok_or(MarketError::Overflow)?;
    }

    // Update user's position...
}
```

The pool totals are updated only after the transfer succeeds. Both additions use `checked_add`; wrapping a pool to zero would distort the implied price and could enable theft.

The position account uses `init_if_needed`. The first bet pays rent and initializes `market`, `user`, and `bump`; later bets reuse the account. Each bet increments either `yes_amount` or `no_amount`.

A participant may hold both YES and NO positions. This supports hedging or changing a view without closing an account. At resolution, only the winning side is paid.

The model is parimutuel, not an AMM. There is no curve, slippage calculation, or price protection. The implied price is derived by the UI from the pool ratio.

### `resolve_market`

Resolution is performed by the market creator after `resolution_time`. The instruction records the winning outcome and sets the market's resolved state.

`claim_winnings` requires `market.resolved` before any payout, so resolution must complete first. Creator-only resolution is a trust assumption; a production system would typically use an oracle or multi-signature process.

### `claim_winnings`

Settles a resolved market. This is the only instruction that moves lamports out of the market PDA. The handler:

1. Verifies that the market is resolved.
2. Verifies that the position has not been claimed.
3. Selects the participant's winning amount and the total winning and losing pools.
4. Computes the payout.
5. Transfers lamports.
6. Marks the position as claimed.

```rust
pub fn claim_winnings(ctx: Context<ClaimWinnings>) -> Result<()> {
    let market = &ctx.accounts.market;
    let position = &ctx.accounts.user_position;

    // Guards
    require!(market.resolved, MarketError::NotResolved);
    require!(!position.claimed, MarketError::AlreadyClaimed);

    // Determine winning side
    let outcome = market.outcome.unwrap();
    let (user_winning_bet, total_winning_pool, total_losing_pool) = if outcome {
        (position.yes_amount, market.yes_pool, market.no_pool)
    } else {
        (position.no_amount, market.no_pool, market.yes_pool)
    };

    require!(user_winning_bet > 0, MarketError::NoWinnings);

    // Calculate payout
    let winnings = (user_winning_bet as u128)
        .checked_mul(total_losing_pool as u128)
        .ok_or(MarketError::Overflow)?
        .checked_div(total_winning_pool as u128)
        .ok_or(MarketError::Overflow)? as u64;
    let total_payout = user_winning_bet
        .checked_add(winnings)
        .ok_or(MarketError::Overflow)?;

    // Transfer from PDA to user
    let market_account_info = ctx.accounts.market.to_account_info();
    let user_account_info = ctx.accounts.user.to_account_info();

    **market_account_info.try_borrow_mut_lamports()? -= total_payout;
    **user_account_info.try_borrow_mut_lamports()? += total_payout;

    let position = &mut ctx.accounts.user_position;
    position.claimed = true;

    Ok(())
}
```

The payout formula is:

```
winnings = (user_bet / winning_pool) * losing_pool
total = user_bet + winnings
```

The implementation multiplies in `u128` before dividing to avoid overflow. Integer division floors the result, so a few lamports may remain in the market PDA as dust. This is deterministic and expected.

Lamports are moved by mutating the account balances directly. The system program only moves lamports from signer accounts, and the market PDA is program-owned. The account constraints validate that the `UserPosition` PDA matches the signer.

After the transfer, `position.claimed` is set to `true`. A repeated claim fails at the guard. The position account is not closed in this version; a `close = user` constraint could reclaim rent after a claim.

If a participant holds both sides, only the winning side pays out. The losing side remains in the pool and is distributed to winners, including the participant if they hold a winning position. All lamports entering the market are either paid out or remain as dust.

## Design Notes

- Validation always runs before state changes.
- PDA seeds make market and position addresses deterministic and discoverable.
- The program does not maintain a global market list. Enumerating accounts is an RPC or indexer concern.
- One `UserPosition` account aggregates all of a participant's bets for a market.
- Both YES and NO positions may be non-zero.
- Payout math uses `u128` intermediates and leaves dust from integer division.
- Closing position accounts after claims is optional and not implemented here.
