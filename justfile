# todo:
# - get rid of log line
# - mode where you fit in instead of fill
# - toggle repeat

watch +args='ltest':
  cargo watch --clear --exec '{{args}}'

run:
  #!/usr/bin/env bash
  set -euo pipefail
  cargo build
  ./target/debug/x 2> >(grep -Ev 'IMKClient|IMKInputSession' >&2)

ci:
  ./bin/forbid
  cargo lclippy --workspace --all-targets -- --deny warnings
  cargo fmt --all -- --check
  cargo ltest --workspace

clippy: (watch 'lclippy --all-targets -- --deny warnings')

outdated:
  cargo outdated --root-deps-only --workspace

unused:
  cargo +nightly udeps --workspace
