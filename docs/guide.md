# Rust and Python guide

This guide shows the same workflow from both public APIs: the Rust crate and the
Python extension module. Pick the language tab you are using; the choice is
remembered across the documentation.

## Installation

=== ":simple-rust: Rust"

    Add the crate to `Cargo.toml`:

    ```toml
    [dependencies]
    fbool = "0.2"
    ```

    For local workspace development:

    ```toml
    [dependencies]
    fbool = { path = "../fbool" }
    ```

    The default feature set includes `optimal5`, so default builds need a C++
    compiler. For a pure-Rust build:

    ```toml
    [dependencies]
    fbool = { version = "0.1", default-features = false, features = ["entanglement", "frontier"] }
    ```

=== ":simple-python: Python"

    `fbool` is published on PyPI, so the normal installation path is:

    ```bash
    uv add fbool
    ```

    Or, with plain `pip`:

    ```bash
    pip install fbool
    ```

    The Python package requires Python 3.11 or newer.

## Create a Boolean function

=== ":simple-rust: Rust"

    ```rust
    use fbool::fvalue::FValue;

    let from_table = FValue::new(vec![false, true, true, false]);
    let from_number = FValue::from_usize(0b0110, 2);
    let majority = FValue::majority(5);
    let parity = FValue::parity(5);
    let prime = FValue::primality(5);
    ```

=== ":simple-python: Python"

    ```python
    from fbool import FBool

    from_table = FBool([False, True, True, False])
    from_number = FBool.from_number(0b0110, 2)
    majority = FBool.majority(5)
    parity = FBool.parity(5)
    prime = FBool.primality(5)
    ```

## Inspect the function

=== ":simple-rust: Rust"

    ```rust
    let f = FValue::majority(4);

    let n_vars = f.n_vars();
    let table = f.repr();
    let first_value = f.get(0);
    ```

=== ":simple-python: Python"

    ```python
    f = FBool.majority(4)

    n_vars = f.n_vars()
    table = f.repr()
    first_value = f.eval(0)
    ```

## Compute partition metrics

=== ":simple-rust: Rust"

    Trait imports make the metric methods available:

    ```rust
    use fbool::entanglement::{Entanglement, Entropy, WithInformation};
    use fbool::fvalue::FValue;

    let f = FValue::majority(4);

    let info = f.information(&[0, 1]);
    let entanglement = f.entanglement();
    let minmax = f.minmax_entanglement();
    let entropy = f.entropy();
    ```

=== ":simple-python: Python"

    Python exposes the methods directly on `FBool`:

    ```python
    from fbool import FBool

    f = FBool.majority(4)

    info = f.information([0, 1])
    entanglement = f.entanglement()
    entropy = f.entropy()
    ```

## Work with witness sets

=== ":simple-rust: Rust"

    ```rust
    for item in f.entanglement_sets() {
        println!("{:?} | {:?}: {}", item.set1, item.set2, item.entanglement);
    }

    for item in f.entropy_sets() {
        println!("{:?} | {:?}: {}", item.set1, item.set2, item.entropy);
    }
    ```

=== ":simple-python: Python"

    ```python
    for item in f.entanglement_sets():
        print(item.set1, item.set2, item.entanglement)

    for item in f.entropy_sets():
        print(item.set1, item.set2, item.entropy)
    ```

## Fragmentation profile

=== ":simple-rust: Rust"

    ```rust
    use fbool::Fragmentation;

    let spectrum = f.fragmentation_spectrum();
    let peak = f.fragmentation_peak();
    let delta = f.fragmentation_delta();
    ```

=== ":simple-python: Python"

    ```python
    spectrum = f.fragmentation_spectrum()
    k_star, s_max = f.fragmentation_peak()
    delta = f.fragmentation_delta()
    ```

## Sensitivity, influence, and spectral metrics

=== ":simple-rust: Rust"

    ```rust
    use fbool::sensitivity::Sensitivity;

    let max_sensitivity = f.max_sensitivity();
    let mean_sensitivity = f.mean_sensitivity();
    let influence_0 = f.influence(0);
    let total_influence = f.total_influence();
    let degree = f.degree();
    let spectral_entropy = f.spectral_entropy();
    let non_linearity = f.no_linearity();
    ```

=== ":simple-python: Python"

    ```python
    max_sensitivity = f.max_sensitivity()
    mean_sensitivity = f.mean_sensitivity()
    influence_0 = f.influence(0)
    total_influence = f.total_influence()
    degree = f.degree()
    spectral_entropy = f.spectral_entropy()
    non_linearity = f.no_linearity()
    ```

## Frontier and certificates

=== ":simple-rust: Rust"

    ```rust
    use fbool::certificate::CertificateComplexity;
    use fbool::frontier::Frontier;

    let frontier_graph = f.frontier_graph();
    let certificate = f.certificate_complexity();
    ```

=== ":simple-python: Python"

    ```python
    frontier_size = f.frontier_size()
    max_frontier_size = f.max_frontier_size()
    certificate = f.certificate_complexity()
    ```

## Structural descriptors

=== ":simple-rust: Rust"

    `counting()` and `repetitiveness()` are Rust-only for now:

    ```rust
    use fbool::StructuralMetrics;

    let f = FValue::majority(5);
    let count_signal = f.counting();
    let repeated_blocks = f.repetitiveness();
    ```

=== ":simple-python: Python"

    The Python binding does not expose `counting()` or `repetitiveness()` yet.
    Use the Rust API when you need those descriptors.

## Minimal gates and NPN representatives

=== ":simple-rust: Rust"

    ```rust
    use fbool::fvalue::FValue;
    use fbool::optimal5::WithMinimalGates;

    let f = FValue::majority(5);

    let gates = f.minimal_gates();
    let npn = f.npn_representant();
    ```

=== ":simple-python: Python"

    ```python
    from fbool import FBool

    f = FBool.majority(5)

    gates = f.minimal_gates()
    npn = f.npn_representant()
    ```

The optimal5 engine supports exactly 5-variable Boolean functions. Calls return
`None` outside that domain.

## Tables and binary serialization

=== ":simple-rust: Rust"

    ```rust
    let restricted_table = f.table(&[0, 2]);

    let raw = bincode::encode_to_vec(&f, bincode::config::standard())?;
    let (same, _): (FValue<bool>, _) =
        bincode::decode_from_slice(&raw, bincode::config::standard())?;
    ```

=== ":simple-python: Python"

    ```python
    table = f.table([0, 2])

    raw = f.encode()
    same = FBool.decode(raw)
    ```

Python returns `table()` as a NumPy array. The binary encoding is shared with
the CLI `bin` input format.
