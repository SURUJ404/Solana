# Security & Trade-offs

This implementation prioritizes a small, auditable program and a clear end-to-end flow. Several production concerns are intentionally out of scope. The sections below describe the resulting trade-offs and the protections that are implemented.

## Design Decisions

| Choice | Trade-off |
| --- | --- |
| Creator resolves market | Simple but centralized trust |
| Polling vs WebSockets | Simpler code, slightly delayed updates |
| All-or-nothing bets | No partial positions, simpler math |
| No fees | No protocol revenue, pure market |

### Resolution

The market creator decides the outcome. This keeps the program small and avoids oracle integration, but it introduces a trust assumption. A production deployment would use an oracle or multi-signature process so no single party can manipulate results.

### Polling

Polling is simple to implement and understand, but it adds latency and extra RPC load. A real-time UI would use WebSockets or an indexer.

### Position Model

All-or-nothing bets keep the accounting simple. There are no partial exits, cash-outs, or liquidity positions. This limits trading flexibility but reduces the state and math the program must handle.

### Fees

The program charges no fees. This keeps the payout math pure and the example clear, but it provides no protocol revenue or sustainability model. A production system would likely add a small fee on bets or winnings.

## Security Protections

| Attack Vector | Protection |
| --- | --- |
| Overflow attacks | `checked_add/mul/div` |
| Double claims | `position.claimed` flag |
| Late bets | Time window validation |
| Unauthorized resolution | Creator-only check |

### Arithmetic Safety

Pool updates use checked arithmetic. In a parimutuel system, overflowing a pool could change implied prices or allow free positions. Checked math prevents that class of attack.

Payout multiplication uses `u128` intermediates before conversion back to `u64`, which avoids overflow when multiplying large pool values.

### Claim Safety

`claim_winnings` requires `market.resolved` and rejects positions where `claimed` is already true. Claims are therefore idempotent and cannot be repeated.

The `UserPosition` PDA is validated against the signer through account constraints, so a caller cannot claim another participant's position.

### Timing and Authorization

`place_bet` rejects transactions at or after `resolution_time`. `resolve_market` is restricted to the market creator and can only run after the deadline.

## Known Limitations

- The creator can resolve early or select an incorrect outcome. The program does not validate the truth of the result.
- There is no cost or stake requirement for creating markets, so spam markets are possible.
- Transaction ordering and deadline races are not mitigated. Participants may compete to place bets near the cutoff.
- There is no fee, treasury, or protocol revenue.
- Positions cannot be partially exited or sold before resolution.
- The frontend polls instead of subscribing to account changes, so updates are delayed by up to the polling interval.
- Position accounts are not closed after claims, so rent is not reclaimed.

## Potential Extensions

- Decentralized oracles such as Switchboard or Pyth for trustless resolution
- A fee mechanism for protocol sustainability
- Partial position exits or liquidity
- Multi-outcome markets instead of binary YES/NO
- Market creation fees or reputation to reduce spam
- An on-chain finalization window to reduce deadline racing
- WebSockets or an indexer for real-time account updates

A fee changes the payout math, so it requires careful rounding and accounting. Oracle resolution changes the trust model and the instruction set. Both are substantial design changes rather than isolated additions.
