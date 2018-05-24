// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;

extern crate serde;
#[macro_use]
extern crate serde_derive;

extern crate chrono;
extern crate serde_xml_rs;
//extern crate env_logger;

mod errors;
mod opendrive;
mod parse_util;
#[cfg(test)]
mod tests;

pub use errors::*;
pub use opendrive::*;

/// Check if a collection is monotonically increasing
fn is_monotonic<T, I>(col: &T) -> bool
where
    for<'a> &'a T: IntoIterator<Item = &'a I>,
    I: PartialOrd,
{
    !col.into_iter()
        .zip(col.into_iter().skip(1))
        .any(|t: (&I, &I)| t.0 > t.1)
}

/// Deserializes LVM file data from the specified reader
pub fn from_reader<R: std::io::Read>(input: R) -> errors::Result<opendrive::Root> {
    let root: opendrive::Root = serde_xml_rs::from_reader(input).unwrap();
    return Ok(root);
}
