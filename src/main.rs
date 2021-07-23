use std::io::{BufRead, Write};

struct VM {
    program: String,
    pp: usize, // ProgramPointer
    data: Vec<u8>,
    #[cfg(test)]
    output: String,
}

impl VM {
    pub fn new() -> Self {
        // TODO: Dynamically grow this if needed & start with smaller defaults
        let data = vec![0; 1024];
        Self {
            program: "".to_string(),
            data,
            pp: 0,
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

    // TODO: rewrite in safe rust (without raw pointers!)
    //       should be easy to do, add a data_pointer int and index the vec with it *shrugs*
    pub unsafe fn run(&mut self) {
        if self.program.is_empty() {
            panic!("No Program specified!");
        }

        let mut pointer = self.data.as_mut_ptr();

        while self.pp < self.program.len() {
            match self
                .program
                .chars()
                .nth(self.pp)
                .expect("End of program reached prematurely!")
            {
                '>' => { // pointer += 1;
                    pointer = pointer.add(1);
                    self.pp += 1;
                }
                '<' => { // pointer -= 1;
                    pointer = pointer.sub(1);
                    self.pp += 1;
                }
                '+' => { // *pointer += 1;
                    *pointer = (*pointer).wrapping_add(1);
                    self.pp += 1;
                }
                '-' => { // *pointer -= 1;
                    *pointer = (*pointer).wrapping_sub(1);
                    self.pp += 1;
                }
                '.' => { // putchar(*pointer)
                    print!("{}", *pointer as char);
                    std::io::stdout().lock().flush().expect("Couldn't lock stdout!");
                    #[cfg(test)]
                    self.output.push(*pointer as char);
                    self.pp += 1;
                }
                ',' => { //getchar(*pointer)
                    let mut input = String::new();
                    std::io::stdin().lock().read_line(&mut input).expect("Couldn't lock stdin");
                    *pointer = input.chars().nth(0).expect("No input??") as u8;
                    self.pp += 1;
                }
                '[' => { // if *pointer == 0: goto end of while)
                    if *pointer == 0 {
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
                    if *pointer != 0 {
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
    let mut vm = VM::new();
    let program = include_str!("../brainfuck_programs/hello_world.bf");
    vm.load(program);
    unsafe { vm.run() };

    vm.reset();
    let program = include_str!("../brainfuck_programs/yapi_4.bf");
    vm.load(program);
    unsafe { vm.run() };

    vm.reset();
    let program = include_str!("../brainfuck_programs/triangle.bf");
    vm.load(program);
    unsafe { vm.run() };
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn reset() {
        let mut vm = VM::new();
        let program = include_str!("../brainfuck_programs/hello_world_smol.bf");
        vm.load(program);
        unsafe { vm.run() };

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
    #[should_panic]
    fn no_program() {
        let mut vm = VM::new();
        unsafe { vm.run() };
    }

    #[test]
    fn hello_world() {
        let mut vm = VM::new();
        let program = include_str!("../brainfuck_programs/hello_world.bf");
        vm.load(program);
        unsafe { vm.run() };

        // Super fucking weird, why tf is it \n\r??? It's from https://de.wikipedia.org/wiki/Brainfuck 
        assert_eq!("Hello World!\n\r", vm.output);
    }

    #[test]
    fn hello_world_smol() {
        let mut vm = VM::new();
        let program = include_str!("../brainfuck_programs/hello_world_smol.bf");
        vm.load(program);
        unsafe { vm.run() };

        assert_eq!("hello world", vm.output);
    }
    
    #[test]
    fn hell() {
        // "Hello world from hell": https://github.com/rdebath/Brainfuck/blob/master/bitwidth.b
        let mut vm = VM::new();
        let program = include_str!("../brainfuck_programs/hell.bf");
        vm.load(program);
        unsafe { vm.run() };

        assert_eq!("Hello World! 255\n", vm.output);
    }
    
    #[test]
    fn squares() {
        let mut vm = VM::new();
        let program = include_str!("../brainfuck_programs/squares.bf");
        vm.load(program);
        unsafe { vm.run() };

        let should_be = include_str!("../brainfuck_programs/squares_output_correct.txt");
        assert_eq!(should_be, vm.output);
    }

    #[test]
    fn quine() {
        // Written by Erik Bosman
        let mut vm = VM::new();
        let program = include_str!("../brainfuck_programs/quine.bf");
        vm.load(program);
        unsafe { vm.run() };

        assert_eq!(program, vm.output);
    }

    #[test]
    fn obscure() {
        let mut vm = VM::new();
        let program = include_str!("../brainfuck_programs/obscure.bf");
        vm.load(program);
        unsafe { vm.run() };

        assert_eq!("H\n", vm.output);
    }

    #[test]
    fn fibonacci() {
        let mut vm = VM::new();
        let program = include_str!("../brainfuck_programs/fibonacci.bf");
        vm.load(program);
        unsafe { vm.run() };

        // yes those are wrong, but that's the programs fault. These numbers are from https://copy.sh/brainfuck which I assume is correct
        assert_eq!("1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 121, 98, 219, ...", vm.output);
    }
}
