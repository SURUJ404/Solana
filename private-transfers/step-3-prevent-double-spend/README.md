# Private Transfers on Solana: Double-Spend Prevention Stage

Privacy-oriented SOL transfers using Noir circuits, Merkle membership proofs, and nullifiers.

> **Demonstration project.** The code is not audited and is not suitable for production use.

This variant adds a nullifier set to the earlier Merkle-root implementation. Withdrawals consume a nullifier so the same note cannot be redeemed twice. Proof verification is not yet connected: the program accepts the nullifier and root as instruction arguments without verifying a Groth16 proof.

## Implementation Status

| Property | Status |
| --- | --- |
| Program ID (`declare_id!`) | `HzEfEnt2E6T6gmy9VQi2d15TN5PYAy78iq7WHPF9ddHB` |
| Commitment deposits | Implemented |
| Merkle root history | Implemented, 10 roots |
| Nullifier replay protection | Implemented, 256-entry set |
| On-chain proof verification | Not implemented |
| Verifier CPI | Not connected; the mock verifier is not called by `withdraw` |
| Backend proof service | Present, but targets the final ABI |
| Frontend | Present, but the generated client targets the final ABI |

## Architecture

```
Deposit
  user -- SOL --> pool vault
  client supplies commitment and new Merkle root
  program stores the root and emits the commitment

Withdrawal
  caller supplies nullifier_hash, root, recipient, and amount
  program checks the nullifier is unused and the root is known
  program marks the nullifier used
  pool vault -- SOL --> recipient
```

### Accounts

| Account | Seeds | Purpose |
| --- | --- | --- |
| `Pool` | `["pool"]` | Stores the authority, deposit counters, leaf index, and root history. |
| `PoolVault` | `["vault", pool]` | Holds deposited SOL and signs withdrawals. |
| `NullifierSet` | `["nullifiers", pool]` | Stores used nullifier hashes. |

### `Pool`

| Field | Type | Description |
| --- | --- | --- |
| `authority` | `Pubkey` | Account that initialized the pool. |
| `total_deposits` | `u64` | Number of deposits. |
| `next_leaf_index` | `u64` | Next Merkle leaf index. |
| `current_root_index` | `u64` | Index of the newest root in the ring buffer. |
| `roots` | `[[u8; 32]; 10]` | Recent Merkle roots. |

### `NullifierSet`

| Field | Type | Description |
| --- | --- | --- |
| `pool` | `Pubkey` | Pool the set belongs to. |
| `nullifiers` | `Vec<[u8; 32]>` | Used nullifier hashes, capped at 256. |

## Instructions

| Instruction | Arguments | Signer | Behavior |
| --- | --- | --- | --- |
| `initialize` | none | Authority | Creates the pool, vault, and nullifier set. |
| `deposit` | `commitment: [u8; 32]`, `new_root: [u8; 32]`, `amount: u64` | Depositor | Transfers SOL into the vault and records a new root. |
| `withdraw` | `nullifier_hash: [u8; 32]`, `root: [u8; 32]`, `recipient: Pubkey`, `amount: u64` | Recipient | Consumes a nullifier and transfers SOL from the vault. |

### `initialize`

- Creates `Pool` at `["pool"]` with the empty Poseidon2 root.
- Creates `PoolVault` at `["vault", pool]`.
- Creates `NullifierSet` at `["nullifiers", pool]`.

### `deposit`

- Requires `amount >= 1,000,000` lamports (0.001 SOL).
- Requires `next_leaf_index < 1024`.
- Transfers SOL from the depositor to the vault.
- Stores `new_root` in the root ring buffer and advances `current_root_index`.
- Emits the commitment, leaf index, new root, and timestamp.
- Increments `total_deposits` and `next_leaf_index`.

### `withdraw`

- Rejects a nullifier hash already present in `NullifierSet`.
- Rejects a root that is not in the 10-entry history.
- Requires the recipient account to match the `recipient` argument.
- Requires the vault balance to cover `amount`.
- Marks the nullifier used.
- Transfers `amount` from the vault to the recipient using the vault PDA as signer.

No proof is checked. The nullifier hash and root are caller-supplied values, so the replay guard only prevents reusing the same nullifier value.

## Project Structure

```
step-3-prevent-double-spend/
├── anchor/
│   ├── Anchor.toml
│   ├── programs/private_transfers/
│   │   ├── src/lib.rs
│   │   └── tests/litesvm_tests.rs
│   └── tests/e2e.ts
├── backend/
│   └── src/server.ts
├── circuits/
│   └── withdrawal/
├── frontend/
│   └── src/
├── instructions/
├── package.json
└── README.md
```

