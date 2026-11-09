CARGO ?= cargo
LLVM_OBJCOPY ?= llvm-objcopy
QEMU ?= qemu-system-riscv64

TARGET := riscv64gc-unknown-none-elf
TARGET_DIR := target
RELEASE_IMAGE := $(TARGET_DIR)/$(TARGET)/release/krunix.bin
DEBUG_IMAGE := $(TARGET_DIR)/$(TARGET)/debug/krunix.bin
QEMU_BOOT := -machine virt -nographic -bios bootloader/rustsbi-qemu.bin

.PHONY: build run debug FORCE
.DELETE_ON_ERROR:

build: $(RELEASE_IMAGE)

run: $(RELEASE_IMAGE)
	$(QEMU) $(QEMU_BOOT) -device "loader,file=$<,addr=0x80200000" $(QEMU_FLAGS)

debug: $(DEBUG_IMAGE)
	$(QEMU) $(QEMU_BOOT) -device "loader,file=$<,addr=0x80200000" -S \
		-chardev socket,id=lldb,host=127.0.0.1,port=1234,server=on,wait=on \
		-gdb chardev:lldb $(QEMU_FLAGS)

$(RELEASE_IMAGE): PROFILE := release
$(DEBUG_IMAGE): PROFILE := dev

# Let Cargo detect source and compile-time environment changes, including LOG.
$(RELEASE_IMAGE) $(DEBUG_IMAGE): FORCE
	$(CARGO) build --profile $(PROFILE) --target $(TARGET) --target-dir "$(TARGET_DIR)"
	$(LLVM_OBJCOPY) --strip-all -O binary "$(@:.bin=)" "$@"
