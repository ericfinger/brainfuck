# Brainfuck
A simple brainfuck interpreter written in Rust.

### Todo:

- [X] Get the basic interpreter working
- [X] Write Tests
- [X] Rewrite in safe Rust
- [X] Remove the panic by requiring a program in the constructor (thus preventing bad states)?
- [ ] Write documentation
- [ ] Cache while-loop "jump points" (maybe even ahead of time?) (Would allow us to catch "openEndedWhile.bf" and "headlessWhile.bf" / mismatched brackets in general)
- [ ] Add better comments to brainfuck files (ignore stuff after ; when parsing)
- [ ] Better Error-handling, backtraces for when bf programs crash etc.
- [ ] Dynamically grow Memory/prevent overflow of Memory Pointer
- [ ] Add support for different cell sizes
- [ ] UI, loading Programms at runtime
- [ ] Run profiler to check performance, mb unsafe is faster?
- [ ] Better I/O for stuff like wc.bf or rot13.bf
- [ ] Better I/O for tests
- [ ] GUI?
- [ ] Debugging capabilities (stepping, memory-state etc.)
- [ ] Probably refactor into lib/mod

### Current State:

- Has no UI
- Can therefore only run hard-coded brainfuck
- Does not handle mismatched While loops (Hangs or crashes)
- Does ignore any non-opcode instructions but does not respect comments (lines starting with ; or // or # etc)
- Has a default (hard-coded) & fixed memory-size of 1KB
- A cell is a byte per brainfuck "spec" (8 bits)
- Might not be the fastest (although with --release it's quite good already, IO is somewhat slow bc we print every char as per spec)
