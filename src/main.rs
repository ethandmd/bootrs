#![no_std]
#![no_main]
//#![feature(lang_items)]

pub mod platform;

use core::arch::{asm, global_asm};
use core::panic::PanicInfo;
// Still need this?
//#[lang = "eh_personality"]
//extern "C" fn eh_personality() {}

fn kernel_init(_start_info: &platform::pvh::StartInfo) {}

// Reference: https://github.com/rust-osdev/bootloader/blob/a1b2eb88fd365fa7b4b35e29495076f22932cb7f/api/src/lib.rs#L110
#[macro_export]
macro_rules! entry_point {
    ($entry:path) => {
        #[panic_handler]
        fn panic(_info: &PanicInfo) -> ! {
            loop {}
        }

        extern "C" {
            //static STACK_TOP: u64;
        }

        // PVH Boot Reference: https://xenbits.xen.org/docs/4.6-testing/misc/pvh.html
        // rust-vmm xen: https://github.com/rust-vmm/xen-sys/blob/main/xen/src/lib.rs
        //elfnote!(6, "asciz", "\"unifire\"");
        //elfnote!(7, "asciz", "\"0.1.0\"");
        //elfnote!(8, "asciz", "\"generic\"");
        //elfnote!(5, "asciz", "\"xen-3.0\"");
        //elfnote!(10, "asciz", "\"!writable_page_tables|pae_pgdir_above_4gb\"");
        //elfnote!(9, "asciz", "\"yes\"");
        elfnote!(18, "quad", "0x100000");

        #[export_name = "_start"] // Review: no_mangle vs export_name
        pub extern "C" fn __impl_start() -> ! {
            // Might need to setup our stack using STACK_TOP symbol from linker script
            // instead of using the stack the loader provides...
            // %rsp points to top of initial single page stack.
            // %rsi points to start_info structure.
            // load start_info address from %rsi onto stack and call kernel_init.
            // load start_info address from %rsi into rust variable:
            let start_info: &platform::pvh::StartInfo = unsafe {
                let start_info: u64;
                asm!(
                    "mov rsi, {}",
                    out(reg) start_info
                );
                &*(start_info as *const platform::pvh::StartInfo)
            };
            kernel_init(start_info);
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
