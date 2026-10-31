# Anchor Escrow

A token escrow program built with [Anchor](https://www.anchor-lang.com/) 1.0.0-rc.2 on Solana. A maker deposits one mint into a PDA-owned vault and specifies how much of a second mint they will accept. A taker completes the swap atomically, or the maker refunds the deposit.

The program uses the SPL Token interface rather than concrete SPL Token types, so the same instructions work with both legacy SPL Token mints and Token-2022 mints.

## Overview

The escrow has two roles:

- **Maker** - creates the escrow, deposits token A, and sets the required amount of token B.
- **Taker** - pays token B to the maker and receives the full token A balance from the vault.

The maker can cancel an unfilled escrow at any time. Both `take` and `refund` close the escrow and vault accounts and return their rent to the maker.

```
make:    maker -- token A --> vault (escrow PDA authority)
take:    taker -- token B --> maker
         vault -- token A --> taker
         vault + escrow closed, rent --> maker
refund:  vault -- token A --> maker
         vault + escrow closed, rent --> maker
```

## Program ID

```
8F3byNyXVHzfmjKK9J2cxvVbKzRiVYh8icoprMUqSFmb
```

`Anchor.toml` configures the program for `localnet`.

## Architecture

| Component | Description |
| --- | --- |
| `Escrow` account | Stores the maker, mints, requested amount, seed, and PDA bump. |
| Vault | An associated token account for token A, owned by the `Escrow` PDA. |
| Token program | Resolved through `Interface<TokenInterface>` and `InterfaceAccount`. |

The escrow account uses seeds `["escrow", maker, seed (little-endian u64)]` and a custom one-byte discriminator. The vault is the associated token account for `mint_a` with the escrow PDA as authority. The program signs vault transfers and closures with the escrow seeds.

All token movements use `transfer_checked` CPIs, which pass the mint decimals to the token program. Account closures use `close_account`.

## Instructions

| Instruction | Arguments | Signer | Purpose |
| --- | --- | --- | --- |
| `make` | `seed: u64`, `receive: u64`, `amount: u64` | Maker | Creates the escrow and vault, then deposits token A. |
| `take` | none | Taker | Pays token B to the maker and releases token A from the vault. |
| `refund` | none | Maker | Returns token A to the maker and closes the escrow. |

### `make`

1. Validates that `receive > 0` and `amount > 0`.
2. Initializes the `Escrow` PDA with `seed`, `maker`, `mint_a`, `mint_b`, `receive`, and `bump`.
3. Initializes the vault as the associated token account for `mint_a` with the escrow PDA as authority.
4. Transfers `amount` of token A from the maker's associated token account to the vault.

The maker's token A account must already exist. The maker pays rent for the escrow and vault accounts.

### `take`

1. Validates the escrow seeds, bump, and `has_one` constraints for `maker`, `mint_a`, and `mint_b`.
2. Transfers `escrow.receive` of token B from the taker's account to the maker's associated token account.
3. Transfers the vault's entire token A balance to the taker's associated token account using the escrow PDA as signer.
4. Closes the vault and sends its rent to the maker.
5. Closes the escrow account and sends its rent to the maker.

The taker's token B account must already exist. The taker pays rent if the maker's token B account or the taker's token A account must be initialized. The transfer amount for token A is read from `vault.amount`, not from an instruction argument, so the full deposit is always released.

### `refund`

1. Validates the escrow seeds, bump, and `has_one` constraints for `maker` and `mint_a`.
2. Transfers the vault's entire token A balance to the maker's associated token account.
3. Closes the vault and sends its rent to the maker.
4. Closes the escrow account and sends its rent to the maker.

Only the maker can sign this instruction. The `close = maker` constraint on the escrow account enforces the authority.

## Accounts

### `Escrow`

Seeds: `["escrow", maker_pubkey, seed.to_le_bytes()]`

| Field | Type | Description |
| --- | --- | --- |
| `seed` | `u64` | Maker-supplied seed that allows multiple escrows per maker. |
| `maker` | `Pubkey` | Wallet that created the escrow and receives refunds and rent. |
| `mint_a` | `Pubkey` | Mint deposited by the maker. |
| `mint_b` | `Pubkey` | Mint the maker requires in return. |
| `receive` | `u64` | Amount of token B required to complete the swap. |
| `bump` | `u8` | PDA bump used for signing. |

### Vault

An associated token account with:

- Mint: `mint_a`
- Authority: the `Escrow` PDA
- Token program: the interface token program supplied by the caller

The vault holds the full token A deposit until `take` or `refund` closes it.

## Token Interface

The program imports `TokenInterface`, `InterfaceAccount<Mint>`, and `InterfaceAccount<TokenAccount>` from `anchor_spl::token_interface`. This keeps the program independent of a specific token program deployment:

- Legacy SPL Token mints work without changes.
- Token-2022 mints work without changes.
- Transfers use `transfer_checked`, so the mint's decimals are enforced by the token program.

The caller supplies the token program account, and the associated token account constraints validate that it matches the mint and authority.

## Error Codes

| Code | Message | Trigger |
| --- | --- | --- |
| `InvalidAmount` | Invalid amount | `receive == 0` or `amount == 0` in `make`. |
| `InvalidMaker` | Invalid maker | Escrow `maker` does not match the supplied maker account. |
| `InvalidMintA` | Invalid mint a | Escrow `mint_a` does not match the supplied mint account. |
| `InvalidMintB` | Invalid mint b | Escrow `mint_b` does not match the supplied mint account. |

## Project Structure

```
escrow/
├── Anchor.toml
├── Cargo.toml
├── programs/anchor_escrow/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── state.rs
│       ├── errors.rs
│       └── instructions/
│           ├── make.rs
│           ├── take.rs
│           └── refund.rs
└── tests/anchor-litesvm-tests.rs
```

## Requirements

- Rust and Cargo
- Solana CLI
- Anchor CLI 1.0.0-rc.2

The workspace enables `overflow-checks = true` for release builds.

## Build

```bash
cd escrow
anchor build
```

The build produces `target/deploy/anchor_escrow.so`, which the LiteSVM test loads directly.

## Test

The repository contains a Rust integration test at `tests/anchor-litesvm-tests.rs`. It covers `make` followed by `take` using LiteSVM and `litesvm-token`.

```bash
cd escrow
anchor build
cargo test --test anchor-litesvm-tests
```

The `test` script in `Anchor.toml` targets `tests/**/*.ts`. The repository does not contain TypeScript tests, so the Rust test target is the executable test suite. There is no automated `refund` test in the current tree.

## Security Notes

- The vault has no private key. Only the escrow PDA can sign for it, and the program signs only with seeds validated by the escrow account constraints.
- `has_one` constraints bind `maker`, `mint_a`, and `mint_b` to the stored escrow state.
- The escrow seeds include the maker key and the user-supplied seed, so one maker can open independent escrows without address collisions.
- `take` transfers exactly `escrow.receive` of token B and the vault's full token A balance. Partial fills are not supported.
- Closing the escrow and vault returns rent to the maker.
- The program does not charge a fee and does not validate that `mint_a` and `mint_b` are different mints.
- Token-2022 extensions such as transfer hooks execute during `transfer_checked` CPIs. The program does not implement a reentrancy guard.

## References

- [Anchor documentation](https://www.anchor-lang.com/)
- [SPL Token interface](https://docs.rs/anchor-spl/latest/anchor_spl/token_interface/index.html)
- [Token-2022 documentation](https://spl.solana.com/token-2022)
- [LiteSVM](https://github.com/LiteSVM/litesvm)
