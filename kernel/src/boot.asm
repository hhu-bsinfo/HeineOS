; Contains the multiboot2 header and the kernel entry point, which is called by the bootloader.
;
; Author: Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-01-07
; License: GPLv3

; Import symbols
[EXTERN ___BSS_START__]
[EXTERN ___BSS_END__]
[EXTERN main]

; Export symbols
[GLOBAL load_gdt]

; Kernel constants
STACK_SIZE equ 0x10000

; Multiboot2 constants
MULTIBOOT2_HEADER_MAGIC equ 0xe85250d6
MULTIBOOT2_HEADER_ARCHITECTURE equ 0
MULTIBOOT2_HEADER_LENGTH equ (start - multiboot2_header)
MULTIBOOT2_HEADER_CHECKSUM equ -(MULTIBOOT2_HEADER_MAGIC + MULTIBOOT2_HEADER_ARCHITECTURE + MULTIBOOT2_HEADER_LENGTH)

; Multiboot2 tag types
MULTIBOOT2_TAG_TERMINATE equ 0
MULTIBOOT2_TAG_INFORMATION_REQUEST equ 1
MULTIBOOT2_TAG_ADDRESS equ 2
MULTIBOOT2_TAG_ENTRY_ADDRESS equ 3
MULTIBOOT2_TAG_FLAGS equ 4
MULTIBOOT2_TAG_FRAMEBUFFER equ 5
MULTIBOOT2_TAG_MODULE_ALIGNMENT equ 6
MULTIBOOT2_TAG_EFI_BOOT_SERVICES equ 7
MULTIBOOT2_TAG_EFI_I386_ENTRY_ADDRESS equ 8
MULTIBOOT2_TAG_EFI_AMD64_ENTRY_ADDRESS equ 9
MULTIBOOT2_TAG_RELOCATABLE_HEADER equ 10

; Multiboot2 request types
MULTIBOOT2_REQUEST_BOOT_COMMAND_LINE equ 1
MULTIBOOT2_REQUEST_BOOT_LOADER_NAME equ 2
MULTIBOOT2_REQUEST_MODULE equ 3
MULTIBOOT2_REQUEST_BASIC_MEMORY_INFORMATION equ 4
MULTIBOOT2_REQUEST_BIOS_BOOT_DEVICE equ 5
MULTIBOOT2_REQUEST_MEMORY_MAP equ 6
MULTIBOOT2_REQUEST_VBE_INFO equ 7
MULTIBOOT2_REQUEST_FRAMEBUFFER_INFO equ 8
MULTIBOOT2_REQUEST_ELF_SYMBOLS equ 9
MULTIBOOT2_REQUEST_APM_TABLE equ 10
MULTIBOOT2_REQUEST_EFI_32_BIT_SYSTEM_TABLE_POINTER equ 11
MULTIBOOT2_REQUEST_EFI_64_BIT_SYSTEM_TABLE_POINTER equ 12
MULTIBOOT2_REQUEST_SMBIOS_TABLES equ 13
MULTIBOOT2_REQUEST_ACPI_OLD_RSDP equ 14
MULTIBOOT2_REQUEST_ACPI_NEW_RSDP equ 15
MULTIBOOT2_REQUEST_NETWORKING_INFORMATION equ 16
MULTIBOOT2_REQUEST_EFI_MEMORY_MAP equ 17
MULTIBOOT2_REQUEST_EFI_BOOT_SERVICES_NOT_TERMINATED equ 18
MULTIBOOT2_REQUEST_EFI_32_BIT_IMAGE_HANDLE_POINTER equ 19
MULTIBOOT2_REQUEST_EFI_64_BIT_IMAGE_HANDLE_POINTER equ 20
MULTIBOOT2_REQUEST_IMAGE_LOAD_BASE_PHYSICAL_ADDRESS equ 21

; Multiboot2 tag flags
MULTIBOOT2_TAG_FLAG_REQUIRED equ 0x00
MULTIBOOT2_TAG_FLAG_OPTIONAL equ 0x01

; Multiboot2 console flags
MULTIBOOT2_CONSOLE_FLAG_FORCE_TEXT_MODE equ 0x01
MULTIBOOT2_CONSOLE_FLAG_SUPPORT_TEXT_MODE equ 0x02

[SECTION .text]
[BITS 64]

