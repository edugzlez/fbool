use crate::traits::encodeable::Encodeable;
use fbool::entanglement::{
    Entanglement, Entropy, EquanimityImportance, NVars, SubInfos, WithEntropy,
};

/// Composite trait that combines all the required functionality for boolean functions
/// This trait is automatically implemented for any type that implements all component traits
pub trait Complete:
    Entanglement + SubInfos + WithEntropy + Entropy + NVars + Encodeable + EquanimityImportance
{
}

impl<T> Complete for T where
    T: Entanglement + SubInfos + WithEntropy + Entropy + NVars + Encodeable + EquanimityImportance
{
}
