use core::arch::asm;

use lazy_static::lazy_static;
use log::info;

use crate::{println, sync::UPSafeCell, trap::context::TrapContext};

const USER_STACK_SIZE: usize = 4096 * 2;
const KERNEL_STACK_SIZE: usize = 4096 * 2;
const MAX_APP_NUM: usize = 16;
const APP_BASE_ADDRESS: usize = 0x80400000;
const APP_SIZE_LIMIT: usize = 0x20000;

#[repr(align(4096))]
struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE],
}

#[repr(align(4096))]
struct UserStack {
    data: [u8; USER_STACK_SIZE],
}

static KERNEL_STACK: KernelStack = KernelStack {
    data: [0; KERNEL_STACK_SIZE],
};

static USER_STACK: UserStack = UserStack {
    data: [0; USER_STACK_SIZE],
};

impl KernelStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }

    fn push_context(&self, cx: TrapContext) -> &'static mut TrapContext {
        // 栈向低地址增长，现场起点是栈顶减去 TrapContext 的大小。
        let cx_ptr = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        // 把初始寄存器状态写到这个地址，供 __restore 按偏移读取。
        unsafe {
            *cx_ptr = cx;
        }
        // as_mut 只按空指针返回 None。创建可变引用还要求这段内存可独占访问。
        unsafe { cx_ptr.as_mut().unwrap() }
    }
}

impl UserStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}

struct AppManager {
    num_app: usize,
    current_app: usize,
    app_start: [usize; MAX_APP_NUM + 1],
}

impl AppManager {
    pub fn print_app_info(&self) {
        info!("Number of apps: {}", self.num_app);
        for i in 0..self.num_app {
            info!(
                "App {}: [{:#010x}, {:#010x}) ({} bytes)",
                i,
                self.app_start[i],
                self.app_start[i + 1],
                self.app_start[i + 1] - self.app_start[i]
            );
        }
    }
    unsafe fn load_app(&self, app_id: usize) {
        if app_id >= self.num_app {
            panic!("Invalid app_id: {}", app_id);
        }
        println!(
            "[Kernel] Loading app {} at address {:#x}",
            app_id, self.app_start[app_id]
        );

        // 清零从 0x80400000 开始的 128 KiB 执行区，擦除上一个 App 的镜像和数据。
        unsafe {
            core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, APP_SIZE_LIMIT).fill(0);
        }

        // 相邻边界给出当前 App 在内核 .rodata 中的起点和长度。
        let app_src = unsafe {
            core::slice::from_raw_parts(
                self.app_start[app_id] as *const u8,
                self.app_start[app_id + 1] - self.app_start[app_id],
            )
        };

        // 只借用执行区中与当前镜像等长的范围，再把镜像复制进去。
        let app_dst =
            unsafe { core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, app_src.len()) };
        app_dst.copy_from_slice(app_src);

        unsafe {
            asm!("fence.i");
        }
    }

    fn get_current_app(&self) -> usize {
        self.current_app
    }

    fn move_to_next_app(&mut self) {
        self.current_app += 1;
    }
}

lazy_static! {
    static ref APP_MANAGER: UPSafeCell<AppManager> = unsafe {
        UPSafeCell::new({
            unsafe extern "C" {
                safe fn _num_app();
            }
            let num_app_ptr = linker_symbol_addr!(_num_app) as *const usize;
            let num_app = num_app_ptr.read_volatile();
            let mut app_start: [usize; MAX_APP_NUM + 1] = [0; MAX_APP_NUM + 1];
            let app_start_raw: &[usize] =
                core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1);
            app_start[..=num_app].copy_from_slice(app_start_raw);
            AppManager {
                num_app,
                current_app: 0,
                app_start,
            }
        })
    };
}

pub fn print_app_info() {
    APP_MANAGER.exclusive_access().print_app_info();
}

pub fn init() {
    print_app_info();
}

pub fn run_next_app() -> ! {
    let mut app_manager = APP_MANAGER.exclusive_access();
    let current_app = app_manager.get_current_app();
    unsafe {
        app_manager.load_app(current_app);
    }
    app_manager.move_to_next_app();
    drop(app_manager);

    unsafe extern "C" {
        fn __restore(cx_addr: usize);
    }

    // 构造 App 的初始现场：入口是 APP_BASE_ADDRESS，x[2] 是用户栈顶，SPP 指定 U-mode。
    // 把现场放到内核栈顶下方，将它的地址传给 __restore，由 sret 启动 App。
    unsafe {
        __restore(KERNEL_STACK.push_context(TrapContext::app_init_context(
            APP_BASE_ADDRESS,
            USER_STACK.get_sp(),
        )) as *const _ as usize);
    }
    panic!("Unreachable in batch::run_current_app!");
}
