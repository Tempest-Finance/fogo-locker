# Fogo Locker

[![FOGO](https://img.shields.io/badge/FOGO-grey?logo=lightning&style=for-the-badge)](https://fogo.io)
[![CI](https://img.shields.io/github/actions/workflow/status/Tempest-Finance/fogo-locker/ci-pr-main-program.yml?logo=githubactions&logoColor=white&style=for-the-badge&label=CI)](https://github.com/Tempest-Finance/fogo-locker/actions/workflows/ci-pr-main-program.yml)

A token vesting and locking program for the FOGO blockchain, forked from [Jupiter Locker](https://github.com/jup-ag/jup-lock). 
Extended with Fogo Sessions support for delegated signing.

## Program ID

| Network  | Address                                        |
| -------- | ---------------------------------------------- |
| Mainnet  | `LockvXm2nWht6EvHf44AmCuS3eMKRiWTuks2x27XRRo` |
| Testnet  | `LockvXm2nWht6EvHf44AmCuS3eMKRiWTuks2x27XRRo` |

## Quick Start

```bash
# Build the program
make build

# Run tests
make test

# See all commands
make help
```

## Components

| Directory      | Description                                       |
| -------------- | ------------------------------------------------- |
| `programs/`    | On-chain Anchor program (Rust)                    |
| `cli/`         | Command-line interface for escrow operations       |
| `sdk/`         | TypeScript SDK and IDL artifacts                   |
| `merkle-tree/` | Merkle tree library for batch escrow creation      |
| `tests/`       | Integration tests (TypeScript/Mocha)               |

## Development

```bash
# Format code
make fmt

# Run linter
make lint

# Unit tests only
make test/unit

# Deploy to testnet
make deploy CLUSTER=testnet
```

## Audits

- **Codespect** (2026-02-09): [`audits/058_CODESPECT_IGNITION_FOGO_LOCKER.pdf`](./audits/058_CODESPECT_IGNITION_FOGO_LOCKER.pdf)
- **Adevar** (2026-02-09): [`audits/Adevar_fogo_lock_after_fix_review_report.pdf`](./audits/Adevar_fogo_lock_after_fix_review_report.pdf)
- **OtterSec** (2024-08-15): [`audits/OtterSec_2024_08_15.pdf`](./audits/OtterSec_2024_08_15.pdf)
- **Sec3** (2024-08-05): [`audits/Sec3_2024_08_05.pdf`](./audits/Sec3_2024_08_05.pdf)

## License

Apache 2.0
