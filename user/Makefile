CARGO ?= cargo
LLVM_OBJCOPY ?= llvm-objcopy

TARGET := riscv64gc-unknown-none-elf
TARGET_DIR ?= ../target
RELEASE_DIR := $(TARGET_DIR)/$(TARGET)/release
BINS := $(patsubst src/bin/%.rs,$(RELEASE_DIR)/%.bin,$(wildcard src/bin/*.rs))

.PHONY: build elf
.DELETE_ON_ERROR:

build: $(BINS)

elf:
	$(CARGO) build --manifest-path Cargo.toml --bins --profile release --target $(TARGET) --target-dir "$(TARGET_DIR)"

$(BINS): %.bin: elf
	$(LLVM_OBJCOPY) --strip-all -O binary "$(@:.bin=)" "$@"
