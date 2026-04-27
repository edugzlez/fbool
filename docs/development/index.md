# Development

## Local checks

```bash
cargo check -p fbool
cargo check -p fbool --no-default-features --features entanglement,frontier
cargo test --workspace
cargo clippy --all-targets --all-features
```

## Documentation

Serve locally:

```bash
uv run --with-requirements docs/requirements.txt mkdocs serve
```

Build strictly:

```bash
uv run --with-requirements docs/requirements.txt mkdocs build --strict
```

## Packaging the Rust crate

```bash
cargo package -p fbool --allow-dirty
```

The publishable crate is `fbool`. The CLI, Python extension, and experiments are
workspace consumers around it.
