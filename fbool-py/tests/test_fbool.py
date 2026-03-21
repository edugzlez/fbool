from fbool import FBool
import pytest


def test_fbool_primality():
    f = FBool.primality(5)

    assert f.n_vars() == 5

    primes = {1, 2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31}
    for p in primes:
        assert f.eval(p)

    no_primes = set(range(2, 32)) - primes
    for n in no_primes:
        assert not f.eval(n)

    with pytest.raises(OverflowError):
        _ = f.eval(-1)

    with pytest.raises(IndexError):
        _ = f.eval(1 << 5)


def test_fbool_majority():
    f = FBool.majority(3)

    assert not f.eval(0b000)
    assert not f.eval(0b001)
    assert not f.eval(0b010)
    assert not f.eval(0b100)
    assert f.eval(0b111)
    assert f.eval(0b011)
    assert f.eval(0b101)
    assert f.eval(0b110)

    with pytest.raises(OverflowError):
        _ = f.eval(-1)

    with pytest.raises(IndexError):
        _ = f.eval(1 << 3)
