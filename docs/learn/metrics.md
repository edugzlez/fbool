# Metrics map

This page is a field guide to the metric families exposed by `fbool`.

| Family | Rust entry point | Python entry point | Good for |
|---|---|---|---|
| Entanglement | `Entanglement` | `entanglement()` | Variable interdependence across bipartitions |
| Entropy | `Entropy` | `entropy()` | Distributional diversity of restrictions |
| Information | `WithInformation`, `SubInfos` | `information(vars)` | Local contribution of variable sets |
| Fragmentation | `Fragmentation` | `fragmentation_spectrum()` | How restriction diversity changes by subset size |
| Frontier | `Frontier` | `frontier_size()` | Boundary structure in the hypercube |
| Sensitivity | `Sensitivity` | `max_sensitivity()`, `mean_sensitivity()` | Response to single-bit flips |
| Influence | inherent methods | `influence(var)`, `total_influence()` | Average impact of variables |
| Spectral | inherent methods | `degree()`, `spectral_entropy()` | Walsh/Fourier structure |
| Structural descriptors | `StructuralMetrics` | not currently exposed | Truth-table count/block regularity |
| Certificate | `CertificateComplexity` | `certificate_complexity()` | Local evidence needed to determine output |
| optimal5 | `WithMinimalGates` | `minimal_gates()`, `npn_representant()` | Exact minimal-gate data for 5-variable functions |

## Entanglement, entropy, fragmentation

These metrics are built around restrictions of the truth table.

- `entanglement()` searches bipartitions and minimizes `I(A) + I(B)`.
- `minmax_entanglement()` minimizes the larger side, `max(I(A), I(B))`.
- `entropy()` replaces counts with Shannon entropy over restricted forms.
- `fragmentation_spectrum()` averages entropy-like restriction behavior by
  subset size.

See [Metric definitions](definitions.md) for the formal version of these
quantities.

## Sensitivity and influence

Sensitivity asks how many neighboring inputs can flip the output. Influence
averages this question per variable.

Use them when you care about local instability, variable importance, or
connections to analysis of Boolean functions.

## Spectral metrics

Spectral metrics use the Walsh-Hadamard/Fourier view.

- `degree()` measures the highest active Fourier degree.
- `spectral_entropy()` summarizes how spread out spectral mass is.
- `no_linearity()` reports distance-like non-linearity information.

## Circuit metrics

`minimal_gates()` comes from the integrated optimal5 C++ engine. It returns
`None` unless the function has exactly 5 variables.

See [Minimal gates and NPN representatives](../guide.md#minimal-gates-and-npn-representatives).

## Structural descriptors

The structural metric set also includes two compact descriptors over the
truth-table string:

- `counting()` combines the binary popcounts of the number of zeros and ones;
- `repetitiveness()` measures repeated 4-bit blocks in 5-variable truth tables.

They are exposed in Rust through `StructuralMetrics`.
