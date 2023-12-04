#![allow(dead_code)]

//_pa(x) ((x) - _KERNEL_BASE) /* from Linux */
pub static PVH_GDT_ENTRY_CS: u32 = 1;
pub static PVH_GDT_ENTRY_DS: u32 = 2;
pub static PVH_CS_SEL: u32 = PVH_GDT_ENTRY_CS * 8;
#[no_mangle]
pub static PVH_DS_SEL: u32 = PVH_GDT_ENTRY_DS * 8;
#[no_mangle]
pub static PAE_BIT: u32 = 1 << 5;

// Stolen from: linux/include/xen/interface/elfnote.h
// The notes should live in a SHT_NOTE segment and have "Xen" in the
// name field.
//
// Numeric types are either 4 or 8 bytes depending on the content of
// the desc field.
//
// LEGACY indicated the fields in the legacy __xen_guest string which
// this a note type replaces.
//
// String values (for non-legacy) are NULL terminated ASCII, also known
// as ASCIZ type.

// NAME=VALUE pair (string).
pub const XEN_ELFNOTE_INFO: u32 = 0;

// The virtual address of the entry point (numeric).
//
// LEGACY: VIRT_ENTRY

pub const XEN_ELFNOTE_ENTRY: u32 = 1;

// The virtual address of the hypercall transfer page (numeric).
//
// LEGACY: HYPERCALL_PAGE. (n.b. legacy value is a physical page
// number not a virtual address)

pub const XEN_ELFNOTE_HYPERCALL_PAGE: u32 = 2;

// The virtual address where the kernel image should be mapped (numeric).
//
// Defaults to 0.
//
// LEGACY: VIRT_BASE

pub const XEN_ELFNOTE_VIRT_BASE: u32 = 3;

// The offset of the ELF paddr field from the acutal required
// pseudo-physical address (numeric).
//
// This is used to maintain backwards compatibility with older kernels
// which wrote __PAGE_OFFSET into that field. This field defaults to 0
// if not present.
//
// LEGACY: ELF_PADDR_OFFSET. (n.b. legacy default is VIRT_BASE)
pub const XEN_ELFNOTE_PADDR_OFFSET: u32 = 4;

// The version of Xen that we work with (string).
//
// LEGACY: XEN_VER
pub const XEN_ELFNOTE_XEN_VERSION: u32 = 5;

// The name of the guest operating system (string).
//
// LEGACY: GUEST_OS
pub const XEN_ELFNOTE_GUEST_OS: u32 = 6;

// The version of the guest operating system (string).
//
// LEGACY: GUEST_VER
pub const XEN_ELFNOTE_GUEST_VERSION: u32 = 7;

// The loader type (string).
//
// LEGACY: LOADER
pub const XEN_ELFNOTE_LOADER: u32 = 8;

// The kernel supports PAE (x86/32 only, string = "yes" or "no").
//
// LEGACY: PAE (n.b. The legacy interface included a provision to
// indicate 'extended-cr3' support allowing L3 page tables to be
// placed above 4G. It is assumed that any kernel new enough to use
// these ELF notes will include this and therefore "yes" here is
// equivalent to "yes[entended-cr3]" in the __xen_guest interface.
pub const XEN_ELFNOTE_PAE_MODE: u32 = 9;

// The features supported/required by this kernel (string).
//
// The string must consist of a list of feature names (as given in
// features.h, without the "XENFEAT_" prefix) separated by '|'
// characters. If a feature is required for the kernel to function
// then the feature name must be preceded by a '!' character.
//
// LEGACY: FEATURES
pub const XEN_ELFNOTE_FEATURES: u32 = 10;

// The kernel requires the symbol table to be loaded (string = "yes" or "no")
// LEGACY: BSD_SYMTAB (n.b. The legacy treated the presence or absence
// of this string as a boolean flag rather than requiring "yes" or
// "no".
pub const XEN_ELFNOTE_BSD_SYMTAB: u32 = 11;

// The lowest address the hypervisor hole can begin at (numeric).
//
// This must not be set higher than HYPERVISOR_VIRT_START. Its presence
// also indicates to the hypervisor that the kernel can deal with the
// hole starting at a higher address.
pub const XEN_ELFNOTE_HV_START_LOW: u32 = 12;

// List of maddr_t-sized mask/value pairs describing how to recognize
// (non-present) L1 page table entries carrying valid MFNs (numeric).
pub const XEN_ELFNOTE_L1_MFN_VALID: u32 = 13;

// Whether or not the guest supports cooperative suspend cancellation.
// This is a numeric value.
//
// Default is 0
pub const XEN_ELFNOTE_SUSPEND_CANCEL: u32 = 14;

// The (non-default) location the initial phys-to-machine map should be
// placed at by the hypervisor (Dom0) or the tools (DomU).
// The kernel must be prepared for this mapping to be established using
// large pages, despite such otherwise not being available to guests.
// The kernel must also be able to handle the page table pages used for
// this mapping not being accessible through the initial mapping.
// (Only x86-64 supports this at present.)
pub const XEN_ELFNOTE_INIT_P2M: u32 = 15;

// Whether or not the guest can deal with being passed an initrd not
// mapped through its initial page tables.
pub const XEN_ELFNOTE_MOD_START_PFN: u32 = 16;

// The features supported by this kernel (numeric).
//
// Other than XEN_ELFNOTE_FEATURES on pre-4.2 Xen, this note allows a
// kernel to specify support for features that older hypervisors don't
// know about. The set of features 4.2 and newer hypervisors will
// consider supported by the kernel is the combination of the sets
// specified through this and the string note.
//
// LEGACY: FEATURES
pub const XEN_ELFNOTE_SUPPORTED_FEATURES: u32 = 17;

// Physical entry point into the kernel.
//
// 32bit entry point into the kernel. When requested to launch the
// guest kernel in a HVM container, Xen will use this entry point to
// launch the guest in 32bit protected mode with paging disabled.
// Ignored otherwise.
pub const XEN_ELFNOTE_PHYS32_ENTRY: u32 = 18;

// The number of the highest elfnote defined.
pub const XEN_ELFNOTE_MAX: u32 = XEN_ELFNOTE_PHYS32_ENTRY;

pub const PVH_BOOT_MAGIC: u32 = 0x336ec578;

/// Reference: https://github.com/xen-project/xen/blob/master/xen/include/public/arch-x86/hvm/start_info.h
/// CH sets at address 0x6000.
pub struct StartInfo {
    pub magic: u32,          // == 0x336ec578
    pub version: u32,        // == version of this struct. PVH should be 1.
    pub flags: u32,          // SIF_xxx flags
    pub nr_modules: u32,     // number of modules passed to the kernel. 0 if no modules.
    pub modlist_paddr: u32, // physical address of an array of hvm_modlist_entry. PVH should have it at 0x6040.
    pub cmdline_paddr: u32, // physical address of the command line
    pub rsdp_paddr: u32,    // physical address of the RSDP ACPI data struct
    pub memmap_paddr: u32,  // physical address of the memory map. PVH should have it at 0x7000.
    pub memmap_entries: u32, // nr entries in memmap table. 0 if no memmap provided.
    pub reserved: u32,      // must be zero.
}

/// Reference: https://github.com/xen-project/xen/blob/master/xen/include/public/arch-x86/hvm/start_info.h
pub struct HvmModListEntry {
    pub paddr: u64,         // physical address of module
    pub size: u64,          // size of module
    pub cmdline_paddr: u64, // physical address of command line
    pub reserved: u64,      // must be zero
}

/// Reference: https://github.com/xen-project/xen/blob/master/xen/include/public/arch-x86/hvm/start_info.h
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
