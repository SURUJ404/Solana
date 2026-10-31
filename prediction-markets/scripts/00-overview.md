# Prediction Markets Documentation

Technical documentation for the full-stack Solana prediction market implementation in this directory.

## Documents

| Document | Topic |
| --- | --- |
| [01-why-and-what.md](./01-why-and-what.md) | Prediction market mechanics and system architecture |
| [02-on-chain-program.md](./02-on-chain-program.md) | Anchor program: accounts, instructions, and payout logic |
| [03-codegen-layer.md](./03-codegen-layer.md) | IDL generation and the Codama TypeScript client |
| [04-frontend-architecture.md](./04-frontend-architecture.md) | Next.js components, data fetching, and transaction flow |
| [05-security-tradeoffs.md](./05-security-tradeoffs.md) | Design decisions, protections, and limitations |
| [06-recap.md](./06-recap.md) | End-to-end flow and key properties |
| [final.md](./final.md) | Combined technical documentation |

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
