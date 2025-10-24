# Umjunsik Lang - Lamina

A compiler for Umjunsik Language (엄랭) using [Lamina](https://github.com/SkuldNorniern/lamina) as the backend.

## Requirements

- **Rust** (for building/installing)
- **clang** (for linking and execution with `--run` flag)

Install clang:
```bash
# macOS
xcode-select --install

# Ubuntu/Debian
sudo apt install clang

# Fedora
sudo dnf install clang
```

## Installation

```bash
cargo install umjunsik
```

Or build from source:

```bash
cargo build --release
```

## Usage

```bash
# Show generated Lamina IR (default)
umjunsik <file.umm>

# Compile and run
umjunsik <file.umm> --run

# Save IR to file
umjunsik <file.umm> --output <file.lamina>

# Run quietly (suppress messages)
umjunsik <file.umm> --quiet
```

## Language Reference

### Numbers
- `.` (dot) = 1
- `,` (comma) = -1
- `!` (exclamation) = multiply by 64
- Space = separate numbers for addition/subtraction

Example: `... ..` = 3 + 2 = 5, `...!` = 3 × 64 = 192

### Keywords
- `어떻게` - Program start
- `이 사람이름이냐ㅋㅋ` - Program end
- `엄` - Assign to variable
- `어` (repeated) - Variable reference (e.g., `어` = var 0, `어어` = var 1)
- `식` - Print number
- `식ㅋ` - Print character (writebyte)
- `동탄` - Conditional (if variable ≠ 0)
- `준` - Input from stdin
- `정` - Goto line
- `나` - Return

### Variables
Variables are indexed by the number of `어` characters:
- First `엄` or `어` = variable 0
- Second `어엄` or `어어` = variable 1
- Third `어어엄` or `어어어` = variable 2

## Implementation

- **Lexer**: Tokenizes Korean keywords and number literals
- **Parser**: Builds AST with operator precedence
- **Codegen**: Two-pass compilation with lazy variable allocation
- **Backend**: Uses Lamina library to compile IR → assembly
- **Linker**: Uses clang to create executable

## License

Apache License 2.0

## References

- [Lamina](https://github.com/SkuldNorniern/lamina) - Compiler backend
- [Umjunsik Language](https://github.com/rycont/umjunsik-lang) - Original specification
