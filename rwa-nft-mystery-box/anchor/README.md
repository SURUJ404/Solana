# Labubu Collection Anchor Workspace

Anchor workspace for the Labubu RWA collection. The workspace package is named `vault`, but the program implements a Token-2022 NFT collection with fixed per-type supply.

## Program ID

```
6RSKyunojsMYHZQWzeP6SPGXnmZwjHAcPcbEumk7Cw2g
```

Configured under `[programs.devnet]` in `Anchor.toml`. The provider targets devnet.

## Program Structure

```
anchor/
├── Anchor.toml
├── Cargo.toml
├── package.json
├── tests/initialize.ts
└── programs/vault/
    ├── Cargo.toml
    └── src/
        ├── lib.rs
        └── tests.rs
```

## Instructions

| Instruction | Arguments | Signer | Behavior |
| --- | --- | --- | --- |
| `initialize_collection` | none | Authority | Creates the collection PDA and sets supply counters. |
| `create_labubu_mint` | `labubu_id: u8` | Collection authority | Creates and initializes the Token-2022 mint for one type. |
| `mint_random` | `labubu_id: u8` | User | Mints one NFT of the requested type to the user. |

### `initialize_collection`

- Creates `LabubuCollection` at `["collection"]`.
- Sets IDs 1 through 10 to 120 editions and ID 11 to 6 editions.
- Sets `total_minted` to 0 and stores the signer as `authority`.

### `create_labubu_mint`

- Requires `1 <= labubu_id <= 11`.
- Requires the signer to match `collection.authority`.
- Derives the mint PDA from `["labubu_mint", [labubu_id]]`.
- Creates an 82-byte account owned by Token-2022.
- Initializes the mint with 0 decimals, the collection PDA as mint authority, and no freeze authority.

### `mint_random`

- Requires `1 <= labubu_id <= 11`.
- Rejects a type whose remaining supply is zero.
- Decrements the remaining supply.
- Mints one token to the user's associated token account using `["collection", bump]` as signer.
- Increments `total_minted`.

The instruction does not generate randomness on-chain. The caller selects `labubu_id`; the frontend performs a weighted selection before calling it.

## Accounts

### `LabubuCollection`

Seeds: `["collection"]`

| Field | Type |
| --- | --- |
| `authority` | `Pubkey` |
| `remaining_supply` | `[u16; 11]` |
| `total_minted` | `u32` |

### Labubu mint

Seeds: `["labubu_mint", [labubu_id]]`

A Token-2022 mint with:

- Decimals: 0
- Mint authority: the collection PDA
- Freeze authority: none

The user receives the NFT in an associated token account created on demand.

## Supply

| IDs | Editions per ID | Total |
| --- | --- | --- |
| 1 through 10 | 120 | 1,200 |
| 11 | 6 | 6 |
| All | | 1,206 |

The frontend metadata file `labubu-metadata.json` mirrors the same supply values.

## Build

```bash
cd anchor
anchor build
```

This produces `target/deploy/vault.so` and `target/idl/vault.json`. The repository root regenerates the TypeScript client from that IDL with `npm run codama:js`.

## Deploy

```bash
solana config set --url devnet
anchor build
anchor deploy --provider.cluster devnet
```

After deployment, initialize the collection:

```bash
anchor run initialize
```

The `initialize` script in `Anchor.toml` runs `tests/initialize.ts` with `ts-mocha`.

Creating the 11 mints is a separate step: call `create_labubu_mint` once for each ID from 1 to 11. No script for this loop is included in the workspace.

## Test

```bash
cd anchor
anchor test --skip-deploy
```

`Anchor.toml` configures the test command as:

```
pnpm exec ts-mocha -p ./tsconfig.json -t 1000000 tests/**/*.ts
```

The only TypeScript test is `tests/initialize.ts`, which initializes the collection and reads back the account. Install pnpm for the configured command, or run `npx ts-mocha -p ./tsconfig.json -t 1000000 tests/initialize.ts`.

The Rust module `programs/vault/src/tests.rs` is retained from an earlier vault template. It constructs `deposit` and `withdraw` instructions for a `["vault", signer]` PDA, which this program does not implement. Do not treat it as a test of the Labubu instructions.

## Security Notes

- The collection authority can create mints for any valid ID at any time.
- Minting is free and unrestricted. Supply counters cap each type but do not enforce fair distribution.
- Random selection is client-side and not verifiable on-chain.
- The program creates bare Token-2022 mints without metadata, royalties, or freeze authority.
- The frontend stores mint addresses in a hardcoded array; deployments must update it.

## References

- [Anchor documentation](https://www.anchor-lang.com/)
- [Token-2022 documentation](https://spl.solana.com/token-2022)
- [Solana documentation](https://solana.com/docs)
