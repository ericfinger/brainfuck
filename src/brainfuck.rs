use std::io::{BufRead, Write};

use bimap::BiMap;
pub struct VM {
    program: Vec<u8>,
    pp: usize,        // ProgramPointer
    mp: usize,        // MemoryPointer
    data: Vec<u8>,
    jump_map: BiMap<usize, usize>,
    // TODO: move this to some kind of config:
    ignore_comments: bool, // wether we should ignore comments (obscure.bf and hell.bf use ';' as non-comment chars)
    #[cfg(test)]
    output: String,
}

impl VM {
    pub fn new(program: &str) -> Self {
        let mut vm = Self {
            program: Vec::<u8>::new(),
            pp: 0,
            mp: 0,
            data: vec![0; 1024], // TODO: Dynamically grow this (static analysis of program possible??) if needed & start with smaller defaults
            jump_map: BiMap::<usize, usize>::new(),
            ignore_comments: true,
            #[cfg(test)]
            output: String::new(),
        };

        vm.parse(program).expect("Couldn't parse program");
        vm
    }

    pub fn load(&mut self, program: &str) {
        self.program.clear();
        self.parse(program).expect("Couldn't parse program");
    }

    pub fn reset(&mut self) {
        self.pp = 0;
        self.mp = 0;
        self.data.fill(0);
        #[cfg(test)]
        self.output.clear();
    }

    /// Turns comment parsing off, meaning ';' and '#' no longer end the line.
    /// Needs to reload the program since it's parsed on load.
    /// TODO: Maybe save 'original' Program so we don't need to reload?
    #[allow(dead_code)]
    pub fn disable_comment_parsing(&mut self, program: &str) {
        self.ignore_comments = false;
        self.load(program);
    }

    /// Turns comment parsing back on, meaning ';' and '#' end the line.
    /// Needs to reload the program since it's parsed on load.
    /// TODO: Maybe save 'original' Program so we don't need to reload?
    #[allow(dead_code)]
    pub fn enable_comment_parsing(&mut self, program: &str) {
        self.ignore_comments = true;
        self.load(program);
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
                    print!("{}", self.data[self.mp] as char);
                    std::io::stdout()
                        .lock()
                        .flush()
                        .expect("Could not flush stdout");

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
                        .nth(0)
                        .expect("No input could be read from stdin?")
                        as u8;
                    self.pp += 1;
                }

                b'[' => {
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

                b']' => {
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

    /// Parses the program and reports errors
    /// TODO: actually report errors & introduce Error Type
    fn parse(&mut self, program: &str) -> Result<(), Box<dyn std::error::Error>> {
        for line in program.lines() {
            for c in line.chars() {
                match c {
                    '<' | '>' | '+' | '-' | '[' | ']' | '.' | ',' => {
                        self.program.push(c as u8);
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

        if !self.check_brackets() {
            Err("Could not parse Program: Mismatched Brackets!")?
        }

        Ok(())
    }

    /// Checks if all '[' brackts have a matching ']' bracket.
    /// Inserts the index of each '[' and it's matching ']' bracket into the given BiMap
    fn check_brackets(&mut self) -> bool {
        let mut count = 0;
        let mut machting_bracket: Option<usize>;
        for (i, op) in self.program.iter().enumerate() {
            match op {
                b'[' => {
                    machting_bracket = None;
                    for (j, c) in self.program.iter().enumerate().skip(i + 1) {
                        if *c == b'[' as u8 {
                            count += 1;
                        } else if *c == b']' as u8 {
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
                            self.jump_map.insert(i, j);
                        }
                    }
                }

                b']' => {
                    machting_bracket = None;
                    for (j, c) in self
                        .program
                        .iter()
                        .rev()
                        .enumerate()
                        .skip(self.program.len() - i)
                    {
                        if *c == ']' as u8 {
                            count += 1;
                        } else if *c == '[' as u8 {
                            if count == 0 {
                                machting_bracket = Some(self.program.len() - j);
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
        let program_vec: Vec<u8> = program.chars().map(|c| c as u8).collect();
        let mut vm = VM::new(program);

        vm.run();

        assert_eq!("hello world", vm.output);

        vm.reset();
        assert_eq!(program_vec, vm.program);
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
    fn comment_semicolon_ignored() {
        let program = include_str!("../brainfuck_programs/comments_ignored_semicolon.bf");
        let mut vm = VM::new(program);
        vm.run();

        assert_eq!("!", vm.output);
    }

    #[test]
    fn comment_poundsign_ignored() {
        let program = include_str!("../brainfuck_programs/comments_ignored_poundsign.bf");
        let mut vm = VM::new(program);
        vm.run();

        assert_eq!("!", vm.output);
    }

    #[test]
    fn comment_semipound_ignored() {
        let program = include_str!("../brainfuck_programs/comments_ignored_semipound.bf");
        let mut vm = VM::new(program);
        vm.run();

        assert_eq!("!", vm.output);
    }

    #[test]
    fn comment_not_ignored() {
        let program = include_str!("../brainfuck_programs/comments_ignored_semipound.bf");
        let mut vm = VM::new(program);
        vm.disable_comment_parsing(program);
        vm.run();

        assert_ne!("!", vm.output);
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
        vm.disable_comment_parsing(program);
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
        vm.disable_comment_parsing(program);
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
