# CLI

The CLI crate is `fbool-cli`.

```bash
cargo build --release -p fbool-cli
```

Run help:

```bash
target/release/fbool-cli --help
```

On Windows:

```powershell
target\release\fbool-cli.exe --help
```

## Commands

| Command | Purpose |
|---|---|
| `encode` | Save a generated function to a binary file |
| `entanglement` | Compute entanglement or min-max entanglement |
| `entropy` | Compute entropy metrics |
| `equanimity-importance` | Compute equanimity importance |
| `sub-info` | Print sub-information values |
| `debug` | Run debug operations |

## Examples

```bash
fbool-cli entanglement majority --n-vars 4
fbool-cli entanglement --sets --sorted --head 10 parity --n-vars 5
fbool-cli entanglement --minmax majority --n-vars 4
fbool-cli entropy --sets majority --n-vars 4
fbool-cli sub-info primality --n-vars 5
```

Encode and reload:

```bash
fbool-cli encode --output-path primality_5.bin primality --n-vars 5
fbool-cli entanglement bin --path primality_5.bin
```

## Function subcommands

Available function subcommands include:

```text
majority, parity, eq, ordered, multiply, sum, max, gcd,
primality, sum-is-prime, coprimes, constant, usize-constant,
raw, bin, find-zero, meta
```

Use command-specific help for exact arguments:

```bash
fbool-cli entanglement majority --help
fbool-cli encode --help
```
