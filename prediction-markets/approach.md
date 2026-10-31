# Prediction Markets: Implementation Plan

Technical plan for a full-stack prediction market on Solana. The implementation includes an Anchor program, an IDL-generated TypeScript client, and a Next.js frontend.

## Objectives

- Implement the complete market lifecycle on-chain.
- Store market and participant state in deterministic PDAs.
- Enforce betting deadlines, resolution authority, and single-claim payouts.
- Generate a type-safe TypeScript client from the program IDL.
- Build a frontend that creates markets, places positions, and displays payouts.
- Keep the account model small enough to audit and extend.

## System Architecture

### System Layers

```
┌────────────────────────────────────────────────────────┐
│                   BROWSER                              │
│  ┌──────────────────────────────────────────────────┐  │
│  │  Next.js App                                     │  │
│  │  • MarketsList, MarketCard, PositionsList        │  │
│  │  • Wallet connection via autoDiscover()          │  │
│  └──────────────────────────────────────────────────┘  │
│                         │                              │
│                         ▼                              │
│  ┌──────────────────────────────────────────────────┐  │
│  │  Generated Client (Codama)                       │  │
│  │  • Type-safe instruction builders                │  │
│  │  • Account encoders/decoders                     │  │
│  └──────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────┘
                          │ JSON-RPC
                          ▼
┌────────────────────────────────────────────────────────┐
│                   SOLANA DEVNET                        │
│  ┌─────────────────┐    ┌─────────────────────────┐   │
│  │ Anchor Program  │───▶│ Accounts (State)        │   │
│  │ (stateless)     │    │ • Market PDAs           │   │
│  │                 │    │ • UserPosition PDAs     │   │
│  └─────────────────┘    └─────────────────────────┘   │
└────────────────────────────────────────────────────────┘
```

### Data Ownership

```
        ┌─────────────┐
        │   Market    │ ← Holds pooled SOL
        │    PDA      │   (no private key!)
        └─────────────┘
              │
    ┌─────────┴─────────┐
    ▼                   ▼
┌─────────┐       ┌─────────┐
│Position │       │Position │  ← Tracks each user's bets
│ User A  │       │ User B  │
└─────────┘       └─────────┘
```

PDAs are addresses with no private key. Only the owning program can sign for them, which allows a market PDA to custody pooled funds without an external key holder.

## On-Chain Program

### Account Structures

`Market` stores global market state:

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

- `creator` is the resolution authority and part of the PDA seed.
- `market_id` is unique per creator.
- `question` is capped at 200 characters.
- `resolution_time` closes betting and gates resolution.
- `yes_pool` and `no_pool` hold lamport totals.
- `resolved` prevents double resolution.
- `outcome` distinguishes unresolved, YES, and NO.
- `bump` supports PDA re-derivation.

`UserPosition` stores a participant's cumulative exposure:

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

Both accounts use Anchor's `InitSpace` and explicit maximum lengths so rent-exempt space is calculated at compile time.

### Instructions

#### `create_market`

Creates the market PDA and initializes all fields.

```rust
pub fn create_market(
    ctx: Context<CreateMarket>,
    market_id: u64,
    question: String,
    resolution_time: i64,
) -> Result<()> {
    // Validation
    require!(resolution_time > Clock::get()?.unix_timestamp,
             ResolutionTimeInPast);
    require!(question.len() <= 200, QuestionTooLong);

    // Initialize state
    let market = &mut ctx.accounts.market;
    market.creator = ctx.accounts.creator.key();
    market.question = question;
    market.yes_pool = 0;
    market.no_pool = 0;
    // ...
}
```

Validation runs before any state change. The account uses `init`, `payer = creator`, and the seeds `b"market"`, creator public key, and `market_id`.

#### `place_bet`

Transfers lamports into the market PDA and updates the selected pool and participant position.

```
   User Wallet                    Market PDA
       │                              │
       │    transfer(amount)          │
       ├─────────────────────────────►│
       │                              │
       ▼                              ▼
  balance -= amt              yes_pool += amt
                                  (or no_pool)
```

```rust
pub fn place_bet(
    ctx: Context<PlaceBet>,
    amount: u64,
    bet_yes: bool,
) -> Result<()> {
    let market = &mut ctx.accounts.market;

    // Time check - can't bet after deadline
    require!(Clock::get()?.unix_timestamp < market.resolution_time,
             BettingClosed);

    // Transfer SOL: user → market PDA
    let transfer_ix = system_instruction::transfer(
        &ctx.accounts.user.key(),
        &ctx.accounts.market.key(),
        amount,
    );
    invoke(/* ... */)?;

    // Update pools
    if bet_yes {
        market.yes_pool = market.yes_pool.checked_add(amount)
            .ok_or(PredictionMarketError::Overflow)?;
    } else {
        market.no_pool = market.no_pool.checked_add(amount)
            .ok_or(PredictionMarketError::Overflow)?;
    }

    // Update user's position...
}
```

