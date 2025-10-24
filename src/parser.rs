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
            &Token::EOF
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
                Token::EOF => break,
                Token::Tilde => {
                    self.advance(); // skip tilde (line separator for one-line code)
                },
                _ => {
                    // Get the line number before parsing the statement
                    let line_num = if self.position < self.tokens.len() {
                        self.tokens[self.position].line
                    } else {
                        1
                    };
                    let stmt = self.parse_statement()?;
                    statements.push((stmt, line_num));
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
            Token::Eom(_) => self.parse_assignment(),
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
        // Get variable index from token
        let var_index = match self.current_token().clone() {
            Token::Eom(eo_count) => {
                // 엄=1, 어엄=2, 어어엄=3, 어어엄어=4, ...
                // eo_count is the total number of 어s (before and after 엄)
                let index = eo_count + 1;
                self.advance();
                index
            },
            _ => return Err("Expected assignment token (Eom)".to_string()),
        };

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

        // Check if there's a value to assign (could be empty/newline)
        if matches!(self.current_token(), Token::Newline | Token::EOF) {
            // Assignment with no value (initialize to 0)
            return Ok(Statement::Assign {
                var_index,
                value: Expr::Number(0),
            });
        }

        // Otherwise, parse the value expression
        let value = self.parse_expr()?;
        Ok(Statement::Assign { var_index, value })
    }

    fn count_eo_sequence(&mut self) -> usize {
        // With new token format, Eo already contains the count
        if let Token::Eo(count) = self.current_token().clone() {
            self.advance();
            count
        } else {
            0
        }
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
            Token::Newline | Token::Tilde | Token::EOF | Token::IEotteonSaram
        ) {
            body.push(self.parse_statement()?);
        }

        Ok(Statement::Conditional { condition, body })
    }

    fn parse_goto(&mut self) -> Result<Statement, String> {
        self.advance(); // skip 준
        let line_expr = self.parse_expr()?;

        // Evaluate expression to get line number
        match Self::eval_const_expr(&line_expr) {
            Some(line) if line > 0 => Ok(Statement::Goto(line as usize)),
            Some(line) => Err(format!("Goto line number must be positive, got {}", line)),
            None => Err("Goto requires a constant expression (no variables)".to_string()),
        }
    }

    fn eval_const_expr(expr: &Expr) -> Option<i64> {
        match expr {
            Expr::Number(n) => Some(*n),
            Expr::Var(_) => None, // Variables are not constant
            Expr::Add(l, r) => {
                let left = Self::eval_const_expr(l)?;
                let right = Self::eval_const_expr(r)?;
                Some(left + right)
            },
            Expr::Sub(l, r) => {
                let left = Self::eval_const_expr(l)?;
                let right = Self::eval_const_expr(r)?;
                Some(left - right)
            },
            Expr::Mul(l, r) => {
                let left = Self::eval_const_expr(l)?;
                let right = Self::eval_const_expr(r)?;
                Some(left * right)
            },
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
        while matches!(self.current_token(), Token::Space) {
            self.advance(); // consume space
            let right = self.parse_additive()?;
            left = Expr::Mul(Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expr, String> {
        let mut dots = 0i64;
        let mut commas = 0i64;
        let mut has_var = false;
        let mut var_index = 0;

        // Count dots and commas, check for variable
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
                Token::Eo(_) => {
                    // Variable reference - can appear before or after dots/commas
                    if !has_var {
                        var_index = self.count_eo_sequence();
                        has_var = true;
                    } else {
                        // Two variables in a row - stop
                        break;
                    }
                },
                _ => break,
            }
        }

        // Build expression: variable + number OR just number OR just variable
        let base_expr = if has_var {
            Expr::Var(var_index)
        } else if dots > 0 || commas > 0 {
            return Ok(Expr::Number(dots - commas));
        } else {
            return Err(format!(
                "Expected expression (dots, commas, or variable), found {:?}",
                self.current_token()
            ));
        };

        // Add dots/commas to variable if present
        if dots > 0 || commas > 0 {
            let number = Expr::Number(dots - commas);
            Ok(Expr::Add(Box::new(base_expr), Box::new(number)))
        } else {
            Ok(base_expr)
        }
    }

}
