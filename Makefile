LIBUNIFIRE=target/x86_64-unknown-none/debug/libunifire.a

ASM-GCC=gcc

LINKER=ld

%.o: %.s
	$(ASM-GCC) -c $^ -o $@

start.o: src/platform/pvh/start.S

$(LIBUNIFIRE): .FORCE
	cargo build

build: src/platform/pvh/pvhboot.ld $(LIBUNIFIRE) start.o .FORCE
	$(LINKER) -Tsrc/platform/pvh/pvhboot.ld -o unifire.ELF \
		start.o $(LIBUNIFIRE)

clean:
	cargo clean
	rm -f *.o *.ELF

.PHONY: .FORCE
