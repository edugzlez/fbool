# fbool for Python

Python bindings for `fbool`, a toolkit for analysing Boolean functions through
truth tables, partitions, entanglement metrics, sensitivity, spectral
properties, certificates, and exact 5-variable circuit data.

```python
from fbool import FBool

f = FBool.majority(5)

print(f.entanglement())
print(f.spectral_entropy())
print(f.minimal_gates())
```

The package requires Python 3.11 or newer and is backed by the Rust `fbool`
crate.

Install from PyPI:

```bash
uv add fbool
pip install fbool
```

Documentation: <https://edugzlez.github.io/fbool/>
