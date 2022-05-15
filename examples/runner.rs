use std::{env, fs};

use bci::vm::BciVm;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 1 {
        println!("An example bci file should be provided.");
    }

    let program = fs::read_to_string(&args[1]).unwrap();

    let mut vm = BciVm::load(&program).unwrap();
    vm.run().unwrap();

    println!("Process is finished with exit code: {}", vm.halt.unwrap());
}
