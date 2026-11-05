# Core Solana Topic Map

A concept-first index for the implementations in this repository. Use it to find the code that demonstrates a specific Solana primitive or design pattern. Project-level setup and behavior live in each topic README.

## Concept index

| Concept | What the code demonstrates | Where to look |
| --- | --- | --- |
| Program-derived addresses | `find_program_address`, deterministic seeds, stored bumps, PDA-owned state | [`voting/`](./voting/README.md), [`escrow/`](./escrow/README.md), [`stablecoin/`](./stablecoin/README.md), [`prediction-markets/`](./prediction-markets/README.md) |
| Account constraints | `#[account(...)]`, `Signer`, `has_one`, `init`, `init_if_needed`, `close` | [`escrow/`](./escrow/README.md), [`prediction-markets/`](./prediction-markets/README.md), [`rwa-nft-mystery-box/`](./rwa-nft-mystery-box/README.md) |
| Cross-program invocation | `CpiContext`, `invoke_signed`, PDA signer seeds, program-to-program calls | [`escrow/`](./escrow/README.md), [`stablecoin/`](./stablecoin/README.md), [`rwa-nft-mystery-box/`](./rwa-nft-mystery-box/README.md), [`private-transfers/main/`](./private-transfers/main/README.md) |
| SPL Token interface | `Interface<TokenInterface>`, `InterfaceAccount`, `transfer_checked`, `close_account` | [`escrow/`](./escrow/README.md) |
| Token-2022 | Token Extensions mints, Token-2022 ATA derivation, PDA mint authority, mint and burn CPIs | [`stablecoin/`](./stablecoin/README.md), [`rwa-nft-mystery-box/`](./rwa-nft-mystery-box/README.md) |
| Token accounts and ATAs | Associated token account constraints, authority checks, rent and closure behavior | [`escrow/`](./escrow/README.md), [`stablecoin/`](./stablecoin/README.md), [`rwa-nft-mystery-box/`](./rwa-nft-mystery-box/README.md) |
| Time-gated logic | `Clock::get()`, voting windows, betting deadlines, resolution timestamps | [`voting/`](./voting/README.md), [`prediction-markets/`](./prediction-markets/README.md) |
| State machines | Market lifecycle, collection supply, pool lifecycle, pause state | [`prediction-markets/`](./prediction-markets/README.md), [`rwa-nft-mystery-box/`](./rwa-nft-mystery-box/README.md), [`stablecoin/`](./stablecoin/README.md) |
| StableSwap math | Hybrid invariant, Newton–Raphson iteration for `D`, LP share accounting, minimum liquidity | [`stableswap/anchor/`](./stableswap/anchor/README.md) |
| Dynamic fees | Imbalance-based fee scaling with a configured cap | [`stableswap/anchor/`](./stableswap/anchor/README.md) |
| Oracle guards | Pyth price feeds, staleness checks, depeg thresholds, risk pauses | [`stableswap/anchor/`](./stableswap/anchor/README.md) |
| Parimutuel accounting | YES/NO pools, proportional payouts, `u128` intermediates, claim flags | [`prediction-markets/`](./prediction-markets/README.md) |
| NFT collections | Fixed per-type supply, mint PDAs, `mint_to` CPIs, client-side weighted selection | [`rwa-nft-mystery-box/`](./rwa-nft-mystery-box/README.md) |
| Commitments | Poseidon2 commitments, deposit events, hidden deposit details | [`private-transfers/main/`](./private-transfers/main/README.md) |
| Merkle membership | Root ring buffer, leaf index tracking, membership proofs | [`private-transfers/main/`](./private-transfers/main/README.md) |
| Nullifiers | Replay protection without linking a withdrawal to a deposit | [`private-transfers/main/`](./private-transfers/main/README.md) |
| Zero-knowledge verification | Noir circuits, Groth16 proofs, Sunspot verifier CPI | [`private-transfers/main/`](./private-transfers/main/README.md) |
| Generated clients | Anchor IDL, Codama generation, typed instruction builders | [`prediction-markets/`](./prediction-markets/README.md), [`rwa-nft-mystery-box/`](./rwa-nft-mystery-box/README.md) |
| Paid HTTP routes | HTTP 402 challenges, payment middleware, facilitator verification, session tokens | [`x402-solana-payments/`](./x402-solana-payments/README.md) |
| Program testing | LiteSVM, local-validator integration tests, end-to-end scripts | [`escrow/`](./escrow/README.md), [`stablecoin/`](./stablecoin/README.md), [`stableswap/anchor/`](./stableswap/anchor/README.md), [`voting/`](./voting/README.md) |

## Entry points by goal

| Goal | Start with |
| --- | --- |
| Learn how PDAs store structured state | [Voting](./voting/README.md) |
| Move SPL tokens safely between accounts | [Token escrow](./escrow/README.md) |
| Issue a controlled Token-2022 asset | [Stablecoin](./stablecoin/README.md) |
| Implement AMM math and oracle guards | [StableSwap](./stableswap/anchor/README.md) |
| Build a full-stack market application | [Prediction markets](./prediction-markets/README.md) |
| Model an NFT collection with fixed supply | [RWA NFT collection](./rwa-nft-mystery-box/README.md) |
| Add privacy with zero-knowledge proofs | [Private transfers](./private-transfers/README.md) |
| Charge for API or content access | [Paid APIs](./x402-solana-payments/README.md) |
