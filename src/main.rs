use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::process;

mod ast;
mod generator;
mod lexer;
mod parser;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command()]
    Run {
        filename: String,

        #[arg(short, long)]
        output: Option<String>,
    },

    #[command()]
    Build {
        filename: String,

        #[arg(short, long)]
        output: Option<String>,
    },
}

fn main() {
    let args = Args::parse();
    match args.command {
        Commands::Run { filename, output } => match build(filename, output) {
            Ok(outputfile) => {
                process::Command::new(outputfile)
                    .spawn()
                    .expect("Failed to run output");
            }
            Err(err) => {
                println!("{}", err);
                return;
            }
        },
        Commands::Build { filename, output } => {
            if let Err(err) = build(filename, output) {
                println!("{}", err);
                return;
            }
        }
    }
}

fn build(filename: String, output: Option<String>) -> Result<String, String> {
    let mut outputfile: String = if output.is_none() {
        let path = std::path::Path::new(&filename);
        if let Some(stem) = path.file_stem() {
            String::from(stem.to_str().unwrap_or_default())
        } else {
            return Err(format!("Couldn't get file stem from {}", filename));
        }
    } else {
        output.clone().unwrap()
    };

    if outputfile == filename {
        outputfile = format!("_{}", outputfile);
    }

    let data = match fs::read_to_string(&filename) {
        Ok(data) => data,
        Err(err) => {
            return Err(err.to_string());
        }
    };

    let tokens = lexer::lex(data).unwrap();
    for token in &tokens {
        println!("{}", token)
    }

    let ast = match parser::parse(tokens) {
        Ok(ast) => ast,
        Err(err) => return Err(format!("{:?}", err)),
    };
    println!("{}", ast);
    let mut env = generator::Environment {
        parent: None,
        base_stack: 0,
        variables: HashMap::new(),
        datatypes: HashMap::from([(String::from("int"), generator::Datatype::Single { size: 4 })]),
    };

    match ast.generate(&mut env) {
        Ok(code) => {
            println!("Variables:");
            for var in env.variables {
                println!(
                    "{}, size: {}, location: {}",
                    var.0,
                    match var.1.datatype {
                        generator::Datatype::Single { size } => size.to_string(),
                        generator::Datatype::Struct { size, offsets } => format!(
                            "{}, offsets: {}",
                            size,
                            offsets
                                .iter()
                                .map(|(s, n)| format!("{}: {}", s, n))
                                .collect::<Vec<String>>()
                                .join(", ")
                        ),
                    },
                    var.1.location
                );
            }

            println!("\nDatatypes:");
            for data in env.datatypes {
                println!(
                    "{}, size: {}",
                    data.0,
                    match data.1 {
                        generator::Datatype::Single { size } => size.to_string(),
                        generator::Datatype::Struct { size, offsets } => format!(
                            "{}, offsets: {}",
                            size,
                            offsets
                                .iter()
                                .map(|(s, n)| format!("{}: {}", s, n))
                                .collect::<Vec<String>>()
                                .join(", ")
                        ),
                    }
                );
            }

            let mut file =
                fs::File::create(format!("{}.asm", outputfile)).expect("Unable to create file");
            file.write_all(code.as_bytes())
                .expect("Unable to write to file");

            process::Command::new("nasm")
                .arg("-felf64")
                .arg(format!("{}.asm", outputfile))
                .spawn()
                .expect("Failed to compile");

            println!(
                "{:?}",
                process::Command::new("ld")
                    .arg(format!("{}.o", outputfile))
                    .arg("-o")
                    .arg(&outputfile)
            );

            process::Command::new("ld")
                .arg(format!("{}.o", outputfile))
                .arg("-o")
                .arg(&outputfile)
                .spawn()
                .expect("Failed to link");

            Ok(outputfile)
        }
        Err(err) => Err(format!("{:?}", err)),
    }
}
