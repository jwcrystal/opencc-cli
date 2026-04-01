mod convert;
mod error;
mod input;

use std::io::IsTerminal;
use std::path::PathBuf;

use clap::Parser;

use convert::create_converter;
use error::AppError;
use input::{collect_dir_files, dirs_overlap, process_dir, process_file, read_file, read_stdin};

const EXAMPLES: &str = "\
Supported modes:
  s2t    Simplified -> Traditional
  t2s    Traditional -> Simplified
  s2tw   Simplified -> Traditional (Taiwan)
  tw2s   Traditional (Taiwan) -> Simplified
  s2hk   Simplified -> Traditional (Hong Kong)
  hk2s   Traditional (Hong Kong) -> Simplified
  s2twp  Simplified -> Traditional (Taiwan, with phrases)
  tw2sp  Traditional (Taiwan, with phrases) -> Simplified
  t2tw   Traditional -> Traditional (Taiwan)
  tw2t   Traditional (Taiwan) -> Traditional
  t2hk   Traditional -> Traditional (Hong Kong)
  hk2t   Traditional (Hong Kong) -> Traditional
  t2jp   Traditional -> Japanese Shinjitai
  jp2t   Japanese Shinjitai -> Traditional

Examples:
  opencc-cli -m s2t -t \"open chinese convert\"
  opencc-cli -m s2t -f input.txt
  opencc-cli -m s2t -f input.txt -o output.txt
  opencc-cli -m s2t -f a.txt -f b.txt -o out/
  opencc-cli -m s2t -d ./folder -o output_folder/
  opencc-cli -m s2t -d ./folder --ext txt,md,csv
  echo \"hanzi\" | opencc-cli -m s2t
  opencc-cli -m s2t -f docs/ --in-place
  opencc-cli -m s2twp -d ./src --in-place";

#[derive(Parser)]
#[command(name = "opencc-cli")]
#[command(about = "Convert Chinese text between Simplified and Traditional using OpenCC")]
#[command(version)]
#[command(after_help = EXAMPLES)]
struct Cli {
    /// Conversion mode
    #[arg(short, long, default_value = "s2t")]
    mode: String,

    /// Direct text input (mutually exclusive with -f/-d)
    #[arg(short, long)]
    text: Option<String>,

    /// Input file(s) (mutually exclusive with -t/-d)
    #[arg(short, long)]
    file: Vec<PathBuf>,

    /// Input directory — recursive (mutually exclusive with -t/-f)
    #[arg(short, long)]
    dir: Option<PathBuf>,

    /// Output path (file for single input, directory for multi/dir input)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// File extension filter for directory mode (comma-separated)
    #[arg(long, default_value = "txt,md,csv,json,xml,html,yaml,yml")]
    ext: String,

    /// Overwrite original files (requires -f or -d, incompatible with -o)
    #[arg(long)]
    in_place: bool,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), AppError> {
    let cli = Cli::parse();

    validate_inputs(&cli)?;

    let converter = create_converter(&cli.mode)?;
    let exts: Vec<&str> = cli.ext.split(',').map(str::trim).collect();

    let has_text = cli.text.is_some();
    let has_files = !cli.file.is_empty();
    let has_dir = cli.dir.is_some();

    if has_text {
        handle_text(&cli, &converter);
        Ok(())
    } else if has_files {
        handle_files(&cli, &converter)
    } else if has_dir {
        handle_dir(&cli, &converter, &exts)
    } else {
        if std::io::stdin().is_terminal() {
            return Err(AppError::NoInput);
        }
        handle_stdin(&converter)
    }
}

