# Prediction Market Anchor Workspace

Anchor workspace for the `prediction_market` program used by the prediction-markets application.

## Program ID

```
33ijjeqroG7Syj3fqJn7oebG6iAwk1pGq5deBNYzkr1h
```

Configured under `[programs.devnet]` in `Anchor.toml`. The workspace targets devnet and uses `~/.config/solana/id.json` as the provider wallet.

## Toolchain

| Component | Version |
| --- | --- |
| Anchor CLI | 0.31.1 |
| `anchor-lang` | 0.31.1 |
| Test command | `cargo test-sbf` |

## Program Structure

```
anchor/
├── Anchor.toml
├── Cargo.toml
└── programs/prediction_market/
    ├── Cargo.toml
    └── src/
        ├── lib.rs
        ├── state.rs
        ├── errors.rs
        └── tests.rs
```

## Instructions

| Instruction | Arguments | Signer | Behavior |
| --- | --- | --- | --- |
| `create_market` | `market_id: u64`, `question: String`, `resolution_time: i64` | Creator | Creates the market PDA and initializes both pools. |
| `place_bet` | `amount: u64`, `bet_yes: bool` | Bettor | Transfers lamports into the market and updates the selected pool. |
| `resolve_market` | `outcome: bool` | Creator | Records the winning outcome after the deadline. |
| `claim_winnings` | none | Position owner | Pays the winning stake plus the proportional share of the losing pool. |

### Validation

- `question.len() <= 200`.
- `resolution_time > Clock::get()?.unix_timestamp` at creation.
- `amount > 0` for bets.
- Bets must be placed before `resolution_time`.
- Only the creator can resolve, and only at or after `resolution_time`.
- A market can be resolved once.
- A position can be claimed once.
- Pool updates use `checked_add`; payout math uses `u128` intermediates.

## Accounts

### `Market`

Seeds: `["market", creator, market_id.to_le_bytes()]`

| Field | Type |
| --- | --- |
| `creator` | `Pubkey` |
| `market_id` | `u64` |
| `question` | `String` (max 200) |
| `resolution_time` | `i64` |
| `yes_pool` | `u64` |
| `no_pool` | `u64` |
| `resolved` | `bool` |
| `outcome` | `Option<bool>` |
| `bump` | `u8` |

### `UserPosition`

Seeds: `["position", market, user]`

| Field | Type |
| --- | --- |
| `market` | `Pubkey` |
| `user` | `Pubkey` |
| `yes_amount` | `u64` |
| `no_amount` | `u64` |
| `claimed` | `bool` |
| `bump` | `u8` |

## Payout

```
winnings = (user_winning_bet * total_losing_pool) / total_winning_pool
total_payout = user_winning_bet + winnings
```

Integer division floors the result, so the market PDA can retain lamport dust.

## Build

```bash
cd anchor
anchor build
```

The IDL is written to `target/idl/prediction_market.json`. The application regenerates its TypeScript client with `npm run codama:js` from the repository root.

## Test

```bash
cd anchor
anchor test --skip-deploy
```

`Anchor.toml` sets the test script to `cargo test-sbf`.

The LiteSVM test module at `programs/prediction_market/src/tests.rs` is present but disabled: `mod tests` is commented out in `lib.rs`, and the `litesvm`/`solana-sdk` dev-dependencies are commented out in the program `Cargo.toml`. The test command builds the program but does not run those tests until the module and dependencies are restored.

## Deploy

```bash
solana config set --url devnet
anchor keys sync
anchor build
anchor deploy
```

`anchor keys sync` updates `declare_id!` and the `Anchor.toml` program entry to the local keypair. Regenerate the client after deployment.

## Security Notes

- The market creator controls resolution. There is no oracle, multisig, or dispute window.
- Market creation is unrestricted and has no fee.
- Positions cannot be closed or transferred before resolution.
- Claimed position accounts remain open and retain their rent.
- The program does not validate the real-world truth of a resolved outcome.

## References

- [Anchor documentation](https://www.anchor-lang.com/)
- [Solana documentation](https://solana.com/docs)
