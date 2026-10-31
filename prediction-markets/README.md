# Prediction Markets

A full-stack binary prediction market implementation on Solana. The repository contains an Anchor program, a Codama-generated TypeScript client, and a Next.js frontend that creates markets, places YES/NO positions in SOL, resolves markets, and claims payouts.

The implementation targets Solana devnet and uses a parimutuel payout model: winners receive their stake plus a proportional share of the losing pool.

## Architecture

| Layer | Technology |
| --- | --- |
| On-chain program | Anchor 0.31.1 (Rust) |
| Program client | Codama-generated TypeScript from the Anchor IDL |
| Frontend | Next.js 16, React 19, TypeScript, Tailwind CSS v4 |
| Solana client | `@solana/client`, `@solana/react-hooks` |
| Network | Devnet (`https://api.devnet.solana.com`), commitment `confirmed` |

The frontend reads directly from the RPC. There is no indexer, database, or backend service.

## Program ID

```
33ijjeqroG7Syj3fqJn7oebG6iAwk1pGq5deBNYzkr1h
```

Configured under `[programs.devnet]` in `anchor/Anchor.toml`.

## On-Chain Model

### Accounts

| Account | Seeds | Purpose |
| --- | --- | --- |
| `Market` | `["market", creator, market_id.to_le_bytes()]` | Global market state: question, deadline, pools, resolution, creator. |
| `UserPosition` | `["position", market, user]` | Cumulative YES and NO stakes for one user in one market. |

### `Market`

| Field | Type | Description |
| --- | --- | --- |
| `creator` | `Pubkey` | Authority allowed to resolve the market. |
| `market_id` | `u64` | Per-creator identifier used in the PDA seed. |
| `question` | `String` | Question text, capped at 200 characters. |
| `resolution_time` | `i64` | Unix timestamp when betting closes and resolution opens. |
| `yes_pool` | `u64` | Total lamports staked on YES. |
| `no_pool` | `u64` | Total lamports staked on NO. |
| `resolved` | `bool` | Whether an outcome has been recorded. |
| `outcome` | `Option<bool>` | `None`, `Some(true)` for YES, or `Some(false)` for NO. |
| `bump` | `u8` | Market PDA bump. |

### `UserPosition`

| Field | Type | Description |
| --- | --- | --- |
| `market` | `Pubkey` | Market the position belongs to. |
| `user` | `Pubkey` | Position owner. |
| `yes_amount` | `u64` | Cumulative lamports staked on YES. |
| `no_amount` | `u64` | Cumulative lamports staked on NO. |
| `claimed` | `bool` | Prevents duplicate payouts. |
| `bump` | `u8` | Position PDA bump. |

A user can hold both YES and NO positions in the same market. Positions are not netted.

## Instructions

| Instruction | Arguments | Signer | Behavior |
| --- | --- | --- | --- |
| `create_market` | `market_id: u64`, `question: String`, `resolution_time: i64` | Creator | Initializes the market PDA and zeroes both pools. |
| `place_bet` | `amount: u64`, `bet_yes: bool` | Bettor | Transfers lamports into the market PDA and updates the selected pool and position. |
| `resolve_market` | `outcome: bool` | Creator | Records the winning outcome after the deadline. |
| `claim_winnings` | none | Position owner | Transfers the winning stake plus the proportional share of the losing pool. |

### `create_market`

- Rejects questions longer than 200 characters.
- Rejects a `resolution_time` that is not in the future.
- Derives the market PDA from `["market", creator, market_id]`.
- Initializes `yes_pool`, `no_pool`, `resolved`, and `outcome` to zero/`false`/`None`.

### `place_bet`

- Rejects `amount == 0`.
- Rejects bets at or after `resolution_time`.
- Transfers lamports from the bettor to the market PDA with the system program.
- Adds the amount to `yes_pool` or `no_pool` with `checked_add`.
- Creates the `UserPosition` with `init_if_needed` on the first bet and increments `yes_amount` or `no_amount` on subsequent bets.

### `resolve_market`

- Requires the market creator as signer.
- Requires `clock.unix_timestamp >= resolution_time`.
- Rejects an already-resolved market.
- Sets `resolved = true` and stores the boolean outcome.

Resolution is creator-controlled. The program does not validate the real-world truth of the outcome.

### `claim_winnings`

- Requires `market.resolved`.
- Rejects a position whose `claimed` flag is already set.
- Requires a non-zero stake on the winning side.
- Computes the payout with `u128` intermediates:

```
winnings = (user_winning_bet * total_losing_pool) / total_winning_pool
total_payout = user_winning_bet + winnings
```

- Moves lamports from the market PDA to the user by mutating account balances.
- Sets `position.claimed = true`.

Integer division floors the result, so the market PDA can retain a small amount of lamport dust.

