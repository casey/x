watch +args='test':
  cargo watch --clear --exec '{{args}}'

outdated:
  cargo outdated --root-deps-only --workspace

unused:
  cargo +nightly udeps --workspace

ci:
  ./bin/forbid
  cargo clippy --workspace --all-targets -- --deny warnings
  cargo fmt --all -- --check
  cargo test --workspace
