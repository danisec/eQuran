# Pull request

## What this changes

<!-- One or two sentences. What behavior is different after this? -->

## Why

<!-- The reason. Link an issue if there is one: "Closes #12". -->

## How it was tested

<!-- Which tests, and any manual steps you ran. -->

## Checklist

- [ ] `cargo fmt --all --check` passes
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes
- [ ] `cargo test --workspace --all-features` passes
- [ ] `npm run check` and `npm test` pass in `frontend/`
- [ ] New behavior has a test; the fix has a test that fails without it
- [ ] Comments explain *why*, not *what*
- [ ] The change is focused on one concern
