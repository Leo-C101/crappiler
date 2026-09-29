mod crappiler;

use std::{fs, path::PathBuf, time::Instant};

use clap::Parser;

use crate::crappiler::CrapError;

#[derive(Debug, Parser)]
struct Args {
    input_path: PathBuf,
    #[arg(long, short, default_value = "out")]
    output_path: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    if !args.input_path.exists() {
        let error = CrapError::InvalidInputFile {
            path: args.input_path,
        };
        eprintln!("error: {}", error);
        return Err(Box::new(error));
    }

    let file_contents = match fs::read_to_string(args.input_path) {
        Ok(contents) => contents,
        Err(e) => {
            eprintln!("error: failed to read input file: {e}");
            return Err(Box::new(e));
        }
    };

    let start = Instant::now();

    println!("beginning compilation");

    let compiled = match crappiler::compile(file_contents) {
        Ok(asm) => asm,
        Err(e) => {
            eprintln!("error: {e}");
            return Err(Box::new(e));
        }
    };

    let elapsed = Instant::now() - start;

    match fs::write("out.asm", compiled) {
        Ok(()) => {
            println!("compilation successful. took {:?}", elapsed);
        }
        Err(e) => {
            println!(
                "error: failed to write to output file '{}': {e}",
                args.output_path.display()
            );
            return Err(Box::new(e));
        }
    }

    let _ = std::process::Command::new("nasm")
        .args(["-felf64", "out.asm", "-o", "out.o"])
        .output()?;

    let _ = std::process::Command::new("ld")
        .args(["out.o", "-o", args.output_path.to_str().unwrap()])
        .output()?;

    let _ = std::process::Command::new("rm")
        .args(["out.asm", "out.o"])
        .output()?;

    Ok(())
}
