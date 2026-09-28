# Contributing to Zephyr Contracts

Thanks for helping build an open, trust-minimised USD ⇄ USDC ramp on Stellar. This repo holds the Soroban **withdrawal escrow** contract, its deploy scripts, and the generated TypeScript client (`@zephyr-ramp/escrow-client`) used by [zephyr-backend](https://github.com/zephyr-ramp/zephyr-backend) and [zephyr-frontend](https://github.com/zephyr-ramp/zephyr-frontend).

## Setup

You need:

- Rust stable (the repo's `rust-toolchain.toml` installs the `wasm32v1-none` target, `rustfmt` and `clippy` for you through `rustup`);
- [Stellar CLI](https://developers.stellar.org/docs/tools/cli/install-cli) **v28.1.0 or newer**. soroban-sdk 28 contracts must be built with `stellar contract build`; plain `cargo build --target wasm32v1-none` fails on purpose;
- Node 20+ (only if you touch `bindings/`).

```bash
git clone https://github.com/<you>/zephyr-contracts
cd zephyr-contracts
stellar contract build     # optimised wasm in target/wasm32v1-none/release/
cargo test                 # runs fully offline in an in-process Soroban host
```

## Picking up an issue

1. Find an issue labelled `help wanted` or `good first issue`.
2. Comment on it, or apply through Drips Wave if it's a Wave issue, and **wait to be assigned before starting**. Unassigned PRs may be closed so that assigned contributors don't lose their work.
3. One issue per PR. If the issue is unclear, ask in the thread before writing code.

### Drips Wave

Wave issues carry a complexity label. Points are awarded when your PR is merged and the issue is resolved **during an active Wave**:

| Label | Points | Typical scope |
|---|---|---|
| `complexity: trivial` | 100 | Docs, a script flag, a small test addition |
| `complexity: medium` | 150 | A contained contract change or new test suite, with tests |
| `complexity: high` | 200 | A new contract feature, storage/event change, or tooling integration |

Apply early so there's time to get a review before the Wave ends.

## Branches, commits and PRs

- Branch from `main`: `feat/short-description`, `fix/...`, `docs/...`, `test/...`.
- Use [Conventional Commits](https://www.conventionalcommits.org): `feat:`, `fix:`, `docs:`, `test:`, `chore:`, `refactor:`. Keep commits small and logical.
- Open the PR against `main`, fill in the template, and link the issue (`Closes #123`).
- CI must be green. A maintainer from `CODEOWNERS` reviews every contract change.

## Code style and contract rules

Run these before you push:

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
stellar contract build && cargo test
```

Contract rules we enforce in review:

- **Amounts are `i128` in stroops.** Never use floats; reject `amount <= 0`.
- **Never panic with a string.** Return an `Error` variant. Error codes are part of the public interface: add new variants at the end and never renumber existing ones.
- **Checks, effects, interactions.** Validate, then write the new status, then transfer tokens.
- **`refund` must always work** for an expired `Locked` escrow, including while paused. Any change that could block it needs a test proving it doesn't.
- **Extend TTLs on every write** (see `storage.rs`), so active escrows are never archived.
- **Every new code path needs a test**, including its error cases and auth (`env.auths()` or `mock_auths`).

### Changing the public interface

If you change a function signature, a `#[contracttype]`, an event or an error code:

1. Update the function table and events section in `README.md`.
2. Regenerate the client: `LOCAL_ONLY=1 ./scripts/bindings.sh`, and commit `bindings/src` and `bindings/dist`.
3. Say so in the PR description, because the backend and frontend will need a matching change.

## Tests

| Command | What it runs |
|---|---|
| `cargo test` | All unit, auth, expiry and property tests |
| `cargo test expiry` | Only tests whose name contains `expiry` |
| `PROPTEST_CASES=1000 cargo test invariant` | The balance-invariant property test, with more cases |

Tests live in `contracts/escrow/src/test/`. Start from the `Setup` fixture in `test/mod.rs`: it registers a USDC-like Stellar Asset Contract, funds a user, and initialises the escrow.

## Maintainers

| Maintainer | GitHub |
|---|---|
| N-thnI | [@N-thnI](https://github.com/N-thnI) |
| nixx | [@N-i-xx](https://github.com/N-i-xx) |

Maintainers assign issues, review PRs (see `.github/CODEOWNERS`) and handle security and conduct reports sent to [niheanyi404@gmail.com](mailto:niheanyi404@gmail.com).

## Security

Never report vulnerabilities in public issues. See [SECURITY.md](SECURITY.md).

## Code of Conduct

This project follows the [Contributor Covenant 2.1](CODE_OF_CONDUCT.md).
