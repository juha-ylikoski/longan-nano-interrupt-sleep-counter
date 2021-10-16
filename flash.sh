#!/bin/bash

objcopy -O binary -I elf32-little target/riscv32imac-unknown-none-elf/$1/first-flash-longan-nano firmware.bin
dfu-util -a 0 -s 0x08000000:leave -D firmware.bin
