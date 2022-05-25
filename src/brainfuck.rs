#[allow(unused_imports)]
use std::io::{BufRead, StdoutLock, Write};

#[cfg(test)]
use newline_converter::dos2unix;

use rustc_hash::FxHashMap;

#[derive(Debug)]
pub struct VMOptions {
    pub program: String,
    pub disable_optimizer: bool,
    pub disable_comments: bool,
}

#[cfg(test)]
impl VMOptions {
    #[cfg(test)]
    fn default(program: String) -> VMOptions {
        VMOptions {
            program,
            disable_comments: false,
            disable_optimizer: true,
        }
    }
}

pub struct VM<'a> {
    program: Vec<u8>,
    pp: usize, // ProgramPointer
    mp: usize, // MemoryPointer
    data: Vec<u8>,
    jump_map: FxHashMap<usize, usize>,
    ignore_comments: bool, // wether we should ignore comments (obscure.bf and hell.bf use ';' as non-comment chars)
    optimize: bool,
    #[allow(dead_code)]
    stdout: StdoutLock<'a>,
    #[cfg(test)]
    output: String,
}

impl<'a> VM<'a> {
    pub fn new(options: VMOptions) -> Self {
        let mut vm = Self {
            program: Vec::<u8>::new(),
            pp: 0,
            mp: 0,
            data: vec![0; 1024], // TODO: Dynamically grow this (static analysis of program possible??) if needed & start with smaller defaults
            jump_map: FxHashMap::default(),
            ignore_comments: !options.disable_comments,
            optimize: !options.disable_optimizer,
            stdout: std::io::stdout().lock(),
            #[cfg(test)]
            output: String::new(),
        };

        vm.parse(options.program).expect("Couldn't parse program");
        vm
    }

