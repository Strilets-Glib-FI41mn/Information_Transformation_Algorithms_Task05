use std::fmt::Debug;

use counting_sort::CountingSort;
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


pub fn bwt_decode<'a, T>(code: &[T], no: usize) -> Vec<T>
where T: Clone + PartialEq + Ord + Default + counting_sort::TryIntoIndex + 'a + Copy + TryInto<usize, Error: Debug>,
&'a mut std::slice::Iter<'a, T>: Clone + Sized +  Iterator<Item = &'a T>,
&'a std::slice::Iter<'a, T>: Iterator<Item = &'a T> + CountingSort<'a, T>
{
    let mut res = vec![T::default(); code.len()];
    let sorted = code.to_vec().iter()
    .cnt_sort().unwrap();
    
    let mut pos = no;
    (0..code.len()).for_each(|i|{
        pos = sorted[pos].try_into().unwrap();
        res[i] = code[pos];
        }
    );
    res
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
        let thing: Vec<_> = "ананас".chars().collect();
        let res = bwt_encode(&thing);
        println!("res: {res:?}");
        let expected_: Vec<_> = "сннааа".chars().collect();
        assert_eq!((expected_, 0usize), res);
        let decoded = bwt_decode(res.0, res.1);
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
