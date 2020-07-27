# 实验报告

博客目录：
1. [环境部署][3] 
1. [lab0实验报告][4]
1. [lab1实验报告][5]
1. [lab2实验报告][7]
1. [lab3实验报告][8](实验三理论部分过多，具体步骤没前面清晰)
1. (lab4-lab5实验报告暂未完成。)
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


# lab 2 
本节主要内容为实现内存管理分配，并以页的方式对内存进行管理。
[lab2实验指导书][6]
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
        ├─algorithm
        │  │  Cargo.toml
        │  │
        │  └─src
        │      │  lib.rs
        │      │  unsafe_wrapper.rs
        │      │
        │      ├─allocator
        │      │      mod.rs
        │      │      segment_tree_allocator.rs
        │      │      stacked_allocator.rs
        │      │
        │      └─scheduler
        │              fifo_scheduler.rs
        │              hrrn_scheduler.rs
        │              mod.rs
        │
        ├─allocator
        │      mod.rs
        │
        ├─interrupt
        │      context.rs
        │      handler.rs
        │      interrupt.asm
        │      mod.rs
        │      timer.rs
        │
        └─memory
            │  address.rs
            │  config.rs
            │  heap.rs
            │  mod.rs
            │  range.rs
            │
            └─frame
                    allocator.rs
                    frame_tracker.rs
                    mod.rs
```

## 一，动态内存分配

### 1.1 动态内存分配简介
动态内存分配指的在程序运行时所进行的动态内存分配，因为有的数据项只有在实际运行中才能确定其所需内存大小。
与静态内存分配相比，动态内存分配可在运行过程中选择合适的实际分配所需的内存大小较为灵活，但相应的会带来一些开销。
在rust中常见的动态内存分配有：
* 智能指针Box<T> ，与C语言的 malloc 功能类似。
* 引用计数 Rc<T>，原子引用计数 Arc<T>，主要用于在引用计数清零，即某对象不再被引用时，对该对象进行自动回收。
* 一些 Rust std 标准库中的数据结构，如 Vec 和 HashMap 等。

按照Rust的语法我们需要实现 Trait GlobalAlloc，将其实例化，主要内容即为实现以下两个函数：
```
unsafe fn alloc(&self, layout: Layout) -> *mut u8;
unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout);

```
其中Layout是一个结构体，其一块连续的、大小至少为 size 字节的虚拟内存，且对齐要求为 align。其主要有两个字段，size 表示要分配的字节数，align 则表示分配的虚拟地址的最小对齐要求。其内容如下：
```
pub struct Layout {
    // size of the requested block of memory, measured in bytes.
    size_: usize,

    // alignment of the requested block of memory, measured in bytes.
    // we ensure that this is always a power-of-two, because API's
    // like `posix_memalign` require it and it is a reasonable
    // constraint to impose on Layout constructors.
    //
    // (However, we do not analogously require `align >= sizeof(void*)`,
    //  even though that is *also* a requirement of `posix_memalign`.)
    align_: NonZeroUsize,
}
```
实现后使用语义项 #[global_allocator] 进行标记，使得编译器将其做为默认的动态内存分配函数。

### 1.2 连续内存分配算法
连续内存分配即为在内存分配时，分配地址连续的内存空间。其中会导致外碎片问题，并需要相应的碎片整合。

外碎片：在连续地址分配中，若系统存在没有被利用且因为容量过小而无法被利用的内存空间，则其被称为外碎片。
碎片整理： 当外部碎片过多时，可通过重新移动进程的内存来使得小的未利用空间聚合到一起变成大的未利用空间，这个过程即为碎片整合。

内存分配算法有很多，这里使用伙伴系统（Buddy System）来解决问题。

__第 1 步__  添加依赖
为了使用已有的伙伴系统，需要在os/Cargo.toml添加依赖buddy_system_allocator = "0.3.9"。
查看官方代码，发现后续要用到的依赖项很多，所以这里将剩下的依赖性全部添加进来了。
```
[dependencies]
bit_field = "0.10.0"
bitflags = "1.2.1"
buddy_system_allocator = "0.3.9"        # 【就是这里】了
hashbrown = "0.7.2"
lazy_static = { version = "1.4.0", features = ["spin_no_std"] }
riscv = { git = "https://github.com/rcore-os/riscv", features = ["inline-asm"] }
spin = "0.5.2"
device_tree = { git = "https://github.com/rcore-os/device_tree-rs" }
virtio-drivers = { git = "https://github.com/rcore-os/virtio-drivers" }
rcore-fs = { git = "https://github.com/rcore-os/rcore-fs"}
rcore-fs-sfs = { git = "https://github.com/rcore-os/rcore-fs"}
xmas-elf = "0.7.0"
```

__第 2 步__ 设计系统堆栈大小
创建 os/src/memory/config.rs，用于存储一些配置相关的信息。 现设定OS堆栈大小为8M。
```
/// 操作系统动态分配内存所用的堆大小（8M）
pub const KERNEL_HEAP_SIZE: usize = 0x80_0000;
```

__第 3 步__ 开辟空间
创建os/src/memory/heap.rs，开辟一个8M静态数组作为堆空间，并实现相应的初始化等操作。
```
/// 进行动态内存分配所用的堆空间
/// 
/// 大小为 [`KERNEL_HEAP_SIZE`]  
/// 这段空间编译后会被放在操作系统执行程序的 bss 段
static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

/// 堆，动态内存分配器
/// 
/// ### `#[global_allocator]`
/// [`LockedHeap`] 实现了 [`alloc::alloc::GlobalAlloc`] trait，
/// 可以为全局需要用到堆的地方分配空间。例如 `Box` `Arc` 等
#[global_allocator]
static HEAP: LockedHeap = LockedHeap::empty();

/// 初始化操作系统运行时堆空间
pub fn init() {
    // 告诉分配器使用这一段预留的空间作为堆
    unsafe {
        HEAP.lock().init(
            HEAP_SPACE.as_ptr() as usize, KERNEL_HEAP_SIZE
        )
    }
}

/// 空间分配错误的回调，直接 panic 退出
#[alloc_error_handler]
fn alloc_error_handler(_: alloc::alloc::Layout) -> ! {
    panic!("alloc error")
}
```


__第 4 步__ 启动特性

将 #![feature(alloc_error_handler)]添加到main.rs里面，启用相关特性。

__第 5 步__ 模块化
在memory文件夹中添加mod.rs，并加入以下内容：
```
//os/src/main.rs
#![allow(dead_code)]
mod heap;
pub mod config;

pub type MemoryResult<T> = Result<T, &'static str>;

pub fn init() {
    heap::init();
    // 允许内核读写用户态内存
    unsafe { riscv::register::sstatus::set_sum() };
    println!("mod memory initialized");
}

