//! Bytecode representation

use anyhow::anyhow;
use std::collections::HashMap;

use crate::{
    lexer::Lexer,
    token::{Op, Token},
};

static ENTRY_POINT: &'static str = "MAIN";

/// Representation of bytecode
#[derive(Debug)]
pub struct Bytecode<'a> {
    /// Array of instructions from top to bottom
    pub instructions: Vec<Instruction<'a>>,
    /// Function table which maps function name to it's attributes
    pub fn_table: HashMap<&'a str, Function<'a>>,
}

/// Function attributes
#[derive(Debug)]
pub struct Function<'a> {
    /// Name of the function
    pub name: &'a str,
    /// Address(line number) of the function.
    pub ptr: usize,
}

/// Supported instructions of the bytecode
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Instruction<'a> {
    /// Call a function
    Call(&'a str),
    /// Halt the program with an exit code
    Halt(i32),
    /// Push string onto memory
    PushStr(&'a str),
    /// Pop string from memory and discard it
    PopStr,
    /// Load a value into memory
    LoadVal(i32),
    /// Create/modify a variable
    WriteVar(&'a str),
    /// Read a variable from memory to memory
    ReadVar(&'a str),
    /// Compare two values on stack
    Cmp,
    /// Compare two strings on stack
    CmpStr,
    /// Unconditionally jump to a location
    Jmp(i32),
    /// Jmp if previous `cmp` is resulted in equal
    Je(i32),
    /// Jmp if previous `cmp` is resulted in not-equal
    Jne(i32),
    /// Jmp if previous `cmp` is resulted in greater
    Jg(i32),
    /// Jmp if previous `cmp` is resulted in less
    Jl(i32),
    /// Add two values
    Add,
    /// Multiply two values
    Mul,
    /// Decrement a value
    Decr,
    /// Increment a value
    Incr,
    /// Return a value
    RetValue,
    // Return
    Ret,
    /// Pass
    Nop,
}

macro_rules! impl_parse_fn {
    ($fn_name:ident;$instruction:ident($token_ident:ident)) => {
        fn $fn_name(&mut self) -> ParseRes<'a> {
            match self.lexer.next_token()? {
                Some(Token::$token_ident(inner_data)) => Ok(Instruction::$instruction(inner_data)),
                token => Err(anyhow!(
                    "Expected {}, got {:?}",
                    stringify!($inner_expr),
                    token
                )),
            }
        }
    };

    ($fn_name:ident;$instruction:ident) => {
        fn $fn_name(&mut self) -> ParseRes<'a> {
            Ok(Instruction::$instruction)
        }
    };
}

impl<'a> Bytecode<'a> {
    fn new() -> Self {
        // This is a small hack to properly end the program. Once the main function returns, `halt 0` will run and
        // properly halt the program.
        let instructions = vec![Instruction::Call(ENTRY_POINT), Instruction::Halt(0)];
        Bytecode {
            instructions,
            fn_table: HashMap::new(),
        }
    }
}

type ParseRes<'a> = anyhow::Result<Instruction<'a>>;
type ParseFn<'a> = fn(&mut Parser<'a>) -> anyhow::Result<Instruction<'a>>;

/// Parser to generate bytecode from text
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    parse_fns: HashMap<Op, ParseFn<'a>>,
}

impl<'a> Parser<'a> {
    /// Initialize the parser and prepare the parser functions
    ///
    /// * `program` program to parse
    pub fn new(program: &'a str) -> Self {
        let mut parse_fns: HashMap<Op, ParseFn> = HashMap::new();
        parse_fns.insert(Op::LoadVal, Self::parse_load_val);
        parse_fns.insert(Op::WriteVar, Self::parse_write_var);
        parse_fns.insert(Op::ReadVar, Self::parse_read_var);
        parse_fns.insert(Op::Call, Self::parse_call);
        parse_fns.insert(Op::Halt, Self::parse_halt);
        parse_fns.insert(Op::Cmp, Self::parse_cmp);
        parse_fns.insert(Op::CmpStr, Self::parse_cmp_str);
        parse_fns.insert(Op::Jmp, Self::parse_jmp);
        parse_fns.insert(Op::Je, Self::parse_je);
        parse_fns.insert(Op::Jne, Self::parse_jne);
        parse_fns.insert(Op::Jl, Self::parse_jl);
        parse_fns.insert(Op::Jg, Self::parse_jg);
        parse_fns.insert(Op::Add, Self::parse_add);
        parse_fns.insert(Op::Mul, Self::parse_mul);
        parse_fns.insert(Op::Decr, Self::parse_decr);
        parse_fns.insert(Op::Incr, Self::parse_incr);
        parse_fns.insert(Op::ReturnValue, Self::parse_ret_value);
        parse_fns.insert(Op::Return, Self::parse_ret);
        parse_fns.insert(Op::Nop, Self::parse_nop);
        parse_fns.insert(Op::PushStr, Self::parse_push_str);
        parse_fns.insert(Op::PopStr, Self::parse_pop_str);

        let lexer = Lexer::new(program);

        Parser { lexer, parse_fns }
    }

