#![no_std]

macro_rules! linker_symbol_addr {
    ($symbol:path) => {
        ($symbol as *const ()).addr()
    };
}

#[macro_use]
pub mod console;
mod lang_items;
mod syscall;

unsafe extern "Rust" {
    fn main() -> i32;
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
extern "C" fn _start() -> ! {
    clear_bss();
    unsafe {
        exit(main());
    }
    panic!("unreachable after sys_exit!");
}

fn clear_bss() {
    unsafe extern "C" {
        safe fn sbss();
        safe fn ebss();
    }
    (linker_symbol_addr!(sbss)..linker_symbol_addr!(ebss))
        .for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}

pub fn write(fd: usize, buf: &[u8]) -> isize {
    syscall::sys_write(fd, buf)
}

pub fn exit(xstate: i32) -> isize {
    syscall::sys_exit(xstate)
}
