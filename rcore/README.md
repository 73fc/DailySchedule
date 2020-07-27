# 实验报告

博客目录：
1.[环境部署][3] 
2.[lab0实验报告][4]
3.[lab1实验报告][5]
4.lab2实验报告
4.lab3实验报告
(lab4-lab5实验报告暂未完成。)
---
# lab0
本实验主要是为了建立os项目，生成内核镜像并在qemu上运行成功。
实验完成后目录结构如下：
```
Project
│  rust-toolchain
│
└─os
    │  .gitignore
    │  Cargo.lock
    │  Cargo.toml
    │  Makefile
    │
    ├─.cargo
    │      config
    │
    └─src
            console.rs
            entry.asm
            linker.ld
            main.rs
            panic.rs
            sbi.rs
```
[lab0实验指导书][1]

## 一，创建项目：
   __第 1 步__：建立Project文件夹，所有文件均保存在其中。

   __第 2 步__：在Project文件夹下建立名为rust-toolchain的文件，指定工具链版本。
```
//rust-toolchain
nightly-2020-06-27
```
   __第 3 步__：运行cargo new os创建os文件夹，具体的os项目在此文件夹中开发。
   此时目录结构如下：
```
Project                 项目目录
├── rust-toolchain      Rust 工具链版本
└── os
    ├── Cargo.toml      项目配置文件
    └── src             源代码路径
        └── main.rs     源程序
``` 
 ## 二，移除环境依赖
因为是自写操作系统，无法使用依赖现成操作系统任何环境，所以必须移除对当前操作系统下的标准库依赖以及运行时环境依赖

 ### 2.1 移除标准库依赖
将std标准库禁用并依赖core库编写代码。

   __第 1 步__，通过#![no_std]将其显式禁用。

 ```
 //os/src/main.rs
 //! # 全局属性
//! - `#![no_std]`  
//!   禁用标准库
#![no_std]

fn main() {
    println!("Hello, rCore-Tutorial!");
}
 ```  

由于println!宏以及默认的panic处理函数均属于std库，所以上述程序无法成功编译。

此外还遇到了语义项（Language Item）的错误，其为编译器内部所需的特殊函数或类型。
禁止std后无法找到eh_personality语义项。
它是一个标某函数用来实现堆栈展开处理功能的语义项。这个语义项也与 panic 有关。


__第 2 步__ 处理上述问题方法为：
1. 删除println!语句。
1. 依赖core库自写panic函数。暂时可在main.rs加入一下代码
    ```
    //os/src/main.rs
    use core::panic::PanicInfo;

    /// 当 panic 发生时会调用该函数
    /// 我们暂时将它的实现为一个死循环
    #[panic_handler]
    fn panic(_info: &PanicInfo) -> ! {
        loop {}
    }
    ```
1. 在项目配置文件中直接将 dev 配置和 release 配置的 panic 的处理策略设为直接终止，不进行堆栈展开。

```
//os/Cargo.toml
...

# panic 时直接终止，因为我们没有实现堆栈展开的功能
[profile.dev]
panic = "abort"

[profile.release]
panic = "abort"
```

### 2.2 移除运行时环境依赖
运行时系统（Runtime System）是一种把半编译的运行码在目标机器上运行的环境，大多数语言在正式执行前都会先配置运行时环境。

Rust(链接标准库的情况下)运行路径为: C 语言运行时环境中的 crt0 --> Rust 运行时环境的入口点（Entry Point）-->main 主函数。

其中Rust 的运行时入口点就是被 start 语义项标记。
由于禁止了std标准库，所以crt0和start均无法使用。
所以必须重写。

__第 1 步__ 删除 main函数，加入_start()函数。将main.rs修改为如下：
```
//os/src/main.rs
//! # 全局属性
//! - `#![no_std]`  
//!   禁用标准库
#![no_std]
//!
//! - `#![no_main]`  
//!   不使用 `main` 函数等全部 Rust-level 入口点来作为程序入口
#![no_main]

use core::panic::PanicInfo;

/// 当 panic 发生时会调用该函数
/// 我们暂时将它的实现为一个死循环
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

