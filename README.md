# Umjunsik Lang - Lamina

A compiler for Umjunsik Language (엄랭) that targets [Lamina IR](https://github.com/SkuldNorniern/lamina).

## Overview

This project compiles Umjunsik Language to Lamina Intermediate Representation. Lamina is a high-performance compiler backend supporting x86_64 and AArch64 architectures.

## Building

```bash
cargo build --release
```

## Usage

```bash
cargo run -- <file.umm>
```

Or use the compiled binary:

```bash
./target/release/umjunsik <file.umm>
```

## Examples

### Hello World (Print number 3)

`examples/hello.umm`:
```
어떻게
식...!
이 사람이름이냐ㅋㅋ
```

### Multiplication (2 * 2 = 4)

`examples/multiply.umm`:
```
어떻게
식.. ..!
이 사람이름이냐ㅋㅋ
```

### Variables

`examples/variable.umm`:
```
어떻게
엄...
엄어어....
식어어!
이 사람이름이냐ㅋㅋ
```

Explanation:
1. `엄...` : Assign 3 to variable 1
2. `엄어어....` : Assign 4 to variable 2
3. `식어어!` : Print variable 2 (outputs 4)

### Character Output

`examples/printchar.umm`:
```
어떻게
식........... .......ㅋ
식ㅋ
이 사람이름이냐ㅋㅋ
```

Explanation:
1. `식........... .......ㅋ` : Print ASCII character 18
2. `식ㅋ` : Print newline

### Conditional

`examples/conditional.umm`:
```
어떻게
엄...
동탄어?식.....!
식..!
이 사람이름이냐ㅋㅋ
```

Explanation:
1. `엄...` : Assign 3 to variable 1
2. `동탄어?식.....!` : If variable 1 is not 0, print 5
3. `식..!` : Print 2

## Implementation Details

### Lexer
The lexer handles Korean characters and special tokens:
- Recognizes keywords: `어떻게`, `엄`, `어`, `준`, `식`, `동탄`, `화이팅`, `이 사람이름이냐ㅋㅋ`
- Handles repeated `어` for variable indexing (e.g., `어어어` = variable 3)
- Processes dots (`.`) and commas (`,`) as number literals
- Supports one-line programs with `~` separator

### Parser
Builds an Abstract Syntax Tree (AST) with:
- Expression nodes: Number, Variable, Add, Sub, Mul
- Statement nodes: Assign, Input, Print, Conditional, Goto, Return
- Handles operator precedence (multiplication before addition)

### Code Generator
Generates optimized Lamina IR:
- **Two-pass compilation**: First pass analyzes which variables are used, second pass generates code
- **Lazy variable allocation**: Only allocates stack space for variables that are actually used in the program
- **SSA form**: Uses Static Single Assignment with proper load/store operations
- **Memory management**: Variables stored on stack using `alloc.ptr.stack`, initialized to 0
- **Control flow**: Generates proper basic blocks for conditionals and goto statements

Example output for a simple variable program:
```lamina
fn @main() -> i64 {
  entry:
    %var_ptr_0 = alloc.ptr.stack i64  # Only allocate used variables
    store.i64 %var_ptr_0, 0

  line_1:
    %t0 = add.i64 3, 0
    store.i64 %var_ptr_0, %t0
    %t1 = load.i64 %var_ptr_0
    print %t1
    ret.i64 0
}
```

## Implementation Status

### Implemented
- ✅ Lexer/Tokenizer with Korean character support
- ✅ Parser (AST generation)
- ✅ Optimized Lamina IR code generation
- ✅ Basic arithmetic operations (add, subtract, multiply)
- ✅ Smart variable management (lazy allocation, only used variables)
- ✅ Console output (numbers and characters)
- ✅ Newline output
- ✅ Conditionals (동탄)
- ✅ GOTO (준)
- ✅ Program exit (화이팅!)

### Not Implemented
- ❌ Console input (식?) - Placeholder only (requires external function call)
- ❌ Full compilation to native code (generates Lamina IR only)

## Project Structure

```
umjunsik-lang-lamina/
├── src/
│   ├── main.rs          # Main executable
│   ├── lib.rs           # Library root
│   ├── token.rs         # Token definitions
│   ├── lexer.rs         # Lexer (tokenization)
│   ├── ast.rs           # Abstract syntax tree
│   ├── parser.rs        # Parser
│   └── codegen.rs       # Lamina IR code generator
├── examples/            # Example programs
├── Cargo.toml
└── README.md
```

## License

Apache License 2.0

## References

- [Lamina](https://github.com/SkuldNorniern/lamina) - High-performance compiler backend
