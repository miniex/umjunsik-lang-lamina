use crate::token::{Token, TokenWithPos};

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    col: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            col: 1,
        }
    }

    fn current_char(&self) -> Option<char> {
        if self.position < self.input.len() {
            Some(self.input[self.position])
        } else {
            None
        }
    }

    fn advance(&mut self) -> Option<char> {
        if self.position < self.input.len() {
            let ch = self.input[self.position];
            self.position += 1;
            if ch == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
            Some(ch)
        } else {
            None
        }
    }

    fn skip_whitespace_except_newline(&mut self) {
        while let Some(ch) = self.current_char() {
            if ch == ' ' || ch == '\t' || ch == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<TokenWithPos>, String> {
        let mut tokens = Vec::new();

        loop {
            self.skip_whitespace_except_newline();

            let line = self.line;
            let col = self.col;

            match self.current_char() {
                None => {
                    tokens.push(TokenWithPos {
                        token: Token::Eof,
                        line,
                        col,
                    });
                    break;
                },
                Some('\n') => {
                    self.advance();
                    tokens.push(TokenWithPos {
                        token: Token::Newline,
                        line,
                        col,
                    });
                },
                Some('~') => {
                    self.advance();
                    tokens.push(TokenWithPos {
                        token: Token::Tilde,
                        line,
                        col,
                    });
                },
                Some('.') => {
                    self.advance();
                    tokens.push(TokenWithPos {
                        token: Token::Dot,
                        line,
                        col,
                    });
                },
                Some(',') => {
                    self.advance();
                    tokens.push(TokenWithPos {
                        token: Token::Comma,
                        line,
                        col,
                    });
                },
                Some('?') => {
                    self.advance();
                    tokens.push(TokenWithPos {
                        token: Token::Question,
                        line,
                        col,
                    });
                },
                Some('!') => {
                    self.advance();
                    tokens.push(TokenWithPos {
                        token: Token::Exclamation,
                        line,
                        col,
                    });
                },
                Some('ㅋ') => {
                    self.advance();
                    tokens.push(TokenWithPos {
                        token: Token::Kek,
                        line,
                        col,
                    });
                },
                Some(ch) if self.is_hangul_start(ch) => {
                    let keyword = self.read_hangul_keyword()?;
                    let token = self.match_keyword(&keyword)?;
                    tokens.push(TokenWithPos { token, line, col });
                },
                Some(ch) => {
                    return Err(format!("Unexpected character '{}' at line {}, col {}", ch, line, col));
                },
            }
        }

        Ok(tokens)
    }

    fn is_hangul_start(&self, ch: char) -> bool {
        matches!(ch, '어' | '엄' | '준' | '식' | '동' | '화' | '이')
    }

    fn read_hangul_keyword(&mut self) -> Result<String, String> {
        let mut keyword = String::new();

        // Special handling for "이 사람이름이냐ㅋㅋ"
        if self.current_char() == Some('이') {
            keyword.push('이');
            self.advance();

            // Check if it's the start of "이 사람이름이냐ㅋㅋ"
            if self.current_char() == Some(' ') {
                // Try to read the full end marker
                let saved_pos = self.position;
                let saved_line = self.line;
                let saved_col = self.col;

                self.advance(); // skip space

                // Try to match "사람이름이냐"
                let mut temp = String::new();
                while let Some(ch) = self.current_char() {
                    if self.is_hangul_char(ch) {
                        temp.push(ch);
                        self.advance();
                    } else {
                        break;
                    }
                }

                if temp.contains("사람이름이냐") {
                    // This is the end marker, consume any trailing ㅋs
                    while self.current_char() == Some('ㅋ') {
                        self.advance();
                    }
                    return Ok("이 사람이름이냐".to_string());
                } else {
                    // Not the end marker, restore position
                    self.position = saved_pos;
                    self.line = saved_line;
                    self.col = saved_col;
                    return Ok(keyword);
                }
            }

            // Continue reading if not followed by space
            while let Some(ch) = self.current_char() {
                if self.is_hangul_char(ch) {
                    keyword.push(ch);
                    self.advance();
                } else {
                    break;
                }
            }
            return Ok(keyword);
        }

        // Read the first character
        if let Some(ch) = self.current_char() {
            keyword.push(ch);
            self.advance();
        }

        // Check if we have a complete keyword
        // Keywords: 어떻게, 준, 식, 동탄, 화이팅, 엄, 어
        match keyword.as_str() {
            "어" => {
                // Could be part of "어떻게" or standalone "어" or repeated "어어어..."
                if self.current_char() == Some('떻') {
                    // Read "떻게"
                    keyword.push('떻');
                    self.advance();
                    if self.current_char() == Some('게') {
                        keyword.push('게');
                        self.advance();
                    }
                } else if self.current_char() == Some('어') {
                    // Continue reading repeated 어s
                    while self.current_char() == Some('어') {
                        keyword.push('어');
                        self.advance();
                    }
                }
                // else: standalone "어"
            },
            "엄" => {
                // Could be followed by "어"s for variable indexing
                while self.current_char() == Some('어') {
                    keyword.push('어');
                    self.advance();
                }
            },
            "준" | "식" => {
                // These are complete keywords, don't continue reading
                // Even if followed by '어', it should be a separate token
            },
            "동" => {
                // Check if it's "동탄"
                if self.current_char() == Some('탄') {
                    keyword.push('탄');
                    self.advance();
                }
            },
            "화" => {
                // Check if it's "화이팅"
                if self.current_char() == Some('이') {
                    keyword.push('이');
                    self.advance();
                    if self.current_char() == Some('팅') {
                        keyword.push('팅');
                        self.advance();
                    }
                }
            },
            _ => {
                // Continue reading other hangul characters
                while let Some(ch) = self.current_char() {
                    if self.is_hangul_char(ch) {
                        keyword.push(ch);
                        self.advance();
                    } else {
                        break;
                    }
                }
            },
        }

        Ok(keyword)
    }

    fn is_hangul_char(&self, ch: char) -> bool {
        // Korean characters range
        ('가'..='힣').contains(&ch) || ('ㄱ'..='ㅎ').contains(&ch) || ('ㅏ'..='ㅣ').contains(&ch)
    }

    fn match_keyword(&self, keyword: &str) -> Result<Token, String> {
        // Check for repeated 어 (variable reference)
        if keyword.chars().all(|c| c == '어') {
            return Ok(Token::Eo);
        }

        // Check for 엄 followed by 어s (assignment)
        if keyword.starts_with('엄') && keyword.chars().skip(1).all(|c| c == '어') {
            return Ok(Token::Eom);
        }

        match keyword {
            "어떻게" => Ok(Token::Eotteohke),
            "준" => Ok(Token::Joon),
            "식" => Ok(Token::Sik),
            "동탄" => Ok(Token::Dongtan),
            "화이팅" => Ok(Token::Hwaiting),
            "엄" => Ok(Token::Eom),
            "어" => Ok(Token::Eo),
            // Handle "이 사람이름이냐" (program end marker)
            "이 사람이름이냐" => Ok(Token::IEotteonSaram),
            _ if keyword.contains("사람이름이냐") => Ok(Token::IEotteonSaram),
            _ => Err(format!("Unknown keyword: {}", keyword)),
        }
    }
}
