use std::collections::HashMap;

use crate::{
    lexer::Lexer,
    token::{Op, Token},
};

static ENTRY_POINT: &'static str = "MAIN";

#[derive(Debug, Clone)]
pub enum Instruction<'a> {
    Call(&'a str),
    Halt(i32), // exit code
    LoadVal(i32),
    Cmp,
    Jmp(i32),
    Je(i32),
    Jne(i32),
    Jg(i32),
    Jge(i32),
    Jl(i32),
    Jle(i32),
    WriteVar(&'a str),
    ReadVar(&'a str),
    Add,
    Mul,
    RetValue,
    Nop,
    Print,
}

macro_rules! impl_parse_fn {
    ($ins_ident:ident;$instruction:ident($token_ident:ident)) => {
        fn $ins_ident(&mut self) -> ParseRes<'a> {
            match self.lexer.next_token()? {
                Some(Token::$token_ident(inner_data)) => Ok(Instruction::$instruction(inner_data)),
                token => Err(format!(
                    "Expected {}, got {:?}",
                    stringify!($inner_expr),
                    token
                )),
            }
        }
    };
}

#[derive(Debug)]
pub struct Function<'a> {
    pub name: &'a str,
    pub ptr: usize,
}

#[derive(Debug)]
pub struct Bytecode<'a> {
    pub instructions: Vec<Instruction<'a>>,
    pub fn_table: HashMap<&'a str, Function<'a>>,
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

type ParseRes<'a> = Result<Instruction<'a>, String>;
type ParseFn<'a> = fn(&mut Parser<'a>) -> ParseRes<'a>;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    parse_fns: HashMap<Op, ParseFn<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(program: &'a str) -> Self {
        let mut parse_fns: HashMap<Op, ParseFn> = HashMap::new();
        parse_fns.insert(Op::LoadVal, Self::parse_load_val);
        parse_fns.insert(Op::WriteVar, Self::parse_write_var);
        parse_fns.insert(Op::ReadVar, Self::parse_read_var);
        parse_fns.insert(Op::Call, Self::parse_call);
        parse_fns.insert(Op::Halt, Self::parse_halt);
        parse_fns.insert(Op::Cmp, Self::parse_cmp);
        parse_fns.insert(Op::Jmp, Self::parse_jmp);
        parse_fns.insert(Op::Je, Self::parse_je);
        parse_fns.insert(Op::Jne, Self::parse_jne);
        parse_fns.insert(Op::Jl, Self::parse_jl);
        parse_fns.insert(Op::Jle, Self::parse_jle);
        parse_fns.insert(Op::Jg, Self::parse_jg);
        parse_fns.insert(Op::Jge, Self::parse_jge);
        parse_fns.insert(Op::Add, Self::parse_add);
        parse_fns.insert(Op::Mul, Self::parse_mul);
        parse_fns.insert(Op::ReturnValue, Self::parse_ret_value);
        parse_fns.insert(Op::Nop, Self::parse_nop);
        parse_fns.insert(Op::Print, Self::parse_print);

        let lexer = Lexer::new(program);

        Parser { lexer, parse_fns }
    }

    pub fn parse(mut self) -> Result<Bytecode<'a>, String> {
        let mut bytecode = Bytecode::new();
        let mut line_ctr = 0;

        while let Some(token) = self.lexer.next_token()? {
            match token {
                Token::Instruction(op) => bytecode
                    .instructions
                    .push((self.parse_fns[&op])(&mut self)?),
                Token::Name(name) => {
                    if self.lexer.next_token()? != Some(Token::Semicolon) {
                        return Err(String::from("':' should come after a label"));
                    }

                    // Redifinition of a function
                    if bytecode.fn_table.contains_key(name) {
                        return Err(format!("Function {} is already defined.", name));
                    }

                    bytecode.fn_table.insert(
                        name,
                        Function {
                            name,
                            ptr: line_ctr + 2, // +2 because we inserted two instructions
                        },
                    );

                    bytecode.instructions.push(Instruction::Nop); // We are adding nop to avoid function address to be wrong
                }
                Token::Newline => {
                    bytecode.instructions.push(Instruction::Nop);
                    line_ctr += 1;
                    continue;
                }
                token => return Err(format!("Expected instruction or label, got {:?}", token)),
            }

            // This instruction is finished so we expect a newline
            match self.lexer.next_token()? {
                Some(Token::Newline) | None => {}
                Some(token) => return Err(format!("Expected '\n', got {:?}", token)),
            }

            line_ctr += 1;
        }

        if !bytecode.fn_table.contains_key("MAIN") {
            return Err(String::from("Could not find the entry point(MAIN)."));
        }

        Ok(bytecode)
    }

    impl_parse_fn! {parse_write_var; WriteVar(StringLiteral)}
    impl_parse_fn! {parse_load_val; LoadVal(Number)}
    impl_parse_fn! {parse_load_val; LoadVal(Number)}

    /*
    fn parse_load_val(&mut self) -> ParseRes<'a> {
        match self.lexer.next_token()? {
            Some(Token::Number(num)) => Ok(Instruction::LoadVal(num)),
            token => Err(format!("Expected number, got {:?}", token)),
        }
    }

    fn parse_call(&mut self) -> ParseRes<'a> {
        match self.lexer.next_token()? {
            Some(Token::Name(name)) => Ok(Instruction::Call(name)),
            token => Err(format!("Expected name, got {:?}", token)),
        }
    }


    fn parse_write_var(&mut self) -> ParseRes<'a> {
        match self.lexer.next_token()? {
            Some(Token::StringLiteral(lit)) => Ok(Instruction::WriteVar(lit)),
            token => Err(format!("Expected string literal, got {:?}", token)),
        }
    }

    fn parse_read_var(&mut self) -> ParseRes<'a> {
        match self.lexer.next_token()? {
            Some(Token::StringLiteral(lit)) => Ok(Instruction::ReadVar(lit)),
            token => Err(format!("Expected string literal, got {:?}", token)),
        }
    }

    fn parse_halt(&mut self) -> ParseRes<'a> {
        match self.lexer.next_token()? {
            Some(Token::Number(number)) => Ok(Instruction::Halt(number)),
            token => Err(format!("Expected number, got {:?}", token)),
        }
    }

    fn parse_jmp(&mut self) -> ParseRes<'a> {
        match self.lexer.next_token()? {
            Some(Token::Number(number)) => Ok(Instruction::Jmp(number)),
            token => Err(format!("Expected number, got {:?}", token)),
        }
    }

    */

    fn parse_add(&mut self) -> ParseRes<'a> {
        Ok(Instruction::Add)
    }

    fn parse_mul(&mut self) -> ParseRes<'a> {
        Ok(Instruction::Mul)
    }

    fn parse_ret_value(&mut self) -> ParseRes<'a> {
        Ok(Instruction::RetValue)
    }

    fn parse_nop(&mut self) -> ParseRes<'a> {
        Ok(Instruction::Nop)
    }

    fn parse_print(&mut self) -> ParseRes<'a> {
        Ok(Instruction::Print)
    }
}
