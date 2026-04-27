# Mental model

## Truth table first

A Boolean function with `n` variables is represented by `2^n` outputs. In Rust,
that is `FValue<bool>`. In Python, it is `FBool`.

```text
input index:  0  1  2  3  4  5  6  7
truth table:  0  1  1  0  1  0  0  1
```

The index encodes the input assignment. Metrics then inspect how the table
changes under restrictions, partitions, transforms, and graph constructions.

## Restrictions and partitions

Many core metrics look at what remains when some variables are fixed.

```mermaid
flowchart LR
    F["Boolean function f"] --> S["choose variables S"]
    S --> R["fix variables outside S"]
    R --> U["count distinct restricted functions"]
    U --> I["information I(S)"]
```

Entanglement uses this idea across balanced bipartitions:

```text
Entanglement(f)        = min over S|Sbar of i(S) + i(Sbar)
MinMax-Entanglement(f) = min over S|Sbar of max(i(S), i(Sbar))
```

Entropy-based entanglement uses the empirical distribution of restricted
subfunctions instead of only counting distinct forms.

## The practical view

Use this rule of thumb:

<span class="metric-pill">entanglement</span>
<span class="metric-pill">entropy</span>
<span class="metric-pill">fragmentation</span>

Ask how variables split the function.

<span class="metric-pill">sensitivity</span>
<span class="metric-pill">influence</span>

Ask how output changes under input flips.

<span class="metric-pill">spectral entropy</span>
<span class="metric-pill">degree</span>
<span class="metric-pill">non-linearity</span>

Ask what the Walsh/Fourier view says.

<span class="metric-pill">frontier</span>
<span class="metric-pill">certificate complexity</span>
<span class="metric-pill">minimal gates</span>
<span class="metric-pill">counting</span>
<span class="metric-pill">repetitiveness</span>

Ask how hard the function looks from graph, decision, or circuit perspectives.

For formulas and references, continue with [Metric definitions](definitions.md).

## Size matters

Most exact metrics grow quickly because the truth table has `2^n` rows and many
metrics inspect subsets or bipartitions. Use small `n` first, then scale with
care.

!!! tip
    For exact `minimal_gates()`, the integrated optimal5 engine is enabled by
    default and only supports 5-variable Boolean functions, so the truth table
    must have length 32.
