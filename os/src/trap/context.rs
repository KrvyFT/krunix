use riscv::register::sstatus::{self, SPP, Sstatus};

#[repr(C)]
pub struct TrapContext {
    // 0x00
    pub x: [usize; 32],
    // 0x100
    pub sstatus: Sstatus,
    // 0x108
    pub sepc: usize,
}

impl TrapContext {
    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }

    pub fn app_init_context(entry: usize, sp: usize) -> Self {
        let mut sstatus = sstatus::read();
        sstatus.set_spp(SPP::User);
        let cx = TrapContext {
            x: [0; 32],
            sstatus,
            sepc: entry,
        };
        let mut cx = cx;
        cx.set_sp(sp);
        cx
    }
}
