pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

pub fn create_all_rotations<T>(input: &[T]) -> Vec<(Vec<&T>, bool)>{
    let lenth = input.len();
    let ps_sl: Vec<&T> = input.iter().collect();
    let table = (0..lenth).into_iter().map(|
        i| {
            let mut c = ps_sl.clone();
            c.rotate_left(i);
        (c, i == 0)
        }).collect();
    table
}
pub fn bwt_encode<T>(input: &[T]) -> (Vec<&T>, usize)
where for<'a> &'a Vec<&'a T>: Ord
{
    let lenth = input.len();
    let mut table = create_all_rotations(input);
    table.sort_by(|(a, _), (b, _)| a.cmp(&b));
    let o = table.iter().position(|(_, i)| *i == true).unwrap();
    let result:  Vec<_> =table.iter().map(|(t, _)| {
        t[lenth - 1]
    }).collect();
    (result, o)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sorted_table(){

        let a: Vec<u8>= vec![3, 4, 2, 1];
        let mut b = create_all_rotations(&a);
        b.sort();
        println!("Sorted? table {b:?}");
        assert_eq!(b[0].0, &[&1, &3, &4, &2])
    }
    #[test]
    fn sorted_text(){
        let thing = "ананас";
        let thing_: Vec<_> = thing.chars().collect();
        let res = bwt_encode(&thing_);
        println!("res: {res:?}");
        let expected_: Vec<_> = "сннааа".chars().collect();
        let expected = expected_.iter().collect();
        assert_eq!((expected, 0usize), res);
    }
}
