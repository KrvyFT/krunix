#!/usr/bin/env bash
set -euo pipefail

# Cargo passes <target-dir>/<triple>/<profile>/krunix as the first argument.
profile_dir=$(dirname -- "$1")
shift
target_output_dir=$(dirname -- "$profile_dir")
target_dir=$(dirname -- "$target_output_dir")
elf="$target_output_dir/release/krunix"
image="$elf.bin"

"${CARGO:-cargo}" build --release --target riscv64gc-unknown-none-elf --target-dir "$target_dir"

tmp_image=$(mktemp "$image.XXXXXX")
trap 'rm -f -- "$tmp_image"' EXIT
llvm-objcopy --strip-all -O binary "$elf" "$tmp_image"
mv -f -- "$tmp_image" "$image"
trap - EXIT

exec qemu-system-riscv64 \
    -machine virt \
    -nographic \
    -bios bootloader/rustsbi-qemu.bin \
    -device "loader,file=$image,addr=0x80200000" \
    "$@"
