#![allow(dead_code)]
pub mod setup;

pub const PVH_BOOT_MAGIC: u32 = 0x336ec578;

#[repr(u32)]
pub enum HvmMemmapType {
    Ram = 1,
    Reserved = 2,
    Acpi = 3,
    Nvs = 4,
    Unusable = 5,
    Disabled = 6,
    Pmem = 7,
}

/// Reference: https://github.com/xen-project/xen/blob/master/xen/include/public/arch-x86/hvm/start_info.h
/// CH sets at address 0x6000.
#[repr(C)]
pub struct HvmStartInfo {
    pub magic: u32,          // == 0x336ec578
    pub version: u32,        // == version of this struct. PVH should be 1.
    pub flags: u32,          // SIF_xxx flags
    pub nr_modules: u32,     // number of modules passed to the kernel. 0 if no modules.
    pub modlist_paddr: u64,  // physical address of an array of hvm_modlist_entry.
    pub cmdline_paddr: u64,  // physical address of the command line, null-terminated ASCII
    pub rsdp_paddr: u64,     // physical address of the RSDP ACPI data struct
    pub memmap_paddr: u64,   // physical address of the memory map. PVH should have it at 0x7000.
    pub memmap_entries: u32, // nr entries in memmap table. 0 if no memmap provided.
    pub _reserved: u32,      // must be zero.
}

impl HvmStartInfo {
    pub fn new(start_info_ptr: *const HvmStartInfo) -> Self {
        if start_info_ptr == core::ptr::null() {
            panic!("Invalid PVH start info pointer");
        }
        let magic = unsafe { (*start_info_ptr).magic };
        if magic != PVH_BOOT_MAGIC {
            panic!("Invalid PVH boot magic");
        }
        let version = unsafe { (*start_info_ptr).version };
        let flags = unsafe { (*start_info_ptr).flags };
        let nr_modules = unsafe { (*start_info_ptr).nr_modules };
        let modlist_paddr = unsafe { (*start_info_ptr).modlist_paddr };
        let cmdline_paddr = unsafe { (*start_info_ptr).cmdline_paddr };
        let rsdp_paddr = unsafe { (*start_info_ptr).rsdp_paddr };
        let memmap_paddr = unsafe { (*start_info_ptr).memmap_paddr };
        let memmap_entries = unsafe { (*start_info_ptr).memmap_entries };
        let _reserved = unsafe { (*start_info_ptr)._reserved };
        Self {
            magic,
            version,
            flags,
            nr_modules,
            modlist_paddr,
            cmdline_paddr,
            rsdp_paddr,
            memmap_paddr,
            memmap_entries,
            _reserved,
        }
    }
}

#[repr(C)]
pub struct HvmModListEntry {
    pub paddr: u64,         // physical address of module
    pub size: u64,          // size of module
    pub cmdline_paddr: u64, // physical address of command line, null-terminated ASCII
    pub reserved: u64,      // must be zero
}

/// Reference: https://uefi.org/specs/ACPI/6.5/15_System_Address_Map_Interfaces.html
#[repr(C)]
pub struct HvmMemMapTableEntry {
    pub addr: u64,                   // start of memory range (bytes)
    pub size: u64,                   // size of memory range (bytes)
    pub mapping_type: HvmMemmapType, // type of memory range
    pub reserved: u32,               // must be zero
}

#[macro_export]
macro_rules! elfnote {
    ($notetype:expr, $valty:expr, $value:expr) => {
        global_asm!(concat!(
            r#"
                .pushsection .note.Xen;
                .align 4;
                .long 2f - 1f;
                .long 4f - 3f;
                .long "#,
            $notetype,
            r#"
            1:  .asciz "Xen";
            2:  .align 4;
            3:  ."#,
            $valty,
            r#" "#,
            $value,
            r#"
            4:  .align 4;
                .popsection;
        "#
        ));
    };
}
