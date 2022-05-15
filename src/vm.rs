//! Virtual machine that runs the bytecode

use crate::bytecode::{Bytecode, Instruction, Parser};
use anyhow::anyhow;
use std::{
    any::Any,
    collections::HashMap,
    fs::{self, File},
    io::{BufRead, BufReader, Lines},
};

/// Frame of memory created for every function at function call
/// and destroyed after the function returns.
#[derive(Debug)]
struct StackFrame {
    ret_addr: usize,                               // instruction to run next
    ret_value: Option<i32>,                        // optional return value
    local_vars: HashMap<String, i32>,              // local variables
    dynamic_objects: HashMap<usize, Box<dyn Any>>, // dynamic objects like iterators
    dyn_obj_index: usize,                          // counter for the next id
}

type BuiltinFn<'a> = fn(&mut BciVm<'a>) -> anyhow::Result<()>;

impl StackFrame {
    fn new(ret_addr: usize) -> Self {
        StackFrame {
            ret_addr,
            ret_value: None,
            local_vars: HashMap::new(),
            dynamic_objects: HashMap::new(),
            dyn_obj_index: 0,
        }
    }
}

/// Virtual machine representation
pub struct BciVm<'a> {
    bytecode: Bytecode<'a>,
    ip: usize,             // instruction pointer
    sp: isize,             // stack pointer
    fp: isize,             // frame pointer
    pub halt: Option<i32>, // halt flag with exit code

    stack: [i32; 1000],                                // the general purpose stack
    frame_stack: Vec<StackFrame>,                      // stack for `StackFrame`'s
    builtin_fns: HashMap<&'static str, BuiltinFn<'a>>, // built-in function map
}

impl<'a> BciVm<'a> {
    pub fn load(program: &'a str) -> anyhow::Result<Self> {
        let bytecode = Parser::new(program).parse()?;

        let mut builtin_fns: HashMap<&'static str, BuiltinFn> = HashMap::new();
        builtin_fns.insert("TRAVERSE_DIR", Self::built_in_traverse_dir);
        builtin_fns.insert("TRAVERSE_DIR_NEXT", Self::built_in_traverse_dir_next);
        builtin_fns.insert("READ_FILE", Self::built_in_read_file);
        builtin_fns.insert("READ_FILE_NEXT", Self::built_in_read_file_next);
        builtin_fns.insert("PRINT", Self::built_in_print);
        builtin_fns.insert("PRINT_STR", Self::built_in_print_str);

        Ok(BciVm {
            bytecode,
            ip: 0,
            sp: -1,
            fp: -1,
            halt: None,
            stack: [0; 1000],
            frame_stack: Vec::new(),
            builtin_fns,
        })
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        if self.halt.is_some() {
            return Err(anyhow!("Program is already ended."));
        }

        while self.halt.is_none() {
            let _ = self.next_instruction()?;
        }
        Ok(())
    }

    pub fn next_instruction(&mut self) -> anyhow::Result<()> {
        let instruction = self.bytecode.instructions[self.ip].clone();

        let prev_ip = self.ip;
        match instruction {
            Instruction::Call(fn_name) => self.ins_call(fn_name)?,
            Instruction::RetValue => self.ins_ret_value()?,
            Instruction::Ret => self.ins_ret()?,
            Instruction::Mul => self.ins_mul()?,
            Instruction::Add => self.ins_add()?,
            Instruction::Incr => self.ins_incr()?,
            Instruction::Decr => self.ins_decr()?,
            Instruction::LoadVal(number) => self.ins_load_val(number)?,
            Instruction::ReadVar(var_name) => self.ins_read_var(var_name)?,
            Instruction::WriteVar(var_name) => self.ins_write_var(var_name)?,
            Instruction::PushStr(s) => self.ins_push_str(s)?,
            Instruction::PopStr => {
                let _ = self.ins_pop_str()?;
            }
            Instruction::Je(number) => self.ins_je(number)?,
            Instruction::Jne(number) => self.ins_jne(number)?,
            Instruction::Jg(number) => self.ins_jg(number)?,
            Instruction::Jl(number) => self.ins_jl(number)?,
            Instruction::Jmp(number) => self.ins_jmp(number)?,
            Instruction::Cmp => self.ins_cmp()?,
            Instruction::CmpStr => self.ins_cmp_str()?,
            Instruction::Halt(exit_code) => self.halt = Some(exit_code),
            Instruction::Nop => {}
        };

        // If the previous instruction pointer is changed, then a jmp/ret or call instruction is
        // called. Then don't change the ip.
        if prev_ip == self.ip {
            self.ip += 1;
        }

        Ok(())
    }

