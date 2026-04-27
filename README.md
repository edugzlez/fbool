# fbool

`fbool` is a Rust workspace for analysing Boolean functions, with support for
entanglement-style measures, information-theoretic metrics, spectral and
structural properties, certificate complexity, frontier graphs, and exact
minimal-gate data for functions of up to 5 variables.

The workspace contains the publishable Rust library crate, a CLI, Python
bindings built with PyO3/maturin, and experiment binaries used by the research
workflow.

## Workspace

```text
fbool/                 # Publishable Rust library crate
fbool/src/metrics/     # Entanglement, entropy, fragmentation, frontier, spectral, sensitivity, influence
fbool/src/optimal5.rs  # Default Rust API for the integrated optimal5 engine
fbool/cpp/             # C++ sources from Adam P. Goucher's optimal5 engine
fbool/knuthies.dat     # Lookup data used by the optimal5 feature
fbool-cli/             # Command-line interface
fbool-py/              # Python extension module named fbool
fbool-experiments/     # Research/analysis binaries
experiments/           # Experiment assets and scripts
clique_solver/         # Standalone clique-solver code retained in the repository
```

The old standalone `optimal5` crate has been folded into `fbool` as the default
`optimal5` feature. There is no separate workspace member for it anymore.

## Rust Crate

Add the library from crates.io once published:

```toml
[dependencies]
fbool = "0.1"
```

The minimal-gate API backed by the integrated C++ engine is enabled by default.
If you want a pure-Rust build without `optimal5`, disable default
features and opt back into the Rust-only metric families you need:

```toml
[dependencies]
fbool = { version = "0.1", default-features = false, features = ["entanglement", "frontier"] }
```

For local development inside this repository:

```toml
[dependencies]
fbool = { path = "../fbool" }
```

### Features

| Feature | Enabled by default | Description |
|---|---:|---|
| `entanglement` | yes | Entanglement, entropy, sub-information, equanimity importance, fragmentation |
| `frontier` | yes | Frontier graph metrics via `petgraph` |
| `optimal5` | yes | C++ optimal5 engine for exact minimal gates up to 5 variables |
| `fmatrix` | no | Experimental multi-output function helpers |
| `clique` | no | Clique-based Boolean function constructor |

### Basic Example

```rust
use fbool::entanglement::{Entanglement, Entropy};
use fbool::fvalue::FValue;

fn main() {
    let f = FValue::majority(4);

    println!("entanglement: {}", f.entanglement());
    println!("minmax entanglement: {}", f.minmax_entanglement());
    println!("entropy: {}", f.entropy());

    for set in f.entanglement_sets() {
        println!("{:?} | {:?} -> {}", set.set1, set.set2, set.entanglement);
    }
}
```

With the default `optimal5` feature:

```rust
use fbool::fvalue::FValue;
use fbool::optimal5::WithMinimalGates;

fn main() {
    let f = FValue::majority(5);
    println!("minimal gates: {:?}", f.minimal_gates());
}
```

### Function Constructors

`FValue<bool>` includes:

| Constructor | Description |
|---|---|
| `majority(n)` | Majority function |
| `parity(n)` | Parity function |
| `primality(n)` | Primality predicate over the truth-table index |
| `equality(n)` | Equality of two `n`-bit inputs |
| `ordered(n)` | Ordered comparison of two `n`-bit inputs |
| `coprimes(n)` | Coprimality of two `n`-bit inputs |
| `sum_is_prime(n)` | Primality of the sum of two `n`-bit inputs |
| `product_is_multiple_of(n, multiple)` | Divisibility predicate |
| `find_zero(vector_size, element_size)` | Search for a zero element in an encoded vector |
| `constant(n, value)` | Constant Boolean function |
| `random(n)` | Random Boolean function |
| `from_usize(fun, n)` | Decode a Boolean truth table from an integer |
| `clique(n)` | Clique predicate, behind the `clique` feature |

`FValue<usize>` includes arithmetic-valued constructors such as `sum`, `product`,
`max`, `gcd`, `multiply`, `sum_some`, and `constant`.

### Metrics

The public API exposes metrics through modules under `fbool::metrics` and
through top-level compatibility re-exports:

| Area | Examples |
|---|---|
| Entanglement and entropy | `entanglement()`, `minmax_entanglement()`, `entropy()`, `entanglement_sets()`, `entropy_sets()` |
| Information | `information(vars)`, `sub_infos()` |
| Fragmentation | `fragmentation_coefficient()`, `fragmentation_spectrum()`, `fragmentation_profile()`, `fragmentation_peak()` |
| Frontier | `frontier_graph()` |
| Sensitivity and influence | `max_sensitivity()`, `mean_sensitivity()`, `influence(var)`, `total_influence()` |
| Spectral analysis | `walsh_coeficients()`, `fourier_coeficients()`, `degree()`, `spectral_entropy()`, `no_linearity()` |
| Certificate complexity | `certificate_complexity()` |
| Optimal 5-variable circuits | `minimal_gates()` and `npn_representant()` with the `optimal5` feature |

