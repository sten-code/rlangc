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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    match args.command {
        Commands::Run { filename, output } => {
            let outputfile = build(filename, output)?;
            process::Command::new(outputfile)
                .spawn()
                .expect("Failed to run output");
        }
        Commands::Build { filename, output } => {
            build(filename, output)?;
        }
    }

    Ok(())
}

fn build(filename: String, output: Option<String>) -> Result<String, String> {
    let mut outputfile = match output {
        Some(_) => output.unwrap(),
        None => {
            let path = std::path::Path::new(&filename);
            match path.file_stem() {
                Some(stem) => stem.to_str().unwrap_or_default().to_owned(),
                None => return Err(format!("Couldn't get file stem from {}", filename)),
            }
        }
    };

    if outputfile == filename {
        outputfile = format!("_{}", outputfile);
    }

    let data = fs::read_to_string(&filename).map_err(|err| err.to_string())?;
    let tokens = lexer::lex(data).map_err(|err| format!("{err:?}"))?;
    for token in &tokens {
        println!("{}", token)
    }

    let ast = parser::parse(tokens).map_err(|err| format!("{err:?}"))?;
    println!("{}", ast);

    let mut env = generator::Environment {
        parent: None,
        base_stack: 0,
        variables: HashMap::new(),
        datatypes: HashMap::from([(String::from("int"), generator::Datatype::Single { size: 4 })]),
    };

    let code = ast.generate(&mut env).map_err(|err| format!("{err:?}"))?;
    println!("Variables: {:#?}", env.variables);
    println!("Datatypes: {:#?}", env.datatypes);

    let asm_output = format!("{outputfile}.asm");
    let ld_output = format!("{outputfile}.o");

    let mut file = fs::File::create(&asm_output).expect("Unable to create file");
    file.write_all(code.as_bytes())
        .expect("Unable to write to file");

    process::Command::new("nasm")
        .args(["-felf64", &asm_output])
        .status()
        .expect("Failed to compile");

    process::Command::new("ld")
        .args([&ld_output, "-o", &outputfile])
        .spawn()
        .expect("Failed to link");

    Ok(outputfile)
}
