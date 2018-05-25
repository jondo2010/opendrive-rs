use types;

#[derive(Debug, Fail)]
pub enum ValidationError {
  #[fail(display = "validation error: \"{}\"", _0)]
  ReferenceLineGeometry(&'static str),

  #[fail(display = "validation error: reference line lengths don't match: {:?}", _0)]
  ReferenceLineLength((types::Length, types::Length)),
}