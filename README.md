# 'Unikernel' Project: PVH Boot Implementation

The long term goal of this project is to create a library OS crate that Rust application developers can link to and create unikernels.

The first milestone in this project is a bootable kernel image -- in other words, a kernel binary that can be directly booted by the hypervisor
without a bootloader or hypervisor firmware. Accomplishing this will be critical in achieving minimal boot times and leverages the unikernel 
paradigm that the underlying platform is a known hypervisor/boot protocol.

## Boot Protocol

After a brief survey of mainstream-ish options for booting the kernel image, I found a few candidates:
- Those leveraging Firmware + BIOS / UEFI Boot (via a bootloader)
- Direct Linux Boot
- Others within the Xen ecosystem:
    - HVM (essentially the same as Firmware + Bootloader)
    - PV (paravirtualized)
    - *PVH* (something of a hybrid between paravirtualization + hardware virtualization extensions)

I initially chose to use a bootloader (albeit the [rust-osdev/bootloader](https://github.com/rust-osdev/bootloader) crate to be fun) to create a bootable kernel
image from my application code linked with the library OS components. However this did not yield the boot times I was looking for, so I decided to implement a feature
which would make the resulting unikernel directly bootable. I chose PVH Boot over Direct Linux Boot since PVH Boot had an easier to understand specification despite Direct
Linux Boot being more widely supported across mainstream hypervisors like QEMU and Firecracker. In hindsight, both are relatively manageable and in the future I would like
to implement both in order to run on platforms that don't support PVH Boot.

Note: I didn't end up installing and configuring Xen on a baremetal x86 machine to test with, instead I opted to test with 
[Cloud-Hypervisor](https://github.com/cloud-hypervisor/cloud-hypervisor) since it support PVH Boot instead of Direct Linux Boot and some of its primary contributors are former
Xen contributors!

## Bootable Kernel Image

Having decided to develop for the Cloud-Hypervisor (CH) platform using the PVH Boot protocol the next step is to create a binary that can be loaded and started by CH. Referencing
the PVH Boot specification the build needs to use something called "ElfNotes" to insert special entries in the `.note` section of the resulting binary. Here's a look at an ElfNote
in the resulting kernel image:
```
unifire/src/main.rs:30

Displaying notes found in: .note
  Owner                Data size 	Description
  Xen                  0x00000008	Unknown note type: (0x00000012)
   description data: 00 00 10 00 00 00 00 00 
```
I use a Rust macro to define the above ElfNote which tells the hypervisor where the kernel
entry point is:
```
elfnote!(18, "quad", "_start"); // XEN_ELFNOTE_PHYS32_ENTRY
```
note that this is the *physical* address of the entry point which is loaded in 32-bit 
protected mode on `x86_64` system. I confirm that CH will read this ElfNote and load our
kernel entry point by referencing their source code, which is encapsulated in the 
`rust-vmm/linux-loader` crate in this section:
```
linux-loader/src/loader/x86_64/elf/mod.rs:251-268

// Read in each section pointed to by the program headers.
for phdr in phdrs {
    if phdr.p_type != elf::PT_LOAD || phdr.p_filesz == 0 {
        if phdr.p_type == elf::PT_NOTE {
            // The PVH boot protocol currently requires that the kernel is loaded at
            // the default kernel load address in guest memory (specified at kernel
            // build time by the value of CONFIG_PHYSICAL_START). Therefore, only
            // attempt to use PVH if an offset from the default load address has not
            // been requested using the kernel_offset parameter.
            if let Some(_offset) = kernel_offset {
                loader_result.pvh_boot_cap = PvhBootCapability::PvhEntryIgnored;
            } else {
                // If kernel_offset is not requested, check if PVH entry point is present
                loader_result.pvh_boot_cap = parse_elf_note(&phdr, kernel_image)?;
            }
        }
        continue;
    }
```
it is clear to see that CH will only care about the `PHYS32_ENTRY` ElfNote to make the kernel image bootable.

Now to make the Rust build system produce a freestanding kernel image (but still relying on SYSTEM V ABI):
+ Use the custom linker script and ensure the kernel image is NOT position independent (as it is not coded to be such) and targex a non-linux ABI:
```
unifire/.cargo/config.toml

[build]
target = "x86_64-unknown-none"
rustflags = [
    "-C", "link-arg=-Tsrc/platform/pvh/pvhboot.ld",
    "-C", "link-arg=--no-pie",
    "-C", "relocation-model=static",
]
```
+ Include Rust's (relatively new) inline assembly features to include the early boot assembly instructions which include our kernel entry point `_start`:
```
unifire/src/main.rs:20

global_asm!(include_str!("platform/pvh/boot.S"), options(att_syntax));
```
Note that this requires AT&T syntax over Intel syntax (Intel didn't seem to play nicely for unknown reasons I didn't bother exploring) and restricts the assembly
code to what the LLVM assembler will support instead of the perhaps more familiar GNU assembler.

+ Lastly, make sure that the Rust binary is not attempting to link the `std` library and doesn't expect a main function:
```
unifire/src/main.rs:1-2

#![no_std]
#![no_main]
```

With this, the resulting kernel image should be bootable on CH as seen in this side by side of a gdb console and CH logs (clearly showing that the kernel was loaded at the expected High RAM Address: 1MiB):
```
GDB                                                 
The target architecture is set to "i386:x86-64".    
0x0000000000100000 in _start ()                     
0x0000000000100000 <_start+0>:mov    $0x10b000,%esp 

CH log
30.727671ms: <vmm> INFO:vmm/src/cpu.rs:758 -- Creating vCPU: cpu_id = 0
31.044563ms: <payload_loader> INFO: -- Kernel loaded: entry_addr = 0x100000
```

## Early Boot Code (Kernel PVH Boot):
Now that the kernel image has the entry point set at 1MiB (thanks to the custom linker script) and an ElfNote instructing the VMM to load the kernel at the entry point, the kernel
needs to perform critical early boot operations before it can continue with regular initialization. The VMM (CH) will configure the system as follows for the unifire kernel:
```
(documented in unifire/src/platform/pvh/boot.S)

 Entry point for PVH guests.

 Xen ABI specifies the following register state when we come here:

 - `ebx`: contains the physical memory a.longress where the loader has placed
          the boot start info structure.
 - `cr0`: bit 0 (PE) must be set. All the other writeable bits are cleared.
 - `cr4`: all bits are cleared.
 - `cs `: must be a 32-bit read/execute code segment with a base of `0` 
          and a limit of `0xFFFFFFFF`. The selector value is unspecified.
 - `ds`, `es`: must be a 32-bit read/write data segment with a base of
               `0` and a limit of `0xFFFFFFFF`. The selector values are all
               unspecified.
 - `tr`: must be a 32-bit TSS (active) with a base of '0' and a limit
         of '0x67'.
 - `eflags`: bit 17 (VM) must be cleared. Bit 9 (IF) must be cleared.
             Bit 8 (TF) must be cleared. Other bits are all unspecified.

 All other processor registers and flag bits are unspecified. The OS is in
 charge of setting up it's own stack, GDT and IDT.
```
The first goal for this kernel is jumping to 64-bit "long mode" and entering the Rust code. NOTE, *nearly* all early boot code could be performed in Rust
instead of asm, this is an ideal end state, however, due to the delicate nature of these operations it has been easier for me to debug when performed in
asm entirely as it is relatively easier to reason about. Additionally, there is no stack, the kernel won't be able to jump into a Rust function if there is
no stack! The general scheme of maneuver for getting the kernel from 32-bit protected mode to 64-bit long mode wil be:
```x86
unifire/src/platform/pvh/boot.S

    .code32
    .section .text.start
    .global _start
    .extern _rust_start
_start:
    /* Setup stack pointer. */
    mov $boot_stack_bottom, %esp
    mov $boot_stack_bottom, %ebp

    call setup_page_tables
    call enable_paging
    /* cli */
    call setup_gdt
    jmp long_mode_entry /* And enter Rust, not coming back! */
```

The internal order of operations here may or may not matter, I did not get an empiric read on whether things breakdown if you setup page tables before
the stack or not. However, these steps took inspiration from the Intel SDM, the Linux kernel (arch/x86/platform/pvh/head.S), and the OSDev Wiki. In this
early boot stage I opt to identity map the entirety of physical memory such that all virtual addresses == physical addresses. In the relatively-near future
this will change for two reasons:
+ I would like to incorporate something like a pre-built page table, since mapping the entire physical address space (even with huge pages) is time-
consuming.
+ Identity maps are not advantageous as they lead to fragmentation down the line which impedes operations like allocating large contiguous regions of memory.+ Lastly, I am not carefully using flags on the page table entries, for example, the stack should be read+write, while the `.text` segment should be
red+execute.

This brings us to the next bit of intracacy: all early boot stack and page table data is in the `.bss` section, eventually the kernel will zero `.bss` and
remap the entire kernel address space, so storing any data that will be long-lived must be done carefully. After setting up the page tables, the system
MUST have paging enabled before jumping to 64-bit mode. This is a relatelivy boring but extremely error prone manuever where the kernel sets the PAE bit 
in `%cr4`, the paging bit in `%cr0` and lastly, writes the address of the top level page table in `%cr3`.

The final requirement before jumping to long mode is configuring a Global Descriptor Table (GDT). Although memory segmentation will *not* be used in this
`x86_64` kernel, the GDT must still be setup and loaded using the `lgdt` command. This entire operation caused me *extreme* headaches. Here is the GDT 
layout I landed on:
```
unifire/src/platform/pvh/boot.S

    .section .rodata
    .balign 8
gdt:
    .word gdt_end - gdt_start /* GDT size. Limit */
    .long gdt_start /* GDT address. Base */
    .word 0x0
gdt_start:
    .quad 0x0               /* null descriptor */
    .quad 0xaf9a000000ffff  /* 64-bit code segment descriptor // CS_SEL := 0x8 */
    .quad 0xc092000000ffff  /* 64-bit data segment descriptor // DS_SEL := 0x10 */
gdt_end:

```
This structure is loaded as follows:
```
setup_gdt:
    lgdt [gdt]
    
    mov $0x10, %eax /* 0x10 is data segment selector in GDT. */
    mov %eax, %ds
    mov %eax, %es
    mov %eax, %ss
    ret
```
When the GDT structure is loaded, the system actually gets a pointer to the *actual* GDT using a few mildly clever asm tricks to create an 
8-byte value which will point the system at the segment descriptors to use. This also took a few stabs in the dark to get right...and may require a few more.

A few notes here: the lower 42 (ish?) bits are *mostly* irrelevant in long mode, but not entirely!! As I spent many painful debugging sessions verifying,
each GDT segment *must* include the limit in addition to the base of the segment despite segmentation not being used in long mode except for possibly one
single instruction that I don't recall making conscious use of. That said, Loading the GDT is worthless for the jump to long mode unless the segment
selectors are loaded into the appropriate registers! This was another initial oversight that led to some peculiar bugs. Long story short, the code segment
ought to point to the offset within the GDT structure where the code segment resides and likewise for the data segment. There are a few ways to do this,
I opt to explicitly set the data segment registers with `0x10` (the data segment in the GDT is the second quad-word) and then use a long jump to *jump* to
long mode which will set the code selector in the code segment register for me (at least I know that now...took me more painful debugging sessions to 
discover this)
```
long_mode_entry:
    ljmp $0x8, $rust_entry /* Jump to 64-bit code segment. */
```
Finally, the kernel can call Rust code in 64-bit mode:
```
    .code64
rust_entry:
    mov %rbx, %rdi /* SYSTEM V ABI passes first six args in regs. */
    jmp _rust_start
```
Note here the use of the `.code64` directive so the assembler can help differentiate the earlier 32-bit sections from the 64-bit ones.

### Debugging Early Boot
Since there is no interrupt handler registered at this point anything from a general protection error or page fault is basically just intuition, as CH
doesn't have as robust debugging facilities as a platform like QEMU, although they're not bad.

## Rust Kernel Initialization
The Rust kernel code begins in earnest once the system is in long mode where the following function is called:
```
unifire/src/platform/pvh/setup.rs

#[no_mangle]
pub extern "C" fn _rust_start(start_info_ptr: *const HvmStartInfo)
```

This function has a few special attributes compared to the average Rust function signature. First, it is decorated with `#[no_mangle]` so that the symbol 
"_rust_start" is included untouched in the kernel image as seen in the following objdump:
```
0000000000100230 g     F .text	000000000000011d _rust_start
```
this lets the asm code in `boot.S` correctly take it on faith that the `.extern _rust_start` symbol exists in order to call the Rust function from asm.

Next, the `extern "C"` tells the Rust compiler to enforce the C ABI for this function, which is particularly useful for the kernel since this ensures we 
can pass arguments from assembly. Given that the SYSTEM V ABI dictates the first six (?) arguments are passed via registers, the `rust_entry:` asm section
copies the contents of `%rbx` to `%rdi` where the Rust function will find it! Recall from the PVH Boot protocol that the pointer to the boot info params
is stored in the `%rbx` register ... better not clobber it!

The Rust kernel initilization must accomplish the following:
+ Read the system memory map
+ Parse the ACPI tables
+ Parse any commandline arguments
+ Check for an initrd

All of these parameters are held in memory (at `0x6000` in CH) in the following struct:
```
#[repr(C)]
pub struct HvmStartInfo {
    pub magic: u32,          // == 0x336ec578
    pub version: u32,        // == version of this struct. PVH should be 1.
    pub flags: u32,          // SIF_xxx flags
    pub nr_modules: u32,     // number of modules passed to the kernel. 0 if no modules.
    pub modlist_paddr: u64,  // pa of an array of hvm_modlist_entry.
    pub cmdline_paddr: u64,  // pa of the command line, null-terminated ASCII
    pub rsdp_paddr: u64,     // pa of the RSDP ACPI data struct
    pub memmap_paddr: u64,   // pa of the memory map. PVH should have it at 0x7000.
    pub memmap_entries: u32, // nr entries in memmap table. 0 if no memmap provided.
    pub _reserved: u32,      // must be zero.
}
```

In order to make use of this struct, the kernel needs to ensure the data is not obviously corrupt or otherwise unusable by checking the magic number
and version number. Then, the fields of interest:
+ `nr_modules, modlist_paddr`: Is used to tell the kernel an initrd is present.
+ `cmdline_paddr`: Just like a C program that takes arguments through `int argc, char** argv`, a PVH bootable kernel takes cmdline args!
+ `rsdp_paddr`: Used to locate and parse ACPI tables.
+ `memmap_paddr, memmap_entries`: The location and length of the memory map.

The first field the kernel reads after magic and version is the memory map, it uses these to detect the system's memory regions (which can be augmented
through the CH interface) and tells the kernel how much RAM there is and which regions of memory can be used for the kernel heap (and eventually address
space reorganization).

### Seemingly Insurmountable Kernel Error
Over the course of days I debugged the `_rust_start` function due to an issue with pointer misalignment. After setting up page tables, enabling paging, 
loading the GDT, and setting up the stack, the Rust code was suffering from debilitating alignment issues which made it impossible to operate the kernel.
For example, the `_rust_start` function could adequately receive the `start_info_ptr` argument and GDB showed a valid start info struct in memory, however
when any of the fields of the struct were read by later Rust code they would be off by one or more bytes. For instance, trying to to use the `memmap_paddr`
field, which is set as `0x7000` by CH, a value like `let mp = start_info.memmap_paddr` would be `0x6fff`! After reviewing the early boot asm code I dove
in deep on the Rust `ptr` core library module to understand if Rust was secretely sabotaging the kernel with hidden pointer metadata as part of its 
provenance experiment or something. This days-long adventure yielded nothing. Finally, after reviewing the early boot code for the umpteenth time I 
discovered what I believe to be the issue: the long jump to 64-bit mode was *not* setting the code segment register with the right selector. This 
tweak seemed to fix it.

Potential confirmation of this fix is that GDB no longer works quite right while debugging the kernel! Why is this a positive sign? Well it turns out
vanilla-GDB can't tell that the processor has switched from 32-bit protected mode to 64-bit long mode and so once the jump has been made it becomes
nearly useless. However, there are manual/automate-able fixes or patches available for this issue.

New issues: Now the kernel doesn't seem to start on its own when booted without GDB when running on CH! This new and exciting issue will consume much more
energy I'm sure, however, with a more robust interrupt handler and eventual serial/console output support it should become easier to at least catch the 
causes of the resulting faults :)


## Future work
I plan to continue my efforts on this project as I am able to, focusing on the following areas:
+ Finish parsing the memory map table to use the appopriate region for a kernel heap and remapping the kernel
+ Device discovery and power management by parsing the ACPI table
+ Loading and using cmdline arguments!

Longer term goals:
+ Use a passed in initrd
+ Basic VIRTIO block / net drivers
+ Device passthrough!
