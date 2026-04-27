import pytest

from fbool import FBool


@pytest.mark.parametrize(
    "fun,expected",
    [(FBool.primality(10), 173), (FBool.majority(7), 44), (FBool.parity(10), 0)],
)
def test_non_linearity(fun: FBool, expected: int):
    assert fun.no_linearity() == expected
