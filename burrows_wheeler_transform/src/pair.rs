use counting_sort::TryIntoIndex;

#[derive(Clone, Copy)]
pub struct Pair<T: TryIntoIndex + Copy + Clone>(pub T, pub usize);
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