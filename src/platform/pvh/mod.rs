#![allow(dead_code)]
pub mod setup;

pub const PVH_BOOT_MAGIC: u32 = 0x336ec578;
pub const XEN_HVM_MEMMAP_TYPE_RAM: u32 = 1;
pub const XEN_HVM_MEMMAP_TYPE_RESERVED: u32 = 2;
pub const XEN_HVM_MEMMAP_TYPE_ACPI: u32 = 3;
pub const XEN_HVM_MEMMAP_TYPE_NVS: u32 = 4;
pub const XEN_HVM_MEMMAP_TYPE_UNUSABLE: u32 = 5;
pub const XEN_HVM_MEMMAP_TYPE_DISABLED: u32 = 6;
pub const XEN_HVM_MEMMAP_TYPE_PMEM: u32 = 7;

/// Reference: https://github.com/xen-project/xen/blob/master/xen/include/public/arch-x86/hvm/start_info.h
/// CH sets at address 0x6000.
#[repr(C)]
pub struct HvmStartInfo {
    pub magic: u32,          // == 0x336ec578
    pub version: u32,        // == version of this struct. PVH should be 1.
    pub flags: u32,          // SIF_xxx flags
    pub nr_modules: u32,     // number of modules passed to the kernel. 0 if no modules.
    pub modlist_paddr: u64, // physical address of an array of hvm_modlist_entry. PVH should have it at 0x6040.
    pub cmdline_paddr: u64, // physical address of the command line
    pub rsdp_paddr: u64,    // physical address of the RSDP ACPI data struct
    pub memmap_paddr: u64,  // physical address of the memory map. PVH should have it at 0x7000.
    pub memmap_entries: u32, // nr entries in memmap table. 0 if no memmap provided.
    pub reserved: u32,      // must be zero.
}

/// Reference: https://github.com/xen-project/xen/blob/master/xen/include/public/arch-x86/hvm/start_info.h
#[repr(C)]
pub struct HvmModListEntry {
    pub paddr: u64,         // physical address of module
    pub size: u64,          // size of module
    pub cmdline_paddr: u64, // physical address of command line
    pub reserved: u64,      // must be zero
}

/// Reference: https://github.com/xen-project/xen/blob/master/xen/include/public/arch-x86/hvm/start_info.h
#[repr(C)]
pub struct HvmMemMapTableEntry {
    pub addr: u64,         // start of memory range (bytes)
    pub size: u64,         // size of memory range (bytes)
    pub mapping_type: u32, // type of memory range
    pub reserved: u32,     // must be zero
}

// CH PVH Setup Reference: https://github.com/cloud-hypervisor/cloud-hypervisor/blob/5f89461a7e95f937733cefa4d2854f98873e3f88/arch/src/x86_64/mod.rs#L929

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
