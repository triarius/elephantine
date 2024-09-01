use std::{
    borrow::Cow,
    fmt::{self, Display, Formatter},
};

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
                s.as_ref().map(|s| format!(" {s}")).unwrap_or_default(),
            ),
            Err(code, msg) => write!(f, "ERR {code} {msg}"),
            D(s) => write!(f, "D {}", escape(s)),
            Comment(s) => write!(f, "# {s}"),
            S(k, v) => write!(f, "S {k} {v}"),
            Inquire(k, v) => write!(f, "INQUIRE {k} {v}"),
        }
    }
}

/// Encode a string to be used in a response. It will percent escape `%`, `\n`, and `\r`.
fn escape(s: &str) -> Cow<'_, str> {
    // TODO: Split into lines of length at most 1000 bytes.
    let mut s = s;
    let mut escaped = String::with_capacity(s.len());

    loop {
        let unescaped_len = s
            .chars()
            .take_while(|c| !matches!(c, '%' | '\n' | '\r'))
            .count();

        let (unescaped, rest) = if unescaped_len >= s.len() {
            if escaped.is_empty() {
                return Cow::from(s);
            }
            (s, "")
        } else {
            s.split_at(unescaped_len)
        };

        if !unescaped.is_empty() {
            escaped.push_str(unescaped);
        }
        if rest.is_empty() {
            break;
        }
        let (first, rest) = rest.split_at(1);
        match first {
            "%" => escaped.push_str("%25"),
            "\n" => escaped.push_str("%0A"),
            "\r" => escaped.push_str("%0D"),
            _ => unreachable!(),
        }
        s = rest;
    }

    Cow::from(escaped)
}

#[cfg(test)]
mod test {
    use std::borrow::Cow;

    #[test]
    fn escape() {
        [
            ("", ""),
            ("a", "a"),
            ("a\n", "a%0A"),
            ("a\r", "a%0D"),
            ("a%", "a%25"),
            ("a%b", "a%25b"),
            ("a%b\n", "a%25b%0A"),
            ("a%b\r", "a%25b%0D"),
            ("a\nb", "a%0Ab"),
            ("a\rb", "a%0Db"),
            ("a\nb\n", "a%0Ab%0A"),
            ("a\rb\r", "a%0Db%0D"),
            ("a\nb\r", "a%0Ab%0D"),
            ("a\rb\n", "a%0Db%0A"),
            ("a\nb\r\n", "a%0Ab%0D%0A"),
            ("a\nb\r\nc", "a%0Ab%0D%0Ac"),
            ("a\nb\r\nc\n", "a%0Ab%0D%0Ac%0A"),
            ("a\nb\r\nc\nd", "a%0Ab%0D%0Ac%0Ad"),
            ("a\nb\r\nc\nd\n", "a%0Ab%0D%0Ac%0Ad%0A"),
        ]
        .into_iter()
        .map(|(input, expected)| (input, Cow::from(expected)))
        .for_each(|(input, expected)| {
            assert_eq!(super::escape(input), *expected);
        });
    }
}
