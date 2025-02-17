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

foo:
  cargo run -- --track '/Users/rodarmor/Music/Music/Media.localized/Music/seagaia/Anodyne/49 Old Generic Boss _The Street_ (Extra).mp3'

bar:
  cargo run -- --track '/Users/rodarmor/Music/Music/Media.localized/Music/t.A.T.u_/200 km_h in the Wrong Lane/02 All The Things She Said.mp3'
