# Brainfuck
A brainfuck interpreter written in Rust.

### Current State/Issues:

- Only has CLI
- Has a default (hard-coded) & fixed memory-size of 1KB
- A cell is a byte (8 bits) as per brainfuck "spec" 
- IO is somewhat slow bc we print every char as per spec
