# Codegen Layer

The codegen layer connects the Rust program to the TypeScript frontend. It removes hand-written PDA derivation, instruction serialization, and account decoding from the application code.

The Rust program is the source of truth. The Anchor IDL is the schema, and the generated TypeScript client is what the frontend imports.

## Pipeline

```
┌─────────────┐      ┌─────────────┐      ┌─────────────┐
│   lib.rs    │      │  IDL.json   │      │ TypeScript  │
│   (Rust)    │─────▶│ (Schema)    │─────▶│  Client     │
└─────────────┘      └─────────────┘      └─────────────┘
     anchor build         codama:js

   Source of           Intermediate        What frontend
   truth               representation      imports
```

Commands:

```bash
npm run anchor-build   # Rust → IDL
npm run codama:js      # IDL → TypeScript
```

`anchor-build` compiles the program and emits an IDL JSON file containing instructions, accounts, types, and errors. `codama:js` reads the IDL and generates a TypeScript client with instruction builders, PDA helpers, account decoders, and error types. The output is written to `app/generated/prediction_market/`.

The generated client must be regenerated whenever a Rust account, instruction, or error changes. Generated files are build output and should not be hand-edited.

A mismatch between the program and the frontend usually means the IDL or generated client is stale. Rebuilding and regenerating resolves that class of deserialization error.

The IDL is also language-neutral. Other clients can be generated from the same schema without duplicating the program's account layout.

The generated IDL is available at `anchor/target/idl/`. It lists every instruction, account layout, field name, and error, which makes it useful for debugging.

Generated instruction files also document the required accounts, writable accounts, and expected signers for each instruction.

## Generated Client

`place_bet` demonstrates the main responsibilities of the generated client: instruction arguments, PDA derivation, and serialization.

IDL snippet:

```json
// IDL snippet (anchor/target/idl/prediction_market.json)
{
  "name": "placeBet",
  "args": [
    { "name": "amount", "type": "u64" },
    { "name": "betYes", "type": "bool" }
  ]
}
```

Generated TypeScript:

```typescript
// Generated: app/generated/prediction_market/instructions/placeBet.ts
export async function getPlaceBetInstructionAsync(
  input: PlaceBetAsyncInput
): Promise<PlaceBetInstruction> {
  // Auto-derives PDAs from seeds
  const marketAddress = await findMarketPda(input.creator, input.marketId);
  const positionAddress = await findPositionPda(marketAddress, input.user);

  // Encodes args to bytes
  const data = getPlaceBetInstructionDataEncoder().encode({
    amount: input.amount,
    betYes: input.betYes,
  });

  return { keys: [...], data, programId };
}
```

The IDL declares the instruction name and arguments. Codama turns that schema into a callable TypeScript function.

The generated client handles:

- **PDA derivation** — It knows the `Market` and `UserPosition` seeds from the IDL and calls `findMarketPda` and `findPositionPda` with the same seeds used by the program. Application code does not reimplement this logic.
- **Serialization** — It encodes arguments in Borsh order so the bytes match the Rust program. `u64` values are represented as `bigint` in TypeScript, avoiding JavaScript number overflow.
- **Account metas** — It orders accounts and sets writable and signer flags correctly. This removes a common source of hand-written instruction errors.
- **Account decoding** — `getMarketDecoder()` reads a `Market` account from raw bytes, including the 8-byte Anchor discriminator. Accounts returned by `getProgramAccounts` can be decoded with the same layout used on-chain.
- **Typed errors** — Program errors such as `MarketError::BettingClosed` map to typed errors on the frontend, allowing specific messages instead of a generic transaction failure.
- **Discriminators** — The client exposes Anchor account discriminators, so RPC filters can select only the expected account type.
- **Program ID** — The generated package exports the program ID as a constant, preventing the frontend from drifting to a different address.
- **Typed account interfaces** — Decoded accounts expose exact field names and types, including `bigint` for `u64` values.
- **Sync and async builders** — Async builders derive PDAs when needed. Sync builders are useful in tests or scripts where addresses are already known.

Encoders and decoders are pure functions and can be tested in isolation. A new field can be verified by encoding a sample or decoding a captured account fixture.

Generated clients may be committed so a clone can run without a codegen step, or regenerated in CI. In both cases the generated directory should be treated as build output rather than hand-written source.

## Key Points

The generated client solves two error-prone problems:

1. **PDA derivation** — deterministic addresses from program seeds.
2. **Serialization** — instruction arguments encoded to bytes matching the program's Borsh layout.

Both are handled by generated code rather than application code.
