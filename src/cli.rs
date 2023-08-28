#![allow(dead_code, unused)]

use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
};

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optionally operate on a file instead of stdin
    #[arg(short, long, value_name = "FILE")]
    file: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(name = "ser")]
    Serialize,
    #[command(name = "de")]
    Deserialize,
}

pub fn do_cli() -> Result<(), std::io::Error> {
    let cli = Cli::parse();

    match &cli.command {
        Some(command) => {
            if let Some(file_paths) = cli.file {
                let mut read = Vec::new();
                for file_path in file_paths.into_iter() {
                    File::open(file_path)?.read_to_end(&mut read)?;

                    match command {
                        Commands::Deserialize => {
                            let output = b"not done";
                            let mut file = OpenOptions::new()
                                .write(true)
                                .create(true)
                                .append(false)
                                .open(&format!("{}-de", file_path.to_string_lossy()))?;
                            file.write_all(output)?;
                            file.flush()?;
                        }
                        Commands::Serialize => {
                            let output = b"not done";
                            let mut file = OpenOptions::new()
                                .write(true)
                                .create(true)
                                .append(false)
                                .open(&format!("{}-ser", file_path.to_string_lossy()))?;
                            file.write_all(output)?;
                            file.flush()?;
                        }
                    }
                }
            } else {
                let mut input = std::io::stdin();
                let mut output = std::io::stdout();
                let mut buf = Vec::new();
                input.read_to_end(&mut buf)?;
                match command {
                    Commands::Serialize => {
                        output.write_all(b"not done");
                    }
                    Commands::Deserialize => {
                        output.write_all(b"not done");
                    }
                }
            }
        }
        None => {}
    }

    Ok(())
}
