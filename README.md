# fbool

> ⚠️ **Stability Notice**  
> This library is currently under active development and **its public API is not stable**.  
> Any version **`< 1.0.0` may introduce breaking changes** without a major version bump.  
> If you depend on specific behavior or interfaces, consider pinning to an exact commit or tag.

A Rust library for analyzing **Boolean functions**, with a focus on **entanglement**, information-theoretic measures, and circuit complexity. Includes Python bindings and a command-line interface.

This library was developed to support research on entanglement measures for Boolean functions. If you use it in your work, please cite the associated paper (see [Citation](#citation)).

---

## Overview

`fbool` provides data structures and algorithms for:

- Representing Boolean functions compactly (truth-table encoding)
- Computing **entanglement** and **min-max entanglement** via bipartitions of variables
- Computing **Shannon entropy** of variable partitions
- Measuring **influence**, **sensitivity**, **spectral** properties (Walsh-Hadamard transform), and **frontier** structure
- Estimating **certificate complexity**
- Finding the **minimum gate count** for functions of up to 5 variables (via an integrated C++ optimizer)

The workspace also ships a **Python package** (`fbool`) built with [PyO3](https://pyo3.rs) and [maturin](https://maturin.rs), and a **CLI tool** (`fbool-cli`).

---

## Repository Structure

```
fbool/                 # Core Rust library
fbool-cli/             # Command-line interface
fbool-py/              # Python bindings (PyO3 + maturin)
clique_solver/         # CLIQUE-problem solver used internally
optimal5/              # Minimal-gate optimizer for 5-variable functions (Rust + C++)
```

---

## Theoretical Background

### Boolean Functions

A Boolean function `f : {0,1}^n -> {0,1}` is represented as a truth table: a vector of `2^n` Boolean values indexed by the binary encoding of each input.

### Entanglement

Given a bipartition `(A, B)` of the `n` input variables, the **information** `I(S)` of a set `S` counts the number of distinct sub-functions obtained by fixing all variables outside `S`. The two entanglement measures defined in this work are:

```
Entanglement(f)        = min_{A,B}  [ I(A) + I(B) ]
MinMax-Entanglement(f) = min_{A,B}  max( I(A), I(B) )
```

Both minimize over all non-trivial bipartitions of `{0, ..., n-1}`.

### Entropy

For a bipartition `(A, B)`, the entropy is computed as the sum of the Shannon entropies of the empirical distributions induced by each partition. The minimum over all bipartitions gives the entropy measure of the function.

---

## Rust Usage

Add `fbool` to your `Cargo.toml`:

```toml
[dependencies]
fbool = { git = "https://github.com/edugzlez/fbool" }
```

### Basic Example

```rust
use fbool::fvalue::FValue;
use fbool::entanglement::{Entanglement, Entropy};

fn main() {
    // Majority function on 4 variables
    let f = FValue::majority(4);

    println!("Entanglement:        {}", f.entanglement());
    println!("MinMax entanglement: {}", f.minmax_entanglement());
    println!("Entropy:             {}", f.entropy());

    // Inspect all bipartitions
    for es in f.entanglement_sets() {
        println!("  {:?} | {:?}  ->  {}", es.set1, es.set2, es.entanglement);
    }
}
```

### Available Boolean Functions

`FValue` provides constructors for many classical families:

| Constructor              | Description              |
| ------------------------ | ------------------------ |
| `FValue::majority(n)`    | Majority function        |
| `FValue::parity(n)`      | Parity (XOR) function    |
| `FValue::primality(n)`   | Primality test           |
| `FValue::zero_search(n)` | Zero-search function     |
| `FValue::sum(n)`         | Arithmetic sum           |
| `FValue::product(n)`     | Arithmetic product       |
| `FValue::gcd(n)`         | GCD function             |
| `FValue::clique(n)`      | Clique decision function |

### Metrics

| Trait                   | Methods                                                                                        |
| ----------------------- | ---------------------------------------------------------------------------------------------- |
| `Entanglement`          | `entanglement()`, `entanglement_sets()`, `minmax_entanglement()`, `minmax_entanglement_sets()` |
| `Entropy`               | `entropy()`, `entropy_sets()`                                                                  |
| `Sensitivity`           | `sensitivity()`                                                                                |
| `Influence`             | `influence()`                                                                                  |
| `CertificateComplexity` | `certificate_complexity()`                                                                     |
| `Frontier`              | `frontier_graph()`                                                                             |
| `WithMinimalGates`      | `minimal_gates()` (5-variable functions only)                                                  |

---

## Python Usage

### Installation

Build and install locally with [maturin](https://maturin.rs):

```bash
cd fbool-py
pip install maturin
maturin develop
```

Or with [uv](https://docs.astral.sh/uv/):

```bash
cd fbool-py
uv sync
uv run maturin develop
```

### Example

```python
import fbool

# Construct a Boolean function (primality on 4 variables)
f = fbool.FBool.primality(4)

# Entanglement
print("Entanglement:", f.entanglement())
print("MinMax entanglement:", f.minmax_entanglement())

# All bipartitions
for es in f.entanglement_sets():
    print(f"  {es.set1} | {es.set2}  ->  {es.entanglement}")

# Entropy
print("Entropy:", f.entropy())

# Truth table as NumPy array
tt = f.truth_table()
print("Truth table shape:", tt.shape)
```

### FBool API

| Method                              | Description                         |
| ----------------------------------- | ----------------------------------- |
| `FBool(repr)`                       | Construct from a truth-table list   |
| `FBool.from_number(n, num_vars)`    | Construct from integer encoding     |
| `FBool.majority(n)`                 | Majority function                   |
| `FBool.parity(n)`                   | Parity function                     |
| `FBool.primality(n)`                | Primality function                  |
| `f.entanglement()`                  | Minimum entanglement value          |
| `f.entanglement_sets()`             | All bipartition entanglement values |
| `f.minmax_entanglement()`           | Minimum max-entanglement value      |
| `f.entropy()`                       | Minimum entropy value               |
| `f.entropy_sets()`                  | All bipartition entropy values      |
| `f.sensitivity()`                   | Sensitivity measure                 |
| `f.truth_table()`                   | Truth table as `numpy.ndarray`      |
| `f.save(path)` / `FBool.load(path)` | Binary serialization                |

---

## CLI Usage

Build and run:

```bash
cargo build --release --bin fbool-cli
./target/release/fbool-cli --help
```

### Examples

```bash
# Entanglement of the majority function with 4 variables
fbool-cli entanglement majority -n 4

# All bipartitions, sorted by entanglement value
fbool-cli entanglement parity -n 5 --sets --sorted

# Entropy
fbool-cli entropy majority -n 4 --sets

# Per-variable information
fbool-cli subinfo primality -n 5

# Serialize a function to binary
fbool-cli encode primality -n 5 -o primality_5.bin
```

---

## Building

**Requirements:** Rust 1.70+, a GCC++ compiler (for `optimal5`), Python 3.11+ (for Python bindings).

```bash
# Build the full workspace
cargo build --release

# Run all tests
cargo test --all

# Run lints
cargo clippy --all-targets --all-features
```

---

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.

---

## Citation

If you use this software in academic work, please cite:

```bibtex
@software{fbool,
  author  = {Eduardo Gonz\'{a}lez-Vaquero and Ricardo Maurizio Paul},
  title   = {fbool: A Rust library for Boolean functions analysis},
  year    = {2026},
  url     = {https://github.com/edugzlez/fbool}
}
```

The associated paper citation will be added upon publication.
