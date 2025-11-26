use std::fmt::Debug;

use counting_sort::{CountingSort, TryIntoIndex};
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

pub fn create_all_rotations<T: Clone>(input: &[T]) -> Vec<(Vec<&T>, bool)>{
    let lenth = input.len();
    let ps_sl: Vec<_> = input.iter().collect();
    // let ps_sl = input.clone();
    let table = (0..lenth).into_iter().map(|
        i| {
            let mut c = ps_sl.clone();
            c.rotate_left(i);
            (c, 0 == i)
        }).collect();
    table
}
pub fn bwt_encode<T>(input: &[T]) -> (Vec<T>, usize)
// where [T; N]: rdxsort::RdxSortTemplate,
where T: Clone + PartialEq + Ord
// for<'a> Vec<Vec<&'a T>>: rdxsort::RdxSort
{

    let lenth = input.len();
    let mut table = create_all_rotations(input);
    table.sort();
    // table.sort_by(|(a, _), (b, _)| a.cmp(&b));
    // table.rdxsort();
    println!("sorted ok");
    let o = table.iter().position(|(_, b)| *b).unwrap();
    let result:  Vec<_> =table.iter().map(|t| {
        t.0[lenth - 1].clone()
    }).collect();
    (result, o)
}

#[derive(Clone, Copy)]
pub struct Pair<T: TryIntoIndex + Copy + Clone>(T, usize);
impl<T: TryIntoIndex + Copy + Clone> TryIntoIndex for Pair<T>{
    type Error = <T as TryIntoIndex>::Error;

    fn try_into_index(value: &Self, min_value: &Self) -> Result<usize, Self::Error> {
        T::try_into_index(&value.0, &min_value.0)
    }
}
impl<T: TryIntoIndex + PartialOrd + std::marker::Copy> PartialOrd for Pair<T>{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.0.partial_cmp(&other.0) {
            Some(order) => {
                Some(order)
            }
            ord => return ord,
        }
    }
}

impl<T: TryIntoIndex + std::marker::Copy + PartialEq> PartialEq for Pair<T>{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl<T: TryIntoIndex + PartialOrd + std::marker::Copy> Eq for Pair<T>{

}
impl<T: TryIntoIndex + Copy + Clone + Ord> Ord for Pair<T>{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}


pub fn bwt_decode<T>(code: Vec<T>, no: usize) -> Vec<T>
where Pair<T>: Clone + PartialEq + Ord + counting_sort::TryIntoIndex + Copy, // + TryInto<usize, Error: Debug>,
T: Clone + PartialEq + Ord + counting_sort::TryIntoIndex +  Copy, 
for<'b> &'b T: Clone + PartialEq + Ord  +  Copy, // counting_sort::TryIntoIndex + 
// for<'a> &'a mut std::slice::Iter<'a, Pair<T>>: Iterator<Item = &'a Pair<T>>,
// for<'c> &'c std::slice::Iter<'c, Pair<T>>: Iterator<Item = &'c Pair<T>> + CountingSort<'c, Pair<T>>
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
    fn sorted_table(){
        let a = [3, 4, 2, 1];
        let mut b = create_all_rotations(&a);
        b.sort();
        println!("Sorted? table {b:?}");
        assert_eq!(b[0].0, [&1, &3, &4, &2])
    }
    #[test]
    fn sorted_text(){
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
    fn sorted_u8(){
        let thing = [99u8, 2, 5, 7, 14];
        println!("{:?}", create_all_rotations(&thing));
        let res = bwt_encode(&thing);
        println!("res: {res:?}");
        let expected_ = vec![99, 2, 5, 7, 14];
        assert_eq!((expected_, 4usize), res);
    }
}
