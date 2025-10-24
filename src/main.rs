use clap::Parser;
use lamina::{compile_lamina_ir_to_assembly, detect_host_architecture};
use std::fs;
use std::path::Path;
use std::process::{self, Command};
use umjunsik::compile_umjunsik;

/// Umjunsik Language Compiler targeting Lamina IR
#[derive(Parser)]
#[command(name = "umjunsik")]
#[command(author = "Han Damin <miniex@daminstudio.net>")]
#[command(version)]
#[command(about = "Compiles Umjunsik language (.umm files) to Lamina IR", long_about = None)]
struct Cli {
    /// Input Umjunsik source file (.umm)
    #[arg(value_name = "FILE")]
    input: String,

    /// Compile and execute instead of showing IR
    #[arg(short, long)]
    run: bool,

    /// Save Lamina IR to file
    #[arg(short, long, value_name = "FILE")]
    output: Option<String>,

    /// Suppress output messages
    #[arg(short, long)]
    quiet: bool,
}

fn main() {
    let cli = Cli::parse();

    // Read the source file
    let source = fs::read_to_string(&cli.input).unwrap_or_else(|err| {
        eprintln!("[umjunsik] Error reading file '{}': {}", cli.input, err);
        process::exit(1);
    });

    // Compile to Lamina IR
    let lamina_ir = match compile_umjunsik(&source) {
        Ok(ir) => ir,
        Err(err) => {
            eprintln!("[umjunsik] Compilation error: {}", err);
            process::exit(1);
        },
    };

    // Save to file if --output is specified
    if let Some(ref output_file) = cli.output {
        fs::write(output_file, &lamina_ir).unwrap_or_else(|err| {
            eprintln!("[umjunsik] Error writing to file '{}': {}", output_file, err);
            process::exit(1);
        });
        if !cli.quiet {
            println!("[umjunsik] Lamina IR written to: {}", output_file);
        }
    }

    // Execute if --run flag is set, otherwise show IR
    if cli.run {
        // Compile and execute
        run_with_lamina(&lamina_ir, &cli.input, cli.quiet);
    } else {
        // Default: show IR
        if !cli.quiet {
            println!("=== Generated Lamina IR ===");
        }
        println!("{}", lamina_ir);
    }
}

fn run_with_lamina(lamina_ir: &str, source_file: &str, quiet: bool) {
    // Detect host architecture
    let target = detect_host_architecture();

    if !quiet {
        println!("[umjunsik] Compiling with lamina for {}...", target);
    }

    // Compile IR to assembly using lamina library
    let mut assembly = Vec::new();
    if let Err(err) = compile_lamina_ir_to_assembly(lamina_ir, &mut assembly) {
        eprintln!("[umjunsik] Lamina compilation error: {}", err);
        process::exit(1);
    }

    // Create temporary files
    let temp_asm = format!(
        "/tmp/{}.s",
        Path::new(source_file).file_stem().unwrap().to_str().unwrap()
    );
    let temp_exe = format!("/tmp/{}", Path::new(source_file).file_stem().unwrap().to_str().unwrap());

    // Write assembly to temp file
    fs::write(&temp_asm, &assembly).unwrap_or_else(|err| {
        eprintln!("[umjunsik] Error writing assembly file: {}", err);
        process::exit(1);
    });

    // Assemble and link with clang
    if !quiet {
        println!("[umjunsik] Assembling and linking...");
    }
    let link_status = Command::new("clang")
        .arg(&temp_asm)
        .arg("-o")
        .arg(&temp_exe)
        .status()
        .unwrap_or_else(|err| {
            eprintln!("[umjunsik] Error running clang: {}", err);
            eprintln!("[umjunsik] Make sure clang is installed");
            process::exit(1);
        });

    if !link_status.success() {
        eprintln!("[umjunsik] Linking failed");
        process::exit(1);
    }

    // Execute
    let run_status = Command::new(&temp_exe).status().unwrap_or_else(|err| {
        eprintln!("[umjunsik] Error executing program: {}", err);
        process::exit(1);
    });

    // Clean up
    let _ = fs::remove_file(&temp_asm);
    let _ = fs::remove_file(&temp_exe);

    if !run_status.success() {
        process::exit(run_status.code().unwrap_or(1));
    }
}
