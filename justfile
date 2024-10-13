watch +args='ltest':
  cargo watch --clear --exec '{{args}}'

outdated:
  cargo outdated --root-deps-only --workspace

unused:
  cargo +nightly udeps --workspace

ci:
  ./bin/forbid
  cargo lclippy --workspace --all-targets -- --deny warnings
  cargo fmt --all -- --check
  cargo ltest --workspace

clippy: (watch 'lclippy --all-targets -- --deny warnings')