    /// Adds a dynamic object to the current frame and pushes the object id to stack.
    fn add_dynamic_object(&mut self, obj: Box<dyn Any>) {
        let index = {
            let mut stack_frame = self.frame_stack.last_mut().unwrap();

            stack_frame
                .dynamic_objects
                .insert(stack_frame.dyn_obj_index, obj);

            let index = stack_frame.dyn_obj_index;
            stack_frame.dyn_obj_index += 1;

            index
        };

        self.push_stack(index as i32);
    }

    /// Returns a dynamic object with the id poped from the stack.
    fn get_dynamic_object(&mut self) -> anyhow::Result<&mut Box<dyn Any>> {
        let obj_ptr = self.pop_stack()?;

        let stack_frame = self.frame_stack.last_mut().unwrap();
        match stack_frame.dynamic_objects.get_mut(&(obj_ptr as usize)) {
            Some(obj) => Ok(obj),
            None => Err(anyhow!("fatal: cannot find the dynamic object".to_string())),
        }
    }

    /// Pops a number and prints it to stdout.
    fn built_in_print(&mut self) -> anyhow::Result<()> {
        let data = self.pop_stack()?;
        println!(">>>>> {}", data);
        Ok(())
    }

    /// Pops a string and prints it to stdout.
    fn built_in_print_str(&mut self) -> anyhow::Result<()> {
        let s = self.ins_pop_str()?;
        println!(">>>>> {}", s);
        Ok(())
    }

    /// Reads the file path from stack, and starts the read file process.
    /// Saves and returns the line-by-line file iterator.
    fn built_in_read_file(&mut self) -> anyhow::Result<()> {
        let file_name = self.ins_pop_str()?;

        let file = File::open(&file_name)?;
        let lines = BufReader::new(file).lines();

        self.ins_push_str(&file_name)?;
        self.add_dynamic_object(Box::new(lines));

        Ok(())
    }

    /// Reads the next line and returns it.
    fn built_in_read_file_next(&mut self) -> anyhow::Result<()> {
        let line_iter = match self
            .get_dynamic_object()?
            .downcast_mut::<Lines<BufReader<File>>>()
        {
            Some(iter) => iter,
            None => return Err(anyhow!("fatal: invalid dynamic object")),
        };

        match line_iter.next() {
            Some(line) => {
                let line = line?;
                self.ins_push_str(line.as_str())?;
                self.push_stack(1); // For Some
            }
            None => self.push_stack(0), // For None
        }

        Ok(())
    }

    /// Reads and returns information about the next file item (dir or file).
    fn built_in_traverse_dir_next(&mut self) -> anyhow::Result<()> {
        let dir_iter = match self.get_dynamic_object()?.downcast_mut::<fs::ReadDir>() {
            Some(iter) => iter,
            None => return Err(anyhow!("fatal: invalid dynamic object")),
        };

        match dir_iter.next() {
            Some(entry) => {
                let entry = entry?;
                let path = entry.path();

                self.ins_push_str(path.to_str().unwrap())?;
                if path.extension().is_some() {
                    self.ins_push_str(path.extension().unwrap().to_str().unwrap())?;
                } else {
                    self.push_stack(0); // No extension
                }
                self.push_stack(entry.metadata()?.is_dir() as i32);
                self.push_stack(1); // For Some
            }
            None => {
                self.push_stack(0); // For None
            }
        }

        Ok(())
    }

    /// Reads a directory path and starts a traverse directory process.
    /// Returns the id for the directory iterator.
    fn built_in_traverse_dir(&mut self) -> anyhow::Result<()> {
        let dir_name = self.ins_pop_str()?;
        let dir_iter = fs::read_dir(dir_name)?.into_iter();

        self.add_dynamic_object(Box::new(dir_iter));

        Ok(())
    }

