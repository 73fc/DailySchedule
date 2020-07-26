use crate::sbi::set_timer;
use riscv::register::{time, sie, sstatus};

pub fn init() {
    unsafe {
        sie::set_stimer();         
        sstatus::set_sie();
    }
    set_next_timeout();
}

static INTERVAL: usize = 100000;

fn set_next_timeout() {
    set_timer(time::read() + INTERVAL);
}

/// 触发时钟中断计数
pub static mut TICKS: usize = 0;

/// 每一次时钟中断时调用
/// 
/// 设置下一次时钟中断，同时计数 +1
pub fn tick() {
    set_next_timeout();
    unsafe {
        TICKS += 1;
        if TICKS % 100 == 0 {
            println!("{} tick", TICKS);
        }
    }
}