use bincode::Encode;

/// Trait for types that can be encoded to binary format
pub trait Encodeable {
    fn encode(&self) -> Result<Vec<u8>, bincode::error::EncodeError>;
}

impl<T> Encodeable for T
where
    T: Encode,
{
    fn encode(&self) -> Result<Vec<u8>, bincode::error::EncodeError> {
        bincode::encode_to_vec(self, bincode::config::standard())
    }
}
