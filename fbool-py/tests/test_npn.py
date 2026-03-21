import pytest

from fbool import FBool


@pytest.mark.parametrize(
    "fun", [FBool.primality(5), FBool.majority(5), FBool.parity(5)]
)
def test_fbool_npn_entanglement(fun: FBool):
    fun_p = fun.npn_representant()
    assert fun_p is not None, "PNP representant is None"

    fun_p2 = fun_p.npn_representant()
    assert fun_p2 is not None, "PNP representant is None"

    assert fun.entanglement() == fun_p.entanglement(), (
        f"Entanglement is not equal: {fun.entanglement()} != {fun_p.entanglement()}"
    )
    assert fun.minimal_gates() == fun_p.minimal_gates(), (
        f"Minimal gates are not equal: {fun.minimal_gates()} != {fun_p.minimal_gates()}"
    )
    assert fun.max_sensitivity() == fun_p.max_sensitivity(), (
        f"Max sensitivity is not equal: {fun.max_sensitivity()} != {fun_p.max_sensitivity()}"
    )
    assert fun.equanimity_importance() == fun_p.equanimity_importance(), (
        f"Equanimity importance is not equal: {fun.equanimity_importance()} != {fun_p.equanimity_importance()}"
    )
    assert fun_p.repr() == fun_p2.repr(), (
        f"Representants are not equal: {fun_p.repr()} != {fun_p2.repr()}"
    )