/// 覆盖 crt0 中的 _start 函数
/// 我们暂时将它的实现为一个死循环
#[no_mangle]
pub extern "C" fn _start() -> ! {
    loop {}
}
```

## 三，编译为裸机目标
Rust 使用一个称为目标三元组（Target Triple）的字符串描述运行时环境内容为(CPU 架构、供应商、操作系统和二进制接口)。

在项目中运行： rustc --version --verbose 可查看当前环境。

实验环境为： riscv64imac-unknown-none-elf ，所以要按以下步骤配置：

__第 1 步__ 在命令行运行以下代码，下载相关环境的标准库和 core 库(此后又可使用标准库进行编程了)。
```
rustup target add riscv64imac-unknown-none-elf
```

__第 2 步__ 在 os 文件夹中创建一个 .cargo 文件夹，并在其中创建一个名为 config 的文件，在其中填入以下内容：
```
//os/.cargo/config
# 编译的目标平台
[build]
target = "riscv64imac-unknown-none-elf"
```
此后编译文件默认在目标环境下编译。

## 四，生成内核镜像

__第 1 步__ 安装 binutils 工具集,命令
```
cargo install cargo-binutils
rustup component add llvm-tools-preview
```
(反汇编的的结果与指导书不同，不过没什么问题应该)

__第 2 步__ 生成镜像
```
rust-objcopy target/riscv64imac-unknown-none-elf/debug/os --strip-all -O binary target/riscv64imac-unknown-none-elf/debug/kernel.bin
```
至此成功生成了镜像文件kernel.bin。


## 五，使用qemu运行内核
为了使镜像在qemu模拟器上运行，还需要完成两个工作：调整内存布局和重写入口函数。

## 5.1 调整内存布局
为对于普通用户程序来说，代码和数据一般放在低地址空间上的。对于 OS 内核，一般都将其地址空间放在高地址上。在 QEMU 模拟的 RISC-V 中，DRAM 内存的物理地址是从 0x80000000 开始，有 128MB 大小。
因为OpenSBI将自身放在 0x80000000，完成初始化后会跳转到 0x80200000，因此 _start 必须位于这个地址。

__第 1 步__ 编写链接脚本

创建文件 os/src/linker.ld
```
/* 有关 Linker Script 可以参考：https://sourceware.org/binutils/docs/ld/Scripts.html */

/* 目标架构 */
OUTPUT_ARCH(riscv)

/* 执行入口 */
ENTRY(_start)

/* 数据存放起始地址 */
BASE_ADDRESS = 0x80200000;

SECTIONS
{
    /* . 表示当前地址（location counter） */
    . = BASE_ADDRESS;

    /* start 符号表示全部的开始位置 */
    kernel_start = .;

    text_start = .;

    /* .text 字段 */
    .text : {
        /* 把 entry 函数放在最前面 */
        *(.text.entry)
        /* 要链接的文件的 .text 字段集中放在这里 */
        *(.text .text.*)
    }

    rodata_start = .;

    /* .rodata 字段 */
    .rodata : {
        /* 要链接的文件的 .rodata 字段集中放在这里 */
        *(.rodata .rodata.*)
    }

    data_start = .;

    /* .data 字段 */
    .data : {
        /* 要链接的文件的 .data 字段集中放在这里 */
        *(.data .data.*)
    }

    bss_start = .;

    /* .bss 字段 */
    .bss : {
        /* 要链接的文件的 .bss 字段集中放在这里 */
        *(.sbss .bss .bss.*)
    }

    /* 结束地址 */
    kernel_end = .;
}
```
__第 2 步__  使用链接脚本

在.cargo/config 文件中加入以下配置
```
# 使用我们的 linker script 来进行链接
[target.riscv64imac-unknown-none-elf]
rustflags = [
    "-C", "link-arg=-Tsrc/linker.ld",
]
```

## 5.2 重写程序入口点 _start

__第 1 步__ 设置内核的运行环境

创建os/src/entry.asm文件，内容如下：
```
# 操作系统启动时所需的指令以及字段
#
# 我们在 linker.ld 中将程序入口设置为了 _start，因此在这里我们将填充这个标签
# 它将会执行一些必要操作，然后跳转至我们用 rust 编写的入口函数
#
# 关于 RISC-V 下的汇编语言，可以参考 https://github.com/riscv/riscv-asm-manual/blob/master/riscv-asm.md

    .section .text.entry
    .globl _start
# 目前 _start 的功能：将预留的栈空间写入 $sp，然后跳转至 rust_main
_start:
    la sp, boot_stack_top
    call rust_main

    # 回忆：bss 段是 ELF 文件中只记录长度，而全部初始化为 0 的一段内存空间
    # 这里声明字段 .bss.stack 作为操作系统启动时的栈
    .section .bss.stack
    .global boot_stack
