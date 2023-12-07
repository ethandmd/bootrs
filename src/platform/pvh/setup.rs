#[no_mangle]
pub extern "C" fn _rust_start(start_info_ptr: u64) {
    //let start_info: &HvmStartInfo;
    //if let Some(start_info_ptr) = unsafe {
    //    let value: u64;
    //    asm!("mov {}, rbx", out(reg) value, options(nomem, nostack));
    //    if value == 0 {
    //        None
    //    } else {
    //        Some(value as *const crate::platform::pvh::HvmStartInfo)
    //    }
    //} {
    //    // SAFETY: Previous check that start_info is a valid pointer
    //    // and PVH Boot Protocol states that start_info is in rbx.
    //    start_info = unsafe { &*start_info_ptr };
    //} else {
    //    panic!("No start info provided by hypervisor");
    //}

    if start_info_ptr == 0 {
        panic!("Invalid PVH start info pointer");
    }

    crate::__impl_main();
}
