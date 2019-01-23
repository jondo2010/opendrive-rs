mod errors;
mod opendrive;
mod parse_util;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests2;
pub mod types;

pub use crate::opendrive::*;

/// Check if a collection is monotonically increasing
trait Monotonic: Iterator {
    fn is_monotonic(self) -> bool
    where
        Self::Item: PartialOrd,
        Self: Sized + Clone,
    {
        self.clone().zip(self.skip(1)).by_ref().all(|(a, b)| a < b)
    }
}
impl<I: Iterator> Monotonic for I {}

/// Deserializes LVM file data from the specified reader
pub fn from_reader<R: std::io::Read>(input: R) -> Result<opendrive::Root, failure::Error> {
    let root: opendrive::Root = serde_xml_rs::from_reader(input).unwrap();
    return Ok(root);
}
