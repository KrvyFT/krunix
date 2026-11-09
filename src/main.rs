#![no_std]
#![no_main]

use core::arch::global_asm;

mod console;
mod lang_items;
mod sbi;

global_asm!(include_str!("arch/riscv64/entry.asm"));

macro_rules! linker_symbol_addr {
    ($symbol:path) => {
        ($symbol as *const ()).addr()
    };
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_main() {
    clear_bss();
    print_kernel_layout();
    println!("Hello World");
    sbi::shutdown(false);
}

fn print_kernel_layout() {
    unsafe extern "C" {
        safe fn stext();
        safe fn etext();
        safe fn srodata();
        safe fn erodata();
        safe fn sdata();
        safe fn edata();
        safe fn skernel();
        safe fn ekernel();
    }
    info!(
        "image layout: [{:#8x} - {:#8x}], start: {:#8x}",
        linker_symbol_addr!(skernel),
        linker_symbol_addr!(ekernel),
        linker_symbol_addr!(skernel)
    );
    info!(
        ".text: [{:#8x} - {:#8x}]",
        linker_symbol_addr!(stext),
        linker_symbol_addr!(etext)
    );
    info!(
        ".rodata: [{:#8x} - {:#8x}]",
        linker_symbol_addr!(srodata),
        linker_symbol_addr!(erodata)
    );
    info!(
        ".data: [{:#8x} - {:#8x}]",
        linker_symbol_addr!(sdata),
        linker_symbol_addr!(edata)
    );
}

fn clear_bss() {
    unsafe extern "C" {
        safe fn sbss();
        safe fn ebss();
    }
    (linker_symbol_addr!(sbss)..linker_symbol_addr!(ebss))
        .for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}