    /// Parse `program` and generate a `Bytecode`
    pub fn parse(mut self) -> anyhow::Result<Bytecode<'a>> {
        let mut bytecode = Bytecode::new();
        let mut line_ctr = 0;

        while let Some(token) = self.lexer.next_token()? {
            match token {
                Token::Instruction(op) => bytecode
                    .instructions
                    .push((self.parse_fns[&op])(&mut self)?),
                Token::Name(name) => {
                    if self.lexer.next_token()? != Some(Token::Colon) {
                        // Eg. "MAIN:"
                        return Err(anyhow!("':' should come after a label"));
                    }

                    // Redifinition of a function
                    if bytecode.fn_table.contains_key(name) {
                        return Err(anyhow!("Function {} is already defined.", name));
                    }

                    bytecode.fn_table.insert(
                        name,
                        Function {
                            name,
                            ptr: line_ctr + 2, // +2 because we inserted two instructions at the begining
                        },
                    );

                    bytecode.instructions.push(Instruction::Nop); // We are adding nop to avoid function address to be shifted up
                }
                Token::Newline => {
                    bytecode.instructions.push(Instruction::Nop);
                    line_ctr += 1;
                    continue;
                }
                token => return Err(anyhow!("Expected instruction or label, got {:?}", token)),
            }

            // This instruction is finished so we expect a newline
            match self.lexer.next_token()? {
                Some(Token::Newline) | None => {}
                Some(token) => return Err(anyhow!("Expected '\n', got {:?}", token)),
            }

            line_ctr += 1;
        }

        if !bytecode.fn_table.contains_key("MAIN") {
            return Err(anyhow!("Could not find the entry point(MAIN)."));
        }

        Ok(bytecode)
    }

    // For instructions that contain data, the generated function:
    // 1. try to read the next token, return on error
    // 2. if the read token is in expected token type, return the
    //    corresponding instruction.
    // 3. Fail otherwise with an appropriate error message.
    impl_parse_fn! {parse_write_var; WriteVar(StringLiteral)}
    impl_parse_fn! {parse_read_var; ReadVar(StringLiteral)}
    impl_parse_fn! {parse_load_val; LoadVal(Number)}
    impl_parse_fn! {parse_call; Call(Name)}
    impl_parse_fn! {parse_halt; Halt(Number)}
    impl_parse_fn! {parse_jmp; Jmp(Number)}
    impl_parse_fn! {parse_je; Je(Number)}
    impl_parse_fn! {parse_jne; Jne(Number)}
    impl_parse_fn! {parse_jg; Jg(Number)}
    impl_parse_fn! {parse_jl; Jl(Number)}
    impl_parse_fn! {parse_push_str; PushStr(StringLiteral)}

    // For instructions that do not contain data, the generated function
    // just returns the given Instruction.
    impl_parse_fn! {parse_add; Add}
    impl_parse_fn! {parse_mul; Mul}
    impl_parse_fn! {parse_decr; Decr}
    impl_parse_fn! {parse_incr; Incr}
    impl_parse_fn! {parse_ret_value; RetValue}
    impl_parse_fn! {parse_ret; Ret}
    impl_parse_fn! {parse_nop; Nop}
    impl_parse_fn! {parse_cmp; Cmp}
    impl_parse_fn! {parse_cmp_str; CmpStr}
    impl_parse_fn! {parse_pop_str; PopStr}
}