    #[cfg(test)]
    pub fn load(&mut self, program: String) {
        self.program.clear();
        self.parse(program).expect("Couldn't parse program");
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.pp = 0;
        self.mp = 0;
        self.data.fill(0);
        #[cfg(test)]
        self.output.clear();
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub fn enable_optimizer(&mut self, program: &str) {
        self.optimize = true;
        self.load(program.to_string());
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub fn disable_optimizer(&mut self, program: &str) {
        self.optimize = false;
        self.load(program.to_string());
    }

    pub fn run(&mut self) {
        while self.pp < self.program.len() {
            match self.program[self.pp] {
                b'>' => {
                    // pointer += 1;
                    self.mp += 1;
                    self.pp += 1;
                }

                b'<' => {
                    // pointer -= 1;
                    self.mp -= 1;
                    self.pp += 1;
                }

                b'+' => {
                    // *pointer += 1;
                    self.data[self.mp] = self.data[self.mp].wrapping_add(1);
                    self.pp += 1;
                }

                b'-' => {
                    // *pointer -= 1;
                    self.data[self.mp] = self.data[self.mp].wrapping_sub(1);
                    self.pp += 1;
                }

                b'.' => {
                    // putchar(*pointer)
                    // print!("{}", self.data[self.mp] as char);
                    #[cfg(not(test))]
                    let _ = self.stdout
                        .write(std::slice::from_mut(&mut self.data[self.mp]))
                        .expect("could not write to stdout");

                    // std::io::stdout().flush().expect("Couldn't flush stdout.");
                    #[cfg(not(test))]
                    self.stdout.flush().expect("Could not flush stdout");

                    self.pp += 1;

                    #[cfg(test)]
                    self.output.push(self.data[self.mp] as char);
                }

                b',' => {
                    //getchar(*pointer)
                    let mut input = String::new();
                    std::io::stdin()
                        .lock()
                        .read_line(&mut input)
                        .expect("Couldn't read from stdin");
                    self.data[self.mp] = input
                        .chars()
                        .next()
                        .expect("No input could be read from stdin?")
                        as u8;
                    self.pp += 1;
                }

                b'[' => {
                    // if *pointer == 0: goto end of while)
                    if self.data[self.mp] == 0 {
                        self.pp = self
                            .jump_map
                            .get(&self.pp)
                            .expect("Incorrect jumpmap?! Please report this error")
                            + 1;
                    } else {
                        self.pp += 1;
                    }
                }

                b']' => {
                    // } (or "if *pointer != 0: goto start of while")
                    if self.data[self.mp] != 0 {
                        self.pp = self
                            .jump_map
                            .get(&self.pp)
                            .expect("Incorrect jumpmap?! Please report this error")
                            + 1;
                    } else {
                        self.pp += 1;
                    }
                }

                _ => {
                    if self.program[self.pp] & 0b10000000 == 0 {
                        // Shouldn't happen after parsing
                        panic!("Parsing silently failed?");
                        //self.pp += 1;
                    } else {
                        let op = self.program[self.pp];
                        if op & 0b01000000 == 0 {
                            // Pointer Arithmetic
                            if op & 0b00100000 == 0 {
                                // Pointer Minus
                                self.mp -= ((op & 0b00011111) + 1) as usize;
                                self.pp += 1;
                            } else {
                                // Pointer Plus
                                self.mp += ((op & 0b00011111) + 1) as usize;
                                self.pp += 1;
                            }
                        } else {
                            // Number Arithmetic
                            if op & 0b00100000 == 0 {
                                // Minus
                                self.data[self.mp] =
                                    self.data[self.mp].wrapping_sub((op & 0b00011111) + 1);
                                self.pp += 1;
                            } else {
                                // Plus
                                self.data[self.mp] =
                                    self.data[self.mp].wrapping_add((op & 0b00011111) + 1);
                                self.pp += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    #[cfg(test)]
    pub fn get_program(&self) -> String {
        let program: String = self.program.iter().map(|op| *op as char).collect();
        program
    }

    /// Parses the program and reports errors
    /// TODO: actually report errors & introduce Error Type
    fn parse(&mut self, program: String) -> Result<(), Box<dyn std::error::Error>> {
        let mut parsed_program: Vec<u8> = Vec::new();
        for line in program.lines() {
            for c in line.chars() {
                match c {
                    '<' | '>' | '+' | '-' | '[' | ']' | '.' | ',' => {
                        parsed_program.push(c as u8);
                    }

                    ';' | '#' => {
                        // ignore until EOL after comment char
                        if self.ignore_comments {
                            break;
                        }
                    }

                    _ => continue,
                }
            }
        }

        if self.optimize {
            self.optimize_successive(parsed_program);
        } else {
            self.program = parsed_program;
        }

        if !self.check_brackets() {
            return Err("Could not parse Program: Mismatched Brackets!".into());
        }

        Ok(())
    }

    /// Checks if all ``[`` brackets have a matching ``]`` bracket.
    /// Inserts the index of each ``[`` and it's matching ``]`` bracket into ``self.jump_map``
    fn check_brackets(&mut self) -> bool {
        let mut count = 0;
        let mut matching_bracket: Option<usize>;
        for (i, op) in self.program.iter().enumerate() {
            match op {
                b'[' => {
                    matching_bracket = None;
                    for (j, c) in self.program.iter().enumerate().skip(i + 1) {
                        if *c == b'[' {
                            count += 1;
                        } else if *c == b']' {
                            if count == 0 {
                                matching_bracket = Some(j);
                                break;
                            } else {
                                count -= 1;
                            }
                        }
                    }

                    match matching_bracket {
                        None => {
                            return false;
                        }
                        Some(j) => {
                            self.jump_map.insert(i, j);
                            self.jump_map.insert(j, i);
                        }
                    }
                }

                b']' => {
                    let mut matching_bracket: bool = false;
                    for c in self.program.iter().rev().skip(self.program.len() - i) {
                        if *c == b']' {
                            count += 1;
                        } else if *c == b'[' {
                            if count == 0 {
                                matching_bracket = true;
                                break;
                            } else {
                                count -= 1;
                            }
                        }
                    }

                    if !matching_bracket {
                        return false;
                    }
                }

                _ => continue,
            }
        }

        true
    }

    /// Optimizes successive '+' '-' '>' and '<' calls by combining them.
    /// For Example, '++++' would turn into something like add(4).
    fn optimize_successive(&mut self, program: Vec<u8>) {
        let mut skip = 0;
        for (i, op) in program.iter().enumerate() {
            if skip > 0 {
                skip -= 1;
                continue;
            }

            match op {
                b'+' => {
                    skip = self.push_special_instruction(i, *op, 0b11100000, &program);
                }

                b'-' => {
                    skip = self.push_special_instruction(i, *op, 0b11000000, &program);
                }

                b'>' => {
                    skip = self.push_special_instruction(i, *op, 0b10100000, &program);
                }

                b'<' => {
                    skip = self.push_special_instruction(i, *op, 0b10000000, &program);
                }

                _ => {
                    self.program.push(*op);
                }
            }
        }
    }

    /// Pushes the special instruction for successive operands.
    fn push_special_instruction(
        &mut self,
        current_pos: usize,
        operator: u8,
        instruction_mask: u8,
        program: &[u8],
    ) -> usize {
        let mut skip = 0;
        if (program.len() - current_pos) > 1 {
            // makes sure we don't try to lookup program [i + 1] if that's oob
            if program[current_pos + 1] == operator {
                let mut count = match program
                    .iter()
                    .skip(current_pos)
                    .position(|op| *op != operator)
                {
                    Some(x) => x,

                    None => program.len() - current_pos,
                };

                skip = count - 1;

                while count > 32 {
                    self.program.push(instruction_mask | (32 - 1) as u8);
                    count -= 32;
                }

                if count != 0 {
                    self.program.push(instruction_mask | (count - 1) as u8);
                }
            } else {
                self.program.push(operator)
            }
        } else {
            self.program.push(operator)
        }

        skip
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reset() {
        let program = include_str!("../brainfuck_programs/hello_world_smol.bf");
        let mut vm = VM::new(VMOptions::default(program.to_string()));
        let program_pre_reset = vm.get_program();

        vm.run();

        assert_eq!("hello world", vm.output);

        vm.reset();
        assert_eq!(program_pre_reset, vm.get_program());
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
        let mut vm = VM::new(VMOptions::default(program.to_string()));
        vm.load(program.to_string());
        vm.run();

        assert_eq!("hello world", vm.output);

        vm.reset();
        let program = include_str!("../brainfuck_programs/yapi_4.bf");
        vm.load(program.to_string());
        vm.run();
    }

    #[test]
    fn no_program() {
        let mut vm = VM::new(VMOptions::default("".to_string()));
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
        let mut vm = VM::new(VMOptions::default(program.to_string()));
        vm.run();
    }

    #[test]
    #[should_panic]
    fn open_ended_while() {
        let program = include_str!("../brainfuck_programs/openEndedWhile.bf");
        let mut vm = VM::new(VMOptions::default(program.to_string()));
        vm.run();
    }

    #[test]
    #[should_panic]
    fn headless_while() {
        let program = include_str!("../brainfuck_programs/headlessWhile.bf");
        let mut vm = VM::new(VMOptions::default(program.to_string()));
        vm.run();
    }

    #[test]
    #[should_panic]
    fn mem_pointer_underflow() {
        let program = include_str!("../brainfuck_programs/underflowMP.bf");
        let mut vm = VM::new(VMOptions::default(program.to_string()));
        vm.run();
    }

    #[test]
    #[should_panic]
    fn mem_pointer_overflow() {
        // TODO: replace this (with an "out of memory check") when we implement dynamic memory sizes
        let program = include_str!("../brainfuck_programs/overflowMP.bf");
        let mut vm = VM::new(VMOptions::default(program.to_string()));
        vm.run();
    }

    #[test]
    fn comment_semicolon_ignored() {
        let program = include_str!("../brainfuck_programs/comments_ignored_semicolon.bf");
        let mut vm = VM::new(VMOptions::default(program.to_string()));
        vm.run();

        assert_eq!("!", vm.output);
    }

    #[test]
    fn comment_poundsign_ignored() {
        let program = include_str!("../brainfuck_programs/comments_ignored_poundsign.bf");
        let mut vm = VM::new(VMOptions::default(program.to_string()));
        vm.run();

        assert_eq!("!", vm.output);
    }

    #[test]
    fn comment_semipound_ignored() {
        let program = include_str!("../brainfuck_programs/comments_ignored_semipound.bf");
        let mut vm = VM::new(VMOptions::default(program.to_string()));
        vm.run();

        assert_eq!("!", vm.output);
    }

    #[test]
    fn comment_not_ignored() {
        let program = include_str!("../brainfuck_programs/comments_ignored_semipound.bf");
        let options = VMOptions {
            program: program.to_string(),
            disable_comments: true,
            disable_optimizer: true,
        };
        let mut vm = VM::new(options);
        vm.run();

        assert_ne!("!", vm.output);
    }

    #[test]
    fn last_char_is_plus() {
        let program = include_str!("../brainfuck_programs/ends_on_plus.bf");
        let mut vm = VM::new(VMOptions::default(program.to_string()));
        vm.enable_optimizer(program);
        vm.run();
    }

    #[test]
    fn optimizer() {
        let program = include_str!("../brainfuck_programs/optimize_me.bf");
        let options = VMOptions {
            program: program.to_string(),
            disable_optimizer: false,
            disable_comments: false,
        };
        let mut vm = VM::new(options);
        vm.run();
        let optimized_program = vec![
            0b11100100, 0b11000100, 0b10100100, 0b10000100, b'+', b'-', b'>', b'<', 0b11100100,
            0b11000100, 0b10100010, b'+', 0b10100001, 0b11000001, b'<', 0b11000001,
        ];
        let optimized_program: String = optimized_program.iter().map(|op| *op as char).collect();
        assert_eq!(vm.get_program(), optimized_program);
    }

    #[test]
    fn hello_world() {
        let program = include_str!("../brainfuck_programs/hello_world.bf");
        let mut vm = VM::new(VMOptions::default(program.to_string()));
        vm.run();

        // Super fucking weird, why tf is it \n\r??? It's from https://de.wikipedia.org/wiki/Brainfuck
        assert_eq!("Hello World!\n\r", vm.output);
    }

    #[test]
    fn hello_world_smol() {
        let program = include_str!("../brainfuck_programs/hello_world_smol.bf");
        let mut vm = VM::new(VMOptions::default(program.to_string()));
        vm.run();

        assert_eq!("hello world", vm.output);
    }

    #[test]
    fn hell() {
        // "Hello world from hell": https://github.com/rdebath/Brainfuck/blob/master/bitwidth.b
        let program = include_str!("../brainfuck_programs/hell.bf");
        let options = VMOptions {
            program: program.to_string(),
            disable_comments: true,
            disable_optimizer: true,
        };
        let mut vm = VM::new(options);
        vm.run();

        assert_eq!("Hello World! 255\n", vm.output);

        vm.reset();
        vm.enable_optimizer(program);
        vm.run();

        assert_eq!("Hello World! 255\n", vm.output);
    }

    #[test]
    fn squares() {
        let program = include_str!("../brainfuck_programs/squares.bf");
        let mut vm = VM::new(VMOptions::default(program.to_string()));
        vm.run();

        let should_be = include_str!("../brainfuck_programs/squares_output_correct.txt");
        let should_be = dos2unix(should_be).to_string();
        assert_eq!(should_be, vm.output);
    }

    #[test]
    fn quine() {
        // Written by Erik Bosman
        let program = include_str!("../brainfuck_programs/quine.bf");
        let mut vm = VM::new(VMOptions::default(program.to_string()));
        vm.run();

        assert_eq!(program, vm.output);
    }

    #[test]
    fn obscure() {
        let program = include_str!("../brainfuck_programs/obscure.bf");
        let options = VMOptions {
            program: program.to_string(),
            disable_comments: true,
            disable_optimizer: true,
        };
        let mut vm = VM::new(options);
        vm.run();

        assert_eq!("H\n", vm.output);

        vm.reset();
        vm.enable_optimizer(program);
        vm.run();

        assert_eq!("H\n", vm.output);
    }

    #[test]
    fn fibonacci() {
        let program = include_str!("../brainfuck_programs/fibonacci.bf");
        let mut vm = VM::new(VMOptions::default(program.to_string()));
        vm.run();

        // yes those are wrong, but that's the programs fault. These numbers are from https://copy.sh/brainfuck which I assume is correct
        assert_eq!(
            "1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 121, 98, 219, ...",
            vm.output
        );

        vm.reset();
        vm.enable_optimizer(program);
        vm.run();

        assert_eq!(
            "1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 121, 98, 219, ...",
            vm.output
        );
    }

    #[test]
    fn benchbf() {
        let program = include_str!("../brainfuck_programs/bench.bf");
        let options = VMOptions {
            program: program.to_string(),
            disable_comments: false,
            disable_optimizer: false,
        };
        let mut vm = VM::new(options);
        vm.run();

        assert_eq!("ZYXWVUTSRQPONMLKJIHGFEDCBA\n", vm.output);
    }

    #[test]
    fn mandel() {
        let program = include_str!("../brainfuck_programs/mandel.bf");
        let options = VMOptions {
            program: program.to_string(),
            disable_comments: false,
            disable_optimizer: false,
        };
        let mut vm = VM::new(options);
        vm.run();

        let expected = include_str!("../brainfuck_programs/mandel_output_correct.txt");
        let expected = dos2unix(expected).to_string();

        assert_eq!(expected, vm.output);
    }
}
