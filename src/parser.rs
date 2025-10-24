use crate::ast::{Expr, Program, Statement};
use crate::token::{Token, TokenWithPos};

pub struct Parser {
    tokens: Vec<TokenWithPos>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<TokenWithPos>) -> Self {
        Parser { tokens, position: 0 }
    }

    fn current_token(&self) -> &Token {
        if self.position < self.tokens.len() {
            &self.tokens[self.position].token
        } else {
            &Token::Eof
        }
    }

    fn advance(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
    }

    fn expect(&mut self, expected: Token) -> Result<(), String> {
        if self.current_token() == &expected {
            self.advance();
            Ok(())
        } else {
            Err(format!(
                "Expected {:?}, found {:?} at position {}",
                expected,
                self.current_token(),
                self.position
            ))
        }
    }

    pub fn parse(&mut self) -> Result<Program, String> {
        // Expect program start: 어떻게
        self.expect(Token::Eotteohke)?;
        self.skip_newlines();

        let mut statements = Vec::new();

        // Parse statements until we hit program end
        loop {
            self.skip_newlines();

            match self.current_token() {
                Token::IEotteonSaram => {
                    self.advance();
                    break;
                },
                Token::Eof => break,
                Token::Tilde => {
                    self.advance(); // skip tilde (line separator for one-line code)
                },
                _ => {
                    let stmt = self.parse_statement()?;
                    statements.push(stmt);
                },
            }
        }

        Ok(Program { statements })
    }

    fn skip_newlines(&mut self) {
        while matches!(self.current_token(), Token::Newline) {
            self.advance();
        }
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        match self.current_token() {
            Token::Eom => self.parse_assignment(),
            Token::Sik => self.parse_console(),
            Token::Dongtan => self.parse_conditional(),
            Token::Joon => self.parse_goto(),
            Token::Hwaiting => self.parse_return(),
            _ => Err(format!(
                "Unexpected token at statement start: {:?}",
                self.current_token()
            )),
        }
    }

    fn parse_assignment(&mut self) -> Result<Statement, String> {
        // Count the number of 어s after 엄 to determine variable index
        self.advance(); // skip 엄

        let var_index = self.count_eo_sequence();

        // Check if it's input (식?)
        if matches!(self.current_token(), Token::Sik) {
            self.advance();
            if matches!(self.current_token(), Token::Question) {
                self.advance();
                return Ok(Statement::Input { var_index });
            } else {
                return Err("Expected '?' after '식' for input".to_string());
            }
        }

        // Otherwise, parse the value expression
        let value = self.parse_expr()?;
        Ok(Statement::Assign { var_index, value })
    }

    fn count_eo_sequence(&mut self) -> usize {
        let mut count = 0;

        // The first Eo token itself can represent multiple 어s
        if matches!(self.current_token(), Token::Eo) {
            self.advance();
            count = 1; // At minimum, one 어

            // Continue counting if there are more Eo tokens
            while matches!(self.current_token(), Token::Eo) {
                self.advance();
                count += 1;
            }
        }

        // Convert to 0-indexed (1어 = index 0, 2어 = index 1, etc.)
        if count > 0 { count - 1 } else { 0 }
    }

    fn parse_console(&mut self) -> Result<Statement, String> {
        self.advance(); // skip 식

        match self.current_token() {
            Token::Question => {
                // Input was already handled in parse_assignment
                Err("식? should be part of assignment".to_string())
            },
            Token::Exclamation => {
                self.advance();
                // Parse expression before !
                // We need to backtrack - the expression is before !
                // Actually식{expr}! means print expr
                // Let's reparse this properly
                Err("PrintNum needs redesign".to_string())
            },
            Token::Kek => {
                self.advance();
                Ok(Statement::PrintNewline)
            },
            _ => {
                // 식{number}ㅋ or 식{expr}!
                let expr = self.parse_expr()?;
                match self.current_token() {
                    Token::Kek => {
                        self.advance();
                        Ok(Statement::PrintChar(expr))
                    },
                    Token::Exclamation => {
                        self.advance();
                        Ok(Statement::PrintNum(expr))
                    },
                    _ => Err(format!(
                        "Expected 'ㅋ' or '!' after expression in console statement, found {:?}",
                        self.current_token()
                    )),
                }
            },
        }
    }

    fn parse_conditional(&mut self) -> Result<Statement, String> {
        self.advance(); // skip 동탄

        let condition = self.parse_expr()?;
        self.expect(Token::Question)?;

        // Parse the body until newline or tilde
        let mut body = Vec::new();
        while !matches!(
            self.current_token(),
            Token::Newline | Token::Tilde | Token::Eof | Token::IEotteonSaram
        ) {
            body.push(self.parse_statement()?);
        }

        Ok(Statement::Conditional { condition, body })
    }

    fn parse_goto(&mut self) -> Result<Statement, String> {
        self.advance(); // skip 준
        let line_expr = self.parse_expr()?;

        // Extract line number
        if let Expr::Number(line) = line_expr {
            Ok(Statement::Goto(line as usize))
        } else {
            Err("Goto requires a constant line number".to_string())
        }
    }

    fn parse_return(&mut self) -> Result<Statement, String> {
        self.advance(); // skip 화이팅
        self.expect(Token::Exclamation)?;
        let value = self.parse_expr()?;
        Ok(Statement::Return(value))
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_multiplicative()
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_additive()?;

        // Space means multiplication
        // In our token stream, we don't have explicit space tokens during normal operation
        // We need to check if the next primary can be multiplied
        // Actually, looking at the grammar: .. .. means 2 * 2 = 4
        // So consecutive number expressions multiply

        while self.is_start_of_primary() {
            let right = self.parse_additive()?;
            left = Expr::Mul(Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expr, String> {
        let mut dots = 0i64;
        let mut commas = 0i64;

        // Count dots and commas
        loop {
            match self.current_token() {
                Token::Dot => {
                    dots += 1;
                    self.advance();
                },
                Token::Comma => {
                    commas += 1;
                    self.advance();
                },
                Token::Eo => {
                    // Variable reference
                    self.position -= dots as usize + commas as usize; // backtrack
                    dots = 0;
                    commas = 0;
                    break;
                },
                _ => break,
            }
        }

        if dots > 0 || commas > 0 {
            return Ok(Expr::Number(dots - commas));
        }

        // Check for variable
        if matches!(self.current_token(), Token::Eo) {
            let var_index = self.count_eo_sequence();
            return Ok(Expr::Var(var_index));
        }

        Err(format!(
            "Expected expression (dots, commas, or variable), found {:?}",
            self.current_token()
        ))
    }

    fn is_start_of_primary(&self) -> bool {
        matches!(self.current_token(), Token::Dot | Token::Comma | Token::Eo)
    }
}