boot_stack:
    # 16K 启动栈大小
    .space 4096 * 16
    .global boot_stack_top
boot_stack_top:
    # 栈结尾
```

__第 2 步__ 设置主函数

将 os/src/main.rs 里面的 _start 函数删除，并换成 rust_main :
```
//! # 全局属性
//! - `#![no_std]`  
//!   禁用标准库
#![no_std]
//!
//! - `#![no_main]`  
//!   不使用 `main` 函数等全部 Rust-level 入口点来作为程序入口
#![no_main]
//!
//! - `#![feature(global_asm)]`  
//!   内嵌整个汇编文件
#![feature(global_asm)]

// 汇编编写的程序入口，具体见该文件
global_asm!(include_str!("entry.asm"));

use core::panic::PanicInfo;

/// 当 panic 发生时会调用该函数
/// 我们暂时将它的实现为一个死循环
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

/// Rust 的入口函数
///
/// 在 `_start` 为我们进行了一系列准备之后，这是第一个被调用的 Rust 函数
#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    loop {}
}
```



## 5.3 使用 QEMU 运行内核

__第 1 步__ 测试qemu
在命令行输入
```
$ qemu-system-riscv64 \
  --machine virt \
  --nographic \
  --bios default
```

若运行成功，则会看到openSBI的字样，接着可以使用 ctrl+a 再按下 x 键退出。 

__第 2 步__ 测试内核是否被加载
修改 os/src/main.rs如下
```
//! # 全局属性
//! - `#![no_std]`  
//!   禁用标准库
#![no_std]
//!
//! - `#![no_main]`  
//!   不使用 `main` 函数等全部 Rust-level 入口点来作为程序入口
#![no_main]
//! # 一些 unstable 的功能需要在 crate 层级声明后才可以使用
//! - `#![feature(llvm_asm)]`  
//!   内嵌汇编
#![feature(llvm_asm)]
//!
//! - `#![feature(global_asm)]`
//!   内嵌整个汇编文件
#![feature(global_asm)]

// 汇编编写的程序入口，具体见该文件
global_asm!(include_str!("entry.asm"));

use core::panic::PanicInfo;

/// 当 panic 发生时会调用该函数
/// 我们暂时将它的实现为一个死循环
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

/// 在屏幕上输出一个字符，目前我们先不用了解其实现原理
pub fn console_putchar(ch: u8) {
    let _ret: usize;
    let arg0: usize = ch as usize;
    let arg1: usize = 0;
    let arg2: usize = 0;
    let which: usize = 1;
    unsafe {
        llvm_asm!("ecall"
             : "={x10}" (_ret)
             : "{x10}" (arg0), "{x11}" (arg1), "{x12}" (arg2), "{x17}" (which)
             : "memory"
             : "volatile"
        );
    }
}

/// Rust 的入口函数
///
/// 在 `_start` 为我们进行了一系列准备之后，这是第一个被调用的 Rust 函数
#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    // 在屏幕上输出 "OK\n" ，随后进入死循环
    console_putchar(b'O');
    console_putchar(b'K');
    console_putchar(b'\n');

    loop {}
}
```
若看到在原openSBI输出下有OK的字眼则表示加载成功。

__第 3 步__ 配置 Makefile文件
在os目录下加入os/Makefile文件，方便后续编译生成项目文件，内容如下:
```
TARGET      := riscv64imac-unknown-none-elf
MODE        := debug
KERNEL_FILE := target/$(TARGET)/$(MODE)/os
BIN_FILE    := target/$(TARGET)/$(MODE)/kernel.bin

OBJDUMP     := rust-objdump --arch-name=riscv64
OBJCOPY     := rust-objcopy --binary-architecture=riscv64

.PHONY: doc kernel build clean qemu run

# 默认 build 为输出二进制文件
build: $(BIN_FILE) 

# 通过 Rust 文件中的注释生成 os 的文档
doc:
    @cargo doc --document-private-items

# 编译 kernel
kernel:
    @cargo build

# 生成 kernel 的二进制文件
$(BIN_FILE): kernel
    @$(OBJCOPY) $(KERNEL_FILE) --strip-all -O binary $@

# 查看反汇编结果
asm:
    @$(OBJDUMP) -d $(KERNEL_FILE) | less

# 清理编译出的文件
clean:
    @cargo clean

