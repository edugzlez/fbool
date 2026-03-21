from fbool import FBool


def test_fbool_majority_entanglement():
    for n in range(2, 10):
        maj = FBool.majority(n)

        assert maj.entanglement() == n + 2
