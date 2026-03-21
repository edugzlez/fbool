from fbool import FBool


def test_fbool_parity_entanglement():
    for n in range(2, 10):
        maj = FBool.parity(n)

        assert maj.entanglement() == 4


def test_fbool_parity_sensitivity():
    for n in range(2, 10):
        maj = FBool.parity(n)

        assert maj.max_sensitivity() == n
