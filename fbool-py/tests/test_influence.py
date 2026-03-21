import pytest

from fbool import FBool


@pytest.mark.parametrize(
    "fun,expected",
    [
        (FBool.primality(10), 2.69140625),
        (FBool.majority(7), 2.1875),
        (FBool.parity(10), 10.0),
    ],
)
def test_total_influences(fun: FBool, expected: int):
    assert abs(fun.total_influence() - expected) < 1e-6


@pytest.mark.parametrize(
    "fun,idx,expected",
    [
        (FBool.primality(10), 3, 0.255859375),
        (FBool.majority(7), 6, 0.3125),
        (FBool.parity(10), 8, 1.0),
    ],
)
def test_idxs_influences(fun: FBool, idx: int, expected: int):
    assert abs(fun.influence(idx) - expected) < 1e-6
