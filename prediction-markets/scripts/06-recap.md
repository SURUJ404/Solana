# Architecture Summary

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

The application has one repeating loop:

1. A user action builds an instruction with the generated client.
2. The wallet signs and submits the transaction.
3. The Anchor program validates the accounts and updates state.
4. The frontend polls for the updated account data and re-renders.

The on-chain program is small: two account types, four instructions, and a fixed set of validation rules. The frontend mirrors that structure with two pages and a shallow component tree.

## Key Properties

1. **Programs are stateless** — All market and position data lives in accounts.
2. **PDAs hold funds** — Market accounts have no private keys; only the program can sign for them.
3. **Codegen removes serialization risk** — Instruction data and account layouts come from the IDL.
4. **The IDL is the interface contract** — It connects the on-chain program to the generated client.

The account model is the source of truth. Market pools, resolution state, and participant positions can all be reconstructed from on-chain accounts without an external database.

## Design Principles

- Keep the state model small and explicit.
- Validate inputs before changing state.
- Prefer deterministic PDA derivation over registry accounts.
- Treat generated client code as build output.
- Keep on-chain and off-chain types synchronized through codegen.

A simple state model makes the program easier to audit and extend. Most integration bugs come from mismatched assumptions between layers, which is why the IDL and generated client are treated as part of the program interface.

## Resources

- [Anchor documentation](https://www.anchor-lang.com/)
- [Solana Cookbook](https://solanacookbook.com/)
- [Codama](https://github.com/codama-idl/codama)

## Extension Ideas

- Add oracle-based resolution to remove creator trust.
- Add a small fee or treasury mechanism.
- Support partial position exits or liquidity.
- Extend the market model beyond binary outcomes.
- Replace polling with WebSockets or an indexer.
- Add market creation costs or reputation checks to reduce spam.

Changes to resolution, fees, or position accounting affect the program, the generated client, and the frontend. They are full-stack changes rather than isolated UI updates.
