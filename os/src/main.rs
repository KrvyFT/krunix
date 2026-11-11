#![no_std]
#![no_main]

use core::arch::global_asm;

use log::info;

macro_rules! linker_symbol_addr {
    ($symbol:path) => {
        ($symbol as *const ()).addr()
    };
}

mod batch;
mod console;
mod lang_items;
mod sbi;
mod sync;
mod syscall;
mod trap;

global_asm!(include_str!("arch/riscv64/entry.asm"));
global_asm!(include_str!(concat!(env!("OUT_DIR"), "/link_app.S")));

#[unsafe(no_mangle)]
pub extern "C" fn rust_main() {
    clear_bss();
    console::init_logger();
    trap::init();
    print_init_info();
    batch::init();
    batch::run_next_app();
}

fn print_init_info() {
    unsafe extern "C" {
        safe fn stext();
        safe fn etext();
        safe fn srodata();
        safe fn erodata();
        safe fn sdata();
        safe fn edata();
        safe fn sbss();
        safe fn ebss();
        safe fn boot_stack_lower_bound();
        safe fn boot_stack_top();
        safe fn skernel();
        safe fn ekernel();
    }

    info!(
        "{} {} booting (riscv64, S-mode)",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );
    info!(
        "kernel image: [{:#010x}, {:#010x})",
        linker_symbol_addr!(skernel),
        linker_symbol_addr!(ekernel)
    );
    for (section, start, end) in [
        (
            ".text",
            linker_symbol_addr!(stext),
            linker_symbol_addr!(etext),
        ),
        (
            ".rodata",
            linker_symbol_addr!(srodata),
            linker_symbol_addr!(erodata),
        ),
        (
            ".data",
            linker_symbol_addr!(sdata),
            linker_symbol_addr!(edata),
        ),
        (
            ".bss (zeroed)",
            linker_symbol_addr!(sbss),
            linker_symbol_addr!(ebss),
        ),
        (
            "boot stack",
            linker_symbol_addr!(boot_stack_lower_bound),
            linker_symbol_addr!(boot_stack_top),
        ),
    ] {
        info!(
            "{section}: [{start:#010x}, {end:#010x}) ({} bytes)",
            end - start
        );
    }

    let stvec = riscv::register::stvec::read();
    info!(
        "trap vector: {:#010x} ({:?})",
        stvec.address(),
        stvec.trap_mode()
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
