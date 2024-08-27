use crate::{
    request::{parse, Request},
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
    repeat: bool,
    qualitybar: Option<String>,
    qualitybar_tt: Option<String>,
    genpin: Option<String>,
    genpin_tt: Option<String>,
    options: HashMap<String, Option<String>>,
}

fn handle_set_req(req: Request, state: &mut State) -> Vec<Response> {
    use crate::request::Request::*;
    match req {
        SetTimeout(timeout) => state.timeout = timeout,
        SetDesc(desc) => state.desc = Some(desc.to_string()),
        SetKeyinfo(keyinfo) => state.keyinfo = Some(keyinfo.to_string()),
        SetPrompt(prompt) => state.prompt = Some(prompt.to_string()),
        SetTitle(title) => state.title = Some(title.to_string()),
        SetOk(ok) => state.ok = Some(ok.to_string()),
        SetCancel(cancel) => state.cancel = Some(cancel.to_string()),
        SetNotok(notok) => state.notok = Some(notok.to_string()),
        SetError(error) => state.error = Some(error.to_string()),
        SetRepeat => state.repeat = true,
        SetQualitybar(qualitybar) => state.qualitybar = qualitybar.map(|s| s.to_string()),
        SetQualitybarTt(qualitybar_tt) => state.qualitybar_tt = Some(qualitybar_tt.to_string()),
        SetGenpin(genpin) => state.genpin = Some(genpin.to_string()),
        SetGenpinTt(genpin_tt) => state.genpin_tt = Some(genpin_tt.to_string()),
        OptionBool(key) => {
            state.options.insert(key.to_string(), None);
        }
        OptionKV(key, value) => {
            state
                .options
                .insert(key.to_string(), Some(value.to_string()));
        }
        _ => {}
    };
    vec![Response::Ok(None)]
}

fn handle_req<F>(req: Request, state: &mut State, get_pin: F) -> Vec<Response>
where
    F: Fn(&State) -> std::result::Result<String, GetPinError>,
{
    use crate::request::Request::*;
    match req {
        message @ (SetTimeout(_)
        | SetDesc(_)
        | SetKeyinfo(_)
        | SetPrompt(_)
        | SetTitle(_)
        | SetOk(_)
        | SetCancel(_)
        | SetNotok(_)
        | SetError(_)
        | SetRepeat
        | SetQualitybar(_)
        | SetQualitybarTt(_)
        | SetGenpin(_)
        | SetGenpinTt(_)
        | OptionBool(_)
        | OptionKV(_, _)) => handle_set_req(message, state),
        Message => {
            // Show a message with the value of the last SETDESC
            vec![Response::Ok(None)]
        }
        Confirm => {
            // Show a confirmation dialog with the value of the last SETDESC
            vec![Response::Ok(None)]
        }
        ConfirmOneButton => {
            // Show a confirmation dialog with the value of the last SETDESC, but with only one
            // button
            vec![Response::Ok(None)]
        }
        GetInfoPid => vec![
            Response::D(format!("{}", std::process::id())),
            Response::Ok(None),
        ],
        GetInfoVersion => vec![
            Response::D(crate::build_info::PKG_VERSION.to_string()),
            Response::Ok(None),
        ],
        GetInfoFlavor => vec![Response::D("walker".to_string()), Response::Ok(None)],
        GetInfoTtyinfo => {
            // TODO Get the terminal size etc
            vec![Response::Ok(None)]
        }
        GetPin => get_pin(state).map_or_else(
            |e| match e {
                GetPinError::Command(e) => {
                    vec![Response::Err(e.code, e.stderr)]
                }
                e => vec![Response::Err(1, e.to_string())],
            },
            |pin| vec![Response::D(pin), Response::Ok(None)],
        ),
        Reset => {
            *state = State::default();
            vec![Response::Ok(None)]
        }
        Help => {
            // TODO Print all available commands
            vec![Response::Ok(None)]
        }
        Bye | End | Quit | Cancel | Auth | Nop => vec![Response::Ok(None)],
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

    let mut state = State::default();
    for line in input.lines() {
        let line = line?;
        let req = parse(&line)?;

        let resps = handle_req(req, &mut state, get_pin);
        for resp in resps {
            writeln!(output, "{resp}")?;
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
            GETPIN
            BYE
        "}));
        let mut output = std::io::Cursor::new(vec![]);
        listen(input, &mut output, get_pin).unwrap();
        let output = String::from_utf8(output.into_inner()).unwrap();

        let pid = std::process::id();

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
                    OK
                    D {}
                    OK
                    OK
                    OK
                    OK
                    D 1234
                    OK
                    OK
                "},
                pid
            ),
        );
    }
}
