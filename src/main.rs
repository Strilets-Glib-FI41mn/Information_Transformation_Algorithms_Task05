use std::{collections::LinkedList, fs::File, io::{Read, Write}, path::PathBuf};
use dialoguer::{Confirm, Editor};
use move_to_front::{move_to_front, move_to_front_decode};
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
            let mut input_buffer = vec![];
            input.read_to_end(&mut input_buffer)?;
            let mut output = File::open(output_path)?;
            match config.bwt{
                true => {
                    let mut res = burrows_wheeler_transform::bwt_encode(&input_buffer);
                    output.write(&[res.1.try_into().unwrap()])?;
                    working_space.append(&mut res.0);
                },
                false => {
                    working_space.append(&mut input_buffer);
                },
            }

            if config.mtf{
                let alph = (0..=u8::MAX).map(|byte| byte).collect::<Vec<u8>>();
                let mut alphabet = LinkedList::new();
                alphabet.extend(&alph);
                let a = move_to_front(&mut alphabet, &working_space);
                let start_alphabet: Vec<_> = alphabet.into_iter().collect();
                output.write(&start_alphabet)?;
                working_space = a.into_iter().map(|u| u as u8).collect();
            }
            match config.encoding{
                Encoding::ZWLU12 => {
                    let mut encoder = zwl_gs::bit_encoder::ZwlBitEncoder::<zwl_gs::like_u12::LikeU12, _>::new(working_space.as_slice(), config.filled_behaviour.into());
                    encoder.encode(output)?;
                },
                Encoding::ZWLU16 => {
                    let mut encoder =  zwl_gs::bit_encoder::ZwlBitEncoder::<zwl_gs::like_u16::LikeU16, _>::new(working_space.as_slice(), config.filled_behaviour.into());
                    encoder.encode(output)?;
                },
                Encoding::ZWLU32 => {
                    let mut encoder = zwl_gs::bit_encoder::ZwlBitEncoder::<zwl_gs::like_u32::LikeU32, _>::new(working_space.as_slice(), config.filled_behaviour.into());
                    encoder.encode(output)?;
                },
                Encoding::ZWLU64 => {
                    let mut encoder = zwl_gs::bit_encoder::ZwlBitEncoder::<zwl_gs::like_u64::LikeU64, _>::new(working_space.as_slice(), config.filled_behaviour.into());
                    encoder.encode(output)?;
                },
                Encoding::Huffman => {
                    huffman_encoding::encoder::encode(working_space.as_slice(), output, true)?;
                },
            }
        },
        Mode::Decode => {
            let mut no = [0];
            let mut alph = [0; u8::MAX as usize];
            if config.bwt{
                input.read_exact(&mut no)?;
            }
            if config.mtf{
                input.read_exact(&mut alph)?;
            }

            let mut input_buffer = vec![];
            input.read_to_end(&mut input_buffer)?;
            let mut working_space = vec![];

            match config.encoding{
                Encoding::ZWLU12 => {
                    let mut decoder = zwl_gs::bit_decoder::ZwlBitDecoder::<zwl_gs::like_u12::LikeU12, _>::new(input_buffer.as_slice(), config.filled_behaviour.into());
                    decoder.decode(&mut working_space)?;
                },
                Encoding::ZWLU16 => {
                    let mut decoder = zwl_gs::bit_decoder::ZwlBitDecoder::<zwl_gs::like_u16::LikeU16, _>::new(input_buffer.as_slice(), config.filled_behaviour.into());
                    decoder.decode(&mut working_space)?;
                },
                Encoding::ZWLU32 => {
                    let mut decoder = zwl_gs::bit_decoder::ZwlBitDecoder::<zwl_gs::like_u32::LikeU32, _>::new(input_buffer.as_slice(), config.filled_behaviour.into());
                    decoder.decode(&mut working_space)?;
                },
                Encoding::ZWLU64 => {
                    let mut decoder = zwl_gs::bit_decoder::ZwlBitDecoder::<zwl_gs::like_u64::LikeU64, _>::new(input_buffer.as_slice(), config.filled_behaviour.into());
                    decoder.decode(&mut working_space)?;
                },
                Encoding::Huffman => {
                    huffman_encoding::decoder::decode(input_buffer.as_slice(), &mut working_space)?;
                },
            }
            
            if config.mtf{
                let mut read_alphabet = LinkedList::from(alph.clone());
                let the_rest: Vec<_> = working_space.iter().map(|u| *u as usize).collect();
                let decoded = move_to_front_decode(&mut read_alphabet, &the_rest);
                working_space = decoded;
            }
            if config.bwt{
                working_space = burrows_wheeler_transform::bwt_decode(working_space, no[0].into());
            }
            let mut output = File::open(output_path)?;
            output.write(&working_space)?;
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