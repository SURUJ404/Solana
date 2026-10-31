# Private Transfers on Solana: Baseline Stage

Baseline SOL pool used as the starting point for the private-transfer implementation.

> **Demonstration project.** The code is not audited and is not suitable for production use.

This variant implements a shared pool with public deposits and public withdrawals. It does not use commitments, Merkle roots, nullifiers, or zero-knowledge proofs. The program demonstrates the vault and PDA structure that later variants extend.

## Implementation Status

| Property | Status |
| --- | --- |
| Program ID (`declare_id!`) | `2QRZu5cWy8x8jEFc9nhsnrnQSMAKwNpiLpCXrMRb3oUn` |
| Commitment deposits | Not implemented |
| Merkle root history | Not implemented |
| Nullifier replay protection | Not implemented |
| On-chain proof verification | Not implemented |
| Backend proof service | Present, but targets the final ABI |
| Frontend | Present, but the generated client targets the final ABI |

## Architecture

```
Deposit
  user -- SOL --> pool vault
  program emits depositor, amount, and timestamp

Withdrawal
  caller supplies recipient and amount
  pool vault -- SOL --> recipient
```

### Accounts

| Account | Seeds | Purpose |
| --- | --- | --- |
| `Pool` | `["pool"]` | Stores the authority and deposit count. |
| `PoolVault` | `["vault", pool]` | Holds deposited SOL and signs withdrawals. |

### `Pool`

| Field | Type | Description |
| --- | --- | --- |
| `authority` | `Pubkey` | Account that initialized the pool. |
| `total_deposits` | `u64` | Number of deposits. |

## Instructions

| Instruction | Arguments | Signer | Behavior |
| --- | --- | --- | --- |
| `initialize` | none | Authority | Creates the pool and vault. |
| `deposit` | `amount: u64` | Depositor | Transfers SOL into the vault and emits a deposit event. |
| `withdraw` | `recipient: Pubkey`, `amount: u64` | Recipient | Transfers SOL from the vault. |

### `initialize`

- Creates `Pool` at `["pool"]`.
- Creates `PoolVault` at `["vault", pool]`.
- Sets `authority` and zeroes `total_deposits`.

### `deposit`

- Requires `amount >= 1,000,000` lamports (0.001 SOL).
- Transfers SOL from the depositor to the vault.
- Emits `DepositEvent` with `depositor`, `amount`, and `timestamp`.
- Increments `total_deposits`.

### `withdraw`

- Requires the recipient account to match the `recipient` argument.
- Requires the vault balance to cover `amount`.
- Transfers `amount` from the vault to the recipient using the vault PDA as signer.
- Emits `WithdrawEvent` with the recipient, amount, and timestamp.

Any caller can withdraw any amount up to the vault balance.

## Project Structure

```
starter/
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
| `lib.rs` (`declare_id!`) | `2QRZu5cWy8x8jEFc9nhsnrnQSMAKwNpiLpCXrMRb3oUn` |
| `Anchor.toml` devnet entry | `2QRZu5cWy8x8jEFc9nhsnrnQSMAKwNpiLpCXrMRb3oUn` |
| `backend/src/server.ts` (`PROGRAM_ID`) | `9T8s1qzKomXR17WgUo9mRwdNwWoQx4xjNtcMqr5pxk2M` |
| `frontend/src/idl/private_transfers.json` | `9T8s1qzKomXR17WgUo9mRwdNwWoQx4xjNtcMqr5pxk2M` |

The program source and the `Anchor.toml` devnet entry agree. The backend and bundled frontend IDL target a separate program. Update both and regenerate the frontend client before running this variant.

## Running the Frontend and Backend

The backend and frontend were copied from the final implementation and expect the four-input withdrawal ABI with `proof`. This variant uses `withdraw(recipient, amount)`.

To run them against this program:

1. Regenerate `frontend/src/generated` from this variant's IDL.
2. Update `backend/src/server.ts` to the variant's program ID and withdrawal arguments.
3. Remove the proof-generation step for this variant.

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

- **Public deposits** - The deposit event records the depositor's public key.
- **Public withdrawals** - Anyone can withdraw from the vault. No commitment, proof, or nullifier is required.
- **No replay protection** - Withdrawals can be repeated until the vault is empty.
- **No privacy** - Deposits and withdrawals are directly linkable.
- **No audit** - Demonstration code only.

This stage demonstrates the shared vault and PDA signer pattern. It provides no privacy guarantees.

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