## Python Package

The Python package is published on PyPI as `fbool` and exports a module named
`fbool`. It requires Python 3.11 or newer.

Install it with `uv`:

```bash
uv add fbool
```

Or with `pip`:

```bash
pip install fbool
```

For local development inside this repository, build the extension from
`fbool-py` with maturin.

### Python Example

```python
from fbool import FBool

f = FBool.primality(4)

print("n vars:", f.n_vars())
print("entanglement:", f.entanglement())
print("min gates:", f.minimal_gates())
print("spectral entropy:", f.spectral_entropy())

for item in f.entanglement_sets():
    print(item.set1, item.set2, item.entanglement)
```

### Python API Highlights

`FBool` supports construction from a truth table or integer encoding:

| Method | Description |
|---|---|
| `FBool(repr)` | Construct from a Boolean truth-table list |
| `FBool.from_number(number, num_vars)` | Construct from integer truth-table encoding |
| `FBool.majority(n)`, `FBool.parity(n)`, `FBool.primality(n)` | Standard Boolean families |
| `FBool.coprimes(n)`, `FBool.sum_is_prime(n)`, `FBool.clique(n)` | Additional constructors |
| `f.repr()`, `f.eval(i)`, `f.size()`, `f.n_vars()` | Basic inspection |
| `f.encode()`, `FBool.decode(raw)` | Binary serialization |
| `f.table(vars)` | NumPy table induced by fixing variables |
| `f.npn_representant()` | NPN representative for 5-variable functions when available |

The Python bindings also expose entanglement, entropy, fragmentation, frontier,
sensitivity, spectral, influence, certificate, and optimal5 metrics.

## CLI

Build the CLI:

```bash
cargo build --release -p fbool-cli
```

Run help:

```bash
./target/release/fbool-cli --help
```

On Windows, the binary is `target\release\fbool-cli.exe`.

### CLI Examples

```bash
# Entanglement of majority on 4 variables
fbool-cli entanglement majority --n-vars 4

# All bipartitions, sorted, keeping the first 10 rows
fbool-cli entanglement --sets --sorted --head 10 parity --n-vars 5

# Min-max entanglement
fbool-cli entanglement --minmax majority --n-vars 4

# Entropy sets
fbool-cli entropy --sets majority --n-vars 4

# Per-subset information
fbool-cli sub-info primality --n-vars 5

# Encode and load a function
fbool-cli encode --output-path primality_5.bin primality --n-vars 5
fbool-cli entanglement bin --path primality_5.bin
```

Available function subcommands include `majority`, `parity`, `eq`, `ordered`,
`multiply`, `sum`, `max`, `gcd`, `primality`, `sum-is-prime`, `coprimes`,
`constant`, `usize-constant`, `raw`, `bin`, `find-zero`, and `meta`.

## Experiments

The `fbool-experiments` crate contains research binaries:

```bash
cargo run -p fbool-experiments --bin npn-create -- --help
cargo run -p fbool-experiments --bin compute-metrics -- --help
cargo run -p fbool-experiments --bin bayes-optimal -- --help
cargo run -p fbool-experiments --bin orthogonality -- --help
```

These tools are part of the repository workflow, but the publishable Rust crate
is `fbool`.

## Documentation

The MkDocs Material documentation lives in `docs/` and is configured by
`mkdocs.yml`.

```bash
uv run --with-requirements docs/requirements.txt mkdocs serve
```

For CI-style validation:

```bash
uv run --with-requirements docs/requirements.txt mkdocs build --strict
```

## Building And Testing

Requirements:

- Rust stable
- A C++ compiler for default `fbool` builds, because `optimal5` is enabled by default
- Python 3.11+ for the Python package
- `uv` is optional but recommended for Python development

Common commands:

```bash
cargo check -p fbool
cargo check -p fbool --no-default-features --features entanglement,frontier
cargo test --workspace
cargo clippy --all-targets --all-features
cargo package -p fbool --allow-dirty
```

## Releases

Versioning is managed from the repository root with Commitizen via `.cz.toml`.

```bash
uv tool install commitizen
cz bump --changelog --yes
```

Publishing is driven by GitHub Actions on `v*` tags:

- `cargo publish -p fbool` publishes the Rust crate to crates.io.
- maturin builds Python wheels for Linux, Windows, and macOS ARM.
- the Python wheels are published to PyPI as `fbool`.

## Citation

If you use this software in academic work, please cite:

```bibtex
@software{fbool,
  author  = {Eduardo Gonz\'{a}lez and Ricardo Maurizio Paul},
  title   = {fbool: A Rust library for Boolean function entanglement analysis},
  year    = {2025},
  url     = {https://github.com/edugzlez/fbool}
}
```

The associated paper citation will be added upon publication.

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE).

The integrated `optimal5` C++ engine is derived from Adam P. Goucher's
[optimal5](https://gitlab.com/apgoucher/optimal5), also used under the MIT
License.

## Authors

- Eduardo Gonzalez-Vaquero ([edugzlez](https://github.com/edugzlez))
- Ricardo Maurizio Paul
