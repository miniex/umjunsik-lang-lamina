#[derive(Debug, Clone)]
pub enum Expr {
    // Number literal from dots/commas
    Number(i64),
    // Variable reference by index (1-indexed in source, 0-indexed internally)
    Var(usize),
    // Binary operations
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum Statement {
    // Assign to variable: 엄.. => vars[0] = 2
    Assign { var_index: usize, value: Expr },
    // Input: 엄식?
    Input { var_index: usize },
    // Print number: 식..!
    PrintNum(Expr),
    // Print char: 식.........ㅋ
    PrintChar(Expr),
    // Print newline: 식ㅋ
    PrintNewline,
    // Conditional: 동탄{expr}?{stmt}
    Conditional { condition: Expr, body: Vec<Statement> },
    // Goto: 준..
    Goto(usize),
    // Return/Exit: 화이팅!..
    Return(Expr),
}

#[derive(Debug)]
pub struct Program {
    pub statements: Vec<(Statement, usize)>, // (statement, line_number)
}