```

### 1.3 动态内存分配测试
本节只是测试而已，内容不多。

__第 1 步__
在main.rs中加入mod memory引用模块;

__第 2 步__
修改 rust_main函数，添加如下测试代码。

```
#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    // 初始化各种模块
    interrupt::init();
    memory::init();
    
    test_dynamic() // 动态内存分配测试
    //test_physics()   // 物理内存分配测试
    //test_page()       //物理页分配
    
}

fn test_dynamic() ->!{
    // 动态内存分配测试
    use alloc::boxed::Box;
    use alloc::vec::Vec;
    let v = Box::new(5);
    assert_eq!(*v, 5);
    core::mem::drop(v);

    let mut vec = Vec::new();
    let mut v2 = Vec::new();
    for i in 0..10000 {
        vec.push(i);
    }
    for i in 0..7 {
        v2.push(i);
    }
    assert_eq!(v2.len(),7);
    assert_eq!(vec.len(), 10000);
    for (i, value) in vec.into_iter().enumerate() {
        assert_eq!(value, i);
    }
    println!("heap test passed");

    panic!()
}
```

## 二，物理内存探测

### 2.1 物理内存的相关概念

对于操作系统而言内存可看作为一个巨大的字节数组，按物理地址可对其字节进行读写访问。

而通过 MMIO（Memory Mapped I/O）技术将可外设映射到一段物理地址，其他的外设也可以被标记上地址，并用地址读写的方式来访问各个外设。

### 2.2 物理地址探测
在 RISC-V 中，这个一般是由 bootloader，即 OpenSBI 固件来完成的。它来完成对于包括物理内存在内的各外设的扫描，将扫描结果以 DTB（Device Tree Blob）的格式保存在物理内存中的某个地方。随后 OpenSBI 固件会将其地址保存在 a1 寄存器中，给我们使用。

我们知道，QEMU 规定的 DRAM 物理内存的起始物理地址为 0x80000000 。而在 QEMU 中，可以使用 -m 指定 RAM 的大小，默认是 128 MB 。因此，默认的 DRAM 物理内存地址范围就是 [0x80000000, 0x88000000)。

本节主要目的是探测内核地址，具体实现如下：

__第 1 步__ 实现PhysicalAddres类
创建文件os/src/memory/address.rs，实现以下代码：
```
//! 定义地址类型和地址常量
//!
//! 我们为虚拟地址和物理地址分别设立两种类型，利用编译器检查来防止混淆。

use super::config::PAGE_SIZE;

/// 物理地址
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PhysicalAddress(pub usize);

/// 物理页号
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PhysicalPageNumber(pub usize);

// 以下是一大堆类型的相互转换、各种琐碎操作

impl PhysicalAddress {
    /// 取得页内偏移
    pub fn page_offset(&self) -> usize {
        self.0 % PAGE_SIZE
    }
}

macro_rules! implement_address_to_page_number {
    // 这里面的类型转换实现 [`From`] trait，会自动实现相反的 [`Into`] trait
    ($address_type: ty, $page_number_type: ty) => {
        impl From<$page_number_type> for $address_type {
            /// 从页号转换为地址
            fn from(page_number: $page_number_type) -> Self {
                Self(page_number.0 * PAGE_SIZE)
            }
        }
        impl From<$address_type> for $page_number_type {
            /// 从地址转换为页号，直接进行移位操作
            ///
            /// 不允许转换没有对齐的地址，这种情况应当使用 `floor()` 和 `ceil()`
            fn from(address: $address_type) -> Self {
                assert!(address.0 % PAGE_SIZE == 0);
                Self(address.0 / PAGE_SIZE)
            }
        }
        impl $page_number_type {
            /// 将地址转换为页号，向下取整
            pub const fn floor(address: $address_type) -> Self {
                Self(address.0 / PAGE_SIZE)
            }
            /// 将地址转换为页号，向上取整
            pub const fn ceil(address: $address_type) -> Self {
                Self(address.0 / PAGE_SIZE + (address.0 % PAGE_SIZE != 0) as usize)
            }
        }
    };
}
implement_address_to_page_number! {PhysicalAddress, PhysicalPageNumber}

// 下面这些以后可能会删掉一些

/// 为各种仅包含一个 usize 的类型实现运算操作
macro_rules! implement_usize_operations {
    ($type_name: ty) => {
        /// `+`
        impl core::ops::Add<usize> for $type_name {
            type Output = Self;
            fn add(self, other: usize) -> Self::Output {
                Self(self.0 + other)
            }
        }
        /// `+=`
        impl core::ops::AddAssign<usize> for $type_name {
            fn add_assign(&mut self, rhs: usize) {
                self.0 += rhs;
            }
        }
        /// `-`
        impl core::ops::Sub<usize> for $type_name {
            type Output = Self;
            fn sub(self, other: usize) -> Self::Output {
                Self(self.0 - other)
            }
        }
        /// `-`
        impl core::ops::Sub<$type_name> for $type_name {
            type Output = usize;
            fn sub(self, other: $type_name) -> Self::Output {
                self.0 - other.0
            }
        }
        /// `-=`
        impl core::ops::SubAssign<usize> for $type_name {
            fn sub_assign(&mut self, rhs: usize) {
                self.0 -= rhs;
            }
        }
        /// 和 usize 相互转换
        impl From<usize> for $type_name {
            fn from(value: usize) -> Self {
                Self(value)
            }
        }
        /// 和 usize 相互转换
        impl From<$type_name> for usize {
            fn from(value: $type_name) -> Self {
                value.0
            }
        }
        impl $type_name {
            /// 是否有效（0 为无效）
            pub fn valid(&self) -> bool {
                self.0 != 0
            }
        }
        /// {} 输出
        impl core::fmt::Display for $type_name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}(0x{:x})", stringify!($type_name), self.0)
            }
        }
    };
}
implement_usize_operations! {PhysicalAddress}
implement_usize_operations! {PhysicalPageNumber}
```

__第 2 步__
在os/src/memory/config.rs中添加如下配置：
```
use super::address::*;
use lazy_static::*;

/// 页 / 帧大小，必须是 2^n
pub const PAGE_SIZE: usize = 4096;
/// 操作系统动态分配内存所用的堆大小（8M）
pub const KERNEL_HEAP_SIZE: usize = 0x80_0000;


/// 可以访问的内存区域起始地址
pub const MEMORY_START_ADDRESS: PhysicalAddress = PhysicalAddress(0x8000_0000);
/// 可以访问的内存区域结束地址
pub const MEMORY_END_ADDRESS: PhysicalAddress = PhysicalAddress(0x8800_0000);

