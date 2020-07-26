#![allow(dead_code)]
mod heap;
mod address;
pub mod config;
pub mod range;
pub mod frame;

pub type MemoryResult<T> = Result<T, &'static str>;

pub use {
    address::*,
    config::*,
    frame::FRAME_ALLOCATOR,
    // mapping::{Flags, MapType, MemorySet, Segment},
    range::Range,
};

pub fn init() {
    heap::init();
    // 允许内核读写用户态内存
    unsafe { riscv::register::sstatus::set_sum() };

    println!("mod memory initialized");
}