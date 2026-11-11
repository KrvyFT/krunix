use core::arch::global_asm;

use riscv::{
    interrupt::{Exception, Interrupt, Trap},
    register::{scause, stval, stvec},
};

use crate::{batch::run_next_app, println, syscall::syscall, trap::context::TrapContext};

pub mod context;

global_asm!(include_str!("trap.S"));

pub fn init() {
    unsafe extern "C" {
        safe fn __alltraps();
    }
    unsafe {
        stvec::write(stvec::Stvec::new(
            linker_symbol_addr!(__alltraps),
            stvec::TrapMode::Direct,
        ));
    }
}

#[unsafe(no_mangle)]
pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    let scause = scause::read();
    let stval = stval::read();
    let trap: Trap<Interrupt, Exception> = match scause.cause().try_into() {
        Ok(trap) => trap,
        Err(_) => panic!(
            "Unsupported trap {:?}, stval = {:#x}!",
            scause.cause(),
            stval
        ),
    };

    match trap {
        Trap::Exception(Exception::UserEnvCall) => {
            cx.sepc += 4;
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
        }
        Trap::Exception(Exception::StoreFault) | Trap::Exception(Exception::StorePageFault) => {
            println!(
                "[kernel] Page fault in application, bad addr = {:#x}, bad instruction = {:#x}, core dumped!",
                stval, cx.sepc
            );
            run_next_app();
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            println!(
                "[kernel] Illegal instruction in application, bad instruction = {:#x}, core dumped!",
                cx.sepc
            );
            run_next_app();
        }
        _ => panic!(
            "[kernel] Unsupported trap {:?}, stval = {:#x}, core dumped!",
            scause.cause(),
            stval
        ),
    }

    cx
}
