/// Reserved keywords of our bytecode
/// ***Note that built-in functions are not reserved keywords***
#[derive(Debug, Hash, Eq, PartialEq)]
pub enum Op {
    LoadVal,
    WriteVar,
    ReadVar,
    PushStr,
    PopStr,
    ReturnValue,
    Return,
    Mul,
    Add,
    Decr,
    Incr,
    Jmp,
    Call,
    Nop,
    Halt,
    Cmp,
    Je,
    Jne,
    Jg,
    Jl,
    CmpStr,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Token<'a> {
    Instruction(Op),

    Newline,

    SingleQuotes,
    Colon,

    StringLiteral(&'a str),
    Name(&'a str),

    Number(i32),
}

impl<'a> Token<'a> {
    pub fn new(token_str: &'a str) -> Self {
        match token_str {
            "LOAD_VAL" => Token::Instruction(Op::LoadVal),
            "WRITE_VAR" => Token::Instruction(Op::WriteVar),
            "READ_VAR" => Token::Instruction(Op::ReadVar),
            "RETURN_VALUE" => Token::Instruction(Op::ReturnValue),
            "MUL" => Token::Instruction(Op::Mul),
            "ADD" => Token::Instruction(Op::Add),
            "JMP" => Token::Instruction(Op::Jmp),
            "CALL" => Token::Instruction(Op::Call),
            "HALT" => Token::Instruction(Op::Halt),
            "CMP" => Token::Instruction(Op::Cmp),
            "CMP_STR" => Token::Instruction(Op::CmpStr),
            "JE" => Token::Instruction(Op::Je),
            "JNE" => Token::Instruction(Op::Jne),
            "JG" => Token::Instruction(Op::Jg),
            "JL" => Token::Instruction(Op::Jl),
            "DECR" => Token::Instruction(Op::Decr),
            "INCR" => Token::Instruction(Op::Incr),
            "RETURN" => Token::Instruction(Op::Return),
            "PUSH_STR" => Token::Instruction(Op::PushStr),
            "POP_STR" => Token::Instruction(Op::PopStr),
            "NOP" => Token::Instruction(Op::Nop),
            _ => Token::Name(token_str),
        }
    }
}
