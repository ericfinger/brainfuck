use std::io::{BufRead, Write};

struct VM {
    program: String,
    pp: usize, // ProgramPointer
    mp: usize, // MemoryPointer
    data: Vec<u8>,
    #[cfg(test)]
    output: String,
}

impl VM {
    pub fn new(program: &str) -> Self {
        // TODO: Dynamically grow this if needed & start with smaller defaults
        let data = vec![0; 1024];
        Self {
            program: program.to_string(),
            pp: 0,
            mp: 0,
            data,
            #[cfg(test)]
            output: String::new(),
        }
    }

    pub fn load(&mut self, program: &str) {
        self.program = program.to_string();
    }

    pub fn reset(&mut self) {
        self.pp = 0;
        self.data.fill(0);
        #[cfg(test)]
        self.output.clear();
    }

    pub fn run(&mut self) {
        while self.pp < self.program.len() {
            match self
                .program
                .chars()
                .nth(self.pp)
                .expect("End of program reached prematurely!")
            {
                '>' => { // pointer += 1;
                    self.mp += 1;
                    self.pp += 1;
                }
                '<' => { // pointer -= 1;
                    self.mp -= 1;
                    self.pp += 1;
                }
                '+' => { // *pointer += 1;
                    self.data[self.mp] = self.data[self.mp].wrapping_add(1);
                    self.pp += 1;
                }
                '-' => { // *pointer -= 1;
                    self.data[self.mp] = self.data[self.mp].wrapping_sub(1);
                    self.pp += 1;
                }
                '.' => { // putchar(*pointer)
                    print!("{}", self.data[self.mp] as char);
                    std::io::stdout().lock().flush().expect("Could not flush stdout");
                    #[cfg(test)]
                    self.output.push(self.data[self.mp] as char);
                    self.pp += 1;
                }
                ',' => { //getchar(*pointer)
                    let mut input = String::new();
                    std::io::stdin().lock().read_line(&mut input).expect("Couldn't read from stdin");
                    self.data[self.mp] = input.chars().nth(0).expect("No input could be read from stdin?") as u8;
                    self.pp += 1;
                }
                '[' => { // if *pointer == 0: goto end of while)
                    if self.data[self.mp] == 0 {
                        let mut count: u32 = 0;
                        // TODO: Cache jump points once we found the matching ']' for quicker repeats (loops in loops)?
                        //       maybe even cache ALL jump points for all loops in the beginning of the program?? self.pp = address
                        for x in self.pp + 1..self.program.len() {
                            let current_char = self
                            .program
                            .chars()
                            .nth(x)
                            .expect("didn't find matching ']'!");

                            if current_char == '[' {
                                count += 1;
                            }

                            if current_char == ']' {
                                if count == 0 {
                                    self.pp = x + 1;
                                    break;
                                } else {
                                    count -= 1;
                                }
                            }
                        }
                    } else {
                        self.pp += 1;
                    }
                }
                ']' => { // } (or "if *pointer != 0: goto start of while")
                    // TODO: Rewrite so we add a "jump point" to a list when entering a while loop
                    // and don't have to scan the program for the beginning '[' every time
                    if self.data[self.mp] != 0 {
                        let mut count: u32 = 0;
                        for x in 1 .. self.pp {
                            let current_char = self.program.chars().nth(self.pp - x).expect("didn't find matching '['!");
                            
                            if current_char == ']' {
                                count += 1;
                            }

                            if current_char == '[' {
                                if count == 0 {
                                    self.pp = (self.pp - x) + 1;
                                    break;
                                } else {
                                    count -= 1;
                                }
                            }
                        }
                    } else {
                        self.pp += 1;
                    }
                }
                _ => {
                    self.pp += 1;
                },
            }
        }
    }
}

