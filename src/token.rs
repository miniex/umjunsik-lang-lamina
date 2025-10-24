#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Eotteohke,         // 어떻게 - program start
    IEotteonSaram,     // 이 사람이름이냐ㅋㅋ - program end
    Eom(usize),        // 엄 - assignment with count of preceding 어s (엄=0, 어엄=1, 어어엄=2)
    Eo(usize),         // 어 - variable reference with count (어=1, 어어=2, 어어어=3)
    Joon,          // 준 - goto
    Sik,           // 식 - console operations
    Dongtan,       // 동탄 - conditional
    Hwaiting,      // 화이팅 - return/exit

    // Operators
    Dot,   // . - increment
    Comma, // , - decrement
    Space, // (space) - multiply
    Tilde, // ~ - line separator (for one-line code)

    // Console
    Question,    // ? - input
    Exclamation, // ! - print number
    Kek,         // ㅋ - print char / end marker

    // Literal
    Number(i64), // calculated number from dots and commas

    // Special
    Newline,
    EOF,
}

#[derive(Debug, Clone)]
pub struct TokenWithPos {
    pub token: Token,
    pub line: usize,
    pub col: usize,
}
