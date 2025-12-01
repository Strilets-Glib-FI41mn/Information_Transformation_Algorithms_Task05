use std::{collections::LinkedList, fs::File, io::{Read, Write}, path::PathBuf};
use dialoguer::{Confirm, Editor};
use move_to_front::{move_to_front, move_to_front_decode};
use burrows_wheeler_transform;
use clap::{Parser, arg};
use serde::Serialize;
use rayon::prelude::*;

fn main() -> std::io::Result<()> {
    let mut config = Config::parse();
    // encode_or_decode(&mut config)?;
    if config.input_file.is_file() {
        let output_path = match find_output_path(&config){
            Some(output) => output,
            None => {
                println!("No name for the output file found! Exiting");
                return Ok(());
            },
        };
        config.output_file = Some(output_path);
        encode_or_decode(&mut config)?;
        return Ok(());
    }
    if let Some(out) = &config.output_file && !out.is_file(){
        if !out.is_dir(){
            std::fs::create_dir(out)?;
        }
        let files = std::fs::read_dir(&config.input_file).unwrap().collect::<Vec<_>>();
        let mut results: Vec<_> = (&files).into_par_iter().filter(|r| r.is_ok())
        .map(|file_path|{
            let mut new_config = config.clone();
            let mut new_pb = out.clone();
            new_pb.as_mut_os_string().push("/");
            new_pb.push(file_path.as_ref().unwrap().file_name());
            // println!("targeted path: {new_pb:?}");
            new_config.input_file = file_path.as_ref().unwrap().path();
            match config.mode{
                Mode::Encode => {
                    let mut new_extension = new_config.input_file.extension().map(|e| e.to_os_string()).unwrap_or_default();
                    match config.encoding{
                        Encoding::Huffman => new_extension.push(".huffman"),
                        Encoding::ZWLU12 | Encoding::ZWLU16 | Encoding::ZWLU32 |Encoding::ZWLU64 => new_extension.push(".zwl")
                    }
                    new_pb.set_extension(new_extension);
                },
                Mode::Decode => {
                    new_pb.set_extension("");
                },
            }
            new_config.output_file = Some(new_pb);
            new_config
        }).filter(|new_config|{
            new_config.input_file.is_file() && !new_config.input_file.ends_with(".DS_Store")
        }).map(|mut new_config|{
            (encode_or_decode(&mut new_config), new_config.input_file.clone(), new_config.output_file.clone().unwrap())
        })
        .filter(|r| r.0.is_err())
        .collect();

        // results.iter().for_each(|a|{
        //     println!("error: {:?}, input path: {:?}, output_path: {:?}", &a.0, &a.1, &a.2)
        // });

        if !results.is_empty(){
            return results.pop().unwrap().0;
        }
    }
    
    Ok(())
}
pub fn find_output_path(config: &Config) -> Option<PathBuf>{
    let input_path = config.input_file.clone();
    let output_path = match &config.output_file{
        Some(output) =>{
            output
        }
        None =>{
            let mut out = input_path.clone();
            &match config.mode{
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
                            return None;
                        }
                    }
                    out
                }
            }
        }
    };
    Some(output_path.clone())
}