lazy_static! {
    /// 内核代码结束的地址，即可以用来分配的内存起始地址
    ///
    /// 因为 Rust 语言限制，我们只能将其作为一个运行时求值的 static 变量，而不能作为 const
    pub static ref KERNEL_END_ADDRESS: PhysicalAddress = PhysicalAddress(kernel_end as usize);
}

extern "C" {
    /// 由 `linker.ld` 指定的内核代码结束位置
    ///
    /// 作为变量存在 [`KERNEL_END_ADDRESS`]
    fn kernel_end();
}
```

__第 3 步__ 修改mod.rs文件

在 os/src/memory/mod.rs加入
```
mod address;
```
__第 4 步__ 修改main.rs 添加测试代码
在main.rs中修改rust_main函数，添加如下代码：
```
#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    // 初始化各种模块
    interrupt::init();
    memory::init();
    
    //test_dynamic() // 动态内存分配测试
    test_physics()   // 物理内存分配测试
   // test_page()       //物理页分配
}

fn test_physics() ->!{
    println!("{}", *memory::config::KERNEL_END_ADDRESS);
    panic!("物理内存地址分配")
}
```

## 三，物理内存管理
物理页（Frame），即连续的 4 KB 字节为的内存分配。
我们希望用物理页号（Physical Page Number，PPN）来代表一物理页，通过设定的兑换公式来完成物理页号与物理页的一一映射。
本节内容为实现以页为单位的内存分配算法。
本实验会用到一些书中未提及的模块，比如algorithm，项目中已经将其导入，并添加了相关的依赖。

__第 1 步__ 修改配置文件config.rs
```
/// 可以访问的内存区域起始地址
pub const MEMORY_START_ADDRESS: PhysicalAddress = PhysicalAddress(0x8000_0000);
/// 可以访问的内存区域结束地址
pub const MEMORY_END_ADDRESS: PhysicalAddress = PhysicalAddress(0x8800_0000);
```


__第 2 步__ 实现物理页追踪器
创建文件os/src/memory/frame/frame_tracker.rs，编辑以下内容：
```
pub struct FrameTracker(pub(super) PhysicalPageNumber);

impl FrameTracker {
    /// 帧的物理地址
    pub fn address(&self) -> PhysicalAddress {
        self.0.into()
    }
    /// 帧的物理页号
    pub fn page_number(&self) -> PhysicalPageNumber {
        self.0
    }
}

/// 帧在释放时会放回 [`static@FRAME_ALLOCATOR`] 的空闲链表中
impl Drop for FrameTracker {
    fn drop(&mut self) {
        FRAME_ALLOCATOR.lock().dealloc(self);
    }
}
```

__第 3 步__ 创建物理页分配器
创建文件os/src/memory/frame/allocator.rs，编辑以下内容：

```
lazy_static! {
    /// 帧分配器
    pub static ref FRAME_ALLOCATOR: Mutex<FrameAllocator<AllocatorImpl>> = Mutex::new(FrameAllocator::new(Range::from(
            PhysicalPageNumber::ceil(PhysicalAddress::from(*KERNEL_END_ADDRESS))..PhysicalPageNumber::floor(MEMORY_END_ADDRESS),
        )
    ));
}

/// 基于线段树的帧分配 / 回收
pub struct FrameAllocator<T: Allocator> {
    /// 可用区间的起始
    start_ppn: PhysicalPageNumber,
    /// 分配器
    allocator: T,
}

impl<T: Allocator> FrameAllocator<T> {
    /// 创建对象
    pub fn new(range: impl Into<Range<PhysicalPageNumber>> + Copy) -> Self {
        FrameAllocator {
            start_ppn: range.into().start,
            allocator: T::new(range.into().len()),
        }
    }

    /// 分配帧，如果没有剩余则返回 `Err`
    pub fn alloc(&mut self) -> MemoryResult<FrameTracker> {
        self.allocator
            .alloc()
            .ok_or("no available frame to allocate")
            .map(|offset| FrameTracker(self.start_ppn + offset))
    }

    /// 将被释放的帧添加到空闲列表的尾部
    ///
    /// 这个函数会在 [`FrameTracker`] 被 drop 时自动调用，不应在其他地方调用
    pub(super) fn dealloc(&mut self, frame: &FrameTracker) {
        self.allocator.dealloc(frame.page_number() - self.start_ppn);
    }
}
```
__第 4 步__ 模块化
创建os/src/memory/frame/mod.rs文件，编辑内容如下：
```
mod allocator;
mod frame_tracker;

pub use allocator::FRAME_ALLOCATOR;
pub use frame_tracker::FrameTracker;
```


__第 5 步__ 实现封装分配器相关的trait
创建文件os/src/algorithm/src/allocator/mod.rs，编辑以下内容：
```
/// 分配器：固定容量，每次分配 / 回收一个元素
pub trait Allocator {
    /// 给定容量，创建分配器
    fn new(capacity: usize) -> Self;
    /// 分配一个元素，无法分配则返回 `None`
    fn alloc(&mut self) -> Option<usize>;
    /// 回收一个元素
    fn dealloc(&mut self, index: usize);
}
```
(算法模块已经导入)

__第 6 步__ 编辑测试代码
修改main.rs如下：
```
#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    // 初始化各种模块
    interrupt::init();
    memory::init();
    
    //test_dynamic() // 动态内存分配测试
    //test_physics()   // 物理内存分配测试
    test_page()       //物理页分配
    
}

fn test_page() -> !{
    for _ in 0..2 {
        let frame_0 = match memory::frame::FRAME_ALLOCATOR.lock().alloc() {
            Result::Ok(frame_tracker) => frame_tracker,
            Result::Err(err) => panic!("{}", err)
        };
        let frame_1 = match memory::frame::FRAME_ALLOCATOR.lock().alloc() {
            Result::Ok(frame_tracker) => frame_tracker,
            Result::Err(err) => panic!("{}", err)
        };
        println!("{} and {}", frame_0.address(), frame_1.address());
    }

    panic!()
}
```
实验结果以截图的方式存储在lab2文件夹中，居然内容见GitHub。

# lab3
[lab3实验指导书][9]
lb3主要涉及以下过程
* 虚拟地址和物理地址的概念和关系
* 利用页表完成虚拟地址到物理地址的映射
* 实现内核的重映射
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
        ├─algorithm
        │  │  Cargo.toml
        │  │  README.md
        │  │
        │  └─src
        │      │  lib.rs
        │      │  unsafe_wrapper.rs
        │      │
        │      ├─allocator
        │      │      mod.rs
        │      │      segment_tree_allocator.rs
        │      │      stacked_allocator.rs
        │      │
        │      └─scheduler
        │              fifo_scheduler.rs
        │              hrrn_scheduler.rs
        │              mod.rs
        │
        ├─interrupt
        │      context.rs
        │      handler.rs
        │      interrupt.asm
        │      mod.rs
        │      timer.rs
        │
        └─memory
            │  address.rs
            │  config.rs
            │  heap.rs
            │  mod.rs
            │  range.rs
            │
            ├─frame
            │      allocator.rs
            │      frame_tracker.rs
            │      mod.rs
            │
            └─mapping
                    mapping.rs
                    memory_set.rs
                    mod.rs
                    page_table.rs
                    page_table_entry.rs
                    segment.rs
```
# 一，从虚拟内存到物理内存