## Requirements

- Noir/Nargo 1.0.0-beta.13
- Sunspot and Go 1.24+ (for the later verification stages)
- Anchor 1.0.0-rc.2
- Solana CLI
- Bun

## Setup

Install the JavaScript dependencies:

```bash
bun install
cd frontend && bun install
cd ../backend && bun install
cd ../anchor && bun install
```

Build the circuit from the repository root:

```bash
cd circuits/withdrawal
nargo compile
nargo test
```

Build and deploy the program from the repository root:

```bash
cd anchor
anchor build
anchor deploy --provider.cluster devnet
```

### Identifier Alignment

| Location | Current value |
| --- | --- |
| `lib.rs` (`declare_id!`) | `HzEfEnt2E6T6gmy9VQi2d15TN5PYAy78iq7WHPF9ddHB` |
| `Anchor.toml` devnet entry | `2QRZu5cWy8x8jEFc9nhsnrnQSMAKwNpiLpCXrMRb3oUn` |
| `backend/src/server.ts` (`PROGRAM_ID`) | `9T8s1qzKomXR17WgUo9mRwdNwWoQx4xjNtcMqr5pxk2M` |
| `frontend/src/idl/private_transfers.json` | `9T8s1qzKomXR17WgUo9mRwdNwWoQx4xjNtcMqr5pxk2M` |

The declared program ID and the `Anchor.toml` devnet entry differ. The backend and bundled frontend IDL target a separate program. Update all four locations and regenerate the frontend client before running this variant.

## Running the Frontend and Backend

The backend and frontend in this directory were copied from the final implementation. Their generated client and proof service use the four-input withdrawal ABI with `proof`, which does not match this variant's program.

To run them against this program:

1. Regenerate `frontend/src/generated` from this variant's IDL.
2. Update `backend/src/server.ts` to the variant's program ID and withdrawal arguments.
3. Remove or adapt the proof-generation call for the three-input withdrawal.

The backend proof service is not required to exercise this variant's `deposit` and `withdraw` instructions directly.

## Testing

Circuit tests:

```bash
cd circuits/withdrawal
nargo test
```

LiteSVM deposit tests:

```bash
cd anchor
anchor build
cargo test --test litesvm_tests
```

The LiteSVM suite covers deposit behavior. The TypeScript end-to-end test in `anchor/tests/e2e.ts` was copied from the final implementation and expects proof arguments; it does not match this program ABI without modification.

## Security and Limitations

- **No proof verification** - `withdraw` does not verify that the caller knows a commitment in the tree. Any caller can supply a known root and a fresh nullifier hash and withdraw vault funds.
- **Trusted Merkle updates** - `deposit` accepts `new_root` from the caller without recomputing the tree.
- **No deposit-to-leaf binding** - The commitment is not connected to the deposited amount.
- **Nullifier capacity** - The set holds at most 256 entries.
- **Variable amounts** - Amounts are public and correlate deposits with withdrawals.
- **No audit** - Demonstration code only.

This stage is useful for demonstrating nullifier storage and replay checks. It is not a private transfer system until proof verification is connected.

## Documentation

| Document | Topic |
| --- | --- |
| [instructions/README.md](./instructions/README.md) | Documentation index |
| [step-0-introduction.md](./instructions/step-0-introduction.md) | Project introduction |
| [step-1-hiding-deposits.md](./instructions/step-1-hiding-deposits.md) | Commitments |
| [step-2-proving-membership.md](./instructions/step-2-proving-membership.md) | Merkle membership |
| [step-3-preventing-double-spend.md](./instructions/step-3-preventing-double-spend.md) | Nullifiers |
| [step-4-zk-circuit.md](./instructions/step-4-zk-circuit.md) | Withdrawal circuit |
| [step-5-sunspot-verifier.md](./instructions/step-5-sunspot-verifier.md) | Verifier generation |
| [step-6-verification-cpi.md](./instructions/step-6-verification-cpi.md) | Verification CPI |
| [step-7-frontend-demo.md](./instructions/step-7-frontend-demo.md) | Frontend demo |

## References

- [Noir](https://noir-lang.org)
- [Sunspot](https://github.com/reilabs/sunspot)
- [Anchor](https://anchor-lang.com)
