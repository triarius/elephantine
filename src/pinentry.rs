use crate::{
    request::{parse, OptionReq, Request, Set},
    response::Response,
};
use color_eyre::Result;
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    io::{BufRead, Write},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum GetPinError {
    Command(CommandError),
    Setup(std::io::Error),
    Output(std::string::FromUtf8Error),
}

impl Display for GetPinError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use GetPinError::*;
        match self {
            Command(e) => write!(f, "{e}"),
            Setup(e) => write!(f, "Setup error: {e}"),
            Output(e) => write!(f, "Output error: {e}"),
        }
    }
}

#[derive(Debug, Error)]
pub(crate) struct CommandError {
    code: i32,
    stderr: String,
}

impl Display for CommandError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "Command failed with code {}:\n{}",
            self.code, self.stderr,
        )
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct State {
    timeout: u64,
    desc: Option<String>,
    keyinfo: Option<String>,
    prompt: Option<String>,
    title: Option<String>,
    ok: Option<String>,
    cancel: Option<String>,
    notok: Option<String>,
    error: Option<String>,
    repeat: Option<String>,
    repeatok: Option<String>,
    repeaterror: Option<String>,
    qualitybar: Option<String>,
    qualitybar_tt: Option<String>,
    genpin: Option<String>,
    genpin_tt: Option<String>,
    options: HashMap<String, Option<String>>,
}

fn handle_set_req(req: Set, state: &mut State) -> Vec<Response> {
    use Set::*;
    match req {
        Timeout(t) => state.timeout = t,
        Desc(m) => state.desc = Some(m.to_string()),
        Keyinfo(m) => state.keyinfo = Some(m.to_string()),
        Prompt(m) => state.prompt = Some(m.to_string()),
        Title(m) => state.title = Some(m.to_string()),
        Ok(m) => state.ok = Some(m.to_string()),
        Cancel(m) => state.cancel = Some(m.to_string()),
        Notok(m) => state.notok = Some(m.to_string()),
        Error(m) => state.error = Some(m.to_string()),
        Repeat(m) => state.repeat = Some(m.to_string()),
        Repeaterror(m) => state.repeaterror = Some(m.to_string()),
        Repeatok(m) => state.repeatok = Some(m.to_string()),
        Qualitybar(m) => state.qualitybar = m.map(|s| s.to_string()),
        QualitybarTt(m) => state.qualitybar_tt = Some(m.to_string()),
        Genpin(m) => state.genpin = Some(m.to_string()),
        GenpinTt(m) => state.genpin_tt = Some(m.to_string()),
    };
    vec![Response::Ok(None)]
}

fn handle_option_req(o: OptionReq, state: &mut State) -> Vec<Response> {
    use OptionReq::*;
    match o {
        Bool(k) => {
            state.options.insert(k.to_string(), None);
        }
        KV(k, v) => {
            state.options.insert(k.to_string(), Some(v.to_string()));
        }
    }
    vec![Response::Ok(None)]
}

#[derive(Debug, PartialEq, Eq)]
enum Action<T> {
    Next(T),
    Stop(T),
}

fn handle_req<F>(req: Request, state: &mut State, get_pin: F) -> Action<Vec<Response>>
where
    F: Fn(&State) -> std::result::Result<String, GetPinError>,
{
    use crate::request::Request::*;
    use Action::*;
    match req {
        Set(s) => Next(handle_set_req(s, state)),
        Option(o) => Next(handle_option_req(o, state)),
        Message => {
            // Show a message with the value of the last SETDESC
            Next(vec![Response::Ok(None)])
        }
        Confirm => {
            // Show a confirmation dialog with the value of the last SETDESC
            Next(vec![Response::Ok(None)])
        }
        ConfirmOneButton => {
            // Show a confirmation dialog with the value of the last SETDESC, but with only one
            // button
            Next(vec![Response::Ok(None)])
        }
        GetInfoPid => Next(vec![
            Response::D(format!("{}", std::process::id())),
            Response::Ok(None),
        ]),
        GetInfoVersion => Next(vec![
            Response::D(crate::build_info::PKG_VERSION.to_string()),
            Response::Ok(None),
        ]),
        GetInfoFlavor => Next(vec![Response::D("walker".to_string()), Response::Ok(None)]),
        GetInfoTtyinfo => {
            // TODO: find out what this is supposed to do by reading more from
            // https://github.com/gpg/pinentry/blob/f4be34f83fd2079fa452525738ef19783c712438/pinentry/pinentry.c#L1896
            Next(vec![
                Response::D(format!(
                    "- - - - {}/{} 0",
                    users::get_current_uid(),
                    users::get_current_gid(),
                )),
                Response::Ok(None),
            ])
        }
        GetPin => Next(get_pin(state).map_or_else(
            |e| match e {
                GetPinError::Command(e) => {
                    vec![Response::Err(e.code, e.stderr)]
                }
                e => vec![Response::Err(1, e.to_string())],
            },
            |pin| vec![Response::D(pin), Response::Ok(None)],
        )),
        Reset => {
            *state = State::default();
            Next(vec![Response::Ok(None)])
        }
        Help => {
            // TODO Print all available commands
            Next(vec![Response::Ok(None)])
        }
        Nop => Next(vec![Response::Ok(None)]),
        Bye | End | Quit | Cancel | Auth => {
            Stop(vec![Response::Ok(Some("closing connection".to_string()))])
        }
    }
}