在之前的Lb2中做了一个简单的内核，但在真正的操作系统中为了让其他的程序能方便的运行在操作系统上，我们需要引入多任务的概念，而其中最为重要的就是让程序各自占有各自的位置，同时运行。而在程序执行的角度上所看到的地址空间，成为虚拟内存，访问虚拟内存的地址也就是虚拟地址，与之对应的是物理地址，为了防止多个应用访问同一段内存，这时就需要引入一种机制，也就是页表，通过页表维护虚拟地址到物理地址的映射，并能有效防止映射到同一段内存地址。
__sv39__
Sv39 模式是基于页的，在这里物理页号为 44 位，每个物理页大小为 4KB。同理，我们对于虚拟内存定义虚拟页（Page）以及虚拟页号（VPN, Virtual Page Number) 。在这里虚拟页号为 27 位，每个虚拟页大小也为 4KB。物理地址和虚拟地址的最后 12 位都表示页内偏移，即表示该地址在所在物理页（虚拟页）上的什么位置。
页表的作用就是将虚拟地址所在的虚拟页映射到一个物理页，然后再在这个物理页上根据页内偏移找到物理地址，从而完成映射。

__页表项__
页表项用来描述一个虚拟页号如何映射到物理页号的。
Sv39 模式里面的一个页表项大小为 64 位（即 8 字节）。其中第 53-10 共 44 位为一个物理页号，表示这个虚拟页号映射到的物理页号。后面的第 9-0 位则描述页的相关状态信息。
* V 表示这个页表项是否合法。如果为 0 表示不合法，此时页表项其他位的值都会被忽略。
* R,W,X 分别表示是否可读（Readable）、可写（Writable）和可执行（Executable）。
* 如果 R,W,X 均为 0，文档上说这表示这个页表项指向下一级页表。
* U 为 1 表示用户态运行的程序可以通过该页表项完成地址映射。需要将 S 态的状态寄存器 sstatus 上的 SUM (permit Supervisor User Memory access) 位手动设置为 1 才可以访问通过这些 U 为 1 的页表项进行映射的用户态内存空间。
* 
__多级页表__
在 Sv39 模式中我们采用三级页表

__页表基址__
页表寄存器 satp：页表的基址（起始地址）一般会保存在一个特殊的寄存器中。

__快表（TLB）__
使用快表（TLB, Translation Lookaside Buffer）来作为虚拟页号到物理页号的映射的缓存。
注意：手动修改一个页表项之后，也修改了映射，但 TLB 并不会自动刷新，我们也需要使用 sfence.vma 指令刷新 TLB。如果不加参数的，sfence.vma 会刷新整个 TLB。你可以在后面加上一个虚拟地址，这样 sfence.vma 只会刷新这个虚拟地址的映射。

__修改内核__
Lb2的内核实现并未使能页表机制，实际上内核是直接在物理地址空间上运行的。
所以首先需要把内核的运行环境从物理地址空间转移到虚拟地址空间：将内核代码放在虚拟地址空间中以 0xffffffff80200000 开头的一段高地址空间中。

# 二，修改内核 
__第 1 步__
修改os/src/linker.ld如下
```
/* Linker Script 语法可以参见：http://www.scoberlin.de/content/media/http/informatik/gcc_docs/ld_3.html */

/* 目标架构 */
OUTPUT_ARCH(riscv)

/* 执行入口 */
ENTRY(_start)

/* 数据存放起始地址 */
BASE_ADDRESS = 0xffffffff80200000; /* 修改为虚拟地址 */

SECTIONS
{
    /* . 表示当前地址（location counter） */
    . = BASE_ADDRESS;

    /* start 符号表示全部的开始位置 */
    kernel_start = .;

    /* 加入对齐 */
    . = ALIGN(4K);
    text_start = .;

    /* .text 字段 */
    .text : {
        /* 把 entry 函数放在最前面 */
        *(.text.entry)
        /* 要链接的文件的 .text 字段集中放在这里 */
        *(.text .text.*)
    }

    /* 加入对齐 */
    . = ALIGN(4K);
    rodata_start = .;

    /* .rodata 字段 */
    .rodata : {
        /* 要链接的文件的 .rodata 字段集中放在这里 */
        *(.rodata .rodata.*)
    }

    /* 加入对齐 */
    . = ALIGN(4K);
    data_start = .;

    /* .data 字段 */
    .data : {
        /* 要链接的文件的 .data 字段集中放在这里 */
        *(.data .data.*)
    }

    /* 加入对齐 */
    . = ALIGN(4K);
    bss_start = .;

    /* .bss 字段 */
    .bss : {
        /* 要链接的文件的 .bss 字段集中放在这里 */
        *(.sbss .bss .bss.*)
    }

    /* 结束地址 */
    /* 加入对齐 */
    . = ALIGN(4K);
    kernel_end = .;
}
```

修改对应 os/src/memory/config.rs 中的 KERNEL_END_ADDRESS 修改为虚拟地址并加入偏移量：
```
lazy_static! {
    /// 内核代码结束的地址，即可以用来分配的内存起始地址
    /// 
    /// 因为 Rust 语言限制，我们只能将其作为一个运行时求值的 static 变量，而不能作为 const
    pub static ref KERNEL_END_ADDRESS: VirtualAddress = VirtualAddress(kernel_end as usize); 
}

/// 内核使用线性映射的偏移量
pub const KERNEL_MAP_OFFSET: usize = 0xffff_ffff_0000_0000;
```
__第 3 步__ 

