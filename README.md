# Brainfuck
A brainfuck interpreter written in Rust.

### Todo:

- [X] Get the basic interpreter working
- [X] Write Tests
- [X] Rewrite in safe Rust
- [X] Remove the panic by requiring a program in the constructor (thus preventing bad states)
- [X] Refactor into mod
- [X] Cache while-loop "jump points"
- [X] Parse programs / allow comments in brainfuck files (ignore stuff after ';' and '#' when parsing)
- [ ] Better Error-handling, backtraces for when bf programs crash etc.
- [ ] Better config management (Disable comments etc)
- [ ] Dynamically grow Memory/prevent overflow of Memory Pointer (we can statically find the needed size by analysing the program?)
- [ ] Write documentation
- [ ] UI, loading Programms at runtime
- [ ] Add support for different cell sizes
- [ ] Add better debug-output/logging
- [ ] Run profiler to check performance for bottlenecks
- [ ] Better I/O for stuff like wc.bf or rot13.bf
- [ ] Better I/O for tests
- [ ] Debugging capabilities (stepping, memory-state etc.)
- [ ] Possible Optimizations (with toggle option?):
  - [ ] Collect successive ops ("++++<" = "add(4)<" etc.) (Would save cycles)
  - [ ] While Loop unroll? (Would eradicate ALL jumps and therefore the Program Vec + Jumpmap)
  - [ ] Simple pattern matching ("[-]" = "*mp = 0") (Manual labor)
  - [ ] Remove never entered Loops (meh)
- [ ] GUI?
- [ ] Visualizer?

### Current State/Issues:

- Has no UI
- Can therefore only run hard-coded brainfuck
- Has a default (hard-coded) & fixed memory-size of 1KB
- A cell is a byte per brainfuck "spec" (8 bits)
- Might not be the fastest (although with --release it's quite good already, IO is somewhat slow bc we print every char as per spec)
