pub mod pair;
use pair::Pair;
use counting_sort::CountingSort;
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

pub fn create_all_rotations<T: Clone>(input: &[T]) -> Vec<(Vec<&T>, bool)>{
    let lenth = input.len();
    let ps_sl: Vec<_> = input.iter().collect();
    let table = (0..lenth).into_iter().map(|
        i| {
            let mut c = ps_sl.clone();
            c.rotate_left(i);
            (c, 0 == i)
        }).collect();
    table
}
pub fn bwt_encode<T>(input: &[T]) -> (Vec<T>, usize)
where T: Clone + PartialEq + Ord
{

    let lenth = input.len();
    let mut table = create_all_rotations(input);
    table.sort();
    // println!("sorted ok");
    let o = table.iter().position(|(_, b)| *b).unwrap();
    let result:  Vec<_> =table.iter().map(|t| {
        t.0[lenth - 1].clone()
    }).collect();
    if o >= input.len(){
        println!("{o} >= {}", input.len());
    }
    (result, o)
}


pub fn bwt_decode<T>(code: Vec<T>, no: usize) -> Vec<T>
where Pair<T>: Clone + PartialEq + Ord + counting_sort::TryIntoIndex + Copy, //+ std::fmt::Debug,
T: Clone + PartialEq + Ord + counting_sort::TryIntoIndex +  Copy, 
for<'b> &'b T: Clone + PartialEq + Ord  +  Copy,
{
    let mut res = vec![None; code.len()];
    let tv = code.to_vec();
    let pairs = tv.iter().enumerate()
    .map(|(u, t)|{Pair(t.clone(), u)}).collect::<Vec<_>>();
    
    match pairs.iter().cnt_sort(){
        Ok(sorted) => {

            let mut pos = no;
            (0..code.len()).for_each(|i|{
                pos = sorted[pos].1;
                res[i] = Some(code[pos]);
                }
            );
            drop(sorted);
            res.iter().map(|o| o.unwrap()).collect()
        },
        Err(error) => {
            match error{
                counting_sort::CountingSortError::SortingUnnecessary(_) => {
                    let mut pos = no;
                    (0..code.len()).for_each(|i|{
                        pos = pairs[pos].1;
                        res[i] = Some(code[pos]);
                        }
                    );
                    drop(pairs);
                    res.iter().map(|o| o.unwrap()).collect()
                },
                _ => Err(error).unwrap(),
            }
        },
    }
}


#[cfg(test)]
mod tests {
    use std::io::Read;

    use super::*;
    #[test]
    fn bwt_text(){
        let original: Vec<_> = "ананас".chars().map(|ch| ch as u32).collect();
        let res = bwt_encode(&original);
        let comp = (res.0.iter().map(|u|  char::from_u32(*u).unwrap()).collect() ,res.1);
        println!("res: {comp:?}");
        let expected_: Vec<_> = "сннааа".chars().collect();
        assert_eq!((expected_, 0usize), comp);
        let decoded = bwt_decode(res.0, res.1);
        let comp_decoded:Vec<_> = decoded.iter().map(|u|  char::from_u32(*u).unwrap()).collect();
        assert_eq!(original, decoded);
    }
    #[test]
    fn bwt_u8(){
        let original = [99u8, 2, 5, 7, 14];
        println!("{:?}", create_all_rotations(&original));
        let res = bwt_encode(&original);
        println!("res: {res:?}");
        let expected_ = vec![99, 2, 5, 7, 14];
        assert_eq!((expected_, 4usize), res);
        let decoded = bwt_decode(res.0, res.1);
        assert_eq!(decoded, original);
    }

