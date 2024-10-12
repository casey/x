watch +args='test':
  cargo watch --clear --exec '{{args}}'

outdated:
  cargo outdated --root-deps-only --workspace