    /// Decrement the last value on stack
    fn ins_decr(&mut self) -> anyhow::Result<()> {
        let mut val = self.pop_stack()?;
        val -= 1;
        self.push_stack(val);

        Ok(())
    }

    /// Increment the last value on stack
    fn ins_incr(&mut self) -> anyhow::Result<()> {
        let mut val = self.pop_stack()?;
        val += 1;
        self.push_stack(val);

        Ok(())
    }

    /// Compare two strings
    fn ins_cmp_str(&mut self) -> anyhow::Result<()> {
        let rhs = self.ins_pop_str()?;
        let lhs = self.ins_pop_str()?;

        if lhs == rhs {
            self.push_stack(0);
        } else if lhs > rhs {
            self.push_stack(1);
        } else {
            self.push_stack(-1);
        }

        Ok(())
    }

    /// Compare two numbers
    fn ins_cmp(&mut self) -> anyhow::Result<()> {
        let rhs = self.pop_stack()?;
        let lhs = self.pop_stack()?;

        if lhs == rhs {
            self.push_stack(0);
        } else if lhs > rhs {
            self.push_stack(1);
        } else {
            self.push_stack(-1);
        }

        Ok(())
    }

    /// Jump if two numbers are equal
    fn ins_je(&mut self, count: i32) -> anyhow::Result<()> {
        if self.pop_stack()? != 0 {
            return Ok(());
        }

        self.ins_jmp(count)
    }

    /// Jump if two numbers are not equal
    fn ins_jne(&mut self, count: i32) -> anyhow::Result<()> {
        if self.pop_stack()? == 0 {
            return Ok(());
        }

        self.ins_jmp(count)
    }

    /// Jump if the first number is greater
    fn ins_jg(&mut self, count: i32) -> anyhow::Result<()> {
        if self.pop_stack()? != 1 {
            return Ok(());
        }

        self.ins_jmp(count)
    }

    /// Jump if the first number is less
    fn ins_jl(&mut self, count: i32) -> anyhow::Result<()> {
        if self.pop_stack()? != -1 {
            return Ok(());
        }

        self.ins_jmp(count)
    }

    /// Jump to a location
    fn ins_jmp(&mut self, count: i32) -> anyhow::Result<()> {
        if count > self.ip as i32 {
            return Err(anyhow!("Invalid jump."));
        }

        let new_ip = ((self.ip as i32) - count) as usize;
        if new_ip >= self.bytecode.instructions.len() {
            Err(anyhow!("Invalid jump."))
        } else {
            self.ip = new_ip;
            Ok(())
        }
    }

    /// Pop a value from stack and write it to variables of the current frame
    fn ins_write_var(&mut self, var_name: &str) -> anyhow::Result<()> {
        let value = self.pop_stack()?;
        let local_vars = &mut self.frame_stack[self.fp as usize].local_vars;

        if let Some(old_value) = local_vars.get_mut(var_name) {
            *old_value = value;
        } else {
            local_vars.insert(var_name.to_string(), value);
        }

        Ok(())
    }

    /// Load a variable from frame to stack
    fn ins_read_var(&mut self, var_name: &str) -> anyhow::Result<()> {
        match self.frame_stack[self.fp as usize].local_vars.get(var_name) {
            Some(&var) => {
                self.push_stack(var);
                Ok(())
            }
            None => Err(anyhow!("Variable '{}' does not exist.", var_name)),
        }
    }

    // Call a function
    fn ins_call(&mut self, fn_name: &str) -> anyhow::Result<()> {
        if self.builtin_fns.contains_key(&fn_name) {
            return self.builtin_fns[fn_name](self);
        }

        // See if there is a builtin function
        let fn_addr = match self.bytecode.fn_table.get(fn_name) {
            Some(func) => func.ptr,
            None => return Err(anyhow!("Function '{}' does not exist.", fn_name)),
        };

        // ip + 1: not to call a function forever
        let stack_frame = StackFrame::new(self.ip + 1);

        self.frame_stack.push(stack_frame);
        self.fp += 1;
        self.ip = fn_addr;
        Ok(())
    }

