# Solana Core Topics

Reference implementations and technical notes for the core concepts behind Solana programs. Each topic is an independent project with its own on-chain program, tests, and optional client. The repository is organized by topic, and every project stands on its own.

> **Demonstration code.** The programs, circuits, and clients are not audited and are not suitable for production use. Each topic README records the verified behavior and known gaps.

## Topics

| Topic | Directory | Core concepts | Stack |
| --- | --- | --- | --- |
| Voting | [`voting/`](./voting/README.md) | PDA-backed state, typed instructions, clock-based access control, vote accounting | Anchor, Jest |
| Token escrow | [`escrow/`](./escrow/README.md) | PDA-owned vaults, SPL Token interface, `transfer_checked` CPIs, account closure | Anchor, LiteSVM |
| Stablecoin | [`stablecoin/`](./stablecoin/README.md) | Token-2022 mint authority, per-minter allowances, pause controls, mint/burn CPIs | Anchor, LiteSVM |
| StableSwap AMM | [`stableswap/anchor/`](./stableswap/anchor/README.md) | Curve-style invariant, Newton's method, LP shares, dynamic fees, Pyth oracle guards | Anchor, LiteSVM |
| Prediction markets | [`prediction-markets/`](./prediction-markets/README.md) | Parimutuel pools, PDA market state, SOL custody, creator resolution, Codama client | Anchor, Next.js |
| RWA NFT collection | [`rwa-nft-mystery-box/`](./rwa-nft-mystery-box/README.md) | Token-2022 NFTs, fixed per-type supply, PDA mint authority, weighted minting | Anchor, Next.js |
| Private transfers | [`private-transfers/`](./private-transfers/README.md) | Commitments, Merkle membership, nullifiers, Noir/Groth16 proofs, verifier CPI | Anchor, Noir, Sunspot, React |
| Paid APIs | [`x402-solana-payments/`](./x402-solana-payments/README.md) | HTTP 402 negotiation, payment middleware, Solana settlement, session tokens | Next.js, x402-next |

## Concept coverage

| Core concept | Where it is implemented |
| --- | --- |
| Accounts, PDAs, seeds, and bumps | voting, escrow, stablecoin, stableswap, prediction-markets, rwa-nft-mystery-box, private-transfers |
| Account constraints and validation | every Anchor program |
| Cross-program invocation and PDA signing | escrow, stablecoin, stableswap, rwa-nft-mystery-box, private-transfers |
| SPL Token and Token-2022 | escrow, stablecoin, stableswap, rwa-nft-mystery-box |
| Token accounts and ATAs | escrow, stablecoin, stableswap, rwa-nft-mystery-box |
| Clock and time-gated logic | voting, prediction-markets |
| AMM math and LP accounting | stableswap |
| Oracle integration and staleness checks | stableswap (Pyth) |
| Parimutuel payout accounting | prediction-markets |
| NFT supply and collection state | rwa-nft-mystery-box |
| Commitments, Merkle trees, and nullifiers | private-transfers |
| Zero-knowledge proofs and on-chain verification | private-transfers |
| Generated TypeScript clients | prediction-markets, rwa-nft-mystery-box |
| Payment-gated HTTP routes | x402-solana-payments |
| LiteSVM program testing | escrow, stablecoin, stableswap |
| Local-validator and end-to-end testing | voting, prediction-markets, rwa-nft-mystery-box, private-transfers |

The [topic map](./TOPICS.md) maps each concept to the specific files and programs that demonstrate it.

## Repository layout

Each topic follows a similar shape:

```text
<topic>/
├── README.md
├── anchor/ or programs/     # Anchor workspace or program crate
├── tests/                   # Integration tests
└── app/ or frontend/        # Optional client
```

Some topics wrap the Anchor workspace in `anchor/`; others keep the workspace at the topic root. The topic README is the source of truth for paths, commands, and program IDs.

## Project status

| Topic | On-chain program | Client | Test coverage |
| --- | --- | --- | --- |
| Voting | Anchor | none | Jest integration tests against a local validator |
| Token escrow | Anchor | none | Rust LiteSVM integration tests |
| Stablecoin | Anchor | none | Rust LiteSVM tests |
| StableSwap AMM | Anchor | none | Rust LiteSVM integration tests |
| Prediction markets | Anchor | Next.js | Build/test pipeline; LiteSVM module currently disabled |
| RWA NFT collection | Anchor | Next.js | Collection initialization test |
| Private transfers | Anchor + Sunspot verifier | React + proof service | Circuit tests and end-to-end withdrawal flow |
| Paid APIs | none | Next.js | Manual devnet payment flow |

## Getting started

Common requirements across topics:

- Rust and Cargo
- Solana CLI
- Anchor CLI (version varies by topic; see the topic README)
- Node.js and a package manager for client topics
- Bun for the private-transfers project

Each topic is independent. There is no repository-wide install or build step. Open the topic README, install that topic's dependencies, and run its documented build and test commands.

## Documentation conventions

- Each topic README documents the on-chain model, instructions, accounts, setup, testing, and security limitations.
- Behavior described in a README is verified against the code in that topic.
- Security sections call out demonstration-only assumptions and known gaps.
- `private-transfers/main/` is the canonical private-transfers implementation. The `starter/` and `step-*` directories are earlier checkpoints retained for comparison, not separate topics.

## References

- [Solana documentation](https://solana.com/docs)
- [Anchor documentation](https://www.anchor-lang.com/)
- [Token-2022 documentation](https://spl.solana.com/token-2022)
- [LiteSVM](https://github.com/LiteSVM/litesvm)
- [Codama](https://github.com/codama-idl/codama)
- [Noir](https://noir-lang.org)
- [Sunspot](https://github.com/reilabs/sunspot)
- [X402](https://github.com/coinbase/x402)
