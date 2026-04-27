import numpy as np
import numpy.typing as npt

class EntanglementSets:
    entanglement: int
    set1: list[int]
    set2: list[int]

class EntropySets:
    entropy: float
    set1: list[int]
    set2: list[int]

class FBool:
    def __init__(self, repr: list[bool]) -> None:
        """
        Initialize the FBool object with a list of boolean values.

        Args:
            repr (list[bool]):
                A list of boolean values representing the FBool object.
        """
        ...

    @staticmethod
    def from_number(n: int, num_vars: int) -> FBool:
        """
        Create an FBool object from a number and a specified size.

        Args:
            n (int):
                The number to convert into an FBool object.
            num_vars (int):
                The number of variables (size) for the FBool object.
        Returns:
            (FBool):
                The created FBool object.
        """
        ...

    def eval(self, i: int) -> bool:
        """
        Evaluates the FBool object and returns its boolean value.

        Returns:
            (bool):
                The boolean value of the FBool object.
        """
        ...

    def repr(self) -> list[bool]:
        pass

    def size(self) -> int:
        """
        Returns the size of the FBool object.

        Returns:
            (int):
                The size of the FBool object.
        """
        ...

    def __len__(self) -> int:
        """
        Returns the size of the FBool object.

        Returns:
            (int):
                The size of the FBool object.
        """
        ...

    @classmethod
    def coprimes(cls, n: int) -> FBool:
        """
        Generates a Boolean formula with 2n variables where each half represents a number, and the result is 1 if and only if the numbers are coprime.

        Args:
            n (int):
                Determines the number of variables (2n) in the formula.

        Returns:
            (FBool):
                A Boolean formula representing the coprime relationship.
        """
        ...

    @classmethod
    def sum_is_prime(cls, n: int) -> FBool:
        """
        Generates a Boolean formula with n variables where the result is 1 if and only if the sum of the variables is prime.

        Args:
            n (int):
                Determines the number of variables (n) in the formula.

        Returns:
            (FBool):
                A Boolean formula representing the prime sum relationship.
        """
        ...

    @classmethod
    def primality(cls, n: int) -> FBool:
        """
        Generates a Boolean formula with n variables where the result is 1 if and only if the number represented by the variables is prime.

        Args:
            n (int):
                Determines the number of variables (n) in the formula.

        Returns:
            (FBool):
                A Boolean formula representing the primality of the number.
        """
        ...

    @classmethod
    def clique(cls, n: int) -> FBool:
        """
        Generates a Boolean formula with n variables where the result is 1 if and only if the variables form a clique in a complete graph.

        Args:
            n (int):
                Determines the number of variables (n) in the formula.

        Returns:
            (FBool):
                A Boolean formula representing the clique relationship.
        """
        ...

    @classmethod
    def majority(cls, n: int) -> FBool:
        """
        Generates a Boolean formula with n variables where the result is 1 if and only if the number of 1s in the variables is greater than n/2.

        Args:
            n (int):
                Determines the number of variables (n) in the formula.

        Returns:
            (FBool):
                A Boolean formula representing the majority condition.
        """
        ...

    @classmethod
    def parity(cls, n: int) -> FBool:
        """
        Generates a Boolean formula with n variables where the result is 1 if and only if the number represented by the variables is even.

        Args:
            n (int):
                Determines the number of variables (n) in the formula.

        Returns:
            (FBool):
                A Boolean formula representing the parity of the number.
        """
        ...

    def entanglement(self) -> int:
        """
        Calculate the entanglement of the FBool object.

        Returns:
            (int):
                The entanglement value of the FBool object.
        """
        ...

    def entanglement_sets(self) -> list[EntanglementSets]:
        """
        Get the sets of variables that contribute to the entanglement.

        Returns:
            (list[EntanglementSets]):
                A list of sets of variables that contribute to the entanglement.

                Each set contains the entanglement value and two sets representing the partition of variables.
        """
        ...

    def entropy(self) -> float:
        """
        Calculate the entropy of the FBool object.

        Returns:
            (float):
                The entropy value of the FBool object.
        """
        ...
    def entropy_sets(self) -> list[EntropySets]:
        """
        Get the sets of variables that contribute to the entropy.

        Returns:
            (list[EntropySets]):
                A list of sets of variables that contribute to the entropy.

                Each set contains the entropy value and two sets representing the partition of variables.
        """
        ...

    def equanimity_importance(self) -> float:
        """
        Calculate the equanimity importance of the FBool object.

        Returns:
            (float):
                The equanimity importance value of the FBool object.
        """
        ...

    def n_vars(self) -> int:
        """
        Get the number of variables in the FBool object.

        Returns:
            (int):
                The number of variables in the FBool object.
        """
        ...

    def information(self, vars: list[int]) -> int:
        """
        Calculate the information of a specific set of variables in the FBool object.

        Args:
            vars (list[int]):
                A list of variable indices for which to calculate the information.

        Returns:
            (int):
                The information value for the specified set of variables.
        """
        ...

    def table(self, vars: list[int]) -> npt.NDArray[np.bool_]:
        """
        Generate the truth table for a specific set of variables in the FBool object.

        Args:
            vars (list[int]):
                A list of variable indices for which to generate the truth table.

        Returns:
            (npt.NDArray):
                A truth table represented as a NumPy array of boolean values.
        """
        ...

    def encode(self) -> bytes:
        """
        Encode the FBool object into a byte representation.
        Returns:
            (bytes):
                The byte representation of the FBool object.
        """

    @classmethod
    def decode(cls, data: bytes) -> FBool:
        """
        Decode a byte representation into an FBool object.

        Args:
            data (bytes):
                The byte representation of the FBool object.

        Returns:
            (FBool):
                The decoded FBool object.
        """
        ...

    def minimal_gates(self) -> int | None:
        """Get the minimal number of gates required to implement the FBool object.

        Returns:
            (int | None):
                The minimal number of gates required to implement the FBool object, or None if the FBool object is empty.
        """
        ...

    def frontier_size(self) -> int:
        """
        Calculate the size of the frontier of the FBool object.

        Returns:
            (int):
                The size of the frontier of the FBool object.
        """
        ...

    def max_frontier_size(self) -> int:
        """
        Calculate the maximum size of the frontier of the FBool object.

        Returns:
            (int):
                The maximum size of the frontier of the FBool object.
        """
        ...

    def npn_representant(self) -> FBool | None:
        """
        Get the NPN representant of the FBool object.

        Returns:
            (FBool):
                The representant of the FBool object.
        """
        ...

    def max_sensitivity(self) -> int:
        """
        Calculate the sensitivity of the FBool object.

        Returns:
            (int):
                The sensitivity of the FBool object.
        """
        ...

    def mean_sensitivity(self) -> float:
        """
        Calculate the mean sensitivity of the FBool object.

        Returns:
            (float):
                The mean sensitivity of the FBool object.
        """
        ...

    def spectral_entropy(self) -> float:
        """
        Calculate the spectral entropy of the FBool object.

        Returns:
            (float):
                The spectral entropy of the FBool object.
        """
        ...

    def degree(self) -> int:
        """
        Calculate the degree of the FBool object.

        Returns:
            (int):
                The degree of the FBool object.
        """
        ...

    def no_linearity(self) -> int:
        """
        Calculate the non-linearity of the FBool object.

        Returns:
            (int):
                The non-linearity of the FBool object.
        """
        ...

    def influence(self, var: int) -> float:
        """
        Calculate the influence of a specific variable in the FBool object.

        Args:
            var (int):
                The index of the variable for which to calculate the influence.
        Returns:
            (float):
                The influence of the specified variable.
        """
        ...

    def total_influence(self) -> float:
        """
        Calculate the total influence of all variables in the FBool object.

        Returns:
            (float):
                The total influence of all variables.
        """
        ...

    def certificate_complexity(self) -> int:
        """
        Calculate the certificate complexity of the FBool object.

        Returns:
            (int):
                The certificate complexity of the FBool object.
        """
        ...

    def fragmentation_coefficient(self, vars: list[int]) -> float:
        """Return the local fragmentation coefficient F(f, S)."""
        ...

    def fragmentation_k(self, k: int) -> float:
        """Return S_k(f), the average fragmentation at subset size k."""
        ...

    def fragmentation_spectrum(self) -> list[float]:
        """Return the full fragmentation spectrum (S_0, ..., S_n)."""
        ...

    def restriction_signature(self) -> list[float]:
        """Alias of fragmentation_spectrum()."""
        ...

    def fragmentation_profile(self) -> list[float]:
        """Alias of fragmentation_spectrum()."""
        ...

    def fragmentation_peak(self) -> tuple[int, float]:
        """Return (k*, Smax) for the fragmentation spectrum."""
        ...

    def fragmentation_delta(self) -> list[float]:
        """Return first discrete derivative of the spectrum."""
        ...

    def fragmentation_delta2(self) -> list[float]:
        """Return second discrete derivative of the spectrum."""
        ...