    /// Return from the function by saving the return value
    fn ins_ret_value(&mut self) -> anyhow::Result<()> {
        match self.frame_stack.pop() {
            Some(mut stack_frame) => {
                stack_frame.ret_value = Some(self.pop_stack()?);
                self.ip = stack_frame.ret_addr;
                self.push_stack(stack_frame.ret_value.unwrap());
                self.fp -= 1;
                Ok(())
            }
            None => Err(anyhow!("Fatal: unexpected return")),
        }
    }

    /// Return from the function
    fn ins_ret(&mut self) -> anyhow::Result<()> {
        match self.frame_stack.pop() {
            Some(stack_frame) => {
                self.ip = stack_frame.ret_addr;
                self.fp -= 1;
                Ok(())
            }
            None => Err(anyhow!("Fatal: unexpected return")),
        }
    }

    /// Push a number to stack
    fn ins_load_val(&mut self, number: i32) -> anyhow::Result<()> {
        self.push_stack(number);
        Ok(())
    }

    /// Add two numbers
    fn ins_add(&mut self) -> anyhow::Result<()> {
        if self.sp < 1 {
            return Err(anyhow!("Fatal: stack is smaller than 2"));
        }

        let lhs = self.pop_stack()?;
        let rhs = self.pop_stack()?;

        self.push_stack(lhs + rhs);

        Ok(())
    }

    /// Multiply two numbers
    fn ins_mul(&mut self) -> anyhow::Result<()> {
        if self.sp < 1 {
            return Err(anyhow!("Fatal: stack size is smaller than 2"));
        }

        let lhs = self.pop_stack()?;
        let rhs = self.pop_stack()?;

        self.push_stack(lhs * rhs);

        Ok(())
    }

    /// Push a string on stack
    ///
    /// To use the least amount of memory, instead of putting 1-byte characters to per memory
    /// cell, it puts 4 character to a memory cell. We can also take advantage of cheap (but dangerous)
    /// copies like this. Because we just map the byte array to the stack as is.
    ///
    /// Eg.
    /// Suppose that we have 4, 4-byte wide memory cells from bottom to top respectively.
    /// |   0   |   0   |   0   |   0   |
    /// PUSH_STR 'hello world!' puts the data and the size of the string.
    /// |  h e l l  |  o _ w o  |  r l d !  |  12  |
    fn ins_push_str<'b>(&mut self, s: &'b str) -> anyhow::Result<()> {
        self.sp += 1;

        if s.len() >= self.sp as usize + self.stack.len() {
            return Err(anyhow!("fatal: out of memory"));
        }

        // Copy the string to the stack as is
        let src = s.as_ptr();
        unsafe {
            let dest = self.stack.as_ptr().offset(self.sp);
            std::ptr::copy_nonoverlapping(src, dest as *mut u8, s.len());
        }

        // Since 4 character fits in a memory cell, divide the string length by 4
        self.sp += s.len() as isize / 4;

        // Finally the string length
        self.push_stack(s.len() as i32);

        Ok(())
    }

    /// Pops a string from stack. Discards the poped string. This is mainly for internal use.
    fn ins_pop_str(&mut self) -> anyhow::Result<String> {
        let str_len = self.pop_stack()?;

        if str_len < 0 {
            return Err(anyhow!("fatal: negative strlen."));
        }

        if str_len == 0 {
            return Ok(String::new());
        }

        let mem_len = str_len / 4 + 1;

        if self.sp as i32 - mem_len + 1 < 0 {
            return Err(anyhow!("fatal: not enough stack."));
        }

        self.sp -= mem_len as isize;

        let mut out_str = String::with_capacity(str_len as usize);
        let str_ptr = self.stack.as_ptr() as *const u8;
        unsafe {
            for i in 0..str_len {
                out_str.push(*str_ptr.offset((self.sp + 1) * 4 + i as isize) as char);
            }
        }

        Ok(out_str)
    }

    fn pop_stack(&mut self) -> anyhow::Result<i32> {
        if self.sp < 0 {
            return Err(anyhow!("Fatal: stack is empty."));
        }

        self.sp -= 1;
        Ok(self.stack[(self.sp + 1) as usize])
    }

    fn push_stack(&mut self, data: i32) {
        self.sp += 1;
        self.stack[self.sp as usize] = data;
    }
}

#[cfg(test)]
mod tests {
    use crate::bytecode::Instruction;
    use std::mem::discriminant;

    use super::*;

