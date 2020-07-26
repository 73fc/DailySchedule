use core::fmt;
use core::mem::zeroed;
use riscv::register::{sstatus::Sstatus, scause::Scause};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Context {
    pub x: [usize; 32],     // 32 个通用寄存器
    pub sstatus: Sstatus,  //状态位，控制全局中断使能
    pub sepc: usize       //记录触发中断的指令的地址
}

impl Default for Context {
    fn default() -> Self {
        unsafe { zeroed() }
    }
}

/// 格式化输出
///
/// # Example
///
/// ```rust
/// println!("{:x?}", Context);   // {:x?} 表示用十六进制打印其中的数值
/// ```
impl fmt::Debug for Context {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Context")
            .field("registers", &self.x)
            .field("sstatus", &self.sstatus)
            .field("sepc", &self.sepc)
            .finish()
    }
}