    #[test]
    fn bwt_text_big(){
        let text = "The Project Gutenberg eBook of The Ethics of Aristotle
    
This ebook is for the use of anyone anywhere in the United States and
most other parts of the world at no cost and with almost no restrictions
whatsoever. You may copy it, give it away or re-use it under the terms
of the Project Gutenberg License included with this ebook or online
at www.gutenberg.org. If you are not located in the United States,
you will have to check the laws of the country where you are located
before using this eBook.".to_string();
        let mut original = text.as_bytes();
        let mut cursor = std::io::Cursor::new(&original);
        let mut buff_sm = [0; 8];
        let mut collecting: Vec<u8> = vec![];
        while let Ok(size) = cursor.read(&mut buff_sm) && size > 0{
            let (mut res, n0) = bwt_encode(&buff_sm[0..size]);
            collecting.push(n0.try_into().unwrap());
            collecting.append(&mut res);
        }
        let mut cursor = std::io::Cursor::new(&collecting);
        let mut out_buff = [0u8; 9];
        let mut decoded:Vec<u8> = vec![];
        while let Ok(size) = cursor.read(&mut out_buff)  && size > 1{
            let mut res = bwt_decode(out_buff[1..size].into(), out_buff[0].into());
            decoded.append(&mut res);
        }

        assert_eq!(decoded, original);
    }
    #[test]
    fn bwt_text_bigger(){
        let text = "The Project Gutenberg eBook of Gulliver's Travels into Several Remote Nations of the World
    
This ebook is for the use of anyone anywhere in the United States and
most other parts of the world at no cost and with almost no restrictions
whatsoever. You may copy it, give it away or re-use it under the terms
of the Project Gutenberg License included with this ebook or online
at www.gutenberg.org. If you are not located in the United States,
you will have to check the laws of the country where you are located
before using this eBook.

Title: Gulliver's Travels into Several Remote Nations of the World

Author: Jonathan Swift

Release date: February 1, 1997 [eBook #829]
                Most recently updated: April 6, 2025

Language: English

Credits: David Price


*** START OF THE PROJECT GUTENBERG EBOOK GULLIVER'S TRAVELS INTO SEVERAL REMOTE NATIONS OF THE WORLD ***




GULLIVER’S TRAVELS

into several

REMOTE NATIONS OF THE WORLD


BY JONATHAN SWIFT, D.D.,

dean of st. patrick’s, dublin.

[_First published in_ 1726–7.]

cover 

Contents


 THE PUBLISHER TO THE READER.
 A LETTER FROM CAPTAIN GULLIVER TO HIS COUSIN SYMPSON.
 PART I. A VOYAGE TO LILLIPUT.
 PART II. A VOYAGE TO BROBDINGNAG.
 PART III. A VOYAGE TO LAPUTA, BALNIBARBI, GLUBBDUBDRIB, LUGGNAGG AND JAPAN.
 PART IV. A VOYAGE TO THE COUNTRY OF THE HOUYHNHNMS.


THE PUBLISHER TO THE READER.

[_As given in the original edition_.]


The author of these Travels, Mr. Lemuel Gulliver, is my ancient and
intimate friend; there is likewise some relation between us on the
mother’s side. About three years ago, Mr. Gulliver growing weary of the
concourse of curious people coming to him at his house in Redriff, made
a small purchase of land, with a convenient house, near Newark, in
Nottinghamshire, his native country; where he now lives retired, yet in
good esteem among his neighbours.

Although Mr. Gulliver was born in Nottinghamshire, where his father
dwelt, yet I have heard him say his family came from Oxfordshire; to
confirm which, I have observed in the churchyard at Banbury in that
county, several tombs and monuments of the Gullivers.

Before he quitted Redriff, he left the custody of the following papers
in my hands, with the liberty to dispose of them as I should think fit.
I have carefully perused them three times. The style is very plain and
simple; and the only fault I find is, that the author, after the manner
of travellers, is a little too circumstantial. There is an air of truth
apparent through the whole; and indeed the author was so distinguished
for his veracity, that it became a sort of proverb among his neighbours
at Redriff, when any one affirmed a thing, to say, it was as true as if
Mr. Gulliver had spoken it.
".to_string();
        let mut original = text.as_bytes();
        let mut cursor = std::io::Cursor::new(&original);
        let mut buff_sm = [0; 8];
        let mut collecting: Vec<u8> = vec![];
        while let Ok(size) = cursor.read(&mut buff_sm) && size > 0{
            let (mut res, n0) = bwt_encode(&buff_sm[0..size]);
            collecting.push(n0.try_into().unwrap());
            collecting.append(&mut res);
        }
        let mut cursor = std::io::Cursor::new(&collecting);
        let mut out_buff = [0u8; 9];
        let mut decoded:Vec<u8> = vec![];
        while let Ok(size) = cursor.read(&mut out_buff)  && size > 1{
            let mut res = bwt_decode(out_buff[1..size].into(), out_buff[0].into());
            decoded.append(&mut res);
        }

        assert_eq!(decoded, original);
    }
}