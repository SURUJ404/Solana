# Solana Technical Topics

A topic-oriented collection of Solana reference implementations and supporting technical documentation. Each topic directory is a self-contained project with its own build, test, and deployment instructions.

## Repository Layout

| Directory | Contents |
| --- | --- |
| [`voting/`](./voting/README.md) | Anchor polling program with PDA state and time-gated voting. |
| [`escrow/`](./escrow/README.md) | Anchor token escrow with PDA vaults and token-interface CPIs. |
| [`private-transfers/`](./private-transfers/main/README.md) | Six private-transfer variants from a public pool to Noir/Groth16 verification. |
| [`stablecoin/`](./stablecoin/README.md) | Token-2022 stablecoin program with issuance and compliance controls. |
| [`stableswap/`](./stableswap/anchor/README.md) | Anchor stable-asset swap program. |
| [`x402-solana-payments/`](./x402-solana-payments/README.md) | Next.js application with X402 payment-gated routes. |
| [`rwa-nft-mystery-box/`](./rwa-nft-mystery-box/README.md) | Token-2022 Labubu collection and mystery-box frontend. |
| [`prediction-markets/`](./prediction-markets/README.md) | Anchor binary prediction market with a Next.js client. |

## Topic Documentation

- **[Voting](./voting/README.md)** - On-chain polls, PDAs, typed instructions, and time-gated access.
- **[Escrow](./escrow/README.md)** - Trustless token swaps, custodial vaults, CPIs, and token interfaces.
- **[Private Transfers](./private-transfers/main/README.md)** - Privacy-oriented transfers, deposit hiding, Merkle membership, nullifiers, and zero-knowledge verification.
- **[Stablecoins](./stablecoin/README.md)** - Token-2022 minting, controlled issuance, allowances, pause controls, and burning.
- **[StableSwap](./stableswap/anchor/README.md)** - Stable-asset swap mechanics and on-chain program patterns.
- **[Solana Payments with X402](./x402-solana-payments/README.md)** - Crypto payment flows and paid application access.
- **[RWA / NFT Mystery Box](./rwa-nft-mystery-box/README.md)** - Token-2022 NFTs, collection state, weighted minting, and wallet integration.
- **[Prediction Markets](./prediction-markets/README.md)** - Binary markets, SOL betting, resolution, and proportional payouts.

## Documentation Scope

Program source, tests, dependencies, and configuration are preserved from the source repository. Documentation changes are limited to Markdown files: project READMEs, step guides, and technical references. Where implementation and documentation differ, the project README records the verified behavior and any known gaps.
