---
hide:
  - navigation
---

# fbool

<div class="hero" markdown>

`fbool` is a compact toolkit for studying Boolean functions from Rust and
Python. It helps you move from a truth table to structure: partitions,
entanglement, entropy, sensitivity, spectra, frontier graphs, certificates, and
exact 5-variable circuit data.

</div>

<div class="quick-grid" markdown>

<div class="quick-card" markdown>
### Learn the model
Start with truth tables, restrictions, partitions, and the way `fbool` turns
them into metrics.

[Open the mental model](learn/mental-model.md)
</div>

<div class="quick-card" markdown>
### Use the APIs
Use the Rust crate or the Python binding with the same conceptual workflow.

[Open the guide](guide.md)
</div>

<div class="quick-card" markdown>
### Compare Rust and Python
Switch between matching examples with language tabs.

[Rust/Python guide](guide.md)
</div>

<div class="quick-card" markdown>
### Run tools
Use the CLI for repeatable metric runs and the experiment binaries for research
pipelines.

[CLI guide](tools/cli.md)
</div>

</div>

## The short version

=== ":simple-rust: Rust"

    ```rust
    use fbool::entanglement::{Entanglement, Entropy};
    use fbool::fvalue::FValue;

    let f = FValue::majority(4);

    println!("{}", f.entanglement());
    println!("{}", f.minmax_entanglement());
    println!("{}", f.entropy());
    ```

=== ":simple-python: Python"

    ```python
    from fbool import FBool

    f = FBool.majority(4)

    print(f.entanglement())
    print(f.entropy())
    print(f.spectral_entropy())
    ```

## What lives where

```text
fbool/                 Rust library crate published to crates.io
fbool-cli/             command-line interface
fbool-py/              Python extension module
fbool-experiments/     research binaries and analysis workflows
```

The old standalone `optimal5` crate is now integrated into `fbool` and enabled
by default.

## Choose a path

=== "I want the concepts"

    Read [Mental model](learn/mental-model.md), then use the
    [Metrics map](learn/metrics.md) as a guide to what each metric measures.

=== "I want the APIs"

    Open the [guide](guide.md) and switch between :simple-rust: Rust and
    :simple-python: Python examples as needed.

=== "I want experiments"

    Use `fbool-cli` for focused metric runs and `fbool-experiments` for larger
    research jobs.