## Payout Model

The market is parimutuel, not an order book or AMM. The ratio between the pools acts as an implied probability, and winners share the losing pool in proportion to their stake.

Example:

```
YES pool: 100 SOL
NO pool:   50 SOL
Outcome: YES

User stake on YES: 10 SOL
Share of YES pool: 10 / 100 = 10%
Share of NO pool:  50 * 10% = 5 SOL
Total payout:      10 + 5 = 15 SOL
```

## Project Structure

```
prediction-markets/
├── anchor/
│   ├── Anchor.toml
│   └── programs/prediction_market/
│       └── src/
│           ├── lib.rs
│           ├── state.rs
│           ├── errors.rs
│           └── tests.rs
├── app/
│   ├── activity/page.tsx
│   ├── components/
│   │   ├── providers.tsx
│   │   ├── create-market-form.tsx
│   │   ├── market-card.tsx
│   │   ├── markets-list.tsx
│   │   ├── position-card.tsx
│   │   ├── positions-list.tsx
│   │   ├── activity-stats.tsx
│   │   └── wallet-button.tsx
│   ├── generated/prediction_market/
│   └── page.tsx
├── codama.json
├── package.json
├── scripts/                 # Technical documentation
├── LECTURE_PLAN.md          # Implementation plan
└── README.md
```

## Requirements

- Node.js and npm
- Rust and Cargo
- Solana CLI
- Anchor CLI 0.31.1

## Setup

```bash
npm install
npm run setup
```

`setup` runs `anchor-build` followed by `codama:js`. The first command compiles the program and emits the IDL; the second regenerates `app/generated/prediction_market/`.

Start the frontend:

```bash
npm run dev
```

The application connects to devnet and requires a funded wallet for market creation and betting.

## Build and Deploy

Build the program only:

```bash
npm run anchor-build
```

Deploy to devnet:

```bash
solana config set --url devnet
cd anchor
anchor keys sync
anchor build
anchor deploy
cd ..
npm run setup
```

`anchor keys sync` updates the declared program ID in the source and `Anchor.toml` to the local keypair. Regenerate the client after deployment so the frontend uses the deployed address.

## Testing

The Anchor test command is:

```bash
npm run anchor-test
```

It runs `anchor test --skip-deploy`, which uses the `test` script configured in `anchor/Anchor.toml` (`cargo test-sbf`).

The LiteSVM test module `anchor/programs/prediction_market/src/tests.rs` is present but currently disabled: `mod tests` is commented out in `lib.rs`, and the `litesvm`/`solana-sdk` dev-dependencies are commented out in the program `Cargo.toml`. The test command therefore exercises the build pipeline but does not execute those tests. Re-enable the module and restore the dev-dependencies to run the LiteSVM suite.

## Security and Limitations

- **Creator-controlled resolution** - The market creator chooses the outcome. The program does not use an oracle, multisig, or dispute period.
- **Unrestricted market creation** - Creating a market has no fee or stake requirement, so spam markets are possible.
- **No fees** - The program has no protocol fee or treasury.
- **No partial exits** - Positions cannot be sold, transferred, or exited before resolution.
- **No position closure** - `claim_winnings` does not close the position account, so rent is not reclaimed.
- **Polling frontend** - The UI refreshes account data on an interval rather than subscribing to account changes.
- **Deadline races** - Transaction ordering around `resolution_time` is not mitigated.
- **Arithmetic** - Pool updates use `checked_add`; payout math uses `u128` intermediates and floors the division.

## Documentation

| Document | Topic |
| --- | --- |
| [scripts/00-overview.md](./scripts/00-overview.md) | Documentation index and file references |
| [scripts/01-why-and-what.md](./scripts/01-why-and-what.md) | Market mechanics and system architecture |
| [scripts/02-on-chain-program.md](./scripts/02-on-chain-program.md) | Accounts, instructions, and payout logic |
| [scripts/03-codegen-layer.md](./scripts/03-codegen-layer.md) | IDL and Codama client generation |
| [scripts/04-frontend-architecture.md](./scripts/04-frontend-architecture.md) | Components, data fetching, and transaction flow |
| [scripts/05-security-tradeoffs.md](./scripts/05-security-tradeoffs.md) | Design decisions and limitations |
| [scripts/06-recap.md](./scripts/06-recap.md) | Architecture summary |
| [scripts/final.md](./scripts/final.md) | Combined technical reference |
| [LECTURE_PLAN.md](./LECTURE_PLAN.md) | Implementation plan (legacy filename) |

## References

- [Anchor documentation](https://www.anchor-lang.com/)
- [Solana documentation](https://solana.com/docs)
- [Codama](https://github.com/codama-idl/codama)
- [framework-kit](https://github.com/solana-foundation/framework-kit)
