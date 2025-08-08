use nom::{
    branch::alt,
    bytes::complete::{tag, take_till},
    character::complete::{not_line_ending, space0, space1, u64},
    combinator::{eof, map, map_res, opt},
    error::Error as NomError,
    sequence::{preceded, separated_pair, terminated, tuple},
    IResult,
};
use paste::paste;
use std::{
    borrow::Cow,
    fmt::{self, Display, Formatter},
};
use thiserror::Error;
use urlencoding::decode;

#[derive(Debug, PartialEq, Eq)]
pub enum Request<'a> {
    Set(Set<'a>),
    Option(OptionReq<'a>),
    Confirm,
    ConfirmOneButton,
    Message,
    GetPin,
    GetInfoFlavor,
    GetInfoVersion,
    GetInfoTtyinfo,
    GetInfoPid,
    Bye,
    Reset,
    End,
    Help,
    Quit,
    Cancel,
    Auth,
    Nop,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Set<'a> {
    Timeout(u64),
    Desc(Cow<'a, str>),
    Prompt(Cow<'a, str>),
    Title(Cow<'a, str>),
    Ok(Cow<'a, str>),
    Cancel(Cow<'a, str>),
    Notok(Cow<'a, str>),
    Error(Cow<'a, str>),
    Keyinfo(Cow<'a, str>),
    Genpin(Cow<'a, str>),
    GenpinTt(Cow<'a, str>),
    Repeat(Cow<'a, str>),
    Repeaterror(Cow<'a, str>),
    Repeatok(Cow<'a, str>),
    Qualitybar(Option<Cow<'a, str>>),
    QualitybarTt(Cow<'a, str>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum OptionReq<'a> {
    Bool(Cow<'a, str>),
    KV(Cow<'a, str>, Cow<'a, str>),
}

#[derive(Debug, Error)]
pub enum Error {
    ParseError(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::ParseError(e) => write!(f, "Parse error: {e}"),
        }
    }
}

/// Parse a command from a string.
///
/// # Examples
/// ```
/// use elephantine::request::{parse, Request, Set};
///
/// let input = parse("SETTITLE title").unwrap();
/// assert_eq!(input, Request::Set(Set::Title(std::borrow::Cow::from("title"))));
/// ```
///
/// # Errors
/// Will return an error if the input string is not a valid command.
pub fn parse(s: &str) -> Result<Request<'_>, Error> {
    parse_command(s).map(|(_, c)| c).map_err(|e| match e {
        nom::Err::Error(NomError { input, .. }) | nom::Err::Failure(NomError { input, .. }) => {
            Error::ParseError(input.to_string())
        }
        nom::Err::Incomplete(_n) => Error::ParseError("Incomplete input".to_string()),
    })
}

fn parse_command(s: &str) -> IResult<&str, Request<'_>> {
    let (s, (cmd, _)) = tuple((
        alt((
            parse_set,
            parse_get,
            parse_confirm,
            parse_option,
            map(tag("MESSAGE"), |_| Request::Message),
            map(tag("BYE"), |_| Request::Bye),
            map(tag("RESET"), |_| Request::Reset),
            map(tag("END"), |_| Request::End),
            map(tag("HELP"), |_| Request::Help),
            map(tag("QUIT"), |_| Request::Quit),
            map(tag("CANCEL"), |_| Request::Cancel),
            map(tag("AUTH"), |_| Request::Auth),
            map(tag("NOP"), |_| Request::Nop),
        )),
        eof,
    ))(s)?;
    Ok((s, cmd))
}

macro_rules! gen_parse_set {
    ($x:expr) => {
        paste! {
            fn [<parse_set_ $x:lower>](s: &str) -> IResult<&str, Set<'_>> {
                map(
                    preceded(
                        terminated(tag($x), space1),
                        map_res(not_line_ending, decode),
                    ),
                    Set::[<$x:camel>],
                )(s)
            }
        }
    };
}

gen_parse_set!("PROMPT");
gen_parse_set!("TITLE");
gen_parse_set!("DESC");
gen_parse_set!("OK");
gen_parse_set!("CANCEL");
gen_parse_set!("NOTOK");
gen_parse_set!("ERROR");
gen_parse_set!("KEYINFO");
gen_parse_set!("GENPIN");
gen_parse_set!("GENPIN_TT");

fn parse_set_timeout(s: &str) -> IResult<&str, Set<'_>> {
    map(
        preceded(terminated(tag("TIMEOUT"), space1), u64),
        Set::Timeout,
    )(s)
}

fn parse_set_repeat(s: &str) -> IResult<&str, Set<'_>> {
    preceded(
        tag("REPEAT"),
        alt((
            map(
                map_res(preceded(space1, not_line_ending), decode),
                Set::Repeat,
            ),
            map(
                map_res(
                    preceded(terminated(tag("ERROR"), space1), not_line_ending),
                    decode,
                ),
                Set::Repeaterror,
            ),
            map(
                map_res(
                    preceded(terminated(tag("OK"), space1), not_line_ending),
                    decode,
                ),
                Set::Repeatok,
            ),
        )),
    )(s)
}

fn parse_set_qualitybar(s: &str) -> IResult<&str, Set<'_>> {
    preceded(
        tag("QUALITYBAR"),
        alt((
            map(eof, |_| Set::Qualitybar(None)),
            map(map_res(preceded(space1, not_line_ending), decode), |val| {
                Set::Qualitybar(Some(val))
            }),
            map(
                map_res(
                    preceded(terminated(tag("_TT"), space1), not_line_ending),
                    decode,
                ),
                Set::QualitybarTt,
            ),
        )),
    )(s)
}

fn parse_set(s: &str) -> IResult<&str, Request<'_>> {
    map(
        preceded(
            tag("SET"),
            alt((
                parse_set_timeout,
                parse_set_desc,
                parse_set_keyinfo,
                parse_set_prompt,
                parse_set_title,
                parse_set_ok,
                parse_set_cancel,
                parse_set_notok,
                parse_set_error,
                parse_set_repeat,
                parse_set_qualitybar,
                parse_set_genpin,
                parse_set_genpin_tt,
            )),
        ),
        Request::Set,
    )(s)
}

fn parse_get(s: &str) -> IResult<&str, Request<'_>> {
    preceded(
        tag("GET"),
        alt((map(tag("PIN"), |_| Request::GetPin), parse_get_info)),
    )(s)
}

fn parse_get_info(s: &str) -> IResult<&str, Request<'_>> {
    preceded(
        terminated(tag("INFO"), space1),
        alt((
            map(tag("flavor"), |_| Request::GetInfoFlavor),
            map(tag("version"), |_| Request::GetInfoVersion),
            map(tag("ttyinfo"), |_| Request::GetInfoTtyinfo),
            map(tag("pid"), |_| Request::GetInfoPid),
        )),
    )(s)
}

fn parse_confirm(s: &str) -> IResult<&str, Request<'_>> {
    preceded(
        tag("CONFIRM"),
        alt((
            map(preceded(space1, tag("--one-button")), |_| {
                Request::ConfirmOneButton
            }),
            map(eof, |_| Request::Confirm),
        )),
    )(s)
}

fn not_whitespace_nor_char(c: char) -> impl Fn(&str) -> IResult<&str, &str> {
    move |s| take_till(|d: char| d.is_whitespace() || d == c)(s)
}

fn parse_option(s: &str) -> IResult<&str, Request<'_>> {
    map(
        preceded(
            tuple((tag("OPTION"), space1)),
            map(
                preceded(
                    opt(tag("--")),
                    separated_pair(
                        map_res(not_whitespace_nor_char('='), decode),
                        tuple((space0, opt(tag("=")), space0)),
                        opt(map_res(not_line_ending, decode)),
                    ),
                ),
                |(key, value)| match value {
                    Some(value) if !value.is_empty() => OptionReq::KV(key, value),
                    _ => OptionReq::Bool(key),
                },
            ),
        ),
        Request::Option,
    )(s)
}

#[cfg(test)]
mod test {
    use super::Request::*;
    use std::borrow::Cow;

    #[test]
    fn parse_command() {
        use super::{OptionReq::*, Set::*};

        let test_cases = vec![
            ("OPTION key", Option(Bool(Cow::from("key")))),
            (
                "OPTION key=value",
                Option(KV(Cow::from("key"), Cow::from("value"))),
            ),
            ("GETINFO flavor", GetInfoFlavor),
            ("GETINFO version", GetInfoVersion),
            ("GETINFO ttyinfo", GetInfoTtyinfo),
            ("GETINFO pid", GetInfoPid),
            ("SETTIMEOUT 10", Set(Timeout(10))),
            ("SETDESC description", Set(Desc(Cow::from("description")))),
            ("SETPROMPT prompt", Set(Prompt(Cow::from("prompt")))),
            ("SETTITLE title", Set(Title(Cow::from("title")))),
            ("SETOK ok", Set(Ok(Cow::from("ok")))),
            (
                "SETCANCEL cancel",
                Set(super::Set::Cancel(Cow::from("cancel"))),
            ),
            ("SETNOTOK notok", Set(Notok(Cow::from("notok")))),
            ("SETERROR error", Set(Error(Cow::from("error")))),
            ("SETREPEAT value", Set(Repeat(Cow::from("value")))),
            ("SETREPEATERROR value", Set(Repeaterror(Cow::from("value")))),
            ("SETREPEATOK value", Set(Repeatok(Cow::from("value")))),
            ("SETQUALITYBAR", Set(Qualitybar(None))),
            (
                "SETQUALITYBAR value",
                Set(Qualitybar(Some(Cow::from("value")))),
            ),
            (
                "SETQUALITYBAR_TT value",
                Set(QualitybarTt(Cow::from("value"))),
            ),
            ("SETGENPIN value", Set(Genpin(Cow::from("value")))),
            ("SETGENPIN_TT value", Set(GenpinTt(Cow::from("value")))),
            ("CONFIRM", Confirm),
            ("CONFIRM --one-button", ConfirmOneButton),
            ("MESSAGE", Message),
            (
                "SETKEYINFO dummy-key-grip",
                Set(Keyinfo(Cow::from("dummy-key-grip"))),
            ),
            ("GETPIN", GetPin),
            ("BYE", Bye),
            ("RESET", Reset),
            ("END", End),
            ("HELP", Help),
            ("QUIT", Quit),
            ("CANCEL", super::Request::Cancel),
            ("AUTH", Auth),
            ("NOP", Nop),
        ];

        for (input, expected) in test_cases {
            let result = super::parse(input).unwrap();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn parse_set_option() {
        use super::{parse_option, OptionReq::*, Request};
        use nom::error::{Error, ErrorKind};

        let test_cases = vec![
            ("OPTION key", Ok(Bool(Cow::from("key")))),
            ("OPTION --key", Ok(Bool(Cow::from("key")))),
            (
                "OPTION key value",
                Ok(KV(Cow::from("key"), Cow::from("value"))),
            ),
            (
                "OPTION --key value",
                Ok(KV(Cow::from("key"), Cow::from("value"))),
            ),
            (
                "OPTION key=value",
                Ok(KV(Cow::from("key"), Cow::from("value"))),
            ),
            (
                "OPTION --key=value",
                Ok(KV(Cow::from("key"), Cow::from("value"))),
            ),
            (
                "OPTION key = value",
                Ok(KV(Cow::from("key"), Cow::from("value"))),
            ),
            (
                "OPTION --key = value",
                Ok(KV(Cow::from("key"), Cow::from("value"))),
            ),
            (
                "OPTIONalkey",
                Err(nom::Err::Error(Error::new("alkey", ErrorKind::Space))),
            ),
        ];

        for (input, expected) in test_cases {
            let result = parse_option(input);
            assert_eq!(result, expected.map(|x| ("", Request::Option(x))));
        }
    }

    #[test]
    fn parse_set_qualitybar() {
        use super::parse_set_qualitybar;
        use super::Set;
        use nom::error::{Error, ErrorKind};

        let test_cases = vec![
            (
                "QUALITYBARa",
                Err(nom::Err::Error(Error::new("a", ErrorKind::Tag))),
            ),
            ("QUALITYBAR", Ok(Set::Qualitybar(None))),
            (
                "QUALITYBAR value",
                Ok(Set::Qualitybar(Some(Cow::from("value")))),
            ),
            (
                "QUALITYBAR_TT value",
                Ok(Set::QualitybarTt(Cow::from("value"))),
            ),
        ];

        for (input, expected) in test_cases {
            let result = parse_set_qualitybar(input);
            assert_eq!(result, expected.map(|x| ("", x)));
        }
    }

    #[test]
    fn parse_confirm() {
        use super::parse_confirm;
        use nom::error::{Error, ErrorKind};

        let test_cases = vec![
            (
                "CONFIRM a",
                Err(nom::Err::Error(Error::new(" a", ErrorKind::Eof))),
            ),
            ("CONFIRM", Ok(Confirm)),
            ("CONFIRM --one-button", Ok(ConfirmOneButton)),
        ];

        for (input, expected) in test_cases {
            let result = parse_confirm(input);
            assert_eq!(result, expected.map(|x| ("", x)));
        }
    }
}
