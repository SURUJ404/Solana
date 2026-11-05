# RWA NFT Mystery Box: Labubu Collection

A Solana RWA demonstration built around a Token-2022 NFT collection. The on-chain program manages a fixed-supply collection of 11 Labubu types, and a Next.js frontend provides a mystery-box minting interface.

The collection contains 10 normal types with 120 editions each and 1 rare type with 6 editions, for a total supply of 1,206 NFTs.

> Part of the [Solana Core Topics](../README.md) reference.

## Core Concepts

- Token-2022 NFT mints with zero decimals
- Fixed per-type supply counters in a collection PDA
- PDA mint authority and `mint_to` CPIs
- Associated token account creation for recipients
- Client-side weighted selection for the mystery-box experience
- Codama-generated TypeScript client and Next.js frontend

## Architecture

| Layer | Technology |
| --- | --- |
| On-chain program | Anchor 0.31.1 (Rust), package name `vault` |
| Token standard | Token-2022 (`TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb`) |
| Client generation | Codama, configured in `codama.json` |
| Frontend | Next.js 16, React 19, TypeScript, Tailwind CSS v4 |
| Solana client | `@solana/client`, `@solana/react-hooks` |

The program package is named `vault`, but its instruction set implements the Labubu collection rather than a SOL vault.

## Program ID

```
6RSKyunojsMYHZQWzeP6SPGXnmZwjHAcPcbEumk7Cw2g
```

Configured under `[programs.devnet]` in `anchor/Anchor.toml`.

## On-Chain Model

### Accounts

| Account | Seeds | Purpose |
| --- | --- | --- |
| `LabubuCollection` | `["collection"]` | Stores the authority, per-type remaining supply, and total minted. |
| Labubu mint | `["labubu_mint", [labubu_id]]` | One Token-2022 mint per Labubu type. |
| User token account | Associated token account | Receives the minted NFT. |

### `LabubuCollection`

| Field | Type | Description |
| --- | --- | --- |
| `authority` | `Pubkey` | Authority allowed to create Labubu mints. |
| `remaining_supply` | `[u16; 11]` | Remaining editions for IDs 1 through 11. |
| `total_minted` | `u32` | Total NFTs minted across all types. |

Supply initialization:

- IDs 1 through 10: 120 each.
- ID 11: 6.
- Total: 1,206.

The collection PDA is also the mint authority for every Labubu mint. The program signs mint instructions with `["collection", bump]`.

## Instructions

| Instruction | Arguments | Signer | Behavior |
| --- | --- | --- | --- |
| `initialize_collection` | none | Authority | Creates the collection PDA and sets all supply counters. |
| `create_labubu_mint` | `labubu_id: u8` | Collection authority | Creates and initializes the Token-2022 mint for one Labubu type. |
| `mint_random` | `labubu_id: u8` | User | Mints one NFT of the requested type to the user's associated token account. |

### `initialize_collection`

- Creates the `LabubuCollection` PDA with seeds `["collection"]`.
- Sets `authority` to the signer.
- Sets IDs 1 through 10 to 120 and ID 11 to 6.
- Sets `total_minted` to 0.

This instruction can run only once because the PDA is initialized with `init`.

### `create_labubu_mint`

- Requires `1 <= labubu_id <= 11`.
- Requires the signer to match `collection.authority`.
- Derives the mint PDA from `["labubu_mint", [labubu_id]]`.
- Creates an 82-byte Token-2022 mint account funded by the authority.
- Initializes the mint with 0 decimals, the collection PDA as mint authority, and no freeze authority.

One mint must exist for each Labubu type before users can mint that type.

### `mint_random`

- Requires `1 <= labubu_id <= 11`.
- Requires `remaining_supply[labubu_id - 1] > 0`.
- Decrements the remaining supply before the CPI.
- Mints exactly one token to the user's associated token account, creating the account if needed.
- Increments `total_minted`.

Despite the name, the instruction does not generate randomness on-chain. The caller supplies `labubu_id`, and the program enforces only the per-type supply limit. Random selection is implemented in the frontend.

## Supply Selection in the Frontend

`app/components/labubu-card.tsx` performs a weighted selection over the 11 types using the supply values from `labubu-metadata.json`:

1. Compute a random integer in `[0, 1206)`.
2. Walk the metadata supply list and select the type whose cumulative supply contains the random value.
3. Call `mint_random` with the selected ID and one of the hardcoded mint addresses.

This makes the displayed mystery-box behavior weighted by supply, but the selection is not verifiable on-chain. A caller can invoke `mint_random` directly with any ID that still has supply.

## Project Structure

