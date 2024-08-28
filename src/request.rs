use nom::{
    branch::alt,
    bytes::complete::{tag, take_till},
    character::complete::{not_line_ending, space0, space1, u64},
    combinator::{eof, flat_map, map, map_res, opt},
    error::Error as NomError,
    sequence::{preceded, separated_pair, tuple},
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
    SetTimeout(u64),
    SetDesc(Cow<'a, str>),
    SetKeyinfo(Cow<'a, str>),
    SetPrompt(Cow<'a, str>),
    SetTitle(Cow<'a, str>),
    SetOk(Cow<'a, str>),
    SetCancel(Cow<'a, str>),
    SetNotok(Cow<'a, str>),
    SetError(Cow<'a, str>),
    SetRepeat,
    SetRepeaterror(Cow<'a, str>),
    SetRepeatok(Cow<'a, str>),
    SetQualitybar(Option<Cow<'a, str>>),
    SetQualitybarTt(Cow<'a, str>),
    SetGenpin(Cow<'a, str>),
    SetGenpinTt(Cow<'a, str>),
    Confirm,
    ConfirmOneButton,
    Message,
    OptionBool(Cow<'a, str>),
    OptionKV(Cow<'a, str>, Cow<'a, str>),
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
/// use elephantine::request::{parse, Request};
///
/// let input = parse("SETTITLE title").unwrap();
/// assert_eq!(input, Request::SetTitle(std::borrow::Cow::from("title")));
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

fn parse_command(s: &str) -> IResult<&str, Request> {
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
            fn [<parse_set_ $x:lower>](s: &str) -> IResult<&str, Request<'_>> {
                let (rem, (_, _, arg)) = tuple((
                    tag($x),
                    space1,
                    map_res(not_line_ending, decode),
                ))(s)?;
                Ok((rem, Request::[<Set $x:camel>](arg)))
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

fn parse_set_timeout(s: &str) -> IResult<&str, Request> {
    let (rem, (_, _, arg)) = tuple((tag("TIMEOUT"), space1, u64))(s)?;
    Ok((rem, Request::SetTimeout(arg)))
}

fn parse_set_repeat(s: &str) -> IResult<&str, Request> {
    let (s, _) = tag("REPEAT")(s)?;
    alt((
        map(eof, |_| Request::SetRepeat),
        map(
            preceded(tuple((tag("ERROR"), space1)), not_line_ending),
            |val| Request::SetRepeaterror(Cow::Borrowed(val)),
        ),
        map(
            preceded(tuple((tag("OK"), space1)), not_line_ending),
            |val| Request::SetRepeatok(Cow::Borrowed(val)),
        ),
    ))(s)
}

fn parse_set_qualitybar(s: &str) -> IResult<&str, Request> {
    let (s, _) = tag("QUALITYBAR")(s)?;
    alt((
        map(eof, |_| Request::SetQualitybar(None)),
        map(preceded(space1, not_line_ending), |val| {
            Request::SetQualitybar(Some(Cow::Borrowed(val)))
        }),
        map(
            preceded(tuple((tag("_TT"), space1)), not_line_ending),
            |val| Request::SetQualitybarTt(Cow::Borrowed(val)),
        ),
    ))(s)
}

fn parse_set(s: &str) -> IResult<&str, Request> {
    let (s, _) = tag("SET")(s)?;
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
    ))(s)
}

fn parse_get(s: &str) -> IResult<&str, Request> {
    let (s, _) = tag("GET")(s)?;
    alt((
        map(tag("PIN"), |_| Request::GetPin),
        flat_map(tag("INFO"), |_| parse_get_info),
    ))(s)
}

fn parse_get_info(s: &str) -> IResult<&str, Request> {
    let (s, _) = space1(s)?;
    alt((
        map(tag("flavor"), |_| Request::GetInfoFlavor),
        map(tag("version"), |_| Request::GetInfoVersion),
        map(tag("ttyinfo"), |_| Request::GetInfoTtyinfo),
        map(tag("pid"), |_| Request::GetInfoPid),
    ))(s)
}

fn parse_confirm(s: &str) -> IResult<&str, Request> {
    let (s, _) = tag("CONFIRM")(s)?;
    map(tag::<&str, &str, NomError<&str>>(" --one-button"), |_| {
        Request::ConfirmOneButton
    })(s)
    .or_else(|_| {
        let (s, _) = eof(s)?;
        Ok((s, Request::Confirm))
    })
}

fn not_whitespace_nor_char(c: char) -> impl Fn(&str) -> IResult<&str, &str> {
    move |s| take_till(|d: char| d.is_whitespace() || d == c)(s)
}

fn parse_option(s: &str) -> IResult<&str, Request> {
    let (s, _) = tuple((tag("OPTION"), space1))(s)?;
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
            Some(value) if !value.is_empty() => Request::OptionKV(key, value),
            _ => Request::OptionBool(key),
        },
    )(s)
}

#[cfg(test)]
mod test {
    use super::Request::*;
    use std::borrow::Cow;

    #[test]
    fn parse_command() {
        let test_cases = vec![
            ("OPTION key", OptionBool(Cow::from("key"))),
            (
                "OPTION key=value",
                OptionKV(Cow::from("key"), Cow::from("value")),
            ),
            ("GETINFO flavor", GetInfoFlavor),
            ("GETINFO version", GetInfoVersion),
            ("GETINFO ttyinfo", GetInfoTtyinfo),
            ("GETINFO pid", GetInfoPid),
            ("SETTIMEOUT 10", SetTimeout(10)),
            ("SETDESC description", SetDesc(Cow::from("description"))),
            ("SETPROMPT prompt", SetPrompt(Cow::from("prompt"))),
            ("SETTITLE title", SetTitle(Cow::from("title"))),
            ("SETOK ok", SetOk(Cow::from("ok"))),
            ("SETCANCEL cancel", SetCancel(Cow::from("cancel"))),
            ("SETNOTOK notok", SetNotok(Cow::from("notok"))),
            ("SETERROR error", SetError(Cow::from("error"))),
            ("SETREPEAT", SetRepeat),
            ("SETQUALITYBAR", SetQualitybar(None)),
            (
                "SETQUALITYBAR value",
                SetQualitybar(Some(Cow::from("value"))),
            ),
            (
                "SETQUALITYBAR_TT value",
                SetQualitybarTt(Cow::from("value")),
            ),
            ("SETGENPIN value", SetGenpin(Cow::from("value"))),
            ("SETGENPIN_TT value", SetGenpinTt(Cow::from("value"))),
            ("SETREPEATERROR value", SetRepeaterror(Cow::from("value"))),
            ("SETREPEATOK value", SetRepeatok(Cow::from("value"))),
            ("CONFIRM", Confirm),
            ("CONFIRM --one-button", ConfirmOneButton),
            ("MESSAGE", Message),
            (
                "SETKEYINFO dummy-key-grip",
                SetKeyinfo(Cow::from("dummy-key-grip")),
            ),
            ("GETPIN", GetPin),
            ("BYE", Bye),
            ("RESET", Reset),
            ("END", End),
            ("HELP", Help),
            ("QUIT", Quit),
            ("CANCEL", Cancel),
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
        use super::parse_option;
        use nom::error::{Error, ErrorKind};

        let test_cases = vec![
            ("OPTION key", Ok(OptionBool(Cow::from("key")))),
            ("OPTION --key", Ok(OptionBool(Cow::from("key")))),
            (
                "OPTION key value",
                Ok(OptionKV(Cow::from("key"), Cow::from("value"))),
            ),
            (
                "OPTION --key value",
                Ok(OptionKV(Cow::from("key"), Cow::from("value"))),
            ),
            (
                "OPTION key=value",
                Ok(OptionKV(Cow::from("key"), Cow::from("value"))),
            ),
            (
                "OPTION --key=value",
                Ok(OptionKV(Cow::from("key"), Cow::from("value"))),
            ),
            (
                "OPTION key = value",
                Ok(OptionKV(Cow::from("key"), Cow::from("value"))),
            ),
            (
                "OPTION --key = value",
                Ok(OptionKV(Cow::from("key"), Cow::from("value"))),
            ),
            (
                "OPTIONalkey",
                Err(nom::Err::Error(Error::new("alkey", ErrorKind::Space))),
            ),
        ];

        for (input, expected) in test_cases {
            let result = parse_option(input);
            assert_eq!(result, expected.map(|x| ("", x)));
        }
    }

    #[test]
    fn parse_set_qualitybar() {
        use super::parse_set_qualitybar;
        use nom::error::{Error, ErrorKind};

        let test_cases = vec![
            (
                "QUALITYBARa",
                Err(nom::Err::Error(Error::new("a", ErrorKind::Tag))),
            ),
            ("QUALITYBAR", Ok(SetQualitybar(None))),
            (
                "QUALITYBAR value",
                Ok(SetQualitybar(Some(Cow::from("value")))),
            ),
            (
                "QUALITYBAR_TT value",
                Ok(SetQualitybarTt(Cow::from("value"))),
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
