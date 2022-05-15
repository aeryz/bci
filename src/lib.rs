//! Bytecode Interpreter (BCI) is a stack-based bytecode interpreter.
//!
//! # Example
//!
//! ```text
//! MAIN:
//!
//! LOAD_VAL 5
//! WRITE_VAR 'inp'
//!
//! READ_VAR 'inp'
//! WRITE_VAR 'result'
//!
//! READ_VAR 'inp'
//! LOAD_VAL 1
//! CMP
//! JE -9
//! READ_VAR 'inp'
//! DECR
//! WRITE_VAR 'inp'
//! READ_VAR 'inp'
//! READ_VAR 'result'
//! MUL
//! WRITE_VAR 'result'
//! JMP 12
//!
//! READ_VAR 'result'
//! PRINT
//!
//! HALT 0
//! ```
//!
//! # Instructions
//!
//! | Instruction | Usage                  | Brief   |
//! |-------------|------------------------|---------|
//! | Call        | CALL '_fn_name_'       | Call the function `fn_name`. |
//! | Halt        | HALT _exit-code_       | Halt the program with an `exit-code`. |
//! | LoadVal     | LOAD_VAL _number_      | Push `number` on top of the stack |
//! | WriteVar    | WRITE_VAR '_var_name_' | Pop a value from stack and create/modify a variable named `var_name` |
//! | ReadVar     | READ_VAR '_var_name_'  | Read the variable named `var_name` and push it on stack |
//! | Cmp         | CMP                    | Pop two values from stack and compare those. Push the result on stack. `lhs <op> rhs` where `lhs` is the first value that is pushed on stack.|
//! | Jmp         | JMP _number_           | Jump to `current instruction + number`. Positive values jump up, negatives down. |
//! | Je          | JE _number_            | Jump if the previous `CMP` resulted in equals. |
//! | Jne         | JNE _number_           | Jump if the previous `CMP` resulted in `not-equals. |
//! | Jg          | JG _number_            | Jump if the previous `CMP` resulted in `greater`. |
//! | Jl          | JL _number_            | Jump if the previous `CMP` resulted in `less`. |
//! | Add         | ADD                    | Pop two values from stack and add them. Push the result on stack. |
//! | Mul         | MUL                    | Pop two values from stack and multiply them. Push the result on stack. |
//! | Decr        | DECR                   | Pop a value from stack and decrement it. Push the result on stack. |
//! | Incr        | INCR                   | Pop a value from stack and increment it. Push the result on stack. |
//! | RetValue    | RETURN_VALUE           | Return a value from a function. Pop a value from stack and save it to stack frame. Jump to the return address. |
//! | Nop         | NOP                    | Do nothing. Newlines are converted to nops. |
//!
//! # Built-in functions
//!
//! ## TRAVERSE_DIR
//! Starts a traverse process through a directory.
//! ### Parameters
//! - _dir_name_: Name of the directory
//! ### Return
//! Object id of the iterator.
//!
//! ## TRAVERSE_DIR_NEXT
//! Gives the next file or directory. Should be called after `TRAVERSE` and until the returned value is `0`.
//! ### Parameters
//! - _iterator_: Object id that is returned from `TRAVERSE_DIR`
//! ### Return
//! If there is a next item:
//! - Path
//! - Extension of the file if any, or `0`.
//! - Whether the item is a directory or not (`1` or `0`).
//! - Item exists (`1`)

//! Else `0` is pushed on stack respectively.
//!
//! ## READ_FILE
//! Starts a read file process. File will be read line-by-line.
//! ### Parameters
//! - _file_path_: Path to file. (absolute or relative)
//! ### Return
//! Object id of the iterator.
//!
//! ## READ_FILE_NEXT
//! Reads the next line. Should be called after `READ_FILE` and until the returned value is `0`.
//! ### Parameters
//! - _iter_: Object id that is returned from `READ_FILE`.
//! ### Return
//! - Line if any.
//! - `1` if there is a line, else `0`.
//!
//! ## PRINT
//! Prints the `number`.
//! ### Parameters
//! - _number_: The number on top of stack.
//!
//! ## PRINT_STR
//! Prints the `string`.
//! ### Parameters
//! - _string_: String to be printed.
//!
//! # Important notes
//!
//! - Entry point is the `MAIN` function. Every program should implement it.
//! - Every piece of code should be written under a function. There is no global code/variable mechanism.
//! - Improper use of stack and call/return flow will result in undefined behaviour.
//! - Each insruction is seperated with newline
//!
//!

pub mod bytecode;
mod lexer;
pub mod token;
pub mod vm;
