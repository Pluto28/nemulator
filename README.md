### What is this?
A nes emulator. I decided to develop this to become more acquainted with
the rust language. That is, this is essentially a toy project made 
to provide nightmarish amounts of fun to the programmer.

## Useful links
# references
https://www.nesdev.org/obelisk-6502-guide/reference.html

http://www.6502.org/tutorials/6502opcodes.html

# tutorials
https://skilldrick.github.io/easy6502/ 


### Useful doc for myself

# Flag register
0bvvvv_vvvv
  |||| ||||-> Carry(1)
  |||| |||
  |||| |||-> Zero(1)
  |||| ||
  |||| ||-> IRQ disable(1)
  |||| ||
  |||| |-> Decimal mode(1)
  ||||
  ||||-> Brk command (1)
  |||
  |||-> Nothing
  ||
  ||-> overflow flag(1)
  |
  |-> negative flag(1)
  

# Page
    The 6502 divides its memory address into 256 pages, each having 256 bytes 
of memory.
    Then the proccessor has access to a memory space of 256\*256=65356 bytes 
of memory in total. The most important pages are:
    * Page 0: Contains usually referenced variables to save space in program code
    * Page 1: Hardwired by the processor to be used as the stack
    * last page: The last 6 bytes of this page contain special addresses
