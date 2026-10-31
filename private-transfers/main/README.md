# Private Transfers on Solana

Privacy-oriented SOL transfers using Noir circuits, Merkle membership proofs, nullifiers, and Groth16 verification through Sunspot.

> **Demonstration project.** The code is not audited and is not suitable for production use.

This directory contains the reference implementation. It includes commitment-based deposits, a Merkle root history, nullifier replay protection, and on-chain proof verification through a fixed Sunspot verifier program.

## Implementation Status

| Property | Status |
| --- | --- |
| Program ID (`declare_id!`) | `8g5mRSVbbZ28fXcHNTwjvA66ZpRRQLkRb6scndFTm5UG` |
| Commitment deposits | Implemented |
| Merkle root history | Implemented, 10 roots |
| Nullifier replay protection | Implemented, 256-entry set |
| On-chain proof verification | Implemented via Sunspot CPI |
| Verifier program constant | `G6ZGZt9iJqejZBdSh4EMXrYiUXkCtedrzxsjorwSm7Un` |
| Backend proof service | Implemented (`backend/src/server.ts`) |
| Frontend | Implemented (Vite + React) |

## Architecture

```
Deposit
  user -- SOL --> pool vault
  client computes commitment and Merkle root
  program stores the new root in a 10-entry ring buffer
  program emits commitment, leaf index, and root

Withdrawal
  client generates a Groth16 proof for:
    commitment = Poseidon2(nullifier, secret, amount)
    commitment is a leaf in root
    nullifier_hash = Poseidon2(nullifier)
  program checks root and nullifier
  program calls the Sunspot verifier via CPI
  program marks the nullifier used
  pool vault -- SOL --> recipient
```

### Accounts

| Account | Seeds | Purpose |
| --- | --- | --- |
| `Pool` | `["pool"]` | Stores the authority, deposit counters, leaf index, root index, and root history. |
| `PoolVault` | `["vault", pool]` | Holds deposited SOL and signs withdrawals. |
| `NullifierSet` | `["nullifiers", pool]` | Stores used nullifier hashes. |
| Verifier program | Program ID constant | Sunspot-generated Groth16 verifier called by CPI. |

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

The nullifier set is a separate account because the list grows, while the pool account has a fixed size.

## Instructions

| Instruction | Arguments | Signer | Behavior |
| --- | --- | --- | --- |
| `initialize` | none | Authority | Creates the pool, vault, and nullifier set. |
| `deposit` | `commitment: [u8; 32]`, `new_root: [u8; 32]`, `amount: u64` | Depositor | Transfers SOL into the vault and records a new root. |
| `withdraw` | `proof: Vec<u8>`, `nullifier_hash: [u8; 32]`, `root: [u8; 32]`, `recipient: Pubkey`, `amount: u64` | Recipient | Verifies membership and transfers SOL from the vault. |

### `initialize`

- Creates `Pool` at `["pool"]` and initializes it with the empty Poseidon2 root.
- Creates `PoolVault` at `["vault", pool]`.
- Creates `NullifierSet` at `["nullifiers", pool]` and stores the pool key.

### `deposit`

- Requires `amount >= 1,000,000` lamports (0.001 SOL).
- Requires `next_leaf_index < 1024`.
- Transfers `amount` from the depositor to `PoolVault`.
- Writes `new_root` into the root ring buffer and advances `current_root_index`.
- Emits `DepositEvent` with the commitment, leaf index, new root, and timestamp.
- Increments `total_deposits` and `next_leaf_index`.

