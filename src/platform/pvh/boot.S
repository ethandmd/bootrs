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
	 *                unspecified.
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
	.global _start
_start:
	/* 
	 * Reference: https://github.com/torvalds/linux/blob/v4.16/arch/x86/boot/compressed/head_64.S
	 * Enable Long mode:
	 * 1. Setup page tables
	 * 2. Enable PAE
	 * 3. Set Long mode enable bit (0x8) in EFER MSR (extended feature enable register @ 0xc0000080)
	 * 4. Enable paging
	 * 5. Load GDT
	 */

	/* 1. Setup page tables. */
	/* Let's use 2MiB hugepages for early boot identity paging of 1st 1GiB in case of small VMs. */

	/* pml4_table[0] = pdp_table */
	mov $pdp_table, %eax
	or $0x3, %eax /* Set present and writable bits. */
	mov %eax, [pml4_table]

	/* pdp_table[0] = pd_table */
	mov $pd_table, %eax
	or $0x3, %eax /* Set present and writable bits. */
	mov %eax, [pdp_table]
	
	/* Map all 512 PTEs in pd_table with 2MiB hugepages. */
	/* The reach of the pd_table is 512 * 2MiB = 1GiB. */
	xor %ecx, %ecx
	mov $pd_table, %edi
	map_pd_table:
		mov $0x200000, %eax /* 2MiB */
		mul %ecx
		or $0x83, %eax /* Set present, writable and 2MiB hugepage bits. */
		mov %eax, (%edi) /* Set PTE address in pd_table at offset. */
		add $0x8, %edi
		inc %ecx
		cmp $0x200, %ecx
		jne map_pd_table
	
	/* Set PML4 base address in %cr3 */
	mov $pml4_table, %eax
	mov %eax, %cr3

	/* 2. Enable PAE. */
	mov %cr4, %eax
	orl $0x20, %eax
	mov %eax, %cr4 /* Enable PAE by setting 5th bit in cr4. */

	/* 3. Set Long mode enable bit (1 << 8) in EFER MSR (extended feature enable register @ 0xc0000080) */
	mov $0xc0000080, %ecx
	rdmsr
	or $0x100, %eax
	wrmsr

	/* Set paging bit in cr0. */
	mov %cr0, %eax /* From GDB I can tell that PE bit is set, as expected. */
	mov $0x80000000, %ecx /* Set paging bit (1 << 31). */
	or %ecx, %eax /* %eax should now be (1 << 31) | (1 << 0) = 0x80000001. */
	mov %eax, %cr0
	
	/* 5. Load GDT Segments. */
	cld
	lgdt [gdt] 
	/* 
	 * Reload segment registers.
	 * CS GDT entry is at offset 0x8.
	 * DS GDT entry is at offset 0x10.
	 */
	mov $0x10, %eax
	mov %eax, %ds
	mov %eax, %es
	mov %eax, %ss
	jmp 1f /* Jump to 64-bit code. */
	.code64
1:  
	/* Set up the stack. */
	mov $boot_stack_top, %eax
	mov %eax, %esp
	mov %esp, %ebp

	/* Set up the stack canary.
	 * mov $0xc0000101, %eax
	 * mov $canary, %eax
	 * xor %edx, %edx /* GS_BASE MSR reads from EDX:EAX...hence clear EDX.
	 * wrmsr
	 */

	/* Jump to 64-bit rust entry point. */
	.extern _rust_start
	mov %rbx, %rdi /* Pass boot info structure to rust in %rdi. */
	jmp _rust_start

	.section .rodata
	.balign 8
	/* Linux GDT64: https://github.com/torvalds/linux/blob/v4.16/arch/x86/boot/compressed/head_64.S
	 * GDT segment layout from Linux and https://wiki.osdev.org/GDT_Tutorial
	 */
gdt:
	.word gdt_end - gdt_start
	.long gdt_start
	.word 0x0000
gdt_start:
	.quad 0x00000000000000 /* Null segment */
	.quad 0xaf9a0000000000 /* 64-bit code segment */
	.quad 0xcf92000000ffff /* 64-bit data segment */
gdt_end:

/* canary:
 *	.fill 48, 1, 0
 */
	
	/* https://web.mit.edu/gnu/doc/html/as_7.html */
	.section .bss
.align 4096
pml4_table:
	.fill 4096, 1, 0
pdp_table:
	.fill 4096, 1, 0
pd_table:
	.fill 4096, 1, 0
boot_stack_bottom:
	.fill 0x4000, 1, 0 /* 4KiB early boot stack. */
boot_stack_top:
