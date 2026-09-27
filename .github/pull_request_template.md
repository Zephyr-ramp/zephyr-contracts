## What

Closes #

## How

<!-- What changed and why. Call out any change to storage layout, events, error codes or auth. -->

## Checklist

- [ ] I was assigned to the linked issue before starting
- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --all-targets -- -D warnings` passes
- [ ] `stellar contract build && cargo test` passes, and new behaviour has tests
- [ ] Amounts stay `i128`; no string panics (use `Error` variants)
- [ ] Status is written **before** any token transfer (checks-effects-interactions)
- [ ] If the public interface changed, I ran `./scripts/bindings.sh` (or said why not)
- [ ] No secret keys or `.stellar/` identity files committed
