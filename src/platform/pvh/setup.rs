use super::*;
use core::ptr;

#[no_mangle]
pub extern "C" fn _rust_start(start_info_ptr: u64) {
    // Rust fn won't let us use a raw pointer as an argument, so we have to;
    if start_info_ptr == 0 {
        panic!("Invalid PVH start info pointer");
    }
    // SAFETY: start_info_ptr is a non-null pointer.
    // We check magic number to make sure data is valid.
    let start_info: HvmStartInfo = unsafe { ptr::read(start_info_ptr as *const HvmStartInfo) };
    if start_info.magic != PVH_BOOT_MAGIC {
        panic!("Invalid PVH start info magic");
    }

    crate::__impl_main();
}
