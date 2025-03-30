set positional-arguments

watch +args='lcheck':
  cargo watch --clear --exec '{{args}}'

run *args:
  #!/usr/bin/env bash
  set -euo pipefail
  cargo build --release
  ./target/release/x "$@" 2> >(grep -Ev 'IMKClient|IMKInputSession' >&2)

forbid:
  ./bin/forbid

ci: forbid
  cargo lclippy --workspace --all-targets -- --deny warnings
  cargo fmt --all -- --check
  cargo ltest --workspace

clippy: (watch 'lclippy --all-targets -- --deny warnings')

outdated:
  cargo outdated --root-deps-only --workspace

unused:
  cargo +nightly udeps --workspace

doc:
  cargo doc --workspace --open

hello:
  cargo run --release -- --song 'old generic boss' --program hello

maria:
  cargo run --release -- --song 'total 4/13 maria'

nobrain:
  cargo run --release -- --song 'no brain$'
