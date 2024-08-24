use nom::{
    branch::alt,
    bytes::complete::{tag, take_till},
    character::complete::{not_line_ending, space0, space1, u64},
    combinator::{eof, flat_map, map, opt},
    error::Error as NomError,
    sequence::{preceded, separated_pair, tuple},
    IResult,
};
use paste::paste;
use std::fmt::{self, Display, Formatter};
use thiserror::Error;

#[derive(Debug, PartialEq, Eq)]
pub enum Request<'a> {
    SetTimeout(u64),
    SetDesc(&'a str),
    SetKeyinfo(&'a str),
    SetPrompt(&'a str),
    SetTitle(&'a str),
    SetOk(&'a str),
    SetCancel(&'a str),
    SetNotok(&'a str),
    SetError(&'a str),
    SetRepeat,
    SetQualitybar(Option<&'a str>),
    SetQualitybarTt(&'a str),
    Confirm,
    ConfirmOneButton,
    Message,
    OptionBool(&'a str),
    OptionKV(&'a str, &'a str),
    GetPin,
    GetInfoFlavor,
    GetInfoVersion,
    GetInfoTtyinfo,
    GetInfoPid,
    Bye,
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
/// assert_eq!(input, Request::SetTitle("title"));
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
                    not_line_ending,
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

fn parse_set_timeout(s: &str) -> IResult<&str, Request> {
    let (rem, (_, _, arg)) = tuple((tag("TIMEOUT"), space1, u64))(s)?;
    Ok((rem, Request::SetTimeout(arg)))
}

fn parse_set_repeat(s: &str) -> IResult<&str, Request> {
    let (rem, _) = tag("REPEAT")(s)?;
    Ok((rem, Request::SetRepeat))
}

fn parse_set_qualitybar(s: &str) -> IResult<&str, Request> {
    let (s, _) = tag("QUALITYBAR")(s)?;
    let res: IResult<&str, &str> = tag("_TT")(s);
    match res {
        Ok((s, _)) => {
            let (s, (_, arg)) = tuple((space1, not_line_ending))(s)?;
            Ok((s, Request::SetQualitybarTt(arg)))
        }
        Err(_) => {
            let res: IResult<&str, &str> = eof(s);
            match res {
                Ok((s, _)) => Ok((s, Request::SetQualitybar(None))),
                Err(_) => {
                    let (s, (_, arg)) = tuple((space1, not_line_ending))(s)?;
                    Ok((s, Request::SetQualitybar(Some(arg))))
                }
            }
        }
    }
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
    let (s, (key, value)) = preceded(
        opt(tag("--")),
        separated_pair(
            not_whitespace_nor_char('='),
            tuple((space0, opt(tag("=")), space0)),
            opt(not_line_ending),
        ),
    )(s)?;
    match value {
        Some(value) if !value.is_empty() => Ok((s, Request::OptionKV(key, value))),
        _ => Ok((s, Request::OptionBool(key))),
    }
}

#[cfg(test)]
mod test {
    use super::Request::*;

    #[test]
    fn parse_command() {
        let test_cases = vec![
            ("OPTION key", OptionBool("key")),
            ("OPTION key=val", OptionKV("key", "val")),
            ("GETINFO flavor", GetInfoFlavor),
            ("GETINFO version", GetInfoVersion),
            ("GETINFO ttyinfo", GetInfoTtyinfo),
            ("GETINFO pid", GetInfoPid),
            ("SETTIMEOUT 10", SetTimeout(10)),
            ("SETDESC description", SetDesc("description")),
            ("SETPROMPT prompt", SetPrompt("prompt")),
            ("SETTITLE title", SetTitle("title")),
            ("SETOK ok", SetOk("ok")),
            ("SETCANCEL cancel", SetCancel("cancel")),
            ("SETNOTOK notok", SetNotok("notok")),
            ("SETERROR error", SetError("error")),
            ("SETREPEAT", SetRepeat),
            ("SETQUALITYBAR", SetQualitybar(None)),
            ("SETQUALITYBAR value", SetQualitybar(Some("value"))),
            ("SETQUALITYBAR_TT value", SetQualitybarTt("value")),
            ("CONFIRM", Confirm),
            ("CONFIRM --one-button", ConfirmOneButton),
            ("MESSAGE", Message),
            ("SETKEYINFO dummy-key-grip", SetKeyinfo("dummy-key-grip")),
            ("GETPIN", GetPin),
            ("BYE", Bye),
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
            ("OPTION key", Ok(OptionBool("key"))),
            ("OPTION --key", Ok(OptionBool("key"))),
            ("OPTION key value", Ok(OptionKV("key", "value"))),
            ("OPTION --key value", Ok(OptionKV("key", "value"))),
            ("OPTION key=value", Ok(OptionKV("key", "value"))),
            ("OPTION --key=value", Ok(OptionKV("key", "value"))),
            ("OPTION key = value", Ok(OptionKV("key", "value"))),
            ("OPTION --key = value", Ok(OptionKV("key", "value"))),
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
                Err(nom::Err::Error(Error::new("a", ErrorKind::Space))),
            ),
            ("QUALITYBAR", Ok(SetQualitybar(None))),
            ("QUALITYBAR value", Ok(SetQualitybar(Some("value")))),
            ("QUALITYBAR_TT value", Ok(SetQualitybarTt("value"))),
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