fn main() {
    let program = include_str!("../brainfuck_programs/hello_world.bf");
    let mut vm = VM::new(program);
    vm.run();

    vm.reset();
    let program = include_str!("../brainfuck_programs/yapi_4.bf");
    vm.load(program);
    vm.run();

    vm.reset();
    let program = include_str!("../brainfuck_programs/triangle.bf");
    vm.load(program);
    vm.run();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reset() {
        let program = include_str!("../brainfuck_programs/hello_world_smol.bf");
        let mut vm = VM::new(program);
        vm.load(program);
        vm.run();

        assert_eq!("hello world", vm.output);
    
        vm.reset();
        assert_eq!(program, vm.program);
        assert_eq!("", vm.output);
        assert_eq!(0, vm.pp);
        
        let mut zerod = true;
        for b in vm.data {
            if b != 0 {
                zerod = false;
            }
        }
        assert!(zerod);
    }

    #[test]
    fn no_program() {
        let mut vm = VM::new("");
        vm.run();

        assert!(vm.output.is_empty() && vm.pp == 0 && vm.mp == 0);
        
        let mut zerod = true;
        for b in vm.data {
            if b != 0 {
                zerod = false;
            }
        }
        assert!(zerod);
    }

    #[test]
    #[should_panic]
    fn mem_pointer_underflow() {
        let program = include_str!("../brainfuck_programs/underflowMP.bf");
        let mut vm = VM::new(program);
        vm.load(program);
        vm.run();
    }

    #[test]
    #[should_panic]
    fn mem_pointer_overflow() {
        // TODO: replace this (with an "out of memory check") when we implement dynamic memory sizes
        let program = include_str!("../brainfuck_programs/overflowMP.bf");
        let mut vm = VM::new(program);
        vm.load(program);
        vm.run();
    }

    #[test]
    fn hello_world() {
        let program = include_str!("../brainfuck_programs/hello_world.bf");
        let mut vm = VM::new(program);
        vm.load(program);
        vm.run();

        // Super fucking weird, why tf is it \n\r??? It's from https://de.wikipedia.org/wiki/Brainfuck 
        assert_eq!("Hello World!\n\r", vm.output);
    }

    #[test]
    fn hello_world_smol() {
        let program = include_str!("../brainfuck_programs/hello_world_smol.bf");
        let mut vm = VM::new(program);
        vm.load(program);
        vm.run();

        assert_eq!("hello world", vm.output);
    }
    
    #[test]
    fn hell() {
        // "Hello world from hell": https://github.com/rdebath/Brainfuck/blob/master/bitwidth.b
        let program = include_str!("../brainfuck_programs/hell.bf");
        let mut vm = VM::new(program);
        vm.load(program);
        vm.run();

        assert_eq!("Hello World! 255\n", vm.output);
    }
    
    #[test]
    fn squares() {
        let program = include_str!("../brainfuck_programs/squares.bf");
        let mut vm = VM::new(program);
        vm.load(program);
        vm.run();

        let should_be = include_str!("../brainfuck_programs/squares_output_correct.txt");
        assert_eq!(should_be, vm.output);
    }

    #[test]
    fn quine() {
        // Written by Erik Bosman
        let program = include_str!("../brainfuck_programs/quine.bf");
        let mut vm = VM::new(program);
        vm.load(program);
        vm.run();

        assert_eq!(program, vm.output);
    }

    #[test]
    fn obscure() {
        let program = include_str!("../brainfuck_programs/obscure.bf");
        let mut vm = VM::new(program);
        vm.load(program);
        vm.run();

        assert_eq!("H\n", vm.output);
    }

    #[test]
    fn fibonacci() {
        let program = include_str!("../brainfuck_programs/fibonacci.bf");
        let mut vm = VM::new(program);
        vm.load(program);
        vm.run();

        // yes those are wrong, but that's the programs fault. These numbers are from https://copy.sh/brainfuck which I assume is correct
        assert_eq!("1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 121, 98, 219, ...", vm.output);
    }
}
