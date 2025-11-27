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
    println!("sorted ok");
    let o = table.iter().position(|(_, b)| *b).unwrap();
    let result:  Vec<_> =table.iter().map(|t| {
        t.0[lenth - 1].clone()
    }).collect();
    (result, o)
}


pub fn bwt_decode<T>(code: Vec<T>, no: usize) -> Vec<T>
where Pair<T>: Clone + PartialEq + Ord + counting_sort::TryIntoIndex + Copy,
T: Clone + PartialEq + Ord + counting_sort::TryIntoIndex +  Copy, 
for<'b> &'b T: Clone + PartialEq + Ord  +  Copy,
{
    let mut res = vec![None; code.len()];
    let tv = code.to_vec();
    let sorted = tv.iter().enumerate()
    .map(|(u, t)|{Pair(t.clone(), u)}).collect::<Vec<_>>().iter()
    .cnt_sort().unwrap();
    let mut pos = no;
    (0..code.len()).for_each(|i|{
        pos = sorted[pos].1;
        res[i] = Some(code[pos]);
        }
    );
    drop(sorted);
    res.iter().map(|o| o.unwrap()).collect()
}



#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bwt_text(){
        let thing: Vec<_> = "ананас".chars().map(|ch| ch as u32).collect();
        let res = bwt_encode(&thing);
        let comp = (res.0.iter().map(|u|  char::from_u32(*u).unwrap()).collect() ,res.1);
        println!("res: {comp:?}");
        let expected_: Vec<_> = "сннааа".chars().collect();
        assert_eq!((expected_, 0usize), comp);
        let decoded = bwt_decode(res.0, res.1);
        let comp_decoded:Vec<_> = decoded.iter().map(|u|  char::from_u32(*u).unwrap()).collect();
        assert_eq!(thing, decoded);
    }
    #[test]
    fn bwt_u8(){
        let thing = [99u8, 2, 5, 7, 14];
        println!("{:?}", create_all_rotations(&thing));
        let res = bwt_encode(&thing);
        println!("res: {res:?}");
        let expected_ = vec![99, 2, 5, 7, 14];
        assert_eq!((expected_, 4usize), res);
        let decoded = bwt_decode(res.0, res.1);
        assert_eq!(decoded, thing);
    }
}
