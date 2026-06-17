ENTRY(main)

PHDRS
{
    text PT_LOAD;
    rodata PT_LOAD;
    data PT_LOAD;
    bss PT_LOAD;
}

SECTIONS {
    . = 0x10000000000;   /* load at address 1 TB */

    ___APP_DATA_START__ = .;

    .text :
        {
            *(.main*)
            *(.text*)
        } :text

        .rodata :
        {
            *(.rodata*)
        } :rodata

        .data :
        {
            *(.data*)
        } :data

       .bss :
        {
          ___BSS_START__ = .;
          *(".bss.*")
          ___BSS_END__ = .;
        } :bss

    ___APP_DATA_END__ = .;
}