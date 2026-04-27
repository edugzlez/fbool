# Metric definitions

This page gives the metric definitions used by the project and maps them to the
implementation in `fbool`. The bibliography at the end lists the sources behind
the standard Boolean-function metrics and the project-specific descriptors.

## Notation

Let \(f : \{0,1\}^n \to \{0,1\}\) be a Boolean function. `fbool` stores \(f\)
as a truth table with \(2^n\) entries.

For a set of variables \(S\), write \(\bar S\) for the complement. Fixing the
variables in \(S\) to a concrete assignment \(x_S\) produces a restricted
subfunction:

\[
g_{x_S}(x_{\bar S}) = f(x).
\]

The set or multiset of these restricted functions is the main object behind
the information, entanglement, entropy-based entanglement, and fragmentation
metrics.

## Entropy

Let \(X\) be uniformly distributed over all inputs. Define:

\[
p_0 = P(f(X)=0), \qquad p_1 = P(f(X)=1).
\]

The output entropy is:

\[
H(f) = -p_0 \log_2 p_0 - p_1 \log_2 p_1.
\]

Intuition:

- constant functions have entropy 0;
- balanced functions have maximal output entropy;
- entropy sees only the number of zeros and ones, not where they occur.

Background: this is the usual Shannon entropy applied to the output random
variable of a Boolean function. General Boolean-function notation and analysis
context follow standard references such as O'Donnell [ODonnell2014].

!!! warning "API naming"

    This definition is the plain output entropy \(H(f)\). It is not what
    `f.entropy()` currently computes.

    In the current API, `entropy()` is reserved for the entropy-based
    entanglement score over balanced variable partitions. This naming will be
    changed in a future version so the API distinguishes plain output entropy
    from entropy-based entanglement more explicitly.

## Information of a variable set

For a variable set \(S\) with \(|S| = k\), fix \(S\) in all \(2^k\) possible
ways and collect the restricted subfunctions. The information value is the
number of distinct restricted functions:

\[
i(S) = |\{g_{x_S} : x_S \in \{0,1\}^k\}|.
\]

Intuition:

- low \(i(S)\): many assignments to \(S\) behave the same once the other
  variables are left free;
- high \(i(S)\): \(S\) creates many genuinely different views of the function.

Usage:

=== ":simple-rust: Rust"

    ```rust
    let info = f.information(&[0, 1]);
    ```

=== ":simple-python: Python"

    ```python
    info = f.information([0, 1])
    ```

## Entanglement

The entanglement metric follows the variable-partition view introduced for
Boolean functions in the equanimity/entanglement work [Carro2023]. Its
communication-style interpretation is related to best-partitioning ideas used
in multilevel logic synthesis [Hwang1990].

It is defined over half-sized partitions:

\[
ent(f) =
\min_{\substack{S \subset \{1,\dots,n\}\\ |S|=\lfloor n/2 \rfloor}}
\left(i(S) + i(\bar S)\right).
\]

The half-size restriction avoids uninformative partitions where one side is too
small or too large.

Intuition:

- if some balanced split makes both sides simple, entanglement is low;
- if every balanced split leaves both sides with many possible subfunctions,
  the variables are strongly interdependent.

Usage:

=== ":simple-rust: Rust"

    ```rust
    let value = f.entanglement();
    let witnesses = f.entanglement_sets();
    ```

=== ":simple-python: Python"

    ```python
    value = f.entanglement()
    witnesses = f.entanglement_sets()
    ```

## Min-max entanglement

`fbool` also exposes a conservative variant:

\[
ent_{\max}(f) =
\min_S \max(i(S), i(\bar S)).
\]

It asks for the best balanced split after judging the split by its harder side,
not by the sum of both sides.

Use it when a single difficult side should dominate the score.

## Entropy-based entanglement

Instead of counting distinct restricted functions, the entropy-based variant
uses their empirical distribution.

Let \(F(S)\) be the multiset of restricted functions obtained by fixing \(S\).
If a restricted function \(g\) appears with multiplicity \(\mathrm{mul}(g)\),
define:

\[
\sigma_g = \frac{\mathrm{mul}(g)}{2^{|S|}}.
\]

The partition entropy is:

\[
E(S) = - \sum_{g \in F(S)} \sigma_g \log_2 \sigma_g.
\]

The entropy-based entanglement score is:

\[
T(f) =
\min_{\substack{S \subset \{1,\dots,n\}\\ |S|=\lfloor n/2 \rfloor}}
\left(E(S) + E(\bar S)\right).
\]

In `fbool`, `entropy()` and `entropy_sets()` use this partition-entropy view
over balanced bipartitions. The fragmentation API generalizes it across all
subset sizes.

This is a project-level extension of the same restriction/partition framework
used by entanglement [Carro2023].

## Fragmentation

Fragmentation asks the same entropy-of-restrictions question as \(E(S)\), but
organizes it by subset size.

For each \(k\):

\[
S_k(f) = \operatorname{avg}_{|S|=k} E(S).
\]

The spectrum is:

\[
(S_0(f), S_1(f), \dots, S_n(f)).
\]

Intuition:

- it shows at which subset sizes the function starts producing diverse
  restrictions;
- the peak identifies the subset size with maximal average fragmentation;
- first and second differences describe how sharply the profile changes.

Usage:

=== ":simple-rust: Rust"

    ```rust
    let spectrum = f.fragmentation_spectrum();
    let peak = f.fragmentation_peak();
    ```

=== ":simple-python: Python"

    ```python
    spectrum = f.fragmentation_spectrum()
    k_star, s_max = f.fragmentation_peak()
    ```

Fragmentation is a project-level profile built from the entropy-based
restriction distribution above.

## Spectral entropy

Let \(\widehat W_\omega(f)\) be the Walsh-Hadamard spectrum. Normalize squared
coefficients into an energy distribution:

\[
p_\omega =
\frac{\widehat W_\omega(f)^2}
{\sum_{\nu \in \{0,1\}^n} \widehat W_\nu(f)^2}.
\]

Spectral entropy is:

\[
H_{spectral}(f) =
- \sum_{\omega \in \{0,1\}^n} p_\omega \log_2 p_\omega.
\]

Intuition:

- low spectral entropy: the function is concentrated on a few linear
  components;
- high spectral entropy: spectral energy is spread across many frequencies.

Background: the Walsh-Hadamard/Fourier view of Boolean functions is standard in
cryptography and Boolean-function analysis [Carlet2010, ODonnell2014].

## Counting

Counting is a structural descriptor over the truth table. Let \(Z\) be the number
of zeros in the truth table and \(U\) the number of ones. Let \(Z_u\) and
\(U_u\) be the number of 1 bits in the binary representations of \(Z\) and
\(U\). Then:

\[
counting(f) = Z_u + U_u.
\]

Intuition: it is a compact handcrafted signal for how simply the output
cardinalities can be expressed as sums of powers of two.

Counting is a project-level descriptor. It is included because it captures a
small amount of truth-table regularity that plain output entropy discards.

Rust:

```rust
use fbool::StructuralMetrics;

let value = f.counting();
```

## Repetitiveness

For 5-variable functions, the truth table has 32 bits. The repetitiveness
descriptor splits it into eight consecutive blocks of four bits:

\[
s_f = S_1 S_2 \dots S_8.
\]

Repetitiveness is the total length covered by blocks that appear at least twice:

\[
rep(f) = \sum_{j : \exists k \ne j, S_j = S_k} |S_j|.
\]

Intuition: it measures repeated local patterns in the truth-table string.

The repetitiveness descriptor follows the earlier experimental work on hard
Boolean functions [Villarubia2021].

Rust:

```rust
use fbool::StructuralMetrics;

let value = f.repetitiveness();
```

## Non-linearity

Non-linearity is the Hamming distance from \(f\) to the closest affine function:

\[
NL(f) = \min_{g \in Affine} d_H(f,g).
\]

An affine Boolean function has the form:

\[
g(x_1,\dots,x_n) =
(a_1x_1 \oplus \dots \oplus a_nx_n) \oplus b.
\]

Intuition: high non-linearity means the function is far from XOR-like
approximations.

Background: non-linearity and affine approximation are standard tools in the
analysis of Boolean functions, especially in cryptographic settings
[Carlet2010].

## Degree

Every Boolean function has a unique multilinear polynomial over the reals:

\[
P(x) = \sum_{S \subseteq \{1,\dots,n\}} a_S \prod_{i \in S} x_i.
\]

The degree is:

\[
deg(f) = \max\{|S| : a_S \ne 0\}.
\]

Intuition: degree measures the highest-order interaction needed to express the
function.

Background: degree is a standard algebraic measure in Boolean-function analysis
[ODonnell2014].

## Influence and total influence

Let \(x^{\oplus i}\) be the input obtained by flipping bit \(i\). Define:

\[
E_i(f) = \{x : f(x) \ne f(x^{\oplus i})\}.
\]

The influence of variable \(i\) is:

\[
Inf_i(f) = \frac{|E_i(f)|}{2^n}.
\]

Total influence is:

\[
I(f) = \sum_{i=1}^n Inf_i(f).
\]

Intuition: influence is the average chance that flipping a variable flips the
output. Total influence is the normalized number of changing hypercube edges.

Background: influence and total influence are central measures in the analysis
of Boolean functions [ODonnell2014].

## Sensitivity

Local sensitivity counts how many single-bit flips change the output at one
input:

\[
s(f,x) = |\{i : f(x) \ne f(x^{\oplus i})\}|.
\]

Sensitivity is the worst case:

\[
s(f) = \max_x s(f,x).
\]

`fbool` exposes both a maximum and a mean sensitivity:

```rust
let max_s = f.max_sensitivity();
let mean_s = f.mean_sensitivity();
```

Background: sensitivity is a standard local instability measure for Boolean
functions [ODonnell2014].

## Certificate complexity

A certificate for input \(x\) is a subset of input positions \(S\) such that
every input \(y\) agreeing with \(x\) on \(S\) has the same output:

\[
(\forall i \in S: y_i = x_i) \Rightarrow f(y) = f(x).
\]

The point certificate complexity \(C(f,x)\) is the smallest such subset size.
The certificate complexity of the function is the worst case:

\[
C(f) = \max_x C(f,x).
\]

Intuition: it measures how many input bits you may need to reveal before the
output is forced.

Background: certificate complexity is a standard decision/circuit-complexity
measure [AroraBarak2009, ODonnell2014].

## Minimal gates

The default `optimal5` feature maps a 5-variable function to its NPN
representative and retrieves exact minimal gate data from the integrated
optimal5 engine.

This is not an asymptotic complexity measure. It is exact reference data for
the finite 5-input domain used in the experiments.

The data comes from Adam P. Goucher's optimal5 database [Goucher2020], which is
based on NPN equivalence classes originally computed by Knuth's BOOLCHAINS work
[Knuth2011].

## References

`[AroraBarak2009]` S. Arora and B. Barak. *Computational Complexity: A Modern
Approach*. Cambridge University Press, 2009.

`[Carlet2010]` C. Carlet, Y. Crama, and P. L. Hammer. "Boolean functions for
cryptography and error-correcting codes." 2010.

`[Carro2023]` E. Carro Garrido and J. R. Cobian Fernandez. "Analysis of Boolean
functions using equanimity and entanglement." Bachelor's thesis, Universidad
Complutense de Madrid, Facultad de Informatica, 2023.
<https://hdl.handle.net/20.500.14352/88009>

`[Goucher2020]` A. P. Goucher. "optimal5: Optimal circuits for all 5-input
1-output Boolean functions." 2020. <https://gitlab.com/apgoucher/optimal5>

`[Hwang1990]` T.-T. Hwang, R. M. Owens, and M. J. Irwin. "Exploiting
communication complexity for multilevel logic synthesis." *IEEE Transactions
on Computer-Aided Design of Integrated Circuits and Systems*, 9(10),
1017-1027, 1990.

`[Knuth2011]` D. E. Knuth. *The Art of Computer Programming*, Volume 4A.
Addison-Wesley, 2011.

`[ODonnell2014]` R. O'Donnell. *Analysis of Boolean Functions*. Cambridge
University Press, 2014.

`[Villarubia2021]` J. Villarubia Elvira. "Identificacion experimental de las
funciones booleanas que requieren circuitos extensos y aplicacion al estudio
de P vs NP." Bachelor's thesis, Universidad Complutense de Madrid, Facultad de
Informatica, 2021.
