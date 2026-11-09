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
    println!("Hello World");
    sbi::shutdown(false);
}

fn clear_bss() {
    unsafe extern "C" {
        safe fn sbss();
        safe fn ebss();
    }
    (linker_symbol_addr!(sbss)..linker_symbol_addr!(ebss))
        .for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}
