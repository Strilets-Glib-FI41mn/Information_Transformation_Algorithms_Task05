use std::{collections::LinkedList, fs::File, io::{BufReader, BufWriter, Read, Seek, Write}, path::PathBuf};
use dialoguer::{Confirm, Editor};
use move_to_front::{move_to_front_decode_r_w, move_to_front_rw};
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
    let input = File::open(&input_path)?;
    let mut input_buf = BufReader::new(input);
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
            input_buf.read_exact(&mut version)?;
            // let read_v = input.read(&mut version)?;
            // println!("read_v: {read_v}");
            if version[0] != 0{
                panic!("Version {} is unsuported", version[0]);
            }
            let mut header_pt2 = [0,0,0];
            input_buf.read_exact(&mut header_pt2)?;
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
                input_buf.read_exact(&mut small)?;
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
            let mut working_temp_file = tempfile::tempfile()?;
            let mut working_buffer =  BufWriter::new(working_temp_file);
            // let mut working_space = vec![];
            // let mut input_buffer = vec![];
            if config.bwt{
                // println!("SIZE: {}", input_buffer.len());
                let mut buff = [0; 8];
                // let mut cursor = std::io::Cursor::new(&input_buffer);
                while let Ok(size) = input_buf.read(&mut buff) && size > 0{
                    // println!("Size:: {size}");
                    let (res, n0) = burrows_wheeler_transform::bwt_encode(&buff[0..size]);
                    // println!("no {}", n0);
                    working_buffer.write_all(&[n0.try_into().unwrap()])?;
                    working_buffer.write_all(&res)?;
                    // working_space.push(n0.try_into().unwrap());
                    // working_space.append(&mut res);
                }
                working_buffer.flush()?;
                input_buf = BufReader::new(working_buffer.into_inner()?);
                working_temp_file = tempfile::tempfile()?;
                working_buffer =  BufWriter::new(working_temp_file);
            }
            // println!("size of space: {}", working_space.len());
            if config.mtf{
                let alph = (0..=u8::MAX).map(|byte| byte).collect::<Vec<u8>>();
                let mut alphabet = LinkedList::new();
                alphabet.extend(&alph);
                move_to_front_rw(&mut alphabet, &mut input_buf, &mut working_buffer)?;
                working_buffer.flush()?;
                input_buf = BufReader::new(working_buffer.into_inner()?);
            }
            // println!("size of space: {}", working_space.len());
            let mut ouptut_buff = BufWriter::new(output);
            match config.encoding{
                Encoding::ZWLU12 => {
                    let mut encoder: zwl_gs::bit_encoder::ZwlBitEncoder<zwl_gs::like_u12::LikeU12, &mut BufReader<File>> = zwl_gs::bit_encoder::ZwlBitEncoder::<zwl_gs::like_u12::LikeU12, _>::new(&mut input_buf, config.filled_behaviour.clone().into());
                    encoder.encode_headerless(&mut ouptut_buff)?;
                    // output.write(&encoded)?;
                },
                Encoding::ZWLU16 => {
                    let mut encoder =  zwl_gs::bit_encoder::ZwlBitEncoder::<zwl_gs::like_u16::LikeU16, _>::new(&mut input_buf, config.filled_behaviour.clone().into());
                    // println!("started to encode zwl16u");
                    encoder.encode_headerless(&mut ouptut_buff)?;
                    // output.write(&encoded)?;
                    // println!("encoded successfully {:?}", config.output_file);
                },
                Encoding::ZWLU32 => {
                    let mut encoder = zwl_gs::bit_encoder::ZwlBitEncoder::<zwl_gs::like_u32::LikeU32, _>::new(&mut input_buf, config.filled_behaviour.clone().into());
                    encoder.encode_headerless(&mut ouptut_buff)?;
                    // output.write(&encoded)?;
                },
                Encoding::ZWLU64 => {
                    let mut encoder = zwl_gs::bit_encoder::ZwlBitEncoder::<zwl_gs::like_u64::LikeU64, _>::new(&mut input_buf, config.filled_behaviour.clone().into());
                    encoder.encode_headerless(&mut ouptut_buff)?;
                    // output.write(&encoded)?;
                },
                Encoding::Huffman => {
                    huffman_encoding::encoder::encode_with_padding(&mut input_buf, &mut ouptut_buff)?;
                    // huffman_encoding::encoder::encode_with_padding(working_space.as_slice(), &mut cursor)?;
                    // output.write(&encoded)?;
                },
            }
            ouptut_buff.flush()?;
        },
        Mode::Decode => {
            let (working_d, working_mtf, working_bwt) = {
                match (config.mtf, config.bwt){
                    (true, true) => (tempfile::tempfile()?, Some(tempfile::tempfile()?), Some(output)),
                    (true, false) => (tempfile::tempfile()?, Some(output), None),
                    (false, true) => (tempfile::tempfile()?, None, Some(output)),
                    (false, false) => (output, None, None),
                }
            };
            let mut working_buffer =  BufWriter::new(working_d);
            // let mut input_buf = BufReader::new(input);
            // let mut input_buffer = vec![];
            // input.read_to_end(&mut input_buffer)?;
            // let mut working_space = vec![];
            
            match config.encoding{
                Encoding::ZWLU12 => {
                    let mut decoder = zwl_gs::bit_decoder::ZwlBitDecoder::<zwl_gs::like_u12::LikeU12, _>::new(input_buf, config.filled_behaviour.clone().into());
                    decoder.decode(&mut working_buffer)?;
                },
                Encoding::ZWLU16 => {
                    let mut decoder = zwl_gs::bit_decoder::ZwlBitDecoder::<zwl_gs::like_u16::LikeU16, _>::new(input_buf, config.filled_behaviour.clone().into());
                    decoder.decode(&mut working_buffer)?;
                },
                Encoding::ZWLU32 => {
                    let mut decoder = zwl_gs::bit_decoder::ZwlBitDecoder::<zwl_gs::like_u32::LikeU32, _>::new(input_buf, config.filled_behaviour.clone().into());
                    decoder.decode(&mut working_buffer)?;
                },
                Encoding::ZWLU64 => {
                    let mut decoder = zwl_gs::bit_decoder::ZwlBitDecoder::<zwl_gs::like_u64::LikeU64, _>::new(input_buf, config.filled_behaviour.clone().into());
                    decoder.decode(&mut working_buffer)?;
                },
                Encoding::Huffman => {
                    huffman_encoding::decoder::decode_with_padding(input_buf, &mut working_buffer)?;
                },
            }
            working_buffer.flush()?;
            if config.mtf{
                input_buf = BufReader::new(working_buffer.into_inner()?);
                // println!("MTF!");
                working_buffer =  BufWriter::new(working_mtf.unwrap());
                let alph = (0..=u8::MAX).map(|byte| byte).collect::<Vec<u8>>();
                let mut alphabet = LinkedList::new();
                alphabet.extend(&alph);
                // let the_rest: Vec<_> = working_space.iter().map(|u| *u as usize).collect();
                move_to_front_decode_r_w(&mut alphabet, &mut input_buf, &mut working_buffer)?;
                working_buffer.flush()?;
            }
            if config.bwt{
                input_buf = BufReader::new(working_buffer.into_inner()?);
                working_buffer =  BufWriter::new(working_bwt.unwrap());
                // let mut decoded = vec![];
                let mut buff = [0; 9];
                // println!("SIZE: {}", input_buffer.len());
                while let Ok(size) = input_buf.read(&mut buff) && size > 1{
                    let res = burrows_wheeler_transform::bwt_decode(buff[1..size].into(), buff[0].into());
                    working_buffer.write(&res)?;
                }
                working_buffer.flush()?;
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
#[derive(Parser, Clone)]
#[command(version, about)]
pub struct Config {
    input_file: PathBuf,
    output_file: Option<PathBuf>,
    #[arg(long, short, default_value_t = Mode::Encode, value_enum)]
    mode: Mode,

    #[arg(long, short, value_enum, help = "Ecnoding used in encoding mode")]
    encoding: Encoding,

    #[arg(long, default_value_t = false, help = "Burrows wheeler transform procedure when encoding")]
    bwt: bool,
    #[arg(long, default_value_t = false, help = "Move to front procedure when encoding")]
    mtf: bool,

    #[arg(long, short, default_value_t = FilledOption::Clear, value_enum, help = "Filled behavior of dictionary used in zwl encoding mode")]
    filled_behaviour: FilledOption,

    #[arg(long, short, default_value_t = false)]
    overwrite: bool,
}




#[cfg(test)]
mod tests {
    use burrows_wheeler_transform::*;
    use zwl_gs::{like_u12, like_u16};
    use std::{collections::LinkedList, io::Read};
    const PREAMBLE: &str =  "The Project Gutenberg eBook of The Ethics of Aristotle
    
This ebook is for the use of anyone anywhere in the United States and
most other parts of the world at no cost and with almost no restrictions
whatsoever. You may copy it, give it away or re-use it under the terms
of the Project Gutenberg License included with this ebook or online
at www.gutenberg.org. If you are not located in the United States,
you will have to check the laws of the country where you are located
before using this eBook...";
    #[test]
    fn huffman_text_big() -> std::io::Result<()>{
        
        let original_vu8 = PREAMBLE.as_bytes();
        println!("{}", original_vu8.len());
        let cursor = std::io::Cursor::new(&original_vu8);
        let mut he_text = vec![];
        let mut cursor_writter = std::io::Cursor::new(&mut he_text);
        huffman_encoding::encoder::encode_with_padding(cursor, &mut cursor_writter)?;
        let cursor = std::io::Cursor::new(&he_text);
        let mut he_dec = vec![];
        huffman_encoding::decoder::decode_with_padding(cursor, &mut he_dec)?;
        
        println!("{:?}", str::from_utf8(&he_dec));
        // assert_eq!(str::from_utf8(&he_dec), Ok(&text).map(|x| x.as_str()));
        assert_eq!(original_vu8, he_dec);
        Ok(())
    }
    #[test]
    fn bwt_huffman_text() -> std::io::Result<()>{
        let text = "ананас".to_string();
        let original_vu8 = text.as_bytes();
        let mut cursor = std::io::Cursor::new(&original_vu8);
        let mut buff_sm = [0; 8];
        let mut collecting: Vec<u8> = vec![];
        while let Ok(size) = cursor.read(&mut buff_sm) && size > 0{
            let (mut res, n0) = bwt_encode(&buff_sm[0..size]);
            collecting.push(n0.try_into().unwrap());
            collecting.append(&mut res);
        }
        let cursor = std::io::Cursor::new(&collecting);
        let mut he_text = vec![];
        let mut cursor_writter = std::io::Cursor::new(&mut he_text);
        huffman_encoding::encoder::encode_with_padding(cursor, &mut cursor_writter)?;
        let mut out_buff = [0u8; 9];
        let cursor = std::io::Cursor::new(&he_text);
        let mut he_dec = vec![];
        huffman_encoding::decoder::decode_with_padding(cursor, &mut he_dec)?;
        let mut cursor = std::io::Cursor::new(&he_dec);
        let mut decoded:Vec<u8> = vec![];
        while let Ok(size) = cursor.read(&mut out_buff) && size > 1{
            let mut res = bwt_decode(out_buff[1..size].into(), out_buff[0].into());
            decoded.append(&mut res);
        }
        
        println!("{:?}", str::from_utf8(&decoded));
        assert_eq!(collecting, he_dec);
        assert_eq!(original_vu8, decoded);
        Ok(())
    }

    #[test]
    fn bwt_huffman_text_big() -> std::io::Result<()>{
        let original_vu8 = PREAMBLE.as_bytes();
        println!("{}", original_vu8.len());
        let mut cursor = std::io::Cursor::new(&original_vu8);
        let mut buff_sm = [0; 8];
        let mut collecting: Vec<u8> = vec![];
        while let Ok(size) = cursor.read(&mut buff_sm) && size > 0{
            let (mut res, n0) = bwt_encode(&buff_sm[0..size]);
            collecting.push(n0.try_into().unwrap());
            collecting.append(&mut res);
        }
        let cursor = std::io::Cursor::new(&collecting);
        let mut he_text = vec![];
        let mut cursor_writter = std::io::Cursor::new(&mut he_text);
        huffman_encoding::encoder::encode_with_padding(cursor, &mut cursor_writter)?;
        let mut out_buff = [0u8; 9];
        let cursor = std::io::Cursor::new(&he_text);
        let mut he_dec = vec![];
        huffman_encoding::decoder::decode_with_padding(cursor, &mut he_dec)?;
        let mut cursor = std::io::Cursor::new(&he_dec);
        let mut decoded:Vec<u8> = vec![];
        while let Ok(size) = cursor.read(&mut out_buff) && size > 1{
            let mut res = bwt_decode(out_buff[1..size].into(), out_buff[0].into());
            decoded.append(&mut res);
        }
        
        println!("{:?}", str::from_utf8(&decoded));
        assert_eq!(collecting, he_dec);
        assert_eq!(original_vu8, decoded);
        Ok(())
    }
    #[test]
    fn mtf_text_big() -> std::io::Result<()>{
        let original_vu8 = PREAMBLE.as_bytes();
        println!("{}", original_vu8.len());
        let alph = (0..=u8::MAX).map(|byte| byte).collect::<Vec<u8>>();
        let mut alphabet = LinkedList::new();
        alphabet.extend(&alph);
        let collecting:Vec<_> = move_to_front::move_to_front(&mut alphabet, &original_vu8);
        
        let mut alphabet_d = LinkedList::new();
        alphabet_d.extend(&alph);
        let decoded = move_to_front::move_to_front_decode(&mut alphabet_d, &collecting);
        
        println!("{:?}", str::from_utf8(&decoded));
        assert_eq!(original_vu8.len(), decoded.len());
        assert_eq!(original_vu8, decoded);
        Ok(())
    }
    #[test]
    fn bwt_mtf_huffman_text_big() -> std::io::Result<()>{
        let original_vu8 = PREAMBLE.as_bytes();


        let mut cursor = std::io::Cursor::new(&original_vu8);
        let mut buff_sm = [0; 8];
        let mut collected_bwt: Vec<u8> = vec![];
        while let Ok(size) = cursor.read(&mut buff_sm) && size > 0{
            let (mut res, n0) = bwt_encode(&buff_sm[0..size]);
            collected_bwt.push(n0.try_into().unwrap());
            collected_bwt.append(&mut res);
        }

        println!("{}", original_vu8.len());
        let alph = (0..=u8::MAX).map(|byte| byte).collect::<Vec<u8>>();
        let mut alphabet = LinkedList::new();
        alphabet.extend(&alph);
        let collected_mtf :Vec<_> = move_to_front::move_to_front(&mut alphabet, &collected_bwt).iter().map(|u| *u as u8).collect();

        let cursor_mtf = std::io::Cursor::new(&collected_mtf);
        let mut he_text = vec![];
        let mut cursor_writter = std::io::Cursor::new(&mut he_text);
        huffman_encoding::encoder::encode_with_padding(cursor_mtf, &mut cursor_writter)?;
        
        let cursor = std::io::Cursor::new(&he_text);
        let mut he_dec = vec![];

        huffman_encoding::decoder::decode_with_padding(cursor, &mut he_dec)?;

        let he_dec: Vec<_> = he_dec.into_iter().map(|u| u as usize).collect();

        let mut alphabet_d = LinkedList::new();
        alphabet_d.extend(&alph);
        let decoded_mtf = move_to_front::move_to_front_decode(&mut alphabet_d, &he_dec);
        let mut cursor = std::io::Cursor::new(&decoded_mtf);

        let mut decoded_bwt:Vec<u8> = vec![];
        let mut out_buff = [0u8; 9];
        while let Ok(size) = cursor.read(&mut out_buff) && size > 1{
            let mut res = bwt_decode(out_buff[1..size].into(), out_buff[0].into());
            decoded_bwt.append(&mut res);
        }
        
        
        println!("{:?}", str::from_utf8(&decoded_bwt));
        assert_eq!(original_vu8, decoded_bwt);
        Ok(())
    }


    #[test]
    fn bwt_mtf_zwlu12_text_big() -> std::io::Result<()>{
        let original_vu8 = PREAMBLE.as_bytes();


        let mut cursor = std::io::Cursor::new(&original_vu8);
        let mut buff_sm = [0; 8];
        let mut collected_bwt: Vec<u8> = vec![];
        while let Ok(size) = cursor.read(&mut buff_sm) && size > 0{
            let (mut res, n0) = bwt_encode(&buff_sm[0..size]);
            collected_bwt.push(n0.try_into().unwrap());
            collected_bwt.append(&mut res);
        }

        println!("{}", original_vu8.len());
        let alph = (0..=u8::MAX).map(|byte| byte).collect::<Vec<u8>>();
        let mut alphabet = LinkedList::new();
        alphabet.extend(&alph);
        let collected_mtf :Vec<_> = move_to_front::move_to_front(&mut alphabet, &collected_bwt).iter().map(|u| *u as u8).collect();

        let cursor_mtf = std::io::Cursor::new(&collected_mtf);
        let mut zwl_enc = vec![];
        let mut cursor_writter = std::io::Cursor::new(&mut zwl_enc);

        let mut enc = zwl_gs::bit_encoder::ZwlBitEncoder::<like_u12::LikeU12, _>::new(cursor_mtf, zwl_gs::dictionary::FilledBehaviour::Clear);
        println!("started encoding");
        // enc.encode_with_padding_headerless(&mut cursor_writter)?;
        enc.encode_headerless(&mut cursor_writter)?;
        // let cursor_zwl_enc = std::io::Cursor::new(&zwl_enc[2..]);
        let cursor_zwl_enc = std::io::Cursor::new(&zwl_enc);
        let mut zwl_dec = vec![];

        println!("started decoding");
        let mut dec = zwl_gs::bit_decoder::ZwlBitDecoder::<like_u12::LikeU12, _>::new(cursor_zwl_enc, zwl_gs::dictionary::FilledBehaviour::Clear);
        // dec.decode_with_padding(&mut zwl_dec)?;
        dec.decode(&mut zwl_dec)?;

        assert_eq!(collected_mtf, zwl_dec);
        let zwl_dec: Vec<_> = zwl_dec.into_iter().map(|u| u as usize).collect();

        let mut alphabet_d = LinkedList::new();// let cursor_zwl_enc = std::io::Cursor::new(&zwl_enc[2..]);zw
        alphabet_d.extend(&alph);
        let decoded_mtf = move_to_front::move_to_front_decode(&mut alphabet_d, &zwl_dec);
        let mut cursor = std::io::Cursor::new(&decoded_mtf);

        let mut decoded_bwt:Vec<u8> = vec![];
        let mut out_buff = [0u8; 9];
        let mut i = 0;
        while let Ok(size) = cursor.read(&mut out_buff) && size > 1{
            i += 1;
            if out_buff[0] > 7 {
                println!("i: {i}");
                continue;
            }
            let mut res = bwt_decode(out_buff[1..size].into(), out_buff[0].into());
            decoded_bwt.append(&mut res);
        }
        
        
        println!("{:?}", str::from_utf8(&decoded_bwt));
        assert_eq!(original_vu8, decoded_bwt);
        Ok(())
    }


    #[test]
    fn bwt_mtf_zwlu16_text_big() -> std::io::Result<()>{
        let original_vu8 = PREAMBLE.as_bytes();


        let mut cursor = std::io::Cursor::new(&original_vu8);
        let mut buff_sm = [0; 8];
        let mut collected_bwt: Vec<u8> = vec![];
        while let Ok(size) = cursor.read(&mut buff_sm) && size > 0{
            let (mut res, n0) = bwt_encode(&buff_sm[0..size]);
            collected_bwt.push(n0.try_into().unwrap());
            collected_bwt.append(&mut res);
        }

        println!("{}", original_vu8.len());
        let alph = (0..=u8::MAX).map(|byte| byte).collect::<Vec<u8>>();
        let mut alphabet = LinkedList::new();
        alphabet.extend(&alph);
        let collected_mtf :Vec<_> = move_to_front::move_to_front(&mut alphabet, &collected_bwt).iter().map(|u| *u as u8).collect();

        let cursor_mtf = std::io::Cursor::new(&collected_mtf);
        let mut zwl_enc = vec![];
        let mut cursor_writter = std::io::Cursor::new(&mut zwl_enc);

        let mut enc = zwl_gs::bit_encoder::ZwlBitEncoder::<like_u16::LikeU16, _>::new(cursor_mtf, zwl_gs::dictionary::FilledBehaviour::Clear);
        println!("started encoding");
        enc.encode_headerless(&mut cursor_writter)?;
        let cursor_zwl_enc = std::io::Cursor::new(&zwl_enc);
        let mut zwl_dec = vec![];

        println!("started decoding");
        let mut dec = zwl_gs::bit_decoder::ZwlBitDecoder::<like_u16::LikeU16, _>::new(cursor_zwl_enc, zwl_gs::dictionary::FilledBehaviour::Clear);
        dec.decode(&mut zwl_dec)?;
        
        assert_eq!(collected_mtf, zwl_dec);

        let zwl_dec: Vec<_> = zwl_dec.into_iter().map(|u| u as usize).collect();

        let mut alphabet_d = LinkedList::new();
        alphabet_d.extend(&alph);
        let decoded_mtf = move_to_front::move_to_front_decode(&mut alphabet_d, &zwl_dec);
        let mut cursor = std::io::Cursor::new(&decoded_mtf);

        let mut decoded_bwt:Vec<u8> = vec![];
        let mut out_buff = [0u8; 9];
        let mut i = 0;
        while let Ok(size) = cursor.read(&mut out_buff) && size > 1{
            i += 1;
            if out_buff[0] > 7 {
                println!("i: {i}");
                continue;
            }
            let mut res = bwt_decode(out_buff[1..size].into(), out_buff[0].into());
            decoded_bwt.append(&mut res);
        }
        
        
        println!("{:?}", str::from_utf8(&decoded_bwt));
        assert_eq!(original_vu8, decoded_bwt);
        Ok(())
    }
    
    #[test]
    fn zwl_u12_test() -> std::io::Result<()> {
        let original_vu8 = PREAMBLE.as_bytes();
        let cursor = std::io::Cursor::new(&original_vu8);
        let mut zwl_enc = vec![];
        let mut cursor_writter = std::io::Cursor::new(&mut zwl_enc);

        // let mut enc = zwl_gs::bit_encoder::ZwlBitEncoder::<like_u12::LikeU12, _>::new(cursor, zwl_gs::dictionary::FilledBehaviour::Clear);
        let mut enc = zwl_gs::bit_encoder::ZwlBitEncoder::<like_u12::LikeU12, _>::new(cursor, zwl_gs::dictionary::FilledBehaviour::Clear);
        
        enc.encode_headerless(&mut cursor_writter)?;
        
        let cursor = std::io::Cursor::new(&zwl_enc);
        let mut zwl_dec = vec![];

        let mut dec = zwl_gs::bit_decoder::ZwlBitDecoder::<like_u12::LikeU12, _>::new(cursor, zwl_gs::dictionary::FilledBehaviour::Clear);
        dec.decode(&mut zwl_dec)?;
        assert_eq!(original_vu8, zwl_dec);
        Ok(())
    }


    #[test]
    fn zwl_u16_test() -> std::io::Result<()> {
        let original_vu8 = PREAMBLE.as_bytes();
        let cursor = std::io::Cursor::new(&original_vu8);
        let mut zwl_enc = vec![];
        let mut cursor_writter = std::io::Cursor::new(&mut zwl_enc);

        let mut enc = zwl_gs::bit_encoder::ZwlBitEncoder::<like_u16::LikeU16, _>::new(cursor, zwl_gs::dictionary::FilledBehaviour::Clear);
        // enc.encode_with_padding_headerless(&mut cursor_writter)?;
        enc.encode_headerless(&mut cursor_writter)?;
        
        let cursor = std::io::Cursor::new(&zwl_enc);
        let mut zwl_dec = vec![];

        let mut dec = zwl_gs::bit_decoder::ZwlBitDecoder::<like_u16::LikeU16, _>::new(cursor, zwl_gs::dictionary::FilledBehaviour::Clear);
        dec.decode(&mut zwl_dec)?;
        assert_eq!(original_vu8, zwl_dec);
        Ok(())
    }

    #[test]
    fn bwt_zwl_u12() -> std::io::Result<()>{
        let original_vu8 = PREAMBLE.as_bytes();
        println!("{}", original_vu8.len());
        let mut cursor = std::io::Cursor::new(&original_vu8);
        let mut buff_sm = [0; 8];
        let mut collecting: Vec<u8> = vec![];
        while let Ok(size) = cursor.read(&mut buff_sm) && size > 0{
            let (mut res, n0) = bwt_encode(&buff_sm[0..size]);
            collecting.push(n0.try_into().unwrap());
            collecting.append(&mut res);
        }
        let cursor = std::io::Cursor::new(&collecting);
        let mut zwl_enc = vec![];
        let mut cursor_writter = std::io::Cursor::new(&mut zwl_enc);
        let mut enc = zwl_gs::bit_encoder::ZwlBitEncoder::<like_u12::LikeU12, _>::new(cursor, zwl_gs::dictionary::FilledBehaviour::Clear);
        println!("started encoding");
        // enc.encode_with_padding_headerless(&mut cursor_writter)?;
        enc.encode_headerless(&mut cursor_writter)?;
        let cursor_zwl_enc = std::io::Cursor::new(&zwl_enc[..]);
        let mut zwl_dec = vec![];

        println!("started decoding");
        let mut dec = zwl_gs::bit_decoder::ZwlBitDecoder::<like_u12::LikeU12, _>::new(cursor_zwl_enc, zwl_gs::dictionary::FilledBehaviour::Clear);
        // dec.decode_with_padding(&mut zwl_dec)?;
        dec.decode(&mut zwl_dec)?;
        // huffman_encoding::encoder::encode_with_padding(cursor, &mut cursor_writter)?;
        let mut out_buff = [0u8; 9];
        let mut cursor = std::io::Cursor::new(&zwl_dec);
        // huffman_encoding::decoder::decode_with_padding(cursor, &mut he_dec)?;
        let mut decoded:Vec<u8> = vec![];
        while let Ok(size) = cursor.read(&mut out_buff) && size > 1{
            let mut res = bwt_decode(out_buff[1..size].into(), out_buff[0].into());
            decoded.append(&mut res);
        }
        
        println!("{:?}", str::from_utf8(&decoded));
        // assert_eq!(collecting, he_dec);
        assert_eq!(original_vu8, decoded);
        Ok(())
    }
    #[test]
    fn mtf_huffman_text_big() -> std::io::Result<()>{
        let original_vu8 = PREAMBLE.as_bytes();
        println!("{}", original_vu8.len());
        let alph = (0..=u8::MAX).map(|byte| byte).collect::<Vec<u8>>();
        let mut alphabet = LinkedList::new();
        alphabet.extend(&alph);
        let collecting:Vec<_> = move_to_front::move_to_front(&mut alphabet, &original_vu8).iter().map(|u| *u as u8).collect();

        let cursor = std::io::Cursor::new(&collecting);
        let mut he_text = vec![];
        let mut cursor_writter = std::io::Cursor::new(&mut he_text);
        huffman_encoding::encoder::encode_with_padding(cursor, &mut cursor_writter)?;
        let cursor = std::io::Cursor::new(&he_text);
        let mut he_dec = vec![];
        huffman_encoding::decoder::decode_with_padding(cursor, &mut he_dec)?;
        let he_dec: Vec<_> = he_dec.into_iter().map(|u| u as usize).collect();
        // let mut decoded:Vec<u8> = vec![];
        let mut alphabet_d = LinkedList::new();
        alphabet_d.extend(&alph);
        let decoded = move_to_front::move_to_front_decode(&mut alphabet_d, &he_dec);
        
        println!("{:?}", str::from_utf8(&decoded));
        assert_eq!(original_vu8.len(), decoded.len());

        let mut alphabet_d = LinkedList::new();
        alphabet_d.extend(&alph);
        let decoded = move_to_front::move_to_front_decode(&mut alphabet_d, &he_dec);
        
        println!("{:?}", str::from_utf8(&decoded));
        assert_eq!(original_vu8, decoded);
        Ok(())
    }
    
}