//use itertools::Itertools;
use super::*;
error_chain! {
  types {
    Error, ErrorKind, ResultExt, Result;
  }
  errors {
    /// A deserialization error occurred
    Deserialize(s: String) {
      description("A deserialization error occurred")
      display("deserialization error: \"{}\"", s)
    }
    /// An invalid separator
    InvalidSeparator(c: char) {
      description("An invalid separator was used by the file")
      display("An invalid separator \"{}\" was used by the file", c)
    }
    /// An error occurred while parsing a floating-point number
    ParseFloatError(e: std::num::ParseFloatError) {
      description("An error occurred while parsing a floating-point number")
      display("parse floating-point error: \"{:?}\"", e)
    }

    /// A validation error occurred
    Validation(s: String){
      description("A validation error occurred")
      display("validation error: \"{}\"", s)
    }
  }

  foreign_links {
    Io(std::io::Error);
    ParseIntError(std::num::ParseIntError);
  }
}

impl serde::de::Error for Error {
    fn custom<T: std::fmt::Display>(i_message: T) -> Self {
        ErrorKind::Deserialize(i_message.to_string()).into()
    }
}
