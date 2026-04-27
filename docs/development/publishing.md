# Publishing

Versioning is managed with Commitizen from the repository root.

```bash
uv tool install commitizen
cz bump --changelog --yes
```

Tags matching `v*` trigger the publish workflow.

## Rust

The workflow publishes only the Rust library crate:

```bash
cargo publish -p fbool
```

The old `optimal5` crate is no longer published separately because its code now
lives behind the default `optimal5` feature in `fbool`.

## Python

The workflow builds wheels from `fbool-py` with maturin for Linux, Windows, and
macOS ARM, then publishes them to PyPI as `fbool`.

## Documentation

Documentation is published automatically to GitHub Pages by the `Docs` workflow
when documentation sources change on `main` or `master`. The workflow builds the
site with:

```bash
uv run --with-requirements docs/requirements.txt mkdocs build --strict
```

It can also be run manually from the GitHub Actions tab.

## Before tagging

Run:

```bash
cargo test --workspace
cargo package -p fbool --allow-dirty
uv run --with-requirements docs/requirements.txt mkdocs build --strict
```
