#[derive(Debug, Fail)]
pub enum ValidationError {
  #[fail(display = "validation error: \"{}\"", _0)]
  ReferenceLineGeometry(&'static str)
}