```
rwa-nft-mystery-box/
├── anchor/
│   ├── Anchor.toml
│   ├── programs/vault/
│   │   └── src/
│   │       ├── lib.rs
│   │       └── tests.rs
│   └── tests/initialize.ts
├── app/
│   ├── components/
│   │   ├── labubu-card.tsx
│   │   ├── vault-card.tsx
│   │   └── providers.tsx
│   ├── generated/vault/
│   ├── layout.tsx
│   └── page.tsx
├── codama.json
├── labubu-metadata.json
├── next.config.ts
├── package.json
└── README.md
```

The Next.js application is defined at the repository root. The `app/` directory contains routes and components but does not have its own `package.json`; dependencies and scripts are defined in the root `package.json`.

## Requirements

- Node.js and npm
- Rust and Cargo
- Solana CLI
- Anchor CLI 0.31.1

## Setup

Install the Next.js dependencies at the repository root:

```bash
npm install
```

Install the Anchor test dependencies:

```bash
cd anchor
npm install
```

Build the program and regenerate the client:

```bash
npm run setup
```

`setup` runs `anchor-build` followed by `codama:js`. The generated client is written to `app/generated/vault/`.

## Deployment

1. Configure the Solana CLI for devnet and fund the deployment wallet.

```bash
solana config set --url devnet
solana airdrop 2
```

2. Build and deploy the program.

```bash
cd anchor
anchor build
anchor deploy --provider.cluster devnet
```

3. Initialize the collection.

```bash
anchor run initialize
```

The `initialize` script is defined in `Anchor.toml` and runs `tests/initialize.ts`.

4. Create the 11 Labubu mints by calling `create_labubu_mint` once for each ID from 1 to 11. The repository does not include a script for this step; use the generated client or an Anchor client call.

5. Update the `MINT_ADRRESSES` array in `app/components/labubu-card.tsx` with the deployed mint addresses in ID order. The array name is spelled `MINT_ADRRESSES` in the source.

6. Regenerate the client if the program ID changed.

```bash
npm run setup
```

7. Start the frontend.

```bash
npm run dev
```

## Testing

The Anchor integration test is `anchor/tests/initialize.ts`. It initializes the collection and reads back the account.

```bash
cd anchor
anchor test --skip-deploy
```

`Anchor.toml` configures the test command as `pnpm exec ts-mocha -p ./tsconfig.json -t 1000000 tests/**/*.ts`. Install pnpm for that exact command, or run the equivalent with npx:

```bash
npx ts-mocha -p ./tsconfig.json -t 1000000 tests/initialize.ts
```

The Rust module `anchor/programs/vault/src/tests.rs` is retained from an earlier vault template. It builds `deposit` and `withdraw` instructions for a `["vault", signer]` PDA, which the current `vault` package does not implement. It is not a valid test of the Labubu instruction set.

## Known Issues

- **Client-side randomness only** - The on-chain program accepts any valid `labubu_id` with remaining supply. It does not verify that the selection was random.
- **No metadata CPI** - The program creates bare Token-2022 mints. `labubu-metadata.json` is consumed by the frontend but is not attached to the mints through a Metaplex metadata instruction.
- **Hardcoded mint addresses** - `labubu-card.tsx` contains a fixed `MINT_ADRRESSES` array. A new deployment requires updating that array.
- **Legacy vault component** - `app/components/vault-card.tsx` imports `getDepositInstructionDataEncoder` and `getWithdrawInstructionDataEncoder` from the generated client. Those instructions do not exist in the current Labubu IDL. The component is not imported by `app/page.tsx`, but `tsconfig.json` includes all `.ts`/`.tsx` files, so `next build` type-checks it. Remove or update the component before relying on a clean production build.
- **No app-local package** - `app/` is not a standalone package. Run npm commands from the repository root.

## Security and Limitations

- The collection authority can create mints for any valid ID at any time, including after initialization.
- Minting is free and unrestricted; there is no payment, allowlist, or per-wallet limit.
- Supply counters prevent over-minting each type but do not enforce fair distribution.
- The collection PDA is the mint authority, so no external key can mint directly.
- The program has no freeze authority and no metadata or royalty configuration.
- The frontend trusts hardcoded mint addresses and metadata; it does not derive mint addresses from the program at runtime.

## References

- [Anchor documentation](https://www.anchor-lang.com/)
- [Token-2022 documentation](https://spl.solana.com/token-2022)
- [Codama](https://github.com/codama-idl/codama)
- [framework-kit](https://github.com/solana-foundation/framework-kit)
- [Solana documentation](https://solana.com/docs)