    fn run_until_instruction<'a>(
        program: &'a str,
        instruction: Instruction,
    ) -> anyhow::Result<BciVm<'a>> {
        let mut vm = BciVm::load(program).unwrap();
        loop {
            vm.next_instruction().unwrap();
            if discriminant(&vm.bytecode.instructions[vm.ip]) == discriminant(&instruction) {
                vm.next_instruction().unwrap();
                return Ok(vm);
            } else if vm.halt.is_some() {
                return Err(anyhow!("process is halted unexpectedly"));
            }
        }
    }

    #[test]
    fn load() {
        let program = "MAIN:\nLOAD_VAL 10\nLOAD_VAL 20\nHALT 0";
        let vm = run_until_instruction(program, Instruction::Halt(0)).unwrap();

        let stack = [10, 20];
        assert_eq!(&vm.stack[0..2], &stack);
        assert_eq!(vm.sp, 1);
    }

    #[test]
    fn read_write() {
        let program = "MAIN:\nLOAD_VAL 10\nWRITE_VAR 'x'\nLOAD_VAL 20\nREAD_VAR 'x'\n";
        let vm = run_until_instruction(program, Instruction::ReadVar("")).unwrap();

        assert_eq!(vm.stack[vm.sp as usize], 10);
        assert_eq!(
            vm.frame_stack[vm.fp as usize].local_vars.get("x"),
            Some(&10)
        );
        assert_eq!(vm.sp, 1);
    }

    #[test]
    fn arithmetic() {
        let base_program = "MAIN:\nLOAD_VAL 6\nLOAD_VAL 4\n";

        let add_prog = base_program.to_string() + "ADD";
        let vm = run_until_instruction(&add_prog, Instruction::Add).unwrap();
        assert_eq!(vm.stack[vm.sp as usize], 10);

        let mul_prog = base_program.to_string() + "MUL";
        let vm = run_until_instruction(&mul_prog, Instruction::Mul).unwrap();
        assert_eq!(vm.stack[vm.sp as usize], 24);
    }

    #[test]
    fn jmp() {
        let program = "MAIN:\nNOP\nLOAD_VAL 1\nJMP 2";
        let vm = run_until_instruction(program, Instruction::Jmp(0)).unwrap();
        assert_eq!(vm.bytecode.instructions[vm.ip], Instruction::Nop);

        let program = "MAIN:\nJMP -2\nLOAD_VAL 1\nNOP";
        let vm = run_until_instruction(program, Instruction::Jmp(0)).unwrap();
        assert_eq!(vm.bytecode.instructions[vm.ip], Instruction::Nop);
    }

    #[test]
    fn push_str() {
        let inp_str = "hello world";
        let program = format!("MAIN:\nPUSH_STR '{}'\nHALT 0", inp_str);

        let mut vm = BciVm::load(&program).unwrap();
        vm.run().unwrap();

        // Check if string contents are equal
        let stack_ptr = vm.stack.as_ptr() as *const u8;
        for i in 0..inp_str.len() {
            unsafe {
                assert_eq!(inp_str.as_bytes()[i], *stack_ptr.offset(i as isize));
            }
        }

        // Check the size
        assert_eq!(vm.stack[inp_str.len() / 4 + 1] as usize, inp_str.len());
    }

    #[test]
    fn pop_str() {
        let program = "MAIN:\nPUSH_STR 'hello world'\nPOP_STR\nHALT 0";
        let mut vm = BciVm::load(program).unwrap();
        vm.run().unwrap();

        // Stack should be empty
        assert_eq!(vm.sp, -1);
    }

    #[test]
    fn cmp() {
        let program = "MAIN:\nLOAD_VAL 1\nLOAD_VAL 1\nCMP";
        let vm = run_until_instruction(program, Instruction::Cmp).unwrap();
        assert_eq!(vm.stack[vm.sp as usize], 0);
        assert_eq!(vm.sp, 0);
    }

    #[test]
    fn cmp_str() {
        let program = "MAIN:\nPUSH_STR 'hello'\nPUSH_STR 'hello'\nCMP_STR";
        let vm = run_until_instruction(program, Instruction::CmpStr).unwrap();
        assert_eq!(vm.stack[vm.sp as usize], 0);
        assert_eq!(vm.sp, 0);
    }
}