multiboot2_header:
    ; Header
    align 8
    dd MULTIBOOT2_HEADER_MAGIC
    dd MULTIBOOT2_HEADER_ARCHITECTURE
    dd MULTIBOOT2_HEADER_LENGTH
    dd MULTIBOOT2_HEADER_CHECKSUM

    ; EFI amd64 entry address tag:
    ; Tell the bootloader where the `start` label is located, so it can jump there.
    align 8
    dw MULTIBOOT2_TAG_EFI_AMD64_ENTRY_ADDRESS
    dw MULTIBOOT2_TAG_FLAG_REQUIRED
    dd 12
    dd (start)

    ; EFI boot services tag:
    ; Tell the bootloader to keep the EFI boot services active.
    align 8
    dw MULTIBOOT2_TAG_EFI_BOOT_SERVICES
    dw MULTIBOOT2_TAG_FLAG_REQUIRED
    dd 8

    ; Information request tag:
    ; Request specific information from the bootloader (boot loader name, command line, modules, framebuffer info).
    align 8
    dw MULTIBOOT2_TAG_INFORMATION_REQUEST
    dw MULTIBOOT2_TAG_FLAG_REQUIRED
    dd 36
    dd MULTIBOOT2_REQUEST_BOOT_LOADER_NAME
    dd MULTIBOOT2_REQUEST_BOOT_COMMAND_LINE
    dd MULTIBOOT2_REQUEST_MODULE
    dd MULTIBOOT2_REQUEST_FRAMEBUFFER_INFO
    dd MULTIBOOT2_REQUEST_EFI_BOOT_SERVICES_NOT_TERMINATED
    dd MULTIBOOT2_REQUEST_EFI_64_BIT_IMAGE_HANDLE_POINTER
    dd MULTIBOOT2_REQUEST_EFI_64_BIT_SYSTEM_TABLE_POINTER

    ; Framebuffer tag:
    ; Request a framebuffer with specific parameters (800x600, 32bpp).
    align 8
    dw MULTIBOOT2_TAG_FRAMEBUFFER
    dw MULTIBOOT2_TAG_FLAG_REQUIRED
    dd 20
    dd 800
    dd 600
    dd 32

    ; Termination tag:
    ; Marks the end of the multiboot2 header.
    align 8
    dw MULTIBOOT2_TAG_TERMINATE
    dw MULTIBOOT2_TAG_FLAG_REQUIRED
    dd 8

; Kernel entry point (called by the bootloader)
start:
    cli ; Disable interrupts

    ; Clear BSS section
    mov rdi, ___BSS_START__
clear_bss:
    mov byte [rdi], 0
    inc rdi
    cmp rdi, ___BSS_END__
    jne clear_bss

    ; Switch stack to our own stack, because the EFI stack may be located inside
    ; reserved memory and will thus be ignored by our paging implementation.
    mov rsp, init_stack.end

    ; Call rust function with multiboot2 magic number and address (initially located in eax and ebx)
    xor rdi, rdi
    xor rsi, rsi
    mov edi, eax
    mov esi, ebx
    call main

load_gdt:
    lgdt [fs:gdt_descriptor]

    ; Clear segment registers, that are not used in 64-bit mode
    mov ax, 0
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    ; Load data segment selector into stack segment register
    mov ax, (2 * 0x08)
    mov ss, ax

    ; Jump into code segment
    push (1 * 0x08)
    push qword load_cs_jmp
    retfq

load_cs_jmp:
    ret;

[SECTION .data]

; Global Descriptor Table (GDT)
align 8
gdt:
	dw  0, 0, 0, 0  ; Null descriptor

	; Kernel code segment descriptor
	dw  0xffff      ; 4 GiB (0x100000 * 0x1000 = 4 GiB)
	dw  0x0000      ; base address = 0
	dw  0x9a00      ; code read/exec
	dw  0x00af      ; granularity = 4096, size flag (+5th nibble of limit), Long-Mode

	; Kernel data segment descriptor
	dw  0xffff      ; 4 GiB (0x100000 * 0x1000 = 4 GiB)
	dw  0x0000      ; base address = 0
	dw  0x9200      ; data read/write
	dw  0x00cf      ; granularity = 4096, size flag (+5th nibble of limit)

; Descriptor for the GDT
align 8
gdt_descriptor:
    dw 3 * 8 - 1    ; GDT has 3 entries of 8 bytes each; Limit = size - 1
    dq gdt          ; Address of the GDT

[SECTION .bss]

; The kernel stack
global init_stack:data (init_stack.end - init_stack)
init_stack:
	  resb STACK_SIZE
.end: