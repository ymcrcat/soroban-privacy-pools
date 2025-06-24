# Soroban Project

## Project Structure

This repository uses the recommended structure for a Soroban project:
```text
.
├── circuits
├── contracts
│   └── privacy_pools
│       ├── src
|		|	├── zk
|       |   |   └── mod.rs
│       │   ├── lib.rs
│       │   └── test.rs
│       └── Cargo.toml
├── Cargo.toml
└── README.md
```

- New Soroban contracts can be put in `contracts`, each in their own directory. There is already a `privacy_pools` contract.
- Contracts should have their own `Cargo.toml` files that rely on the top-level `Cargo.toml` workspace for their dependencies.