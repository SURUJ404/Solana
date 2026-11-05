# Private Transfers on Solana with Noir ZK

Privacy-preserving SOL transfers using Noir ZK circuits and onchain Groth16 verification via Sunspot.

This is a demonstration project for zero-knowledge proofs on Solana. It is not intended for production use.

## Contents

- [Overview](#overview)
- [How It Works](#how-it-works)
- [Prerequisites](#prerequisites)
- [Documentation](#documentation)
- [Quick Start](#quick-start)
- [Architecture](#architecture)
- [Onchain Accounts](#onchain-accounts)
- [Project Structure](#project-structure)
- [Implementation Notes](#implementation-notes)

## Overview

The project implements a shared pool for private SOL transfers. A sender deposits into the pool, and a recipient withdraws from it. Zero-knowledge proofs break the link between the deposit and the withdrawal.

The implementation covers:

- A Solana program that accepts private deposits
- ZK circuits that prove withdrawal rights without revealing the deposit
- Onchain verification using Sunspot (Groth16 proofs on Solana)
- A frontend for the complete flow

## How It Works

The system uses several cryptographic primitives together:

1. **Commitments** — Hide deposit details in a hash.
2. **Nullifiers** — Prevent double-spending without revealing which deposit is spent.
3. **Merkle Trees** — Efficiently prove that a deposit exists in the pool.
4. **ZK Circuits** — Prove the required statements without revealing private data.
5. **Sunspot Verification** — Verify Groth16 proofs onchain via CPI.

## Prerequisites

Solana knowledge:

- Accounts and PDAs
- CPI basics
- Anchor program structure

Tools:

- Anchor v1.0.0-rc.2, Solana CLI, Bun
- [Noir/Nargo v1.0.0-beta.13](https://noir-lang.org/docs) — ZK circuit compiler
- [Sunspot CLI](https://github.com/reilabs/sunspot) — Groth16 proving for Solana
- Go 1.24+ — required for building Sunspot

## Documentation

The filenames retain their original step prefixes for continuity. Each document is a standalone technical note, and the table is organized by concept.

| Document | Topic |
| --- | --- |
| [step-0-introduction.md](./step-0-introduction.md) | Introduction and project structure |
| [step-1-hiding-deposits.md](./step-1-hiding-deposits.md) | Hiding deposit details with hashes |
| [step-2-proving-membership.md](./step-2-proving-membership.md) | Membership proofs with Merkle trees |
| [step-3-preventing-double-spend.md](./step-3-preventing-double-spend.md) | Double-spend prevention with nullifiers |
| [step-4.1-zk-circuit.md](./step-4.1-zk-circuit.md) | The withdrawal proof circuit |
| [step-4.2-sunspot-verifier.md](./step-4.2-sunspot-verifier.md) | Sunspot verifier generation and deployment |
| [step-4.3-verification-cpi.md](./step-4.3-verification-cpi.md) | Onchain proof verification via CPI |
| [step-4.4-frontend-demo.md](./step-4.4-frontend-demo.md) | Frontend demo |

## Quick Start

```bash
# Install dependencies
bun install

# Build the Anchor program
cd anchor
anchor build

# Start the frontend
cd ../frontend && bun run dev
```

Circuit compilation (`nargo compile`) and Sunspot setup are covered in [step-4.1-zk-circuit.md](./step-4.1-zk-circuit.md) and [step-4.2-sunspot-verifier.md](./step-4.2-sunspot-verifier.md).

## Architecture

```
┌─────────────────┐      ┌─────────────────┐      ┌─────────────────┐
│                 │      │                 │      │                 │
│    Frontend     │─────▶│    Backend      │─────▶│  Noir Circuits  │
│    (React)      │      │   (Express)     │      │   (nargo CLI)   │
│                 │◀─────│                 │◀─────│                 │
└─────────────────┘      └─────────────────┘      └─────────────────┘
        │                                                  │
        │                                                  │
        ▼                                                  ▼
┌─────────────────┐                              ┌─────────────────┐
│                 │                              │                 │
│  Solana RPC     │◀────────────────────────────│ Sunspot Verifier│
│                 │                              │   (onchain)    │
└─────────────────┘                              └─────────────────┘
```

## Onchain Accounts

| Account          | Type    | Seeds                  | Purpose                                                |
| ---------------- | ------- | ---------------------- | ------------------------------------------------------ |
| **Pool**         | PDA     | `["pool"]`             | Stores Merkle root history, leaf index, deposit count  |
| **PoolVault**    | PDA     | `["vault", pool]`      | Holds deposited SOL                                    |
| **NullifierSet** | PDA     | `["nullifiers", pool]` | Tracks used nullifier hashes to prevent double-spend   |
| **Verifier**     | Program | N/A                    | Sunspot-generated program that verifies Groth16 proofs |

### Account Roles

**Pool** — The state of the privacy pool. Stores:

- `roots[10]`: Ring buffer of recent Merkle roots (allows proof timing flexibility)
- `next_leaf_index`: Where the next deposit goes in the tree
- `current_root_index`: Which root is newest

**PoolVault** — Holds the actual SOL. Separate from Pool because:

- Pool stores data, Vault holds lamports
- Vault is a PDA so the program can sign transfers out

**NullifierSet** — Prevents double-spending:

- Withdrawals add the `nullifier_hash` to this account
- Future withdrawals check whether the nullifier hash is already used
- Stored separately because it grows (`Vec`) while Pool is fixed-size

> **Scaling note:** With 256 max nullifiers, `contains()` is sufficient. At larger scale, one PDA per nullifier (hash as seed) provides O(1) lookup.

**Verifier** — External program generated by Sunspot:

- Contains the verification key
- Called by the program via CPI
- Returns success or error based on proof validity

## Project Structure

```
noir-solana-private-transfers/
├── circuits/                    # Noir ZK circuits
│   ├── hasher/                  # Computes commitment and nullifier hash
│   ├── merkle-hasher/           # Computes Merkle roots
│   └── withdrawal/              # Main withdrawal proof circuit
├── anchor/                      # Solana program
│   └── programs/private_transfers/src/lib.rs  # Program implementation
├── backend/                     # Proof generation server
├── frontend/                    # React UI
└── instructions/                # Documentation
```

## Implementation Notes

The Noir circuits are complete. The Solana program integration is centered on `anchor/programs/private_transfers/src/lib.rs`, which connects the circuits, the pool accounts, and the Sunspot verifier.
