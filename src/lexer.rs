use super::token::Token;
use anyhow::anyhow;
use std::str;

pub struct Lexer<'a> {
    program: &'a str,
    cursor: usize,
}

type LResult<'a> = anyhow::Result<Option<Token<'a>>>;

impl<'a> Lexer<'a> {
    pub fn new(program: &'a str) -> Self {
        Lexer { program, cursor: 0 }
    }

    /// Get the next token. This consumes the tokens.
    pub fn next_token(&mut self) -> LResult<'a> {
        self.trim();

        match self.next_char(false) {
            Some(b'\'') => self.read_str_literal(),
            Some(b':') => Ok(Some(Token::Colon)),
            Some(b'\n') => Ok(Some(Token::Newline)),
            Some(ch) => {
                if ch.is_ascii_digit() || ch == b'-' {
                    self.read_number()
                } else {
                    self.read_token()
                }
            }

            None => Ok(None),
        }
    }

    /// Trim whitespaces, tabs, carriage returns, control chars
    fn trim(&mut self) {
        while let Some(ch) = self.next_char(true) {
            if ch != b'\t' && ch != b'\r' && ch != b'\x0C' && ch != b' ' {
                break;
            }
            self.cursor += 1;
        }
    }

    /// Read a decimal number
    fn read_number(&mut self) -> LResult<'a> {
        let start_pos = self.cursor - 1;
        while let Some(ch) = self.next_char(false) {
            if !ch.is_ascii_digit() {
                self.cursor -= 1;
                break;
            }
        }

        let number =
            str::from_utf8(&self.program.as_bytes()[start_pos..self.cursor])?.parse::<i32>()?;

        Ok(Some(Token::Number(number)))
    }

    /// Read a string literal that starts and ends with "'"
    fn read_str_literal(&mut self) -> LResult<'a> {
        let mut finished = false;
        let _ = self.next_char(false);
        let start_pos = self.cursor - 1;
        while let Some(ch) = self.next_char(false) {
            if ch == b'\n' {
                // Strings cannot continue from next line
                break;
            } else if ch == b'\'' {
                finished = true;
                break;
            }
        }

        if !finished {
            Err(anyhow!("String literal is not finished properly."))
        } else {
            let str_lit = str::from_utf8(&self.program.as_bytes()[start_pos..self.cursor - 1])?;
            Ok(Some(Token::StringLiteral(str_lit)))
        }
    }

    /// Read any other token
    fn read_token(&mut self) -> LResult<'a> {
        let start_pos = self.cursor - 1;
        while let Some(ch) = self.next_char(false) {
            // Only alphanumberic characters and '_'
            if !ch.is_ascii_alphanumeric() && ch != b'_' {
                self.cursor -= 1;
                break;
            }
        }

        let token_str = str::from_utf8(&self.program.as_bytes()[start_pos..self.cursor])?;
        Ok(Some(Token::new(token_str)))
    }

    /// Get the next char and increase the cursor if `peek` is false
    fn next_char(&mut self, peek: bool) -> Option<u8> {
        if let Some(ch) = self.program.as_bytes().get(self.cursor) {
            if !peek {
                self.cursor += 1;
            }
            Some(*ch)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::token::Op;
    use super::*;

    #[test]
    fn trim() {
        let program = "\t\r\x0C PUSH_STR 'hello'";
        let mut lexer = Lexer::new(program);
        lexer.trim();
        assert_eq!(&lexer.program[lexer.cursor..], "PUSH_STR 'hello'");
    }

    #[test]
    fn read_str_literal() {
        let program = "'test_00_me'";
        let mut lexer = Lexer::new(program);
        let _ = lexer.next_char(false);
        let token = lexer.read_str_literal().unwrap();
        assert_eq!(Token::StringLiteral("test_00_me"), token.unwrap());
    }

    #[test]
    fn read_number() {
        let program = "123 -456";
        let mut lexer = Lexer::new(program);
        let _ = lexer.next_char(false);
        assert_eq!(Token::Number(123), lexer.read_number().unwrap().unwrap());
        lexer.trim();
        let _ = lexer.next_char(false);
        assert_eq!(Token::Number(-456), lexer.read_number().unwrap().unwrap());
    }

    #[test]
    fn read_token() {
        let program = "MAIN:\nMUL\n";
        let mut lexer = Lexer::new(program);
        let _ = lexer.next_char(false);
        assert_eq!(Token::Name("MAIN"), lexer.read_token().unwrap().unwrap());
        let _ = lexer.next_char(false);
        let _ = lexer.next_char(false);
        let _ = lexer.next_char(false);
        assert_eq!(
            Token::Instruction(Op::Mul),
            lexer.read_token().unwrap().unwrap()
        );
        let _ = lexer.next_char(false);
    }

    #[test]
    fn read_program() {
        let program = r"
            CUSTOM_FN:
            LOAD_VAL 1
            WRITE_VAR 'x'
            READ_VAR 'x'
            ADD
            RETURN_VALUE

            MAIN:
            PUSH_STR 'hello world'
            CALL PRINT_STR
            CALL CUSTOM_FN
        ";

        let mut lexer = Lexer::new(program);

        let tokens = vec![
            Token::Newline,
            Token::Name("CUSTOM_FN"),
            Token::Colon,
            Token::Newline,
            Token::Instruction(Op::LoadVal),
            Token::Number(1),
            Token::Newline,
            Token::Instruction(Op::WriteVar),
            Token::StringLiteral("x"),
            Token::Newline,
            Token::Instruction(Op::ReadVar),
            Token::StringLiteral("x"),
            Token::Newline,
            Token::Instruction(Op::Add),
            Token::Newline,
            Token::Instruction(Op::ReturnValue),
            Token::Newline,
            Token::Newline,
            Token::Name("MAIN"),
            Token::Colon,
            Token::Newline,
            Token::Instruction(Op::PushStr),
            Token::StringLiteral("hello world"),
            Token::Newline,
            Token::Instruction(Op::Call),
            Token::Name("PRINT_STR"),
            Token::Newline,
            Token::Instruction(Op::Call),
            Token::Name("CUSTOM_FN"),
            Token::Newline,
        ];

        let mut tokens = tokens.into_iter();
        while let Some(token) = tokens.next() {
            assert_eq!(token, lexer.next_token().unwrap().unwrap());
        }

        // No tokens left
        assert_eq!(lexer.next_token().unwrap(), None);
    }
}