最后需要在启动时、在进入 rust_main 之前我们要完成一个从物理地址访存模式到虚拟访存模式的转换，同时这也意味着，我们要写一个简单的页表，完成这个线性映射。
修改os/src/entry.asm
```
# 操作系统启动时所需的指令以及字段
#
# 我们在 linker.ld 中将程序入口设置为了 _start，因此在这里我们将填充这个标签
# 它将会执行一些必要操作，然后跳转至我们用 rust 编写的入口函数
#
# 关于 RISC-V 下的汇编语言，可以参考 https://github.com/riscv/riscv-asm-manual/blob/master/riscv-asm.md
# %hi 表示取 [12,32) 位，%lo 表示取 [0,12) 位

    .section .text.entry
    .globl _start
# 目前 _start 的功能：将预留的栈空间写入 $sp，然后跳转至 rust_main
_start:
    # 计算 boot_page_table 的物理页号
    lui t0, %hi(boot_page_table)
    li t1, 0xffffffff00000000
    sub t0, t0, t1
    srli t0, t0, 12
    # 8 << 60 是 satp 中使用 Sv39 模式的记号
    li t1, (8 << 60)
    or t0, t0, t1
    # 写入 satp 并更新 TLB
    csrw satp, t0
    sfence.vma

    # 加载栈地址
    lui sp, %hi(boot_stack_top)
    addi sp, sp, %lo(boot_stack_top)
    # 跳转至 rust_main
    lui t0, %hi(rust_main)
    addi t0, t0, %lo(rust_main)
    jr t0

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

    # 初始内核映射所用的页表
    .section .data
    .align 12
boot_page_table:
    .quad 0
    .quad 0
    # 第 2 项：0x8000_0000 -> 0x8000_0000，0xcf 表示 VRWXAD 均为 1
    .quad (0x80000 << 10) | 0xcf
    .zero 507 * 8
    # 第 510 项：0xffff_ffff_8000_0000 -> 0x8000_0000，0xcf 表示 VRWXAD 均为 1
    .quad (0x80000 << 10) | 0xcf
    .quad 0
```
# 三，实现页表
思路是把一个分配好的物理页（即会自动销毁的 FrameTracker）拿来把数据填充作为页表，而页表中的每一项是一个 8 字节的页表项。

__第 1 步__
加入两个关于位操作的 crate，在os/Cargo.toml中加入依赖。
（由于之前以及将本项目需要的依赖全加入了，所以这一步可跳过）
```
bitflags = "1.2.1"
bit_field = "0.10.0"
```

__第 2 步__
修改os/src/memory/address.rs。构建通过虚拟页号获得三级 VPN 的函数。
```
impl VirtualPageNumber {
    /// 得到一、二、三级页号
    pub fn levels(self) -> [usize; 3] {
        [
            self.0.get_bits(18..27),
            self.0.get_bits(9..18),
            self.0.get_bits(0..9),
        ]
    }
}
```

__第 3 步__ 构建页表项
建立os/src/memory/mapping/page_table_entry.rs
```
/// Sv39 结构的页表项
#[derive(Copy, Clone, Default)]
pub struct PageTableEntry(usize);

/// Sv39 页表项中标志位的位置
const FLAG_RANGE: core::ops::Range<usize> = 0..8;
/// Sv39 页表项中物理页号的位置
const PAGE_NUMBER_RANGE: core::ops::Range<usize> = 10..54;

impl PageTableEntry {
    /// 将相应页号和标志写入一个页表项
    pub fn new(page_number: Option<PhysicalPageNumber>, mut flags: Flags) -> Self {
        // 标志位中是否包含 Valid 取决于 page_number 是否为 Some
        flags.set(Flags::VALID, page_number.is_some());
        Self(
            *0usize
                .set_bits(FLAG_RANGE, flags.bits() as usize)
                .set_bits(PAGE_NUMBER_RANGE, page_number.unwrap_or_default().into()),
        )
    }
    /// 设置物理页号，同时根据 ppn 是否为 Some 来设置 Valid 位
    pub fn update_page_number(&mut self, ppn: Option<PhysicalPageNumber>) {
        if let Some(ppn) = ppn {
            self.0
                .set_bits(FLAG_RANGE, (self.flags() | Flags::VALID).bits() as usize)
                .set_bits(PAGE_NUMBER_RANGE, ppn.into());
        } else {
            self.0
                .set_bits(FLAG_RANGE, (self.flags() - Flags::VALID).bits() as usize)
                .set_bits(PAGE_NUMBER_RANGE, 0);
        }
    }
    /// 获取页号
    pub fn page_number(&self) -> PhysicalPageNumber {
        PhysicalPageNumber::from(self.0.get_bits(10..54))
    }
    /// 获取地址
    pub fn address(&self) -> PhysicalAddress {
        PhysicalAddress::from(self.page_number())
    }
    /// 获取标志位
    pub fn flags(&self) -> Flags {
        unsafe { Flags::from_bits_unchecked(self.0.get_bits(..8) as u8) }
    }
    /// 是否为空（可能非空也非 Valid）
    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }
}

impl core::fmt::Debug for PageTableEntry {
    fn fmt(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter
            .debug_struct("PageTableEntry")
            .field("value", &self.0)
            .field("page_number", &self.page_number())
            .field("flags", &self.flags())
            .finish()
    }
}

bitflags! {
    /// 页表项中的 8 个标志位
    #[derive(Default)]
    pub struct Flags: u8 {
        /// 有效位
        const VALID =       1 << 0;
        /// 可读位
        const READABLE =    1 << 1;
        /// 可写位
        const WRITABLE =    1 << 2;
        /// 可执行位
        const EXECUTABLE =  1 << 3;
        /// 用户位
        const USER =        1 << 4;
        /// 全局位，我们不会使用
        const GLOBAL =      1 << 5;
        /// 已使用位，用于替换算法
        const ACCESSED =    1 << 6;
        /// 已修改位，用于替换算法
        const DIRTY =       1 << 7;
    }
}
```
__第 4 步__  建立页表
多个页表项组成物理页，再加上多级添加映射封装成页表。
创建os/src/memory/mapping/page_table.rs文件，编辑内容如下：
```
/// 存有 512 个页表项的页表
///
/// 注意我们不会使用常规的 Rust 语法来创建 `PageTable`。相反，我们会分配一个物理页，
/// 其对应了一段物理内存，然后直接把其当做页表进行读写。我们会在操作系统中用一个「指针」
/// [`PageTableTracker`] 来记录这个页表。
#[repr(C)]
pub struct PageTable {
    pub entries: [PageTableEntry; PAGE_SIZE / 8],
}

impl PageTable {
    /// 将页表清零
    pub fn zero_init(&mut self) {
        self.entries = [Default::default(); PAGE_SIZE / 8];
    }
}

/// 类似于 [`FrameTracker`]，用于记录某一个内存中页表
///
/// 注意到，「真正的页表」会放在我们分配出来的物理页当中，而不应放在操作系统的运行栈或堆中。
/// 而 `PageTableTracker` 会保存在某个线程的元数据中（也就是在操作系统的堆上），指向其真正的页表。
///
/// 当 `PageTableTracker` 被 drop 时，会自动 drop `FrameTracker`，进而释放帧。
pub struct PageTableTracker(pub FrameTracker);

impl PageTableTracker {
    /// 将一个分配的帧清零，形成空的页表
    pub fn new(frame: FrameTracker) -> Self {
        let mut page_table = Self(frame);
        page_table.zero_init();
        page_table
    }
    /// 获取物理页号
    pub fn page_number(&self) -> PhysicalPageNumber {
        self.0.page_number()
    }
}
```

