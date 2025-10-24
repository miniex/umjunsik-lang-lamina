pub mod ast;
pub mod codegen;
pub mod lexer;
pub mod parser;
pub mod token;

use codegen::CodeGenerator;
use lexer::Lexer;
use parser::Parser;

pub fn compile_umjunsik(source: &str) -> Result<String, String> {
    // Lexical analysis
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;

    // Parsing
    let mut parser = Parser::new(tokens);
    let program = parser.parse()?;

    // Code generation
    let mut codegen = CodeGenerator::new();
    let lamina_ir = codegen.generate(&program)?;

    Ok(lamina_ir)
}
