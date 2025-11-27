use std::path::PathBuf;

use clap::{Parser, arg};
use serde::Serialize;

fn main() {
    println!("Hello, world!");
}



#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(
    clap::ValueEnum, Clone, Default, Serialize
)]
#[serde(rename_all = "kebab-case")]
enum Mode{
    #[default]
    Encode,
    Decode
}


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(
    clap::ValueEnum, Clone, Serialize
)]
#[serde(rename_all = "kebab-case")]
enum Encoding{
    ZWLU12,
    ZWLU16,
    ZWLU32,
    ZWLU64,
    Huffman
}


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(
    clap::ValueEnum, Clone, Default, Serialize
)]
#[serde(rename_all = "kebab-case")]
enum FilledOption{
    #[default]
    Clear,
    Freeze
}

impl From<FilledOption> for zwl_gs::dictionary::FilledBehaviour{
    fn from(val: FilledOption) -> Self{
        match val{
            FilledOption::Clear => Self::Clear,
            FilledOption::Freeze => Self::Freeze,
        }
    }
}


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    input_file: PathBuf,
    output_file: Option<PathBuf>,
    #[arg(long, short, default_value_t = Mode::Encode, value_enum)]
    mode: Mode,
    #[arg(long, short, default_value_t = false)]
    bwt: bool,
    #[arg(long, short, default_value_t = false)]
    overwrite: bool,
    #[arg(long, short, default_value_t = false)]
    mtf: bool,
    #[arg(long, short, default_value_t = FilledOption::Clear, value_enum, help = "Filled behavior of dictionary used in encoding mode")]
    filled: FilledOption,
    #[arg(long, short, value_enum, help = "Ecnoding used in encoding mode")]
    encoding: Encoding,
}