# 四，实现内核重映射
构造了一个简单映射使得内核能够运行在虚拟空间上，但是这个映射是比较粗糙的
我们知道一个程序通常含有下面几段：

* .text 段：存放代码，需要可读、可执行的，但不可写；
*.rodata 段：存放只读数据，顾名思义，需要可读，但不可写亦不可执行；
* .data 段：存放经过初始化的数据，需要可读、可写；
* .bss 段：存放零初始化的数据，需要可读、可写。
我们看到各个段之间的访问权限是不同的。在现在的映射，我们甚至可以修改内核 .text 段的代码。因为我们通过一个标志位 W 为 1 的页表项完成映射。

因此，我们考虑对这些段分别进行重映射，使得他们的访问权限被正确设置。

__第 1 步__ 创建内存段
创建os/src/memory/mapping/segment.rs，编辑内容如下：
```
//! 映射类型 [`MapType`] 和映射片段 [`Segment`]

use crate::memory::{address::*, mapping::Flags, range::Range};

/// 映射的类型
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MapType {
    /// 线性映射，操作系统使用
    Linear,
    /// 按帧分配映射
    Framed,
}

/// 一个映射片段（对应旧 tutorial 的 `MemoryArea`）
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Segment {
    /// 映射类型
    pub map_type: MapType,
    /// 所映射的虚拟地址
    pub range: Range<VirtualAddress>,
    /// 权限标志
    pub flags: Flags,
}

impl Segment {
    /// 遍历对应的物理地址（如果可能）
    pub fn iter_mapped(&self) -> Option<impl Iterator<Item = PhysicalPageNumber>> {
        match self.map_type {
            // 线性映射可以直接将虚拟地址转换
            MapType::Linear => Some(self.page_range().into().iter()),
            // 按帧映射无法直接获得物理地址，需要分配
            MapType::Framed => None,
        }
    }

    /// 将地址相应地上下取整，获得虚拟页号区间
    pub fn page_range(&self) -> Range<VirtualPageNumber> {
        Range::from(
            VirtualPageNumber::floor(self.range.start)..VirtualPageNumber::ceil(self.range.end),
        )
    }
}

```