pub(crate) fn walker_get_pin(_state: &State) -> std::result::Result<String, GetPinError> {
    std::process::Command::new("walker")
        .arg("--password")
        .output()
        .map_err(GetPinError::Setup)
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).map_err(GetPinError::Output)
            } else {
                Err(GetPinError::Command(CommandError {
                    code: output.status.code().unwrap_or(1),
                    stderr: String::from_utf8(output.stderr).unwrap_or_default(),
                }))
            }
        })
}

pub(crate) fn listen<F>(input: impl BufRead, output: &mut impl Write, get_pin: F) -> Result<()>
where
    F: Fn(&State) -> std::result::Result<String, GetPinError> + Copy,
{
    writeln!(
        output,
        "{}",
        Response::Ok(Some("Greetings from Elephantine".to_string())),
    )?;
    log::debug!("Started Assuan server...");

    let mut state = State::default();
    for line in input.lines() {
        let line = line?;
        log::debug!("Request: {}", line);

        let req = parse(&line)?;
        match handle_req(req, &mut state, get_pin) {
            Action::Next(resps) => {
                for resp in resps {
                    writeln!(output, "{resp}")?;
                }
            }
            Action::Stop(resps) => {
                for resp in resps {
                    writeln!(output, "{resp}")?;
                }
                return Ok(());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::listen;
    use indoc::indoc;

    fn get_pin(_state: &super::State) -> std::result::Result<String, super::GetPinError> {
        Ok("1234".to_string())
    }

    #[test]
    fn test_listen() {
        let uid = users::get_current_uid();
        let gid = users::get_current_gid();
        let pid = std::process::id();

        let input = std::io::BufReader::new(std::io::Cursor::new(indoc! {"
            OPTION no-grab
            OPTION ttyname=not a tty
            OPTION ttytype=dumb
            OPTION lc-ctype=en_AU.UTF8
            OPTION lc-messages=en_AU.UTF8
            OPTION default-ok=_OK
            OPTION default-cancel=_Cancel
            OPTION default-yes=_Yes
            OPTION default-no=_No
            OPTION default-prompt=PIN:
            OPTION default-cf-visi=Do you really want to make your passphrase visible on the screen?
            OPTION default-tt-visi=Make passphrase visible
            OPTION default-tt-hide=Hide passphrase
            OPTION default-capshint=Caps Lock is on
            OPTION touch-file=/run/user/1000/gnupg/d.e59j34m8zuain4ytq5zumaf5/S.gpg-agent
            OPTION owner=1577791/1000 quirinus
            GETINFO flavor
            GETINFO version
            GETINFO ttyinfo
            GETINFO pid
            SETKEYINFO n/B830C0023090DD5DC5F5D2EFFD00168706E40708
            SETDESC Please enter the passphrase to unlock the OpenPGP secret key:%0A%22Narthana Epa <narthana.epa@gmail.com>%22%0A255-bit EDDSA key, ID 0FA72769B0697155,%0Acreated 2022-09-30 (main key ID BF82195DF1BD0789).%0A
            SETPROMPT Passphrase:
            SETREPEATERROR does not match - try again
            SETREPEATOK Passphrase match.
            GETPIN
            BYE
        "}));
        let mut output = std::io::Cursor::new(vec![]);

        listen(input, &mut output, get_pin).unwrap();

        let output = String::from_utf8(output.into_inner()).unwrap();

        assert_eq!(
            output,
            format!(
                indoc! {"
                    OK Greetings from Elephantine
                    OK
                    OK
                    OK
                    OK
                    OK
                    OK
                    OK
                    OK
                    OK
                    OK
                    OK
                    OK
                    OK
                    OK
                    OK
                    OK
                    D walker
                    OK
                    D 0.1.0
                    OK
                    D - - - - {}/{} 0
                    OK
                    D {}
                    OK
                    OK
                    OK
                    OK
                    OK
                    OK
                    D 1234
                    OK
                    OK closing connection
                "},
                uid, gid, pid,
            ),
        );
    }
}
