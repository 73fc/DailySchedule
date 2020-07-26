#![no_std]
#![no_main]
#![feature(llvm_asm)]
#![feature(global_asm)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(naked_functions)]
#![feature(slice_fill)]

#[macro_use]
mod console;
mod panic;
mod sbi;
mod interrupt;
mod memory;
extern crate alloc;
mod process;
mod drivers;
mod fs;
mod kernel;

use alloc::sync::Arc;
use fs::{INodeExt, ROOT_INODE};
use memory::PhysicalAddress;
use process::*;
use xmas_elf::ElfFile;

// 汇编编写的程序入口，具体见该文件
global_asm!(include_str!("entry.asm"));

/// 加上 #[no_mangle] 告诉编译器对于此函数禁用编译期间的名称重整（Name Mangling），即确保编译器生成一个名为 _start 的函数
#[no_mangle]
pub extern "C" fn rust_main(_hart_id: usize, dtb_pa: PhysicalAddress) -> ! {
    memory::init();
    interrupt::init();
    drivers::init(dtb_pa);
    
    println!("{} {}",_hart_id,dtb_pa);
    panic!("end of rust_main")
}

/// 内核线程需要调用这个函数来退出
fn kernel_thread_exit() {
    // 当前线程标记为结束
    PROCESSOR.lock().current_thread().as_ref().inner().dead = true;
    // 制造一个中断来交给操作系统处理
    unsafe { llvm_asm!("ebreak" :::: "volatile") };
}

/// 测试任何内核线程都可以操作文件系统和驱动
fn simple(id: usize) {
    println!("hello from thread id {}", id);
    // 新建一个目录
    fs::ROOT_INODE
        .create("tmp", rcore_fs::vfs::FileType::Dir, 0o666)
        .expect("failed to mkdir /tmp");
    // 输出根文件目录内容
    //fs::ls("/");

    loop {}
}