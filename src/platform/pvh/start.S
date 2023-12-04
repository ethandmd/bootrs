	/*
	 * Copied from linux/arch/x86/platform/pvh/head.S
	 * 
	 *  Entry point for PVH guests.
	 * 
	 *  Xen ABI specifies the following register state when we come here:
	 * 
	 *  - `ebx`: contains the physical memory address where the loader has placed
	 *           the boot start info structure.
	 *  - `cr0`: bit 0 (PE) must be set. All the other writeable bits are cleared.
	 *  - `cr4`: all bits are cleared.
	 *  - `cs `: must be a 32-bit read/execute code segment with a base of `0`
	 *           and a limit of `0xFFFFFFFF`. The selector value is unspecified.
	 *  - `ds`, `es`: must be a 32-bit read/write data segment with a base of
	 *                `0` and a limit of `0xFFFFFFFF`. The selector values are all
	 *                unspecified.Linux PVH Boot head.S:
	 *  - `tr`: must be a 32-bit TSS (active) with a base of '0' and a limit
	 *          of '0x67'.
	 *  - `eflags`: bit 17 (VM) must be cleared. Bit 9 (IF) must be cleared.
	 *              Bit 8 (TF) must be cleared. Other bits are all unspecified.
	 * 
	 *  All other processor registers and flag bits are unspecified. The OS is in
	 *  charge of setting up it's own stack, GDT and IDT.
	 */
	.code32
	.section .text.start
/* 
 * #define PVH_GDT_ENTRY_CS	1
 * #define PVH_GDT_ENTRY_DS	2
 * #define PVH_CS_SEL		(PVH_GDT_ENTRY_CS * 8)
 * #define PVH_DS_SEL		(PVH_GDT_ENTRY_DS * 8)
 * #define PAE_BIT			(1 << 5)
 */
	.global _start
_start:
	/* How does linux handle GDT address?
	 * https://github.com/torvalds/linux/blob/0adb32858b0bddf4ada5f364a84ed60b196dbcda/arch/x86/boot/compressed/head_64.S#L540
	 */
	cld
	lgdt gdt64 /* Use _pa(gdt64) here? */

	/* Set up segment registers, which we won't use in favor of paging. */
	.extern PVH_DS_SEL
	mov PVH_DS_SEL, eax
	mov eax, ds
	mov eax, es
	mov eax, ss

	/* Set up the stack. */
	.extern _rust_start
	mov _STACK_TOP, esp
	push ebx
	call _rust_start

	/* Set up the page tables for long mode. */
	.extern PAE_BIT
	mov cr4, eax
	or PAE_BIT, eax
	mov eax, cr4 /* Enable PAE by setting 5th bit in cr4. */

	.section .rodata
/* 64-bit GDT  from Linux */
gdt64:
    .word    gdt_end - gdt
    .long    0
    .word    0
    .quad	 0
gdt:
    .word    gdt_end - gdt
    .long    gdt
    .word    0
    .quad    0x00cf9a000000ffff    /* __KERNEL32_CS */
    .quad    0x00af9a000000ffff    /* __KERNEL_CS */
    .quad    0x00cf92000000ffff    /* __KERNEL_DS */
    .quad    0x0080890000000000    /* TS descriptor */
    .quad    0x0000000000000000    /* TS continued */
gdt_end:
