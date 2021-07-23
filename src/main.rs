mod brainfuck;

use brainfuck::VM;

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
