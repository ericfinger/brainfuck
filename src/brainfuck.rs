use std::io::{BufRead, Write};

use bimap::BiMap;
pub struct VM {
    program: String,
    pp: usize, // ProgramPointer
    mp: usize, // MemoryPointer
    data: Vec<u8>,
    jump_map: BiMap<usize, usize>,
    #[cfg(test)]
    output: String,
}

impl VM {
    pub fn new(program: &str) -> Self {
        // TODO: Dynamically grow this if needed & start with smaller defaults
        let data = vec![0; 1024];
        let mut jump_map = BiMap::<usize, usize>::new();

        if !Self::parse(program, &mut jump_map) {
            // TODO: Give better Debug output
            panic!("Mismatched Brackets!");
        }

        Self {
            program: program.to_string(),
            pp: 0,
            mp: 0,
            data,
            jump_map,
            #[cfg(test)]
            output: String::new(),
        }
    }

    pub fn load(&mut self, program: &str) {
        if !Self::parse(program, &mut self.jump_map) {
            // TODO: Give better ~~Debug~~ output
            panic!("Mismatched Brackets!");
        }

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
                '>' => {
                    // pointer += 1;
                    self.mp += 1;
                    self.pp += 1;
                }
                '<' => {
                    // pointer -= 1;
                    self.mp -= 1;
                    self.pp += 1;
                }
                '+' => {
                    // *pointer += 1;
                    self.data[self.mp] = self.data[self.mp].wrapping_add(1);
                    self.pp += 1;
                }
                '-' => {
                    // *pointer -= 1;
                    self.data[self.mp] = self.data[self.mp].wrapping_sub(1);
                    self.pp += 1;
                }
                '.' => {
                    // putchar(*pointer)
                    print!("{}", self.data[self.mp] as char);
                    std::io::stdout()
                        .lock()
                        .flush()
                        .expect("Could not flush stdout");
                    #[cfg(test)]
                    self.output.push(self.data[self.mp] as char);
                    self.pp += 1;
                }
                ',' => {
                    //getchar(*pointer)
                    let mut input = String::new();
                    std::io::stdin()
                        .lock()
                        .read_line(&mut input)
                        .expect("Couldn't read from stdin");
                    self.data[self.mp] = input
                        .chars()
                        .nth(0)
                        .expect("No input could be read from stdin?")
                        as u8;
                    self.pp += 1;
                }
                '[' => {
                    // if *pointer == 0: goto end of while)
                    if self.data[self.mp] == 0 {
                        self.pp = self
                            .jump_map
                            .get_by_left(&self.pp)
                            .expect("Incorrect jumpmap?! Please report this error")
                            + 1;
                    } else {
                        self.pp += 1;
                    }
                }
                ']' => {
                    // } (or "if *pointer != 0: goto start of while")
                    if self.data[self.mp] != 0 {
                        self.pp = self
                            .jump_map
                            .get_by_right(&self.pp)
                            .expect("Incorrect jumpmap?! Please report this error")
                            + 1;
                    } else {
                        self.pp += 1;
                    }
                }
                _ => {
                    self.pp += 1;
                }
            }
        }
    }

    // Parses the program and reports errors
    // TODO: actually report errors & introduce Error Type
    fn parse(program: &str, jump_map: &mut BiMap<usize, usize>) -> bool {
        if !Self::check_brackets(program, jump_map) {
            return false;
        }

        true
    }

    /// Checks if all '[' brackts have a matching ']' bracket.
    /// Inserts the index of each '[' and it's matching ']' bracket into the given BiMap
    fn check_brackets(program: &str, jump_map: &mut BiMap<usize, usize>) -> bool {
        let mut count = 0;
        let mut machting_bracket: Option<usize>;
        for (i, op) in program.chars().enumerate() {
            match op {
                '[' => {
                    machting_bracket = None;
                    for (j, c) in program.chars().enumerate().skip(i + 1) {
                        if c == '[' {
                            count += 1;
                        } else if c == ']' {
                            if count == 0 {
                                machting_bracket = Some(j);
                                break;
                            } else {
                                count -= 1;
                            }
                        }
                    }

                    match machting_bracket {
                        None => {
                            return false;
                        }
                        Some(j) => {
                            //println!("('[' at {}): Found matching Bracket ']' at {}", i, j);
                            jump_map.insert(i, j);
                        }
                    }
                }
                ']' => {
                    machting_bracket = None;
                    for (j, c) in program.chars().rev().enumerate().skip(program.len() - i) {
                        if c == ']' {
                            count += 1;
                        } else if c == '[' {
                            if count == 0 {
                                machting_bracket = Some(program.len() - j);
                                break;
                            } else {
                                count -= 1;
                            }
                        }
                    }

                    match machting_bracket {
                        None => {
                            return false;
                        }
                        Some(_j) => {
                            //println!("('[' at {}): Found matching Bracket ']' at {}", i, j);

                            // Ignoring value for now bc we already added it in the open-Bracket loop
                            // but might be useful for debugger later
                        }
                    }
                }
                _ => continue,
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reset() {
        let program = include_str!("../brainfuck_programs/hello_world_smol.bf");
        let mut vm = VM::new(program);
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
    fn reset_reuse() {
        let program = include_str!("../brainfuck_programs/hello_world_smol.bf");
        let mut vm = VM::new(program);
        vm.load(program);
        vm.run();

        assert_eq!("hello world", vm.output);

        vm.reset();
        let program = include_str!("../brainfuck_programs/yapi_4.bf");
        vm.load(program);
        vm.run();
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
    fn layered_brackets() {
        let program = include_str!("../brainfuck_programs/layeredBracketsTest.bf");
        let mut vm = VM::new(program);
        vm.run();
    }

    #[test]
    #[should_panic]
    fn open_ended_while() {
        let program = include_str!("../brainfuck_programs/openEndedWhile.bf");
        let mut vm = VM::new(program);
        vm.run();
    }

    #[test]
    #[should_panic]
    fn headless_while() {
        let program = include_str!("../brainfuck_programs/headlessWhile.bf");
        let mut vm = VM::new(program);
        vm.run();
    }

    #[test]
    #[should_panic]
    fn mem_pointer_underflow() {
        let program = include_str!("../brainfuck_programs/underflowMP.bf");
        let mut vm = VM::new(program);
        vm.run();
    }

    #[test]
    #[should_panic]
    fn mem_pointer_overflow() {
        // TODO: replace this (with an "out of memory check") when we implement dynamic memory sizes
        let program = include_str!("../brainfuck_programs/overflowMP.bf");
        let mut vm = VM::new(program);
        vm.run();
    }

    #[test]
    fn hello_world() {
        let program = include_str!("../brainfuck_programs/hello_world.bf");
        let mut vm = VM::new(program);
        vm.run();

        // Super fucking weird, why tf is it \n\r??? It's from https://de.wikipedia.org/wiki/Brainfuck
        assert_eq!("Hello World!\n\r", vm.output);
    }

    #[test]
    fn hello_world_smol() {
        let program = include_str!("../brainfuck_programs/hello_world_smol.bf");
        let mut vm = VM::new(program);
        vm.run();

        assert_eq!("hello world", vm.output);
    }

    #[test]
    fn hell() {
        // "Hello world from hell": https://github.com/rdebath/Brainfuck/blob/master/bitwidth.b
        let program = include_str!("../brainfuck_programs/hell.bf");
        let mut vm = VM::new(program);
        vm.run();

        assert_eq!("Hello World! 255\n", vm.output);
    }

    #[test]
    fn squares() {
        let program = include_str!("../brainfuck_programs/squares.bf");
        let mut vm = VM::new(program);
        vm.run();

        let should_be = include_str!("../brainfuck_programs/squares_output_correct.txt");
        assert_eq!(should_be, vm.output);
    }

    #[test]
    fn quine() {
        // Written by Erik Bosman
        let program = include_str!("../brainfuck_programs/quine.bf");
        let mut vm = VM::new(program);
        vm.run();

        assert_eq!(program, vm.output);
    }

    #[test]
    fn obscure() {
        let program = include_str!("../brainfuck_programs/obscure.bf");
        let mut vm = VM::new(program);
        vm.run();

        assert_eq!("H\n", vm.output);
    }

    #[test]
    fn fibonacci() {
        let program = include_str!("../brainfuck_programs/fibonacci.bf");
        let mut vm = VM::new(program);
        vm.run();

        // yes those are wrong, but that's the programs fault. These numbers are from https://copy.sh/brainfuck which I assume is correct
        assert_eq!(
            "1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 121, 98, 219, ...",
            vm.output
        );
    }
}
