use super::*;
use core::mem;
use core::ptr;

fn iter_mem_regions(mementry_ptr: usize, mpe: u32) {
    if mementry_ptr == 0 || mpe == 0 {
        panic!("Invalid PVH memory map pointer");
    }
    let mut i = 0;
    while i < mpe {
        // SAFETY: mementry is a non-null pointer.
        // Mem table entries are valid every size_of::<HvmMemMapTableEntry>() bytes
        // for mpe entries.
        unsafe {
            let next_entry = (mementry_ptr as *const HvmMemMapTableEntry).add(i as usize);
            let entry = ptr::read(next_entry);
            let _addr = entry.addr;
            let _size = entry.size;
            let _entry_type = entry.mapping_type;
        }
        i += 1;
    }
}

#[no_mangle]
pub extern "C" fn _rust_start(start_info_ptr: usize) {
    // Check against zero instead of is_null() because is_null() was true for non-null pointers.
    if start_info_ptr == 0 {
        panic!("Invalid PVH start info pointer");
    }
    let _info_size = mem::size_of::<HvmStartInfo>();
    // SAFETY: start_info_ptr is a non-null pointer.
    let start_info = unsafe { ptr::read(start_info_ptr as *const HvmStartInfo) };
    let mementry = start_info.memmap_paddr as usize;
    if mementry == 0 {
        panic!("Invalid PVH memory map pointer");
    }
    let mpe = start_info.memmap_entries;
    iter_mem_regions(mementry, mpe);
    crate::__impl_main();
}