fn validate_inputs(cli: &Cli) -> Result<(), AppError> {
    let has_text = cli.text.is_some();
    let has_files = !cli.file.is_empty();
    let has_dir = cli.dir.is_some();

    if has_text && (has_files || has_dir) {
        return Err(AppError::ConflictingInput);
    }
    if has_files && has_dir {
        return Err(AppError::ConflictingInput);
    }
    if cli.in_place && cli.output.is_some() {
        return Err(AppError::InPlaceAndOutput);
    }
    if cli.in_place && !has_files && !has_dir {
        return Err(AppError::InPlaceRequiresFiles);
    }
    if has_files && cli.file.len() > 1 && cli.output.is_none() && !cli.in_place {
        return Err(AppError::MultiFileNoOutput);
    }

    Ok(())
}

fn handle_text(cli: &Cli, converter: &ferrous_opencc::OpenCC) {
    let text = cli.text.as_ref().unwrap();
    let result = convert::convert_text(converter, text);
    println!("{result}");
}

fn handle_stdin(converter: &ferrous_opencc::OpenCC) -> Result<(), AppError> {
    let content = read_stdin()?;
    let result = convert::convert_text(converter, &content);
    print!("{result}");
    Ok(())
}

fn handle_files(cli: &Cli, converter: &ferrous_opencc::OpenCC) -> Result<(), AppError> {
    let files = &cli.file;

    if files.len() == 1 {
        let input_path = &files[0];
        if !input_path.exists() {
            return Err(AppError::FileNotFound(input_path.clone()));
        }

        match (&cli.output, cli.in_place) {
            (Some(out), false) => {
                process_file(converter, input_path, Some(out), false)?;
            }
            (None, true) => {
                process_file(converter, input_path, None, true)?;
            }
            (None, false) => {
                let content = read_file(input_path)?;
                let result = convert::convert_text(converter, &content);
                println!("{result}");
            }
            (Some(_), true) => unreachable!("validated in validate_inputs"),
        }
    } else if cli.in_place {
        for file_path in files {
            if !file_path.exists() {
                return Err(AppError::FileNotFound(file_path.clone()));
            }
            process_file(converter, file_path, None, true)?;
        }
    } else if let Some(out_dir) = &cli.output {
        if !out_dir.exists() {
            return Err(AppError::OutputDirNotFound(out_dir.clone()));
        }
        if !out_dir.is_dir() {
            return Err(AppError::OutputNotDir(out_dir.clone()));
        }

        let mut seen = std::collections::HashSet::new();
        for file_path in files {
            let basename = file_path
                .file_name()
                .ok_or_else(|| AppError::NoFileName(file_path.clone()))?
                .to_string_lossy()
                .to_string();
            if !seen.insert(basename.clone()) {
                return Err(AppError::BasenameConflict(PathBuf::from(basename)));
            }
        }

        for file_path in files {
            if !file_path.exists() {
                return Err(AppError::FileNotFound(file_path.clone()));
            }
            let basename = file_path
                .file_name()
                .ok_or_else(|| AppError::NoFileName(file_path.clone()))?;
            let out_path = out_dir.join(basename);
            process_file(converter, file_path, Some(&out_path), false)?;
        }
    }

    Ok(())
}

fn handle_dir(
    cli: &Cli,
    converter: &ferrous_opencc::OpenCC,
    exts: &[&str],
) -> Result<(), AppError> {
    let input_dir = cli.dir.as_ref().unwrap();
    if !input_dir.is_dir() {
        return Err(AppError::DirNotFound(input_dir.clone()));
    }

    match (&cli.output, cli.in_place) {
        (Some(out_dir), false) => {
            if dirs_overlap(input_dir, out_dir) {
                return Err(AppError::OutputOverlapsInput(out_dir.clone()));
            }
            process_dir(converter, input_dir, out_dir, exts, false)
        }
        (None, true) => process_dir(converter, input_dir, input_dir, exts, true),
        (None, false) => {
            let files = collect_dir_files(input_dir, exts)?;
            for file_path in &files {
                let content = read_file(file_path)?;
                let result = convert::convert_text(converter, &content);
                println!("--- {} ---", file_path.display());
                println!("{result}");
            }
            Ok(())
        }
        (Some(_), true) => unreachable!("validated in validate_inputs"),
    }
}
