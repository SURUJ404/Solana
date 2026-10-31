# Frontend Architecture

## Component Structure

The frontend uses the Next.js App Router. The root layout wraps the application in `Providers`, which supplies the Solana connection and wallet state to every page.

The component tree is intentionally shallow:

```
App (layout.tsx)
 └─ Providers (Solana client + wallet)
     ├─ HomePage (page.tsx)
     │   ├─ CreateMarketForm
     │   └─ MarketsList
     │       └─ MarketCard (×N)
     │
     └─ ActivityPage (activity/page.tsx)
         └─ PositionsList
             └─ PositionCard (×N)
```

`Providers` is a client component. Wallet adapters depend on `window`, so the client boundary is kept explicit while the layout can remain server-rendered.

The home page has two responsibilities:

- `CreateMarketForm` collects a question and resolution time, calls `create_market`, and resets the form.
- `MarketsList` fetches markets and renders a `MarketCard` for each one.

The activity page uses `PositionsList` to fetch the connected wallet's positions. Each `PositionCard` displays the position amount, market outcome, and claim status.

`Providers` also centralizes the cluster and RPC connection in `app/components/providers.tsx`. The repository targets devnet, but the same structure supports mainnet, localnet, or a custom RPC.

State is kept local to the components that need it. `MarketCard` receives a decoded market and callbacks; there is no global store or additional state-management layer.

## Data Fetching

The frontend reads directly from the RPC. There is no indexer or database layer.

`markets-list.tsx` calls `getProgramAccounts`, filters by the `Market` discriminator, and decodes each account with the generated client.

```typescript
// Fetch all Market accounts from the program
const fetchMarkets = async () => {
  const response = await fetch(RPC_URL, {
    method: 'POST',
    body: JSON.stringify({
      method: 'getProgramAccounts',
      params: [
        PROGRAM_ID,
        {
          filters: [{
            memcmp: {
              offset: 0,
              bytes: "dkokXHR3DTw"  // Market discriminator
            }
          }]
        }
      ]
    })
  });

  // Decode each account
  const markets = accounts.map(acc => {
    const data = base64ToBytes(acc.data);
    return getMarketDecoder().decode(data);
  });
};

// Poll every 3 seconds
useEffect(() => {
  const interval = setInterval(fetchMarkets, 3000);
  return () => clearInterval(interval);
}, []);
```

`getProgramAccounts` returns every account owned by the program. Anchor prepends an 8-byte discriminator to each account type. Filtering on the `Market` discriminator excludes `UserPosition` accounts.

The generated `getMarketDecoder()` matches the Rust struct layout, so account bytes are not parsed by hand.

The fetching function should be wrapped in error handling with loading and retry states. The polling interval is cleared on unmount so navigation does not leave multiple intervals running.

Polling every three seconds is a deliberate trade-off: it is simple and reliable but not real-time. WebSockets or an indexer would reduce latency in a larger deployment.

Derived values such as implied probability (`yes_pool / (yes_pool + no_pool)`) and total liquidity (`yes_pool + no_pool`) are computed in the client and do not need on-chain storage.

React Query or SWR can add caching if needed. Extracting the fetch logic into a `useMarkets` hook is also a reasonable refactor for a larger application.

The activity page follows the same pattern for `UserPosition` accounts. Positions can be fetched and filtered on the client, or filtered by the user public key with a `memcmp` offset after the discriminator and market key.

Markets can be grouped into active and resolved by comparing `resolution_time` with `Date.now() / 1000`. Pool values are stored in lamports and should be converted with `lamports / LAMPORTS_PER_SOL` for display.

As the number of markets grows, `getProgramAccounts` becomes heavier. An indexer or cached result set is the next step for larger deployments.

### Discriminator Filtering

```
Program owns many accounts:
┌──────────────────┐
│ [disc: Market]   │ ← matches filter ✓
│ question: "..."  │
└──────────────────┘
┌──────────────────┐
│ [disc: Position] │ ← doesn't match ✗
│ user: 0x...      │
└──────────────────┘
┌──────────────────┐
│ [disc: Market]   │ ← matches filter ✓
│ question: "..."  │
└──────────────────┘
```

The filter reduces the RPC payload and keeps the decoder focused on a single account type.

## Transaction Flow

All instructions follow the same transaction pattern: build an instruction with the generated client, send it through the wallet, and refresh the UI on the next poll.

Betting handler:

```typescript
const handleBet = async (betYes: boolean) => {
  // 1. Build instruction using generated client
  const instruction = await getPlaceBetInstructionAsync({
    market: marketAddress,
    user: wallet.address,
    amount: BigInt(solAmount * LAMPORTS_PER_SOL),
    betYes,
  });

  // 2. Send transaction
  await sendTransaction({
    instructions: [instruction],
  });

  // 3. UI updates on next poll (3s)
};
```

The instruction builder is fully typed. Incorrect argument types or missing accounts are reported by TypeScript at compile time.

After `sendTransaction`, the UI can show a toast, set a pending flag, or optimistically update the affected market. This implementation relies on polling so the displayed state stays consistent with on-chain state.

`u64` values map to `bigint` in TypeScript. The handler converts the SOL amount to lamports and then to `BigInt` to avoid precision loss.

For stronger transaction feedback, wait for confirmation with `connection.confirmTransaction(signature)` and clear the pending state after the transaction lands.

The same pattern applies to the other instructions:

- `create_market` builds an instruction with a question and resolution time.
- `resolve_market` requires the creator's signature.
- `claim_winnings` uses the participant's position PDA.

Handlers should catch wallet rejections and program errors. The generated client exposes typed errors, which can be mapped to messages such as "Betting is closed" instead of a generic failure.

For `create_market`, the date input must be converted to a Unix timestamp:

```typescript
Math.floor(date.getTime() / 1000)
```

The value must match the program's `resolution_time`; otherwise the transaction fails with `ResolutionTimeInPast`.

Useful UI details include disabling bet buttons while a transaction is pending and displaying current pool sizes on each card.

A "Transaction too large" or "Blockhash not found" error during testing is usually a devnet or wallet timing issue; retrying with a fresh blockhash resolves it. This program is small enough that a compute-budget instruction is not required.

Optimistic UI is optional: pool totals can be updated locally after submission and reconciled on the next poll.

### Full Round Trip

```
User clicks          Build            Sign &           Program          Account
"Bet YES"      →   Instruction   →    Send       →   Executes     →    Updated
                        │                               │
                   Generated               Validation + state change
                   client                  (checked_add, time check)
```

The same flow applies to every instruction in the application.
