TARGET := riscv64gc-unknown-none-elf
MODE ?= release

ifeq ($(MODE),release)
CARGO_MODE := --release
else
CARGO_MODE :=
endif

KERNEL_ELF := target/$(TARGET)/$(MODE)/tinyos-rv64
KERNEL_BIN := target/$(TARGET)/$(MODE)/kernel.bin

.PHONY: all build bin run debug gdb clean

all: bin

build:
	cargo build $(CARGO_MODE)

bin: build
	# Requires: cargo install cargo-binutils && rustup component add llvm-tools-preview
	cargo objcopy $(CARGO_MODE) -- -O binary $(KERNEL_BIN)

run: bin
	qemu-system-riscv64 -machine virt -nographic -bios default -kernel $(KERNEL_BIN)

debug: bin
	# QEMU waits for gdb at tcp::1234
	qemu-system-riscv64 -machine virt -nographic -bios default -kernel $(KERNEL_BIN) -s -S

gdb:
	# You may need: sudo apt install gdb-multiarch
	gdb-multiarch -ex "file $(KERNEL_ELF)" -ex "target remote :1234"

clean:
	cargo clean
