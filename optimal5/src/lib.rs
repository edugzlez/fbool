use std::ffi::c_uint;

pub trait WithMinimalGates {
    fn minimal_gates(&self) -> Option<u32>;
    fn minimal_depth(&self) -> Option<u32>;
    fn npn_representant(&self) -> Option<u32>;
}

#[repr(C)]
pub struct WrapperOptimiser {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

unsafe extern "C" {
    fn create_wrapper(bv: *const u8) -> *mut WrapperOptimiser;
    fn num_gates(wrapper: *mut WrapperOptimiser, fun: u32) -> c_uint;
    fn calculate_depth(wrapper: *mut WrapperOptimiser, fun: u32) -> c_uint;
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

impl WithMinimalGates for fbool::fvalue::FValue<bool> {
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

    fn minimal_depth(&self) -> Option<u32> {
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
            let depth = unsafe { calculate_depth(WRAPPER, fun) };
            Some(depth)
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
