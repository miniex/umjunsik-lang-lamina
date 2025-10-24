use std::env;
use std::fs;
use std::process;
use umjunsik::compile_umjunsik;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <file.umm>", args[0]);
        eprintln!("   or: {} --help", args[0]);
        process::exit(1);
    }

    if args[1] == "--help" || args[1] == "-h" {
        print_help();
        return;
    }

    let filename = &args[1];

    // Read the source file
    let source = fs::read_to_string(filename).unwrap_or_else(|err| {
        eprintln!("Error reading file '{}': {}", filename, err);
        process::exit(1);
    });

    // Compile to Lamina IR
    let lamina_ir = match compile_umjunsik(&source) {
        Ok(ir) => ir,
        Err(err) => {
            eprintln!("Compilation error: {}", err);
            process::exit(1);
        },
    };

    // Output the generated Lamina IR
    println!("=== Generated Lamina IR ===");
    println!("{}", lamina_ir);

    // TODO: Pass to Lamina compiler to generate assembly/executable
    // For now, we just output the IR
}

fn print_help() {
    println!("Umjunsik Language Compiler (using Lamina)");
    println!();
    println!("USAGE:");
    println!("    umjunsik <file.umm>         Compile an Umjunsik source file");
    println!("    umjunsik --help             Show this help message");
    println!();
    println!("DESCRIPTION:");
    println!("    Compiles Umjunsik language (.umm files) to Lamina IR.");
    println!();
    println!("EXAMPLES:");
    println!("    umjunsik hello.umm          Compile hello.umm");
    println!();
}
