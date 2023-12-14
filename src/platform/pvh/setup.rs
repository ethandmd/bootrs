use super::*;
use core::ptr;
//use x86_64::registers;

#[no_mangle]
pub extern "C" fn _rust_start(start_info_ptr: *const HvmStartInfo) {
    if start_info_ptr == ptr::null() {
        panic!("Invalid PVH start info pointer");
    }
    let version = unsafe { (*start_info_ptr).version };
    let memmap_paddr = unsafe { (*start_info_ptr).memmap_paddr };
    let start_info = HvmStartInfo::new(start_info_ptr);
    crate::__impl_main();
}