# 运行 QEMU
qemu: build
    @qemu-system-riscv64 \
            -machine virt \
            -nographic \
            -bios default \
            -device loader,file=$(BIN_FILE),addr=0x80200000

# 一键运行
run: build qemu
```

接下来的实验，均可以使用make run来用qemu加载内核镜像并运行


## 六，接口封装和代码整理

__第 1 步__ 使用 OpenSBI 提供的服务
建立os/src/sbi.rs文件，内容如下：
```
#![allow(unused)]

/// SBI 调用
#[inline(always)]
fn sbi_call(which: usize, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let ret;
    unsafe {
        llvm_asm!("ecall"
            : "={x10}" (ret)
            : "{x10}" (arg0), "{x11}" (arg1), "{x12}" (arg2), "{x17}" (which)
            : "memory"      // 如果汇编可能改变内存，则需要加入 memory 选项
            : "volatile");  // 防止编译器做激进的优化（如调换指令顺序等破坏 SBI 调用行为的优化）
    }
    ret
}

const SBI_SET_TIMER: usize = 0;
const SBI_CONSOLE_PUTCHAR: usize = 1;
const SBI_CONSOLE_GETCHAR: usize = 2;
const SBI_CLEAR_IPI: usize = 3;
const SBI_SEND_IPI: usize = 4;
const SBI_REMOTE_FENCE_I: usize = 5;
const SBI_REMOTE_SFENCE_VMA: usize = 6;
const SBI_REMOTE_SFENCE_VMA_ASID: usize = 7;
const SBI_SHUTDOWN: usize = 8;

/// 向控制台输出一个字符
///
/// 需要注意我们不能直接使用 Rust 中的 char 类型
pub fn console_putchar(c: usize) {
    sbi_call(SBI_CONSOLE_PUTCHAR, c, 0, 0);
}

/// 从控制台中读取一个字符
///
/// 没有读取到字符则返回 -1
pub fn console_getchar() -> usize {
    sbi_call(SBI_CONSOLE_GETCHAR, 0, 0, 0)
}

/// 调用 SBI_SHUTDOWN 来关闭操作系统（直接退出 QEMU）
pub fn shutdown() -> ! {
    sbi_call(SBI_SHUTDOWN, 0, 0, 0);
    unreachable!()
}
```

__第 2 步__ 实现格式化输出
将格式化输出的内容整理到os/src/console.rs，自写输入print 和 println。内容如下：
```
//! 实现控制台的字符输入和输出
//! 
//! # 格式化输出
//! 
//! [`core::fmt::Write`] trait 包含
//! - 需要实现的 [`write_str`] 方法
//! - 自带实现，但依赖于 [`write_str`] 的 [`write_fmt`] 方法
//! 
//! 我们声明一个类型，为其实现 [`write_str`] 方法后，就可以使用 [`write_fmt`] 来进行格式化输出
//! 
//! [`write_str`]: core::fmt::Write::write_str
//! [`write_fmt`]: core::fmt::Write::write_fmt

use crate::sbi::*;
use core::fmt::{self, Write};

/// 一个 [Zero-Sized Type]，实现 [`core::fmt::Write`] trait 来进行格式化输出
/// 
/// ZST 只可能有一个值（即为空），因此它本身就是一个单件
struct Stdout;

impl Write for Stdout {
    /// 打印一个字符串
    ///
    /// [`console_putchar`] sbi 调用每次接受一个 `usize`，但实际上会把它作为 `u8` 来打印字符。
    /// 因此，如果字符串中存在非 ASCII 字符，需要在 utf-8 编码下，对于每一个 `u8` 调用一次 [`console_putchar`]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let mut buffer = [0u8; 4];
        for c in s.chars() {
            for code_point in c.encode_utf8(&mut buffer).as_bytes().iter() {
                console_putchar(*code_point as usize);
            }
        }
        Ok(())
    }
}

/// 打印由 [`core::format_args!`] 格式化后的数据
/// 
/// [`print!`] 和 [`println!`] 宏都将展开成此函数
/// 
/// [`core::format_args!`]: https://doc.rust-lang.org/nightly/core/macro.format_args.html
pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

/// 实现类似于标准库中的 `print!` 宏
/// 
/// 使用实现了 [`core::fmt::Write`] trait 的 [`console::Stdout`]
#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
    }
}

