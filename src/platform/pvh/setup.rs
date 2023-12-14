use super::*;
//use x86_64::registers;

#[no_mangle]
pub extern "C" fn _rust_start(start_info_ptr: *const HvmStartInfo) {
    let start_info = HvmStartInfo::new(start_info_ptr);
    let memmap_paddr = start_info.memmap_paddr as *const HvmMemMapTableEntry;
    let memmap_entries = start_info.memmap_entries;
    let memtable = HvmMemMapTable::new(memmap_paddr, memmap_entries);
    for entry in memtable.into_iter() {
        match entry.mapping_type {
            HvmMemmapType::Ram => {
                let start = entry.addr;
                let end = entry.addr + entry.size;
            }
            _ => (),
        }
    }
    crate::__impl_main();
}