The amount must be greater than zero, and the transfer occurs only after the deadline check. Pool updates use `checked_add`.

`UserPosition` uses `init_if_needed`, so the first bet creates the account and later bets update it. A participant may hold both YES and NO positions.

#### `resolve_market`

The market creator records the outcome after `resolution_time`. The instruction sets `resolved` and `outcome`. `claim_winnings` requires `resolved` to be true.

Creator-only resolution is a trust assumption. A production system would use an oracle or multi-signature process.

#### `claim_winnings`

Verifies resolution and claim state, computes the payout, transfers lamports from the market PDA, and marks the position as claimed.

```
Example Market:
┌─────────────────────────────────────┐
│  YES Pool: 100 SOL                  │
│  NO Pool:   50 SOL                  │
│  Outcome: YES wins                  │
└─────────────────────────────────────┘

User bet 10 SOL on YES:
┌─────────────────────────────────────┐
│  Share of winners: 10/100 = 10%     │
│  Claim from losers: 50 × 10% = 5    │
│  Total payout: 10 + 5 = 15 SOL      │
└─────────────────────────────────────┘
```

```rust
pub fn claim_winnings(ctx: Context<ClaimWinnings>) -> Result<()> {
    let market = &ctx.accounts.market;
    let position = &mut ctx.accounts.position;

    // Guards
    require!(market.resolved, NotResolved);
    require!(!position.claimed, AlreadyClaimed);

    // Determine winning side
    let (user_bet, winning_pool, losing_pool) = match market.outcome {
        Some(true) => (position.yes_amount, market.yes_pool, market.no_pool),
        Some(false) => (position.no_amount, market.no_pool, market.yes_pool),
        None => return err!(NotResolved),
    };

    require!(user_bet > 0, NoWinnings);

    // Calculate payout
    let winnings = user_bet
        .checked_mul(losing_pool).ok_or(Overflow)?
        .checked_div(winning_pool).ok_or(Overflow)?;
    let total_payout = user_bet.checked_add(winnings).ok_or(Overflow)?;

    // Transfer from PDA to user...
    position.claimed = true;

    Ok(())
}
```

The payout formula is:

```
winnings = (user_bet / winning_pool) * losing_pool
total = user_bet + winnings
```

Integer division floors the result, leaving possible dust in the market PDA. Lamports are moved by directly mutating the program-owned market account and the recipient account.

### Validation Rules

| Rule | Purpose |
| --- | --- |
| Question length ≤ 200 | Keeps account size bounded |
| Resolution time in the future | Prevents closed markets at creation |
| Bet amount > 0 | Prevents empty positions |
| Bet before resolution time | Closes betting at the deadline |
| Creator-only resolution | Restricts outcome authority |
| Resolution before claims | Prevents payouts from unresolved markets |
| `claimed` flag | Prevents double claims |
| Checked arithmetic | Prevents overflow and pool corruption |

## Codegen Layer

### Pipeline

```
┌─────────────┐      ┌─────────────┐      ┌─────────────┐
│   lib.rs    │      │  IDL.json   │      │ TypeScript  │
│   (Rust)    │─────▶│ (Schema)    │─────▶│  Client     │
└─────────────┘      └─────────────┘      └─────────────┘
     anchor build         codama:js

   Source of           Intermediate        What frontend
   truth               representation      imports
```

```bash
npm run anchor-build   # Rust → IDL
npm run codama:js      # IDL → TypeScript
```

`anchor-build` compiles the program and emits the IDL. `codama:js` reads the IDL and generates instruction builders, PDA helpers, account decoders, and error types in `app/generated/prediction_market/`.

### Generated Responsibilities

The generated client handles:

- PDA derivation from program seeds
- Borsh serialization of instruction arguments
- Account ordering and signer/writable flags
- Account decoding, including the 8-byte discriminator
- Typed program errors
- Program ID and account discriminators
- Sync and async instruction builders

Generated files are build output and should be regenerated whenever the program changes, rather than edited by hand.

Example IDL declaration:

```json
{
  "name": "placeBet",
  "args": [
    { "name": "amount", "type": "u64" },
    { "name": "betYes", "type": "bool" }
  ]
}
```

Generated instruction builder:

