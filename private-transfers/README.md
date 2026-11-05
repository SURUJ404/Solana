# Private Transfers

Private SOL transfers using commitments, Merkle membership proofs, nullifiers, and Noir/Groth16 verification through Sunspot.

> Part of the [Solana Core Topics](../README.md) reference.

## Canonical implementation

The reference implementation is in [`main/`](./main/README.md). It contains the Anchor program, the Noir withdrawal circuit, Sunspot verifier integration, the proof-generation service, and the React frontend.

This is demonstration code. It is not audited and is not suitable for production use.

## Core concepts

- Commitment-based deposits that hide deposit details
- Merkle root history and membership proofs
- Nullifier replay protection
- Noir zero-knowledge circuits and Groth16 proofs
- On-chain proof verification through a Sunspot verifier CPI
- Off-chain proof generation and a browser client

## Other directories

`starter/` and the `step-*` directories are earlier checkpoints retained for comparison. They are not separate topics and are not required to build or run the reference implementation. Use `main/` unless you are comparing how the design evolved.

## Documentation

| Document | Topic |
| --- | --- |
| [`main/README.md`](./main/README.md) | Program architecture, instructions, accounts, setup, testing, and limitations |
| [`main/instructions/README.md`](./main/instructions/README.md) | Technical notes index for the protocol design |
| [`main/instructions/step-1-hiding-deposits.md`](./main/instructions/step-1-hiding-deposits.md) | Commitments |
| [`main/instructions/step-2-proving-membership.md`](./main/instructions/step-2-proving-membership.md) | Merkle membership |
| [`main/instructions/step-3-preventing-double-spend.md`](./main/instructions/step-3-preventing-double-spend.md) | Nullifiers |
| [`main/instructions/step-4.1-zk-circuit.md`](./main/instructions/step-4.1-zk-circuit.md) | Withdrawal circuit |
| [`main/instructions/step-4.2-sunspot-verifier.md`](./main/instructions/step-4.2-sunspot-verifier.md) | Verifier generation and deployment |
| [`main/instructions/step-4.3-verification-cpi.md`](./main/instructions/step-4.3-verification-cpi.md) | Verification CPI |
| [`main/instructions/step-4.4-frontend-demo.md`](./main/instructions/step-4.4-frontend-demo.md) | Frontend flow |

## References

- [Noir](https://noir-lang.org)
- [Sunspot](https://github.com/reilabs/sunspot)
- [Anchor](https://anchor-lang.com)
- [Poseidon2](https://github.com/zkpassport/poseidon2)
