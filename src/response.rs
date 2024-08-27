use std::fmt::{self, Display, Formatter};

#[derive(Debug, PartialEq, Eq)]
pub enum Response {
    Ok(Option<String>),
    Err(i32, String),
    D(String),
    Comment(String),
    S(String, String),
    Inquire(String, String),
}

impl Display for Response {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use Response::*;
        match self {
            Ok(s) => write!(
                f,
                "OK{}",
                s.as_ref().map(|s| format!(" {}", s)).unwrap_or_default(),
            ),
            Err(code, msg) => write!(f, "ERR {} {}", code, msg),
            D(s) => write!(f, "D {}", s),
            Comment(s) => write!(f, "# {}", s),
            S(k, v) => write!(f, "S {} {}", k, v),
            Inquire(k, v) => write!(f, "INQUIRE {} {}", k, v),
        }
    }
}
