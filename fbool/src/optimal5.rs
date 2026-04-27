pub trait WithMinimalGates {
    fn minimal_gates(&self) -> Option<u32>;
    fn npn_representant(&self) -> Option<u32>;
}

#[repr(C)]
pub struct WrapperOptimiser {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

unsafe extern "C" {
    fn create_wrapper(bv: *const u8) -> *mut WrapperOptimiser;
    fn num_gates(wrapper: *mut WrapperOptimiser, fun: u32) -> u32;
    fn npn_representant(wrapper: *mut WrapperOptimiser, fun: u32) -> u32;
}

static mut WRAPPER: *mut WrapperOptimiser = std::ptr::null_mut();
static BYTES: &[u8; 9241860] = include_bytes!("../knuthies.dat");

pub fn init_wrapper() {
    unsafe {
        if WRAPPER.is_null() {
            WRAPPER = create_wrapper(BYTES.as_ptr());
        }
    }
}

impl WithMinimalGates for crate::fvalue::FValue<bool> {
    fn minimal_gates(&self) -> Option<u32> {
        init_wrapper();
        if self.repr().len() != 32 {
            None
        } else {
            let usize_repr = self.repr();

            let mut fun: u32 = 0;
            for i in 0..usize_repr.len() {
                let bit = usize_repr[usize_repr.len() - i - 1] as u32;
                fun |= bit << i;
            }
            let ngates = unsafe { num_gates(WRAPPER, fun) };
            Some(ngates)
        }
    }

    fn npn_representant(&self) -> Option<u32> {
        init_wrapper();
        if self.repr().len() != 32 {
            None
        } else {
            let usize_repr = self.repr();

            let mut fun: u32 = 0;
            for i in 0..usize_repr.len() {
                let bit = usize_repr[usize_repr.len() - i - 1] as u32;
                fun |= bit << i;
            }
            let representant = unsafe { npn_representant(WRAPPER, fun) };
            Some(representant)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::WithMinimalGates;
    use crate::fvalue::FValue;

    #[test]
    fn minimal_gates_handles_basic_5var_functions() {
        let zero = FValue::from_usize(0, 5);
        let one = FValue::from_usize(u32::MAX as usize, 5);

        assert_eq!(zero.minimal_gates(), Some(0));
        assert_eq!(one.minimal_gates(), Some(0));
    }

    #[test]
    fn minimal_gates_stays_in_reasonable_range() {
        // Regression guard for ABI/signature issues across toolchains.
        // For 5-variable functions this value should always be small.
        let sample = [
            0x0000_0000u32,
            0xFFFF_FFFFu32,
            0x6996_6996u32,
            0x8000_0001u32,
            0xA5A5_5A5Au32,
            0xDEAD_BEEFu32,
            0x1234_5678u32,
            0x0F0F_F0F0u32,
        ];

        for fun in sample {
            let f = FValue::from_usize(fun as usize, 5);
            let gates = f.minimal_gates().expect("n=5 must be supported");
            assert!(
                gates <= 64,
                "unexpected minimal_gates={gates} for 0x{fun:08X}"
            );
        }
    }
}
