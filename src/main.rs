use std::{collections::LinkedList, fs::File, io::Read, path::PathBuf};
use dialoguer::{Confirm, Editor};
use move_to_front;
use burrows_wheeler_transform;
use clap::{Error, Parser, arg};
use serde::Serialize;
use zwl_gs::like_u12::LikeU12;

fn main() -> std::io::Result<()> {
    let mut config = Cli::parse();
    let mut ver = 0;

    #[cfg(debug_assertions)]
    println!("{:?}", config);
    let input_path = config.input_file.clone();

    let output_path = match config.output_file{
        Some(output) =>{
            output
        }
        None =>{
            let mut out = input_path.clone();
            match config.mode{
                Mode::Encode => {
                    let mut new_extension = out.extension().map(|e| e.to_os_string()).unwrap_or_default();
                    match config.encoding{
                        Encoding::Huffman => new_extension.push(".huffman"),
                        Encoding::ZWLU12 | Encoding::ZWLU16 | Encoding::ZWLU32 |Encoding::ZWLU64 => new_extension.push(".zwl")
                    }
                    out.set_extension(new_extension);
                    out
                }
                Mode::Decode => {
                    out.set_extension("");

                    let confirmation = Confirm::new()
                        .with_prompt(format!("Should the name of new file be {:?}", &out))
                        .interact()
                        .unwrap();
                    if !confirmation{
                        if let Some(rv) = Editor::new().edit(out.to_str().unwrap()).unwrap() {
                            println!("The file will become:");
                            println!("{}", rv);
                            out = rv.into();
                        } else {
                            println!("No name for the output file found! Exiting");
                            return Ok(());
                        }
                    }
                    out
                }
            }
        }
    };
    let mut input = File::open(input_path)?;
    let mut header = vec![ver];
    match config.mode{
        Mode::Encode => {
            header.push((&config.encoding).into());
            header.push(config.bwt.into());
            header.push(config.mtf.into());
            if &config.encoding != &Encoding::Huffman{
                match config.filled_behaviour{
                    FilledOption::Clear => header.push(0),
                    FilledOption::Freeze => header.push(1),
                }
            }
        },
        Mode::Decode => {
            let mut version = [0];
            input.read_exact(&mut version)?;
            if version[0] != 0{
                panic!("Version {} is unsuported", version[0]);
            }
            let mut header_pt2 = [0,0,0];
            input.read_exact(&mut header_pt2)?;
            config.encoding = header_pt2[0].try_into().unwrap();
            config.bwt = match header_pt2[1]{
                0 => false,
                1 => true,
                _ => panic!("Bad header!")
            };
            config.mtf = match header_pt2[2]{
                0 => false,
                1 => true,
                _ => panic!("Bad header!")
            };
            if &config.encoding != &Encoding::Huffman{
                let mut small = vec![0];
                input.read_exact(&mut small)?;
                config.filled_behaviour = match small[0] {
                    0 =>{
                        FilledOption::Clear
                    },
                    1 =>{
                        FilledOption::Freeze
                    }
                    _ => panic!("Bad header!")
                }
            }
        },
    }
    match config.mode{
        Mode::Encode => {
            let mut working_space = vec![];

            let mut buffer = vec![0; 128];
            while let Ok(size) = input.read(&mut buffer){
                match config.bwt{
                    true => {
                        let mut res = burrows_wheeler_transform::bwt_encode(&buffer[0..size]);
                        working_space.append(&mut res.0);
                        working_space.push(res.1 as u8);
                    },
                    false => {
                        if size < 128{
                            working_space.extend_from_slice(&buffer[0..size]);
                            continue;
                        }
                        working_space.append(&mut buffer);
                    },
                }
            }

            if config.mtf{
                let alph = (0..=u8::MAX).map(|byte| byte).collect::<Vec<u8>>();
                let mut alphabet = LinkedList::new();
                alphabet.extend(&alph);
                let a = move_to_front::move_to_front(&mut alphabet, &working_space);
                let start_alphabet: Vec<_> = alphabet.into_iter().collect();
                working_space = start_alphabet;
                working_space.append(&mut a.into_iter().map(|u| u as u8).collect());
            }
            match config.encoding{
                Encoding::ZWLU12 => {
                    zwl_gs::bit_encoder::ZwlBitEncoder::<zwl_gs::like_u12::LikeU12, _>::new(working_space.as_slice(), config.filled_behaviour.into());
                },
                Encoding::ZWLU16 => {
                    zwl_gs::bit_encoder::ZwlBitEncoder::<zwl_gs::like_u16::LikeU16, _>::new(working_space.as_slice(), config.filled_behaviour.into());
                },
                Encoding::ZWLU32 => {
                    zwl_gs::bit_encoder::ZwlBitEncoder::<zwl_gs::like_u32::LikeU32, _>::new(working_space.as_slice(), config.filled_behaviour.into());
                },
                Encoding::ZWLU64 => {
                    zwl_gs::bit_encoder::ZwlBitEncoder::<zwl_gs::like_u64::LikeU64, _>::new(working_space.as_slice(), config.filled_behaviour.into());
                },
                Encoding::Huffman => {
                    todo!("Huffman encoding not implemented yet");
                    // huffman_encoding::encoder
                },
            }
        },
        Mode::Decode => {
             match config.bwt{
                true => todo!(),
                false => todo!(),
            }

            match config.mtf{
                true => todo!(),
                false => todo!(),
            }
        },
    }
    Ok(())
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
#[derive(PartialEq, Eq)]
enum Encoding{
    ZWLU12,
    ZWLU16,
    ZWLU32,
    ZWLU64,
    Huffman
}
impl TryFrom<u8> for Encoding{
    type Error = ();
    fn try_from(value: u8) -> Result<Encoding, ()> {
        match value{
            0 => Ok(Encoding::ZWLU12),
            1 => Ok(Encoding::ZWLU16),
            2 => Ok(Encoding::ZWLU32),
            3 => Ok(Encoding::ZWLU64),
            4 => Ok(Encoding::Huffman),
            _ => Err(())
        }
    }
}
impl Into<u8> for &Encoding{
    fn into(self) -> u8 {
        match self{
            &Encoding::ZWLU12 => 0,
            &Encoding::ZWLU16 => 1,
            &Encoding::ZWLU32 => 2,
            &Encoding::ZWLU64 => 3,
            &Encoding::Huffman => 4,
        }
    }
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

    #[arg(long, short, value_enum, help = "Ecnoding used in encoding mode")]
    encoding: Encoding,

    #[arg(long, short, default_value_t = false)]
    bwt: bool,
    #[arg(long, short, default_value_t = false)]
    mtf: bool,

    #[arg(long, short, default_value_t = FilledOption::Clear, value_enum, help = "Filled behavior of dictionary used in encoding mode")]
    filled_behaviour: FilledOption,

    #[arg(long, short, default_value_t = false)]
    overwrite: bool,

}