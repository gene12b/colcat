//idk what I'm doing here
//use itertools::Itertools;
//
//pub trait ResultIteratools: Iterator {
//    //type Item;
//    type Error;
//    type ItemRes = Result<Self::Item, Self::Error>;
//    fn collect_res_vec(self) -> Result<Vec<Self::Item>, Self::Error>
//    where
//        Self: std::marker::Sized,
//    {
//        self.collect::<Result<Vec<Self::Item>, Self::Error>>()
//    }
//}

pub trait VecEmptyOr {
    fn empty_or(self, default: impl FnOnce() -> Self) -> Self;
}

impl<T> VecEmptyOr for Vec<T> {
    fn empty_or(self, default: impl FnOnce() -> Self) -> Self {
        if self.is_empty() { default() } else { self }
    }
}