/// 实现类似于标准库中的 `println!` 宏
/// 
/// 使用实现了 [`core::fmt::Write`] trait 的 [`console::Stdout`]
#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}
```

__第 3 步__  整理 panic 处理模块
将处理panic有关的内容提出单独放在os/src/panic.rs中，在添加内容成如下：
```
//! 代替 std 库，实现 panic 和 abort 的功能

use core::panic::PanicInfo;
use crate::sbi::shutdown;

/// 打印 panic 的信息并 [`shutdown`]
///
/// ### `#[panic_handler]` 属性
/// 声明此函数是 panic 的回调
#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    // `\x1b[??m` 是控制终端字符输出格式的指令，在支持的平台上可以改变文字颜色等等，这里使用红色
    // 参考：https://misc.flogisoft.com/bash/tip_colors_and_formatting
    //
    // 需要全局开启 feature(panic_info_message) 才可以调用 .message() 函数
    println!("\x1b[1;31mpanic: '{}'\x1b[0m", info.message().unwrap());
    shutdown()
}

/// 终止程序
/// 
/// 调用 [`panic_handler`]
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!("abort()")
}
```
__第 4 步__ 修改main.rs 
去掉输出以及panic有关的内容,将main.rs修改为如下：
```
//! # 全局属性
//! - `#![no_std]`  
//!   禁用标准库
#![no_std]
//!
//! - `#![no_main]`  
//!   不使用 `main` 函数等全部 Rust-level 入口点来作为程序入口
#![no_main]
//! # 一些 unstable 的功能需要在 crate 层级声明后才可以使用
//! - `#![feature(llvm_asm)]`  
//!   内嵌汇编
#![feature(llvm_asm)]
//!
//! - `#![feature(global_asm)]`  
//!   内嵌整个汇编文件
#![feature(global_asm)]
//!
//! - `#![feature(panic_info_message)]`  
//!   panic! 时，获取其中的信息并打印
#![feature(panic_info_message)]

#[macro_use]
mod console;
mod panic;
mod sbi;

// 汇编编写的程序入口，具体见该文件
global_asm!(include_str!("entry.asm"));

/// Rust 的入口函数
///
/// 在 `_start` 为我们进行了一系列准备之后，这是第一个被调用的 Rust 函数
#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    println!("Hello rCore-Tutorial!");
    println!("finish the lab0(fc)");
    panic!("end of rust_main")
}
```

使用make run，看到println!的内容正常输出，则lab0完成。
代码与运行结果图均在lab0项目文件夹中



# lab1
本实验目标是实现中断系统。
[lab1实验指导书][2]
实验完成后目录结构如下：
```
Project
│  rust-toolchain
│
└─os
    │  .gitignore
    │  Cargo.lock
    │  Cargo.toml
    │  Makefile
    │
    ├─.cargo
    │      config
    │
    └─src
        │  console.rs
        │  entry.asm
        │  linker.ld
        │  main.rs
        │  panic.rs
        │  sbi.rs
        │
        └─interrupt
                context.rs
                handler.rs
                interrupt.asm
                mod.rs
                timer.rs
