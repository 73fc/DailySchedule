
/*目标架构*/
OUTPUT_ARCH(riscv)

/*程序入口*/
ENTRY(_start)

/*数据起始地址*/
BASE_ADDRESS = 0x80200000;

SECTIONS 
{
    . = BASE_ADDRESS;

    kernel_start = .;

    text_start = .;

    .text :{
        *(.text.entry)
        *(.text .text.*)
    }

    rodata_start = .;

    .rodata : {
        *(.rodata .rodata.*)
    }


    data_start = .;

    .data :{
        *(.data .data.*)
    }
     
    bss_start = .;

    .bss :{
       *(.sbss .bss .bss.*)
    }

    kernel_end = .;
}