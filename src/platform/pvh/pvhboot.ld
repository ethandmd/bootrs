/* Reference GNU Linker Script: https://sourceware.org/binutils/docs/ld/PHDRS.html */
/* Reference QEMU test: https://gitlab.com/qemu-project/qemu/-/blob/master/tests/tcg/x86_64/system/kernel.ld?ref_type=heads */
/* name type [ FILEHDR ] [ PHDRS ] [ AT ( address ) ] [ FLAGS ( flags ) ] ; */
PHDRS
{
	text PT_LOAD;
	rodata PT_LOAD;
	data PT_LOAD;
	bss PT_LOAD;
	note PT_NOTE;
}

ENTRY(_start)
SECTIONS
{
	/* Low RAM start: 0x0 */
	/* High RAM start: 0x10_0000 */
	/* 64-bit RAM start: 0x1_0000_0000 */
	. = 0x100000;
	_KERNEL_BASE = .;
	.text :
	{
		*(.text.start, .text.start.*)
		*(.text .text.*)
		. = ALIGN(4096);
	} :text

	.rodata :
	{
		*(.rodata .rodata.*)
		. = ALIGN(4096);
	} :rodata

	.data :
	{
		*(.data .data.*)
		. = ALIGN(4096);
	} :data
	
	.bss :
	{
		. = ALIGN(4096);
		/*_STACK_BOTTOM = .;
		 * . = . + 4096;
		 *_STACK_TOP = .;
		 */
		*(.bss .bss.*)
		. = ALIGN(4096);
	} :bss

	.note :
	{
		*(.note .note.*)
	} :note
}
