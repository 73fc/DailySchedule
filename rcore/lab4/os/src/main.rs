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
pub extern "C" fn rust_main() -> ! {
    println!("Hello rCore-Tutorial !!!");
    
    // 初始化各种模块
    interrupt::init();
    memory::init();

    test_restore_thread();
    
    panic!("end of rust_main")
}

/// 内核线程需要调用这个函数来退出
fn kernel_thread_exit() {
    // 当前线程标记为结束
    PROCESSOR.lock().current_thread().as_ref().inner().dead = true;
    // 制造一个中断来交给操作系统处理
    unsafe { llvm_asm!("ebreak" :::: "volatile") };
}

/// 创建一个内核进程
pub fn create_kernel_thread(
    process: Arc<Process>,
    entry_point: usize,
    arguments: Option<&[usize]>,
) -> Arc<Thread> {
    // 创建线程
    let thread = Thread::new(process, entry_point, arguments).unwrap();
    // 设置线程的返回地址为 kernel_thread_exit
    thread.as_ref().inner().context.as_mut().unwrap()
        .set_ra(kernel_thread_exit as usize);
    thread
}
fn sample_process(message: usize) -> usize {
    println!("hello from kernel thread {}", message);
    message
}

pub fn test_restore_thread(){
    {
        let mut processor = PROCESSOR.lock();
        // 创建一个内核进程
        let kernel_process = Process::new_kernel().unwrap();
        // 为这个进程创建多个线程，并设置入口均为 sample_process，而参数不同
        for i in 1..9usize {
            processor.add_thread(create_kernel_thread(
                kernel_process.clone(),
                sample_process(i as usize),
                Some(&[i]),
            ));
        }
    }

    extern "C" {
        fn __restore(context: usize);
    }
    // 获取第一个线程的 Context
    let context = PROCESSOR.lock().prepare_next_thread();
    // 启动第一个线程
    unsafe { __restore(context as usize) };

    // unreachable!()
}