pub fn encode_or_decode(config: &mut Config) -> std::io::Result<()>{
    let ver = 0;

    #[cfg(debug_assertions)]
    println!("{:?}", config);
    let input_path = config.input_file.clone();

    let output_path = match find_output_path(&config){
        Some(output) => output,
        None => {
            println!("No name for the output file found! Exiting");
            return Ok(());
        },
    };
    // println!("Input file: {input_path:?}, output file: {output_path:?}");
    let mut input = File::open(&input_path)?;
    let mut output = File::create(output_path)?;
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
            output.write_all(&header)?;
        },
        Mode::Decode => {
            let mut version = [0];
            input.read_exact(&mut version)?;
            // let read_v = input.read(&mut version)?;
            // println!("read_v: {read_v}");
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
            // let read = 
            input.read_to_end(&mut input_buffer)?;
            // println!("read: {read}");
            match config.bwt{
                true => {
                    // println!("SIZE: {}", input_buffer.len());
                    let mut buff = [0; 8];
                    let mut cursor = std::io::Cursor::new(&input_buffer);
                    while let Ok(size) = cursor.read(&mut buff) && size > 0{
                        // println!("Size:: {size}");
                        let (mut res, n0) = burrows_wheeler_transform::bwt_encode(&buff[0..size]);
                        println!("no {}", n0);
                        working_space.push(n0.try_into().unwrap());
                        working_space.append(&mut res);
                    }
                },
                false => {
                    working_space.append(&mut input_buffer);
                },
            }
            // println!("size of space: {}", working_space.len());
            if config.mtf{
                let alph = (0..=u8::MAX).map(|byte| byte).collect::<Vec<u8>>();
                let mut alphabet = LinkedList::new();
                alphabet.extend(&alph);
                let a = move_to_front(&mut alphabet, &working_space);
                let start_alphabet: Vec<_> = alphabet.into_iter().collect();
                output.write(&start_alphabet)?;
                working_space = a.into_iter().map(|u| u as u8).collect();
            }
            // println!("size of space: {}", working_space.len());
            match config.encoding{
                Encoding::ZWLU12 => {
                    let mut encoder = zwl_gs::bit_encoder::ZwlBitEncoder::<zwl_gs::like_u12::LikeU12, _>::new(working_space.as_slice(), config.filled_behaviour.clone().into());
                    encoder.encode(output)?;
                },
                Encoding::ZWLU16 => {
                    let mut encoder =  zwl_gs::bit_encoder::ZwlBitEncoder::<zwl_gs::like_u16::LikeU16, _>::new(working_space.as_slice(), config.filled_behaviour.clone().into());
                    encoder.encode(output)?;
                },
                Encoding::ZWLU32 => {
                    let mut encoder = zwl_gs::bit_encoder::ZwlBitEncoder::<zwl_gs::like_u32::LikeU32, _>::new(working_space.as_slice(), config.filled_behaviour.clone().into());
                    encoder.encode(output)?;
                },
                Encoding::ZWLU64 => {
                    let mut encoder = zwl_gs::bit_encoder::ZwlBitEncoder::<zwl_gs::like_u64::LikeU64, _>::new(working_space.as_slice(), config.filled_behaviour.clone().into());
                    encoder.encode(output)?;
                },
                Encoding::Huffman => {
                    huffman_encoding::encoder::encode(working_space.as_slice(), output, true)?;
                },
            }
        },
        Mode::Decode => {
            // let mut no = [0];
            let mut alph = [0; u8::MAX as usize + 1];
            // if config.bwt{
            //     input.read_exact(&mut no)?;
            // }
            if config.mtf{
                input.read_exact(&mut alph)?;
            }

            let mut input_buffer = vec![];
            input.read_to_end(&mut input_buffer)?;
            let mut working_space = vec![];

            match config.encoding{
                Encoding::ZWLU12 => {
                    let mut decoder = zwl_gs::bit_decoder::ZwlBitDecoder::<zwl_gs::like_u12::LikeU12, _>::new(input_buffer.as_slice(), config.filled_behaviour.clone().into());
                    decoder.decode(&mut working_space)?;
                },
                Encoding::ZWLU16 => {
                    let mut decoder = zwl_gs::bit_decoder::ZwlBitDecoder::<zwl_gs::like_u16::LikeU16, _>::new(input_buffer.as_slice(), config.filled_behaviour.clone().into());
                    decoder.decode(&mut working_space)?;
                },
                Encoding::ZWLU32 => {
                    let mut decoder = zwl_gs::bit_decoder::ZwlBitDecoder::<zwl_gs::like_u32::LikeU32, _>::new(input_buffer.as_slice(), config.filled_behaviour.clone().into());
                    decoder.decode(&mut working_space)?;
                },
                Encoding::ZWLU64 => {
                    let mut decoder = zwl_gs::bit_decoder::ZwlBitDecoder::<zwl_gs::like_u64::LikeU64, _>::new(input_buffer.as_slice(), config.filled_behaviour.clone().into());
                    decoder.decode(&mut working_space)?;
                },
                Encoding::Huffman => {
                    huffman_encoding::decoder::decode(input_buffer.as_slice(), &mut working_space)?;
                },
            }
            
            if config.mtf{
                // println!("MTF!");
                let mut read_alphabet = LinkedList::from(alph.clone());
                let the_rest: Vec<_> = working_space.iter().map(|u| *u as usize).collect();
                let decoded = move_to_front_decode(&mut read_alphabet, &the_rest);
                working_space = decoded;
            }
            if config.bwt{
                let mut buff = [0; 9];
                // println!("SIZE: {}", input_buffer.len());
                let mut cursor = std::io::Cursor::new(&input_buffer);
                while let Ok(size) = cursor.read(&mut buff) && size > 1{
                    // println!("{size}, {}", input_buffer[1..size].len());
                    println!("buff {:?}", buff);
                    println!("no {}, pos1: {}", buff[0], input_buffer[1..size][buff[0] as usize]);
                    let mut res = burrows_wheeler_transform::bwt_decode(input_buffer[1..size].into(), input_buffer[0].into());
                    working_space.append(&mut res);
                }
            }
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
#[derive(Parser, Clone)]
#[command(version, about)]
pub struct Config {
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




#[cfg(test)]
mod tests {
    use burrows_wheeler_transform::*;
    use std::io::Read;



    #[test]
    fn bwt_huffman_text() -> std::io::Result<()>{
        let text = "ананас".to_string();
        let thing = text.as_bytes();
        let mut cursor = std::io::Cursor::new(&thing);
        let mut buff_sm = [0; 8];
        let mut collecting: Vec<u8> = vec![];
        while let Ok(size) = cursor.read(&mut buff_sm) && size > 0{
            let (mut res, n0) = bwt_encode(&buff_sm[0..size]);
            collecting.push(n0.try_into().unwrap());
            collecting.append(&mut res);
        }
        let cursor = std::io::Cursor::new(&collecting);
        let mut he_text = vec![];
        huffman_encoding::encoder::encode(cursor, &mut he_text, true)?;
        let mut out_buff = [0u8; 9];
        let cursor = std::io::Cursor::new(&he_text);
        let mut he_dec = vec![];
        huffman_encoding::decoder::decode(cursor, &mut he_dec)?;
        assert_eq!(collecting, he_dec[0..he_dec.len() - 3]);
        let mut cursor = std::io::Cursor::new(&he_dec[0..he_dec.len() - 3]);
        let mut decoded:Vec<u8> = vec![];
        while let Ok(size) = cursor.read(&mut out_buff)  && size > 1{
            let mut res = bwt_decode(out_buff[1..size].into(), out_buff[0].into());
            decoded.append(&mut res);
        }

        assert_eq!(decoded, thing);
        Ok(())
    }

    #[test]
    fn bwt_huffman_text_big() -> std::io::Result<()>{
        let text = "The Project Gutenberg eBook of The Ethics of Aristotle
    
This ebook is for the use of anyone anywhere in the United States and
most other parts of the world at no cost and with almost no restrictions
whatsoever. You may copy it, give it away or re-use it under the terms
of the Project Gutenberg License included with this ebook or online
at www.gutenberg.org. If you are not located in the United States,
you will have to check the laws of the country where you are located
before using this eBook.".to_string();
        let thing = text.as_bytes();
        let mut cursor = std::io::Cursor::new(&thing);
        let mut buff_sm = [0; 8];
        let mut collecting: Vec<u8> = vec![];
        while let Ok(size) = cursor.read(&mut buff_sm) && size > 0{
            let (mut res, n0) = bwt_encode(&buff_sm[0..size]);
            collecting.push(n0.try_into().unwrap());
            collecting.append(&mut res);
        }
        let cursor = std::io::Cursor::new(&collecting);
        let mut he_text = vec![];
        huffman_encoding::encoder::encode(cursor, &mut he_text, true)?;
        let mut out_buff = [0u8; 9];
        let cursor = std::io::Cursor::new(&he_text);
        let mut he_dec = vec![];
        huffman_encoding::decoder::decode(cursor, &mut he_dec)?;
        let mut cursor = std::io::Cursor::new(&he_dec);
        let mut decoded:Vec<u8> = vec![];
        while let Ok(size) = cursor.read(&mut out_buff)  && size > 1{
            let mut res = bwt_decode(out_buff[1..size].into(), out_buff[0].into());
            decoded.append(&mut res);
        }
        println!("{:?}", str::from_utf8(&decoded));
        assert_eq!(collecting, he_dec);
        assert_eq!(decoded, thing);
        Ok(())
    }


    #[test]
    fn huffman_text_big() -> std::io::Result<()>{
        let text = "The Project Gutenberg eBook of The Ethics of Aristotle
    
This ebook is for the use of anyone anywhere in the United States and
most other parts of the world at no cost and with almost no restrictions
whatsoever. You may copy it, give it away or re-use it under the terms
of the Project Gutenberg License included with this ebook or online
at www.gutenberg.org. If you are not located in the United States,
you will have to check the laws of the country where you are located
before using this eBook.".to_string();
        let thing = text.as_bytes();
        let cursor = std::io::Cursor::new(&thing);
        let mut he_text = vec![];
        huffman_encoding::encoder::encode(cursor, &mut he_text, true)?;
        let cursor = std::io::Cursor::new(&he_text);
        let mut he_dec = vec![];
        huffman_encoding::decoder::decode(cursor, &mut he_dec)?;
        
        println!("{:?}", str::from_utf8(&he_dec));
        assert_eq!(he_dec, thing);
        Ok(())
    }
    // #[test]
    // fn huffman_text_small() -> std::io::Result<()>{
    //     let text = "Ананас".to_string();
    //     let thing = text.as_bytes();
    //     let cursor = std::io::Cursor::new(&thing);
    //     let mut he_text = vec![];
    //     huffman_encoding::encoder::encode(cursor, &mut he_text, true)?;
    //     let cursor = std::io::Cursor::new(&he_text);
    //     let mut he_dec = vec![];
    //     huffman_encoding::decoder::decode(cursor, &mut he_dec)?;
        
    //     println!("{:?}", str::from_utf8(&he_dec));
    //     assert_eq!(he_dec, thing);
    //     Ok(())
    // }
}