__第 2 步__ 创建映射
创建os/src/memory/mapping/mapping.rs文件，编辑内容如下：
```
#[derive(Default)]
/// 某个线程的内存映射关系
pub struct Mapping {
    /// 保存所有使用到的页表
    page_tables: Vec<PageTableTracker>,
    /// 根页表的物理页号
    root_ppn: PhysicalPageNumber,
    /// 所有分配的物理页面映射信息
    mapped_pairs: VecDeque<(VirtualPageNumber, FrameTracker)>,
}


impl Mapping {
    /// 将当前的映射加载到 `satp` 寄存器并记录
    pub fn activate(&self) {
        // satp 低 27 位为页号，高 4 位为模式，8 表示 Sv39
        let new_satp = self.root_ppn.0 | (8 << 60);
        unsafe {
            // 将 new_satp 的值写到 satp 寄存器
            llvm_asm!("csrw satp, $0" :: "r"(new_satp) :: "volatile");
            // 刷新 TLB
            llvm_asm!("sfence.vma" :::: "volatile");
        }
    }

    /// 创建一个有根节点的映射
    pub fn new() -> MemoryResult<Mapping> {
        let root_table = PageTableTracker::new(FRAME_ALLOCATOR.lock().alloc()?);
        let root_ppn = root_table.page_number();
        Ok(Mapping {
            page_tables: vec![root_table],
            root_ppn,
            mapped_pairs: VecDeque::new(),
        })
    }

    /// 加入一段映射，可能会相应地分配物理页面
    ///
    /// 未被分配物理页面的虚拟页号暂时不会写入页表当中，它们会在发生 PageFault 后再建立页表项。
    pub fn map(&mut self, segment: &Segment, init_data: Option<&[u8]>) -> MemoryResult<()> {
        match segment.map_type {
            // 线性映射，直接对虚拟地址进行转换
            MapType::Linear => {
                for vpn in segment.page_range().iter() {
                    self.map_one(vpn, Some(vpn.into()), segment.flags | Flags::VALID)?;
                }
                // 拷贝数据
                if let Some(data) = init_data {
                    unsafe {
                        (&mut *slice_from_raw_parts_mut(segment.range.start.deref(), data.len()))
                            .copy_from_slice(data);
                    }
                }
            }
            // 需要分配帧进行映射
            MapType::Framed => {
                for vpn in segment.page_range().iter() {
                    // 如果有初始化数据，找到相应的数据
                    let page_data = if init_data.is_none() || init_data.unwrap().is_empty() {
                        [0u8; PAGE_SIZE]
                    } else {
                        // 这里必须进行一些调整，因为传入的数据可能并非按照整页对齐

                        // 传入的初始化数据
                        let init_data = init_data.unwrap();
                        // 整理后将要返回的一整个页面的数据
                        let mut page_data = [0u8; PAGE_SIZE];

                        // 拷贝时必须考虑区间与整页不对齐的情况
                        //    start（仅第一页时非零）
                        //      |        stop（仅最后一页时非零）
                        // 0    |---data---|          4096
                        // |------------page------------|
                        let page_address = VirtualAddress::from(vpn);
                        let start = if segment.range.start > page_address {
                            segment.range.start - page_address
                        } else {
                            0
                        };
                        let stop = min(PAGE_SIZE, segment.range.end - page_address);
                        // 计算来源和目标区间并进行拷贝
                        let dst_slice = &mut page_data[start..stop];
                        let src_slice = &init_data[(page_address + start - segment.range.start)
                            ..(page_address + stop - segment.range.start)];
                        dst_slice.copy_from_slice(src_slice);

                        page_data
                    };

                    // 建立映射
                    let mut frame = FRAME_ALLOCATOR.lock().alloc()?;
                    // 更新页表
                    self.map_one(vpn, Some(frame.page_number()), segment.flags)?;
                    // 写入数据
                    (*frame).copy_from_slice(&page_data);
                    // 保存
                    self.mapped_pairs.push_back((vpn, frame));
                }
            }
        }
        Ok(())
    }

    /// 移除一段映射
    pub fn unmap(&mut self, segment: &Segment) {
        for vpn in segment.page_range().iter() {
            let entry = self.find_entry(vpn).unwrap();
            assert!(!entry.is_empty());
            // 从页表中清除项
            entry.clear();
        }
    }

    /// 找到给定虚拟页号的三级页表项
    ///
    /// 如果找不到对应的页表项，则会相应创建页表
    pub fn find_entry(&mut self, vpn: VirtualPageNumber) -> MemoryResult<&mut PageTableEntry> {
        // 从根页表开始向下查询
        // 这里不用 self.page_tables[0] 避免后面产生 borrow-check 冲突（我太菜了）
        let root_table: &mut PageTable = PhysicalAddress::from(self.root_ppn).deref_kernel();
        let mut entry = &mut root_table.entries[vpn.levels()[0]];
        for vpn_slice in &vpn.levels()[1..] {
            if entry.is_empty() {
                // 如果页表不存在，则需要分配一个新的页表
                let new_table = PageTableTracker::new(FRAME_ALLOCATOR.lock().alloc()?);
                let new_ppn = new_table.page_number();
                // 将新页表的页号写入当前的页表项
                *entry = PageTableEntry::new(Some(new_ppn), Flags::VALID);
                // 保存页表
                self.page_tables.push(new_table);
            }
            // 进入下一级页表（使用偏移量来访问物理地址）
            entry = &mut entry.get_next_table().entries[*vpn_slice];
        }
        // 此时 entry 位于第三级页表
        Ok(entry)
    }

    /// 查找虚拟地址对应的物理地址
    pub fn lookup(va: VirtualAddress) -> Option<PhysicalAddress> {
        let mut current_ppn;
        unsafe {
            llvm_asm!("csrr $0, satp" : "=r"(current_ppn) ::: "volatile");
            current_ppn ^= 8 << 60;
        }

        let root_table: &PageTable =
            PhysicalAddress::from(PhysicalPageNumber(current_ppn)).deref_kernel();
        let vpn = VirtualPageNumber::floor(va);
        let mut entry = &root_table.entries[vpn.levels()[0]];
        // 为了支持大页的查找，我们用 length 表示查找到的物理页需要加多少位的偏移
        let mut length = 12 + 2 * 9;
        for vpn_slice in &vpn.levels()[1..] {
            if entry.is_empty() {
                return None;
            }
            if entry.has_next_level() {
                length -= 9;
                entry = &mut entry.get_next_table().entries[*vpn_slice];
            } else {
                break;
            }
        }
        let base = PhysicalAddress::from(entry.page_number()).0;
        let offset = va.0 & ((1 << length) - 1);
        Some(PhysicalAddress(base + offset))
    }

    /// 为给定的虚拟 / 物理页号建立映射关系
    fn map_one(
        &mut self,
        vpn: VirtualPageNumber,
        ppn: Option<PhysicalPageNumber>,
        flags: Flags,
    ) -> MemoryResult<()> {
        // 定位到页表项
        let entry = self.find_entry(vpn)?;
        assert!(entry.is_empty(), "virtual address is already mapped");
        // 页表项为空，则写入内容
        *entry = PageTableEntry::new(ppn, flags);
        Ok(())
    }
}
```
__第 3 步__  创建MemorySet
创建os/src/memory/mapping/memory_set.rs，编辑内容如下:
```
//! 一个线程中关于内存空间的所有信息 [`MemorySet`]
//!

use crate::memory::{
    address::*,
    config::*,
    mapping::{Flags, MapType, Mapping, Segment},
    range::Range,
    MemoryResult,
};
use alloc::{vec, vec::Vec};

/// 一个进程所有关于内存空间管理的信息
pub struct MemorySet {
    /// 维护页表和映射关系
    pub mapping: Mapping,
    /// 每个字段
    pub segments: Vec<Segment>,
}

impl MemorySet {
    /// 创建内核重映射
    pub fn new_kernel() -> MemoryResult<MemorySet> {
        // 在 linker.ld 里面标记的各个字段的起始点，均为 4K 对齐
        extern "C" {
            fn text_start();
            fn rodata_start();
            fn data_start();
            fn bss_start();
        }

        // 建立字段
        let segments = vec![
            // .text 段，r-x
            Segment {
                map_type: MapType::Linear,
                range: Range::from((text_start as usize)..(rodata_start as usize)),
                flags: Flags::READABLE | Flags::EXECUTABLE,
            },
            // .rodata 段，r--
            Segment {
                map_type: MapType::Linear,
                range: Range::from((rodata_start as usize)..(data_start as usize)),
                flags: Flags::READABLE,
            },
            // .data 段，rw-
            Segment {
                map_type: MapType::Linear,
                range: Range::from((data_start as usize)..(bss_start as usize)),
                flags: Flags::READABLE | Flags::WRITABLE,
            },
            // .bss 段，rw-
            Segment {
                map_type: MapType::Linear,
                range: Range::from(VirtualAddress::from(bss_start as usize)..*KERNEL_END_ADDRESS),
                flags: Flags::READABLE | Flags::WRITABLE,
            },
            // 剩余内存空间，rw-
            Segment {
                map_type: MapType::Linear,
                range: Range::from(*KERNEL_END_ADDRESS..VirtualAddress::from(MEMORY_END_ADDRESS)),
                flags: Flags::READABLE | Flags::WRITABLE,
            },
        ];
        let mut mapping = Mapping::new()?;

        // 每个字段在页表中进行映射
        for segment in segments.iter() {
            mapping.map(segment, None)?;
        }
        Ok(MemorySet {
            mapping,
            segments,
        })
    }

    /// 替换 `satp` 以激活页表
    ///
    /// 如果当前页表就是自身，则不会替换，但仍然会刷新 TLB。
    pub fn activate(&self) {
        self.mapping.activate();
    }

    /// 添加一个 [`Segment`] 的内存映射
    pub fn add_segment(&mut self, segment: Segment, init_data: Option<&[u8]>) -> MemoryResult<()> {
        // 检测 segment 没有重合
        assert!(!self.overlap_with(segment.page_range()));
        // 映射
        self.mapping.map(&segment, init_data)?;
        self.segments.push(segment);
        Ok(())
    }

    /// 移除一个 [`Segment`] 的内存映射
    ///
    /// `segment` 必须已经映射
    pub fn remove_segment(&mut self, segment: &Segment) -> MemoryResult<()> {
        // 找到对应的 segment
        let segment_index = self
            .segments
            .iter()
            .position(|s| s == segment)
            .expect("segment to remove cannot be found");
        self.segments.remove(segment_index);
        // 移除映射
        self.mapping.unmap(segment);
        Ok(())
    }

    /// 检测一段内存区域和已有的是否存在重叠区域
    pub fn overlap_with(&self, range: Range<VirtualPageNumber>) -> bool {
        for seg in self.segments.iter() {
            if range.overlap_with(&seg.page_range()) {
                return true;
            }
        }
        false
    }
}
```