```
## 一，中断原理简介
### 1.1 中断简介
中断是操作系统所有功能的基础，其决定了操作系统的模式切换以及各种资源的调度实现。

__中断主要分为三种__
* 异常（Exception） : 执行指令时产生的，通常无法预料的错误。例如：访问无效内存地址、执行非法指令（除以零）等。
* 陷阱（Trap）: 陷阱是一系列强行导致中断的指令，例如：系统调用（Syscall）等。
* 硬件中断（Hardware Interrupt）: 硬件中断是由 CPU 之外的硬件产生的异步中断，例如：时钟中断、外设发来数据等。

### 1.2 RISC-V 与中断相关的寄存器和指令
在次只列举部分实验常用的寄存器与指令，更多信息请查阅官方文档。
#### 寄存器

__线程相关寄存器__
1. sscratch : 在用户态，sscratch 保存内核栈的地址；在内核态，sscratch 的值为 0。

__发生中断时，硬件自动填写的寄存器__
1. sepc : 即 Exception Program Counter，用来记录触发中断的指令的地址。
1. scause: 记录中断是否是硬件中断，以及具体的中断原因。
1. stval :scause 不足以存下中断所有的必须信息。例如缺页异常，就会将 stval 设置成需要访问但是不在内存中的地址，以便于操作系统将这个地址所在的页面加载进来。

__指导硬件处理中断的寄存器__
1. stvec: 设置内核态中断处理流程的入口地址。存储了一个基址 BASE 和模式 MODE：
          * MODE 为 0 表示 Direct 模式，即遇到中断便跳转至 BASE 进行执行。
          * MODE 为 1 表示 Vectored 模式，此时 BASE 应当指向一个向量，存有不同处理流程的地址，遇到中断会跳转至 BASE + 4 * cause 进行处理流程。
1. sstatus: 具有许多状态位，控制全局中断使能等。
1. sie :即 Supervisor Interrupt Enable，用来控制具体类型中断的使能，例如其中的 STIE 控制时钟中断使能。
1. sip :即 Supervisor Interrupt Pending，和 sie 相对应，记录每种中断是否被触发。仅当 sie 和 sip 的对应位都为 1 时，意味着开中断且已发生中断，这时中断最终触发。

### 1.3 与中断相关的指令

__进入和退出中断__
1. ecall:触发中断，进入更高一层的中断处理流程之中。用户态进行系统调用进入内核态中断处理流程，内核态进行 SBI 调用进入机器态中断处理流程，使用的都是这条指令。
1. sret:从内核态返回用户态，同时将 pc 的值设置为 sepc。（如果需要返回到 sepc 后一条指令，就需要在 sret 之前修改 sepc 的值）
1. ebreak:触发一个断点。
1. mret 从机器态返回内核态，同时将 pc 的值设置为 mepc。

__操作 CSR__(读写重置)
1. csrrw dst, csr, src（CSR Read Write）:同时读写的原子操作，将指定 CSR 的值写入 dst，同时将 src 的值写入 CSR。
1. csrr dst, csr（CSR Read）:仅读取一个 CSR 寄存器。
1. csrw csr, src（CSR Write）:仅写入一个 CSR 寄存器。
1. csrc(i) csr, rs1（CSR Clear）:将 CSR 寄存器中指定的位清零，csrc 使用通用寄存器作为 mask，csrci 则使用立即数。
1. csrs(i) csr, rs1（CSR Set）:将 CSR 寄存器中指定的位置 1，csrc 使用通用寄存器作为 mask，csrci 则使用立即数。

## 二，程序运行状态

### 3.1 上下文设计
在程序运行中，各种寄存器存储着当前程序的运行时信息，包括PC，返回值等，这些信息被统称为程序的上下文。当中断发生时，操作系统必须将当前程序的上下文保存，以便于中断完成后会恢复现场；接着会将中断程序的上下文赋值到寄存器中。
本节的目的在于设计上下文信息。
__第 1 步__ 设计 Context类。在os/src/interrupt/context.rs内添加如下代码：
```
use riscv::register::{sstatus::Sstatus, scause::Scause};

#[repr(C)]
pub struct Context {
    pub x: [usize; 32],     // 32 个通用寄存器
    pub sstatus: Sstatus,  //状态位，控制全局中断使能
    pub sepc: usize       //记录触发中断的指令的地址
}
```

__第 2 步__ 添加依赖
为了使用riscv的寄存器，必须在os/Cargo.toml 中添加依赖，将依赖修改如下：
```
[dependencies]
riscv = { git = "https://github.com/rcore-os/riscv", features = ["inline-asm"] }
```

### 3.2 状态的保存与恢复
状态保存：先用栈上的一小段空间来把需要保存的全部通用寄存器和 CSR 寄存器保存在栈上，保存完之后在跳转到 Rust 编写的中断处理函数。
状态恢复：直接把备份在栈上的内容写回寄存器。由于涉及到了寄存器级别的操作，我们需要用汇编来实现。


__第 1 步__ 编写汇编代码实现保存与恢复
建立os/src/interrupt/interrupt.asm文件，编写以下内容：
(本版本中保存运用循环的方式保存和恢复#3-31号通用寄存器，旧版的采用一次列出所有寄存器的方式。)
```
# 我们将会用一个宏来用循环保存寄存器。这是必要的设置
.altmacro
# 寄存器宽度对应的字节数
.set    REG_SIZE, 8
# Context 的大小
.set    CONTEXT_SIZE, 34

# 宏：将寄存器存到栈上
.macro SAVE reg, offset
    sd  \reg, \offset*8(sp)
.endm

.macro SAVE_N n
    SAVE  x\n, \n
.endm


# 宏：将寄存器从栈中取出
.macro LOAD reg, offset
    ld  \reg, \offset*8(sp)
