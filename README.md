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
  
