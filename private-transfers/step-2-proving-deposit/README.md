# Private Transfers on Solana: Merkle Membership Stage

Privacy-oriented SOL transfers using Noir circuits and Merkle membership proofs.

> **Demonstration project.** The code is not audited and is not suitable for production use.

This variant adds commitment deposits and a Merkle root history. Withdrawals require a root that the pool has seen before. Nullifier tracking and proof verification are not implemented at this stage; the program accepts the root and amount directly from the caller.

## Implementation Status

| Property | Status |
| --- | --- |
| Program ID (`declare_id!`) | `HzEfEnt2E6T6gmy9VQi2d15TN5PYAy78iq7WHPF9ddHB` |
| Commitment deposits | Implemented |
| Merkle root history | Implemented, 10 roots |
| Nullifier replay protection | Not implemented |
| On-chain proof verification | Not implemented |
| Mock verifier | Present under `anchor/programs/mock_verifier`, not called by `withdraw` |
| Backend proof service | Present, but targets the final ABI |
| Frontend | Present, but the generated client targets the final ABI |

## Architecture

```
Deposit
  user -- SOL --> pool vault
  client supplies commitment and new Merkle root
  program stores the root and emits the commitment

Withdrawal
  caller supplies root, recipient, and amount
  program checks the root is known
  pool vault -- SOL --> recipient
```

### Accounts

| Account | Seeds | Purpose |
| --- | --- | --- |
| `Pool` | `["pool"]` | Stores the authority, deposit counters, leaf index, and root history. |
| `PoolVault` | `["vault", pool]` | Holds deposited SOL and signs withdrawals. |

### `Pool`

| Field | Type | Description |
| --- | --- | --- |
| `authority` | `Pubkey` | Account that initialized the pool. |
| `total_deposits` | `u64` | Number of deposits. |
| `next_leaf_index` | `u64` | Next Merkle leaf index. |
| `current_root_index` | `u64` | Index of the newest root in the ring buffer. |
| `roots` | `[[u8; 32]; 10]` | Recent Merkle roots. |

## Instructions

| Instruction | Arguments | Signer | Behavior |
| --- | --- | --- | --- |
| `initialize` | none | Authority | Creates the pool and vault with the empty Poseidon2 root. |
| `deposit` | `commitment: [u8; 32]`, `new_root: [u8; 32]`, `amount: u64` | Depositor | Transfers SOL into the vault and records a new root. |
| `withdraw` | `root: [u8; 32]`, `recipient: Pubkey`, `amount: u64` | Recipient | Transfers SOL when the root is known. |

### `initialize`

- Creates `Pool` at `["pool"]`.
- Sets `roots[0]` to the empty Poseidon2 root.
- Creates `PoolVault` at `["vault", pool]`.

### `deposit`

- Requires `amount >= 1,000,000` lamports (0.001 SOL).
- Requires `next_leaf_index < 1024`.
- Transfers SOL from the depositor to the vault.
- Stores `new_root` in the ring buffer and advances `current_root_index`.
- Emits the commitment, leaf index, new root, and timestamp.
- Increments `total_deposits` and `next_leaf_index`.

### `withdraw`

- Rejects a root that is not in the 10-entry history.
- Requires the recipient account to match the `recipient` argument.
- Requires the vault balance to cover `amount`.
- Transfers `amount` from the vault to the recipient using the vault PDA as signer.

There is no nullifier and no proof check. A known root can be reused for any number of withdrawals.

## Project Structure

```
step-2-proving-deposit/
├── anchor/
│   ├── Anchor.toml
│   ├── programs/
│   │   ├── mock_verifier/src/lib.rs
│   │   └── private_transfers/
│   │       ├── src/lib.rs
│   │       └── tests/litesvm_tests.rs
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
- Anchor 1.0.0-rc.2
- Solana CLI
- Bun
- Sunspot and Go 1.24+ only for later verification stages

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

The backend and frontend were copied from the final implementation and expect the four-input withdrawal ABI with `proof`. This variant uses `withdraw(root, recipient, amount)`.

To run them against this program:

1. Regenerate `frontend/src/generated` from this variant's IDL.
2. Update `backend/src/server.ts` to the variant's program ID and withdrawal arguments.
3. Remove the proof-generation step for this variant.

The backend proof service is not required to exercise `deposit` and `withdraw` directly.

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

The TypeScript end-to-end test in `anchor/tests/e2e.ts` was copied from the final implementation and does not match this program ABI.

## Security and Limitations

- **No proof verification** - A caller does not need to know a commitment in the tree. Any known root allows a withdrawal.
- **No replay protection** - Without nullifiers, the same root can be used repeatedly until the vault is empty.
- **Trusted Merkle updates** - `deposit` accepts `new_root` from the caller without recomputing the tree.
- **No deposit-to-leaf binding** - The commitment is not connected to the deposited amount.
- **Variable amounts** - Amounts are public and correlate deposits with withdrawals.
- **No audit** - Demonstration code only.

This stage is useful for demonstrating root history and membership checks. It does not provide privacy or solvency guarantees.

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