.endm

.macro LOAD_N n
    LOAD  x\n, \n
.endm

    .section .text
    .globl __interrupt
# 进入中断
# 保存 Context 并且进入 Rust 中的中断处理函数 interrupt::handler::handle_interrupt()
__interrupt:
    # 在栈上开辟 Context 所需的空间
    addi    sp, sp, -34*8

    # 保存通用寄存器，除了 x0（固定为 0）
    SAVE    x1, 1
    # 将原来的 sp（sp 又名 x2）写入 2 位置
    addi    x1, sp, 34*8
    SAVE    x1, 2
    # 保存 x3 至 x31
    .set    n, 3
    .rept   29
        SAVE_N  %n
        .set    n, n + 1
    .endr

    # 取出 CSR 并保存
    csrr    s1, sstatus
    csrr    s2, sepc
    SAVE    s1, 32
    SAVE    s2, 33

    # 调用 handle_interrupt，传入参数
    # context: &mut Context
    mv      a0, sp
    # scause: Scause
    csrr    a1, scause
    # stval: usize
    csrr    a2, stval
    jal  handle_interrupt

    .globl __restore
# 离开中断
# 从 Context 中恢复所有寄存器，并跳转至 Context 中 sepc 的位置
__restore:
    # 恢复 CSR
    LOAD    s1, 32
    LOAD    s2, 33
    csrw    sstatus, s1
    csrw    sepc, s2

    # 恢复通用寄存器
    LOAD    x1, 1
    # 恢复 x3 至 x31
    .set    n, 3
    .rept   29
        LOAD_N  %n
        .set    n, n + 1
    .endr

    # 恢复 sp（又名 x2）这里最后恢复是为了上面可以正常使用 LOAD 宏
    LOAD    x2, 2
    sret
```



## 三，中断处理

__第 1 步__ 开启和处理中断
新建os/src/interrupt/handler.rs文件，在其中编写以下内容开启和处理中断：
```
use super::context::Context;
use riscv::register::stvec;

global_asm!(include_str!("./interrupt.asm"));

/// 初始化中断处理
///
/// 把中断入口 `__interrupt` 写入 `stvec` 中，并且开启中断使能
pub fn init() {
    unsafe {
        extern "C" {
            /// `interrupt.asm` 中的中断入口
            fn __interrupt();
        }
        // 使用 Direct 模式，将中断入口设置为 `__interrupt`
        stvec::write(__interrupt as usize, stvec::TrapMode::Direct);
    }
}
/// 中断的处理入口
/// 
/// `interrupt.asm` 首先保存寄存器至 Context，其作为参数和 scause 以及 stval 一并传入此函数
/// 具体的中断类型需要根据 scause 来推断，然后分别处理
#[no_mangle]
pub fn handle_interrupt(context: &mut Context, scause: Scause, stval: usize) {
    panic!("Interrupted: {:?}", scause.cause());
}
```

__第 2 步__ 模块初始化
基于Rust的语法，新建os/src/interrupt/mod.rs文件添加以下代码初始化interrupt模块：
```
//! 中断模块
//! 
//! 

mod handler;
mod context;

/// 初始化中断相关的子模块
/// 
/// - [`handler::init`]
/// - [`timer::init`]
pub fn init() {
    handler::init();
    println!("mod interrupt initialized");
}
```

__第 3 步__ 触发中断
在os/src/main.rs中添加 mod interrupt; 并使用ebreak来触发中断。修改代码如下：
```
...
mod interrupt;
...

/// Rust 的入口函数
///
/// 在 `_start` 为我们进行了一系列准备之后，这是第一个被调用的 Rust 函数
pub extern "C" fn rust_main() -> ! {
    // 初始化各种模块
    interrupt::init();

    unsafe {
        llvm_asm!("ebreak"::::"volatile");
    };

    unreachable!();
}
```

## 四，时钟中断
时钟中断是操作系统能够进行线程调度的基础，操作系统会在每次时钟中断时被唤醒，暂停正在执行的线程，并根据调度算法选择下一个应当运行的线程。本节目标在于实现时钟中断。

__第 1 步__ 开启与设置时钟中断
新建os/src/interrupt/timer.rs文件，编辑如下代码：
```
//! 预约和处理时钟中断

use crate::sbi::set_timer;
use riscv::register::{time, sie, sstatus};



