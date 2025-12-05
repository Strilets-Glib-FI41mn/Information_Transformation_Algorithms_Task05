use std::collections::LinkedList;

pub fn move_to_front_rw<I, O>(alphabet: &mut LinkedList<u8>, input: I, mut output: O) -> std::io::Result<()>
where I: std::io::Read, O: std::io::Write
{
    for byte in input.bytes(){
        let symbol = byte?;
        let pos = alphabet.iter().position(|q| q == &symbol).unwrap();
        let mut second = alphabet.split_off(pos);
        alphabet.push_front(second.pop_back().unwrap());
        alphabet.append(&mut second);
        output.write(&[pos.try_into().unwrap()])?;
    };
    return Ok(())
}

pub fn move_to_front_decode_r_w<I, O>(alphabet: &mut LinkedList<u8>, input: I, mut output: O) -> std::io::Result<()>
where I: std::io::Read, O: std::io::Write
{
    // let mut output = vec![];
    for byte in input.bytes(){
        let b = byte?;
        let u = b.try_into().unwrap();
        let symbol = alphabet.iter().nth(u)
        .expect(&format!("Position failed: {u}")).clone();
        let mut second = alphabet.split_off(u);
        alphabet.push_front(second.pop_back().unwrap());
        alphabet.append(&mut second);
        output.write(&[symbol])?;
    }
    return Ok(());
}

pub fn move_to_front<T>(alphabet: &mut LinkedList<T>, input: &[T]) -> Vec<usize>
where for<'a> &'a T: Eq
{
    let mut output = vec![];
    input.iter().for_each(|symbol|{
        let pos = alphabet.iter().position(|q| q == &symbol).unwrap();
        let mut second = alphabet.split_off(pos);
        alphabet.push_front(second.pop_back().unwrap());
        alphabet.append(&mut second);
        output.push(pos);
    });
    return output;
}

pub fn move_to_front_decode<T>(alphabet: &mut LinkedList<T>, input: &[usize]) -> Vec<T>
where T: Clone, for<'a> &'a T: std::ops::Deref
{
    let mut output = vec![];
    input.iter().for_each(|u|{
        let symbol = alphabet.iter().nth(*u)
        .expect(&format!("Position failed: {u}")).clone();
        let mut second = alphabet.split_off(*u);
        alphabet.push_front(second.pop_back().unwrap());
        alphabet.append(&mut second);
        output.push(symbol);
    });
    return output;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forward_encode_decode() {
        let input: Vec<char> = "ccccbdbdbdeee".chars().collect();
        let alphabet = ['a', 'b', 'c', 'd', 'e', 'f'];
        let mut alph_1 = LinkedList::from(alphabet.clone());
        let mut alph_2 = LinkedList::from(alphabet.clone());
        let encoded = move_to_front(&mut alph_1, input.as_slice());

        println!("encoded: {}", encoded.iter().map(|u| {alphabet[*u]}).collect::<String>());
        let dec = move_to_front_decode(&mut alph_2, &encoded);
        
        println!("decoded: {}", dec.iter().collect::<String>());
        assert_eq!(dec, input)
    }
}