__第 4 步__ 修改物理页追踪器
在os/src/memory/frame/frame_tracker.rs 中加入如下代码：
```

/// `FrameTracker` 可以 deref 得到对应的 `[u8; PAGE_SIZE]`
impl core::ops::Deref for FrameTracker {
    type Target = [u8; PAGE_SIZE];
    fn deref(&self) -> &Self::Target {
        self.page_number().deref_kernel()
    }
}

/// `FrameTracker` 可以 deref 得到对应的 `[u8; PAGE_SIZE]`
impl core::ops::DerefMut for FrameTracker {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.page_number().deref_kernel()
    }
}
```
__第 5 步__ 编辑测试代码
修改main.rs中的rust_main函数，内容如下：
```
/// Rust 的入口函数
///
/// 在 `_start` 为我们进行了一系列准备之后，这是第一个被调用的 Rust 函数
#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    // 初始化各种模块
    interrupt::init();
    memory::init();

    let remap = memory::mapping::MemorySet::new_kernel().unwrap();
    remap.activate();

    println!("kernel remapped");

    panic!()
}
```

# 四，页面置换算法
（虽说花了不少时间但是还是不能完全掌握）
__第 1 步__
在磁盘中建立一个页面置换文件，来保存所有换出的页面。user/Makefile内容如下：
```
# 编译、打包、格式转换、预留空间
build: dependency
    @cargo build
    @echo Targets: $(patsubst $(SRC_DIR)/%.rs, %, $(SRC_FILES))
    @rm -rf $(OUT_DIR)
    @mkdir -p $(OUT_DIR)
    @cp $(BIN_FILES) $(OUT_DIR)
-->    @dd if=/dev/zero of=$(OUT_DIR)/SWAP_FILE bs=1M count=16
    @rcore-fs-fuse --fs sfs $(IMG_FILE) $(OUT_DIR) zip
    @qemu-img convert -f raw $(IMG_FILE) -O qcow2 $(QCOW_FILE)
    @qemu-img resize $(QCOW_FILE) +1G
```

__第 2 步__
创建os/src/fs/swap.rs文件。
其中SwapTracker 记录了一个被置换出物理内存的页面，并提供一些便捷的操作接口。内容如下(详细内容见项目代码)：
```
/// 类似于 [`FrameTracker`]，相当于 `Box<置换文件中的一个页面>`
///
/// 内部保存该置换页面在文件中保存的 index
///
/// [`FrameTracker`]: crate::memory::frame::FrameTracker
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct SwapTracker(pub(super) usize);

impl SwapTracker {
    /// 从置换文件分配一个页面空间
    pub fn new() -> MemoryResult<Self> {
        ...
    }

    /// 读取页面数据
    pub fn read(&self) -> [u8; PAGE_SIZE] {
        ...
    }

    /// 写入页面数据
    pub fn write(&self, data: &[u8; PAGE_SIZE]) {
        ...
    }
}

impl Drop for SwapTracker {
    fn drop(&mut self) {
        ...
    }
}
```
__第 3 步__
创建swapper.rs，内容如下(具体内容见项目源码)：
（其中Swapper 就替代了 Mapping 中的 mapped_pairs: Vec<(VirtualPageNumber, FrameTracker)> 的作用。）
```
/// 管理一个线程所映射的页面的置换操作
pub trait Swapper {
    /// 新建带有一个分配数量上限的置换器
    fn new(quota: usize) -> Self;

    /// 是否已达到上限
    fn full(&self) -> bool;

    /// 取出一组映射
    fn pop(&mut self) -> Option<(VirtualPageNumber, FrameTracker)>;

    /// 添加一组映射（不会在以达到分配上限时调用）
    fn push(&mut self, vpn: VirtualPageNumber, frame: FrameTracker);

    /// 只保留符合某种条件的条目（用于移除一段虚拟地址）
    fn retain(&mut self, predicate: impl Fn(&VirtualPageNumber) -> bool);
}
```

__第 4 步__
修改os/src/memory/mapping/mapping.rs文件，mapping中被替换的内容：
```
impl Mapping {
    /// 处理缺页异常
    pub fn handle_page_fault(&mut self, stval: usize) -> MemoryResult<()> {
        let vpn = VirtualPageNumber::floor(stval.into());
        let swap_tracker = self
            .swapped_pages
            .remove(&vpn)
            .ok_or("stval page is not mapped")?;
        let page_data = swap_tracker.read();

        if self.mapped_pairs.full() {
            // 取出一个映射
            let (popped_vpn, mut popped_frame) = self.mapped_pairs.pop().unwrap();
            // print!("{:x?} -> {:x?}", popped_vpn, vpn);
            // 交换数据
            swap_tracker.write(&*popped_frame);
            (*popped_frame).copy_from_slice(&page_data);
            // 修改页表映射
            self.invalidate_one(popped_vpn)?;
            self.remap_one(vpn, popped_frame.page_number())?;
            // 更新记录
            self.mapped_pairs.push(vpn, popped_frame);
            self.swapped_pages.insert(popped_vpn, swap_tracker);
        } else {
            // 如果当前还没有达到配额，则可以继续分配物理页面。这种情况目前还不会出现
            // 添加新的映射
            let mut frame = FRAME_ALLOCATOR.lock().alloc()?;
            // 复制数据
            (*frame).copy_from_slice(&page_data);
            // 更新映射
            self.remap_one(vpn, frame.page_number())?;
            // 更新记录
            self.mapped_pairs.push(vpn, frame);
        }
        Ok(())
    }
}
```
__第 5 步__
修改os/src/interrupt/handler.rs文件，修改内容如下：
```
/// 处理缺页异常
///
/// todo: 理论上这里需要判断访问类型，并与页表中的标志位进行比对
fn page_fault(context: &mut Context, stval: usize) -> Result<*mut Context, String> {
    println!("page_fault");
    let current_thread = PROCESSOR.get().current_thread();
    let memory_set = &mut current_thread.process.write().memory_set;
    memory_set.mapping.handle_page_fault(stval)?;
    memory_set.activate();
    Ok(context)
}

```
(~~说实话，页面置换这块确实没做好。~~)
代码的内容可见github上lab3文件夹，已实现部分实验结果以截图的方式存储。



[1]:https://rcore-os.github.io/rCore-Tutorial-deploy/docs/lab-0/guide/intro.html
[2]:https://rcore-os.github.io/rCore-Tutorial-deploy/docs/lab-1/guide/intro.html
[3]:https://blog.csdn.net/weixin_41542958/article/details/107577542
[4]:https://blog.csdn.net/weixin_41542958/article/details/107612922
[5]:https://blog.csdn.net/weixin_41542958/article/details/107617342
[6]:https://rcore-os.github.io/rCore-Tutorial-deploy/docs/lab-2/guide/intro.html
[7]:https://blog.csdn.net/weixin_41542958/article/details/107624186
[8]:https://blog.csdn.net/weixin_41542958/article/details/107625823
[9]:https://rcore-os.github.io/rCore-Tutorial-deploy/docs/lab-2/guide/intro.html