/// 初始化时钟中断
/// 
/// 开启时钟中断使能，并且预约第一次时钟中断
pub fn init() {
    unsafe {
        // 开启 STIE，允许时钟中断
        sie::set_stimer(); 
        // 开启 SIE（不是 sie 寄存器），允许内核态被中断打断
        sstatus::set_sie();
    }
    // 设置下一次时钟中断
    set_next_timeout();
}


/// 时钟中断的间隔，单位是 CPU 指令
static INTERVAL: usize = 100000;

/// 设置下一次时钟中断
/// 
/// 获取当前时间，加上中断间隔，通过 SBI 调用预约下一次中断
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
```


__第 2 步__ 修改sbi
为简化操作系统实现，操作系统可请求(sbi_call调用ecall指令)SBI服务来完成时钟中断的设置。
在os/src/sbi.rs文件添加如下代码。
```
/// 设置下一次时钟中断的时间
pub fn set_timer(time: usize) {
    sbi_call(SBI_SET_TIMER, time, 0, 0);
}
```

__第 3 步__ 实现时钟中断的处理流程
修改os/src/interrupt/handler.rs文件中的handle_interrupt()函数。
```
/// 中断的处理入口
/// 
/// `interrupt.asm` 首先保存寄存器至 Context，其作为参数和 scause 以及 stval 一并传入此函数
/// 具体的中断类型需要根据 scause 来推断，然后分别处理
#[no_mangle]
pub fn handle_interrupt(context: &mut Context, scause: Scause, stval: usize) {
    // 可以通过 Debug 来查看发生了什么中断
    // println!("{:x?}", context.scause.cause());
    match scause.cause() {
        // 断点中断（ebreak）
        Trap::Exception(Exception::Breakpoint) => breakpoint(context),
        // 时钟中断
        Trap::Interrupt(Interrupt::SupervisorTimer) => supervisor_timer(context),
        // 其他情况，终止当前线程
        _ => fault(context, scause, stval),
    }
}

/// 处理 ebreak 断点
/// 
/// 继续执行，其中 `sepc` 增加 2 字节，以跳过当前这条 `ebreak` 指令
fn breakpoint(context: &mut Context) {
    println!("Breakpoint at 0x{:x}", context.sepc);
    context.sepc += 2;
}

/// 处理时钟中断
/// 
/// 目前只会在 [`timer`] 模块中进行计数
fn supervisor_timer(_: &Context) {
    timer::tick();
}

/// 出现未能解决的异常
fn fault(context: &mut Context, scause: Scause, stval: usize) {
    panic!(
        "Unresolved interrupt: {:?}\n{:x?}\nstval: {:x}",
        scause.cause(),
        context,
        stval
    );
}
```

### 补充内容

__1.修改mod.rs__ 
因为加入了timer.rs,使用需要加入相应的初始化操作。修改os/src/interrupt/mod.rs如下。
```
//! 中断模块
//! 
//! 

mod handler;
mod context;
mod timer;
pub use context::Context;
/// 初始化中断相关的子模块
/// 
/// - [`handler::init`]
/// - [`timer::init`]
pub fn init() {
    handler::init();
    timer::init();
    println!("mod interrupt initialized");
}
```

__2. 修改Context__
因为要输出，所以要实现相应的trait。
修改os/src/interrupt/context.rs，内容如下：
```
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
```



__3.修改 handle__
因为调整了包的结构，以及修改了中断处理函数，所以需要修改包含的"头文件"。
os/src/interrupt/handler.rs“头文件”修改如下：
```
use super::context::Context;
use super::timer;
use riscv::register::{
    scause::{Exception, Interrupt, Scause, Trap},
    sie, stvec,
};

```

__4.修改主函数__
为了方便测试，需要修改main函数内容，加入无限循环。
os/src/main.rs中main函数修改如下：
#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    interrupt::init();

    unsafe {
        llvm_asm!("ebreak"::::"volatile");
    };
    loop{};
    unreachable!();
}


至此，lab1实验使用完成。源代码于实验题代码均在github中。





[1]:https://rcore-os.github.io/rCore-Tutorial-deploy/docs/lab-0/guide/intro.html
[2]:https://rcore-os.github.io/rCore-Tutorial-deploy/docs/lab-1/guide/intro.html
[3]:https://blog.csdn.net/weixin_41542958/article/details/107577542
[4]:https://blog.csdn.net/weixin_41542958/article/details/107612922
[5]:https://blog.csdn.net/weixin_41542958/article/details/107617342