The depositor supplies `new_root`. The program does not recompute the Merkle tree on-chain. See [Security and Limitations](#security-and-limitations).

### `withdraw`

- Rejects a nullifier hash already present in `NullifierSet`.
- Rejects a root that is not in the 10-entry history.
- Requires the supplied `recipient` account to match the `recipient` argument.
- Requires the vault balance to cover `amount`.
- Encodes public inputs as `root || nullifier_hash || recipient || amount` with a 12-byte Groth16 header.
- Calls the Sunspot verifier with instruction data `proof || public_inputs`.
- Marks the nullifier used only after the verifier succeeds.
- Transfers `amount` from the vault to the recipient using the vault PDA as signer.
- Emits `WithdrawEvent` with the nullifier hash, recipient, and timestamp.

The verifier program account is constrained to `SUNSPOT_VERIFIER_ID`. Groth16 verification is compute-intensive; the frontend requests 1,400,000 compute units.

## Project Structure

```
main/
├── anchor/
│   ├── Anchor.toml
│   ├── programs/private_transfers/src/lib.rs
│   ├── scripts/init-pool.ts
│   └── tests/e2e.ts
├── backend/
│   └── src/server.ts          # Proof generation API on port 4001
├── circuits/
│   └── withdrawal/            # Noir circuit and Merkle helpers
├── frontend/
│   └── src/                   # Vite application and generated client
├── instructions/              # Step-by-step technical documentation
├── package.json
└── README.md
```

## Requirements

- Noir/Nargo 1.0.0-beta.13
- Sunspot and Go 1.24+ for proof generation and verifier deployment
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

Build and test the circuit from the repository root:

```bash
cd circuits/withdrawal
nargo compile
nargo test
```

Generate and deploy the verifier from `circuits/withdrawal`:

```bash
sunspot compile target/withdrawal.json
sunspot setup target/withdrawal.ccs
sunspot deploy target/withdrawal.vk
solana program deploy target/withdrawal.so --url devnet
```

Set `GNARK_VERIFIER_BIN` if `sunspot deploy` requires the gnark verifier binary:

```bash
export GNARK_VERIFIER_BIN=/path/to/sunspot/gnark-solana/crates/verifier-bin
```

Build and deploy the Anchor program from the repository root:

```bash
cd anchor
anchor build
anchor deploy --provider.cluster devnet
```

### Identifier Alignment

The repository contains several program and verifier identifiers. Align them before running the full flow:

| Location | Current value |
| --- | --- |
| `anchor/programs/private_transfers/src/lib.rs` (`declare_id!`) | `8g5mRSVbbZ28fXcHNTwjvA66ZpRRQLkRb6scndFTm5UG` |
| `anchor/Anchor.toml` devnet entry | `8g5mRSVbbZ28fXcHNTwjvA66ZpRRQLkRb6scndFTm5UG` |
| `backend/src/server.ts` (`PROGRAM_ID`) | `9T8s1qzKomXR17WgUo9mRwdNwWoQx4xjNtcMqr5pxk2M` |
| `frontend/src/idl/private_transfers.json` | `9T8s1qzKomXR17WgUo9mRwdNwWoQx4xjNtcMqr5pxk2M` |
| `lib.rs` (`SUNSPOT_VERIFIER_ID`) | `G6ZGZt9iJqejZBdSh4EMXrYiUXkCtedrzxsjorwSm7Un` |
| `frontend/src/constants.ts` (`SUNSPOT_VERIFIER_ID`) | `Amugr8yL9EQVAgGwqds9gCmjzs8fh6H3wjJ3eB4pBhXV` |
| `anchor/tests/e2e.ts` verifier | `Amugr8yL9EQVAgGwqds9gCmjzs8fh6H3wjJ3eB4pBhXV` |

The backend and frontend are not aligned with the declared program ID, and the frontend test verifier differs from the program constant. Update the backend constant, regenerate the frontend client from this variant's IDL, and set the verifier ID consistently before running against a local deployment.

## Running the Application

Start the proof service:

```bash
cd backend
bun run dev
```

The service listens on `http://localhost:4001`. It invokes the `sunspot` CLI to generate proofs and reads the circuit from `circuits/withdrawal`.

Start the frontend:

```bash
cd frontend
bun run dev
```

The frontend calls the proof service and submits deposits and withdrawals to the configured Solana RPC.

## Testing

Circuit tests:

```bash
cd circuits/withdrawal
nargo test
```

End-to-end test:

```bash
cd anchor
ANCHOR_PROVIDER_URL=https://api.devnet.solana.com \
ANCHOR_WALLET=~/.config/solana/id.json \
npx ts-mocha -p ./tsconfig.json -t 1000000 tests/e2e.ts
```

The end-to-end test requires:

- A running backend at `http://localhost:4001`.
- A compiled circuit.
- A deployed verifier and program.
- Matching verifier identifiers in the program, test, and backend.

This variant does not include a LiteSVM test module; the end-to-end test is the integration test.

## Security and Limitations

- **Trusted Merkle updates** - `deposit` accepts `new_root` from the caller and does not verify that it results from inserting `commitment` into the previous tree. A malicious depositor can insert a root for fabricated leaves, produce a valid membership proof, and withdraw funds from the shared vault. The circuit proves membership in the supplied root, not that the root was built only from valid deposits.
- **No deposit-to-leaf binding** - The program does not verify that `commitment` corresponds to the deposited amount.
- **Fixed verifier** - The verifier program ID is a compile-time constant. Rotating the verifier requires a program upgrade.
- **Nullifier capacity** - The set holds at most 256 nullifiers. The pool stops accepting withdrawals that require a new nullifier once full.
- **Variable amounts** - Deposit and withdrawal amounts are public, so amounts can be correlated.
- **No audit** - The circuits, verifier integration, and program are demonstration code.
- **No relayer** - The recipient pays the transaction fee and signs the withdrawal, which links the recipient to the withdrawal transaction.
- **Test coverage** - The repository does not include an automated negative-test suite for forged roots or malformed proofs.

## Documentation

| Document | Topic |
| --- | --- |
| [instructions/README.md](./instructions/README.md) | Documentation index and architecture |
| [step-0-introduction.md](./instructions/step-0-introduction.md) | Project introduction |
| [step-1-hiding-deposits.md](./instructions/step-1-hiding-deposits.md) | Commitments |
| [step-2-proving-membership.md](./instructions/step-2-proving-membership.md) | Merkle membership |
| [step-3-preventing-double-spend.md](./instructions/step-3-preventing-double-spend.md) | Nullifiers |
| [step-4.1-zk-circuit.md](./instructions/step-4.1-zk-circuit.md) | Withdrawal circuit |
| [step-4.2-sunspot-verifier.md](./instructions/step-4.2-sunspot-verifier.md) | Verifier generation |
| [step-4.3-verification-cpi.md](./instructions/step-4.3-verification-cpi.md) | Verification CPI |
| [step-4.4-frontend-demo.md](./instructions/step-4.4-frontend-demo.md) | Frontend flow |

## References

- [Noir](https://noir-lang.org)
- [Sunspot](https://github.com/reilabs/sunspot)
- [Anchor](https://anchor-lang.com)
- [Poseidon2](https://github.com/zkpassport/poseidon2)
