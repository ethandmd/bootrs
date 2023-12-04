LIBUNIFIRE=target/x86_64-unknown-none/debug/libunifire.a

ASM-GCC=gcc

LINKER=ld

%.o: %.s
	$(ASM-GCC) -c $^ -o $@

src/platform/pvh/start.o: src/platform/pvh/start.S

$(LIBUNIFIRE): .FORCE
	cargo build

build: src/platform/pvh/pvhboot.ld $(LIBUNIFIRE) src/platform/pvh/start.o .FORCE
	$(LINKER) -Tsrc/platform/pvh/pvhboot.ld -o unifire.ELF \
		src/platform/pvh/start.o $(LIBUNIFIRE)

clean:
	cargo clean
	rm -f *.o *.ELF
	rm -f src/platform/pvh/*.o

.PHONY: .FORCE
