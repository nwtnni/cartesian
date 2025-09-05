pub use cartesian_derive::Cartesian;

pub type IntoIter<T> = <T as Cartesian>::IntoIter;
pub type Item<T> = <T as Cartesian>::Item;

pub trait Cartesian {
    type IntoIter: IntoIterCartesian<Item = Self::Item>;
    type Item;
}

pub trait IntoIterCartesian {
    type Item;
    fn into_iter_cartesian(self) -> impl Iterator<Item = Self::Item>;
}
