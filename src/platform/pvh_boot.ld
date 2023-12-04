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
	. = 0x100000; /* 1MB */
	.text :
	{
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
		STACK_BOTTOM = .;
		. += 4096; /* Create our own single page stack */
		STACK_TOP = .;
		*(.bss .bss.*)
		. = ALIGN(4096);
	} :bss

	.note :
	{
		*(.note .note.*)
	} :note
}
