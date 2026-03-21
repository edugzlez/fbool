#[macro_export]
macro_rules! bv {
    ($bits:expr) => {{
        let bits = stringify!($bits);
        let size = bits.len();
        let mut repr = Vec::new();
        let mut current_byte = 0u8;
        let mut bit_count = 0;

        for (i, bit) in bits.chars().rev().enumerate() {
            if bit == '1' {
                current_byte |= 1 << (i % 8);
            }
            bit_count += 1;
            if bit_count == 8 || i == size - 1 {
                repr.push(current_byte);
                current_byte = 0;
                bit_count = 0;
            }
        }

        $crate::bitvector::BitVector::from_vec(repr, size)
    }};
}
