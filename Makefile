TARGET_DIR ?= target

.PHONY: build kernel user run debug

build: kernel

kernel:
	$(MAKE) -C os build TARGET_DIR="$(abspath $(TARGET_DIR))"

user:
	$(MAKE) -C user build TARGET_DIR="$(abspath $(TARGET_DIR))"

run debug:
	$(MAKE) -C os $@ TARGET_DIR="$(abspath $(TARGET_DIR))"
