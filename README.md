# Brainfuck
A simple brainfuck interpreter written in Rust.

### Todo:

- [X] Get the basic interpreter working
- [X] Write Tests
- [ ] Rewrite in safe Rust
- [ ] Remove the panic by requiring a program in the constructor (thus preventing bad states)?
- [ ] Cache while-loop "jump points" (maybe even ahead of time?)
- [ ] Run profiler to check performance, mb unsafe is faster?
- [ ] Dynamically grow Memory
- [ ] Add support for different cell sizes
- [ ] UI, loading Programms at runtime
- [ ] Better Error-handling, backtraces for when bf programs crash etc.
- [ ] Better I/O for stuff like wc.bf or rot13.bf
- [ ] Better I/O for tests
- [ ] GUI?
- [ ] Debugging capabilities (stepping, memory-state etc.)