```typescript
export async function getPlaceBetInstructionAsync(
  input: PlaceBetAsyncInput
): Promise<PlaceBetInstruction> {
  const marketAddress = await findMarketPda(input.creator, input.marketId);
  const positionAddress = await findPositionPda(marketAddress, input.user);

  const data = getPlaceBetInstructionDataEncoder().encode({
    amount: input.amount,
    betYes: input.betYes,
  });

  return { keys: [...], data, programId };
}
```

## Frontend Architecture

### Component Hierarchy

```
App (layout.tsx)
 └─ Providers (Solana client + wallet)
     ├─ HomePage (page.tsx)
     │   ├─ CreateMarketForm
     │   └─ MarketsList
     │       └─ MarketCard (×N)
     │
     └─ ActivityPage (activity/page.tsx)
         └─ PositionsList
             └─ PositionCard (×N)
```

`Providers` is a client component that supplies the Solana client context and wallet connection. The home page creates and lists markets. The activity page lists the connected wallet's positions.

### Data Fetching

`markets-list.tsx` fetches all program accounts, filters by the `Market` discriminator, and decodes each account with the generated client.

```typescript
const fetchMarkets = async () => {
  const response = await fetch(RPC_URL, {
    method: 'POST',
    body: JSON.stringify({
      method: 'getProgramAccounts',
      params: [
        PROGRAM_ID,
        {
          filters: [{
            memcmp: {
              offset: 0,
              bytes: "dkokXHR3DTw"  // Market discriminator
            }
          }]
        }
      ]
    })
  });

  const markets = accounts.map(acc => {
    const data = base64ToBytes(acc.data);
    return getMarketDecoder().decode(data);
  });
};

useEffect(() => {
  const interval = setInterval(fetchMarkets, 3000);
  return () => clearInterval(interval);
}, []);
```

The discriminator filter excludes `UserPosition` accounts. Decoding uses the same layout as the Rust struct. The interval is cleared on unmount.

### Transaction Flow

```typescript
const handleBet = async (betYes: boolean) => {
  const instruction = await getPlaceBetInstructionAsync({
    market: marketAddress,
    user: wallet.address,
    amount: BigInt(solAmount * LAMPORTS_PER_SOL),
    betYes,
  });

  await sendTransaction({
    instructions: [instruction],
  });
};
```

```
User clicks          Build            Sign &           Program          Account
"Bet YES"      →   Instruction   →    Send       →   Executes     →    Updated
                        │                               │
                   Generated               Validation + state change
                   client                  (checked_add, time check)
```

The same pattern applies to `create_market`, `resolve_market`, and `claim_winnings`. `u64` values are converted to `bigint` in TypeScript.

## Security and Trade-offs

### Design Decisions

| Choice | Trade-off |
| --- | --- |
| Creator resolves market | Simple but centralized trust |
| Polling vs WebSockets | Simpler code, delayed updates |
| All-or-nothing bets | No partial positions, simpler math |
| No fees | No protocol revenue, pure market |

### Protections

| Attack Vector | Protection |
| --- | --- |
| Overflow attacks | `checked_add/mul/div` |
| Double claims | `position.claimed` flag |
| Late bets | Time window validation |
| Unauthorized resolution | Creator-only check |

### Limitations

- The creator can resolve early or select an incorrect outcome.
- Market creation is unrestricted, so spam markets are possible.
- Deadline racing and transaction ordering are not mitigated.
- There are no fees, partial exits, or secondary liquidity.
- Position accounts are not closed after claims.
- The frontend polls rather than subscribing to account updates.

### Potential Extensions

- Oracle-based resolution
- Protocol fees or a treasury
- Partial position exits
- Multi-outcome markets
- Market creation fees or reputation checks
- WebSockets or an indexer

## End-to-End Flow

```
┌─────────────────────────────────────────────────────────────┐
│  1. User interacts with React component                     │
│  2. Component calls generated instruction builder           │
│  3. Wallet signs, transaction sent to Solana                │
│  4. Anchor program validates & mutates account state        │
│  5. Frontend polls for updated state, re-renders            │
└─────────────────────────────────────────────────────────────┘
```

The user action builds an instruction, the wallet signs and submits it, the program validates and mutates accounts, and the frontend refreshes from RPC state.

## File References

| Topic | File Path |
| --- | --- |
| Account structures | `anchor/programs/prediction_market/src/state.rs` |
| Program instructions | `anchor/programs/prediction_market/src/lib.rs` |
| Error definitions | `anchor/programs/prediction_market/src/errors.rs` |
| Generated client | `app/generated/prediction_market/` |
| Markets list component | `app/components/markets-list.tsx` |
| Market card component | `app/components/market-card.tsx` |
| Wallet providers | `app/components/providers.tsx` |
| Codama config | `codama.json` |
