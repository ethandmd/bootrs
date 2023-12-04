#![no_std]
#![no_main]
#![allow(internal_features)]
#![feature(lang_items)]

pub mod platform;

use core::arch::global_asm;
use core::panic::PanicInfo;

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

// Reference: https://github.com/rust-osdev/bootloader/blob/a1b2eb88fd365fa7b4b35e29495076f22932cb7f/api/src/lib.rs#L110
#[macro_export]
macro_rules! entry_point {
    ($entry:path) => {
        // Include in order to get _start symbol for kernel entry.
        //global_asm!(include_str!("platform/pvh/start.S"));

        #[panic_handler]
        fn panic(_info: &PanicInfo) -> ! {
            loop {}
        }
        // PVH Boot Reference: https://xenbits.xen.org/docs/4.6-testing/misc/pvh.html
        // Linux PVH Boot: https://github.com/torvalds/linux/blob/master/arch/x86/platform/pvh/head.S
        // CH uses linux-loader::loader::Elf::load() to load the kernel ELF image.
        // load() uses parse_elf_note() which only checks for XEN_ELFNOTE_PHYS32_ENTRY.
        elfnote!(18, "quad", "_start"); // XEN_ELFNOTE_PHYS32_ENTRY.

        #[no_mangle] // Review: no_mangle vs export_name
        pub extern "C" fn _rust_start(start_info_ptr: *const platform::pvh::HvmStartInfo) -> ! {
            if start_info_ptr.is_null() {
                panic!("HVM start info is null.");
            }
            //SAFETY: Already checked that start info is not null.
            let start_info: &platform::pvh::HvmStartInfo = unsafe { &*start_info_ptr };
            if start_info.magic != platform::pvh::PVH_BOOT_MAGIC {
                panic!("HVM start info magic is not PVH_BOOT_MAGIC.");
            }
            // Validate entry point function signature.
            let f: fn() -> ! = $entry;
            f();
            //loop {}
        }
    };
}

fn main() -> ! {
    loop {}
}
entry_point!(main);
