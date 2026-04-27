# Experiments

The `fbool-experiments` crate contains research binaries that sit next to the
library and CLI.

```bash
cargo run -p fbool-experiments --bin npn-create -- --help
cargo run -p fbool-experiments --bin compute-metrics -- --help
cargo run -p fbool-experiments --bin bayes-optimal -- --help
cargo run -p fbool-experiments --bin orthogonality -- --help
```

## Binaries

| Binary | Role |
|---|---|
| `npn-create` | NPN-class related dataset creation |
| `compute-metrics` | Batch metric computation |
| `bayes-optimal` | Bayesian/optimality experiment entry point |
| `orthogonality` | Orthogonality experiment entry point |

These tools are not the published Rust crate. They are workspace members for
research workflows.
