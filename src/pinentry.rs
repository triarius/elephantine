use crate::{
    request::{parse, Request},
    response::Response,
};
use color_eyre::Result;
use std::{
    collections::HashMap,
    io::{BufRead, Write},
};

#[derive(Debug, Default, PartialEq, Eq)]
struct State {
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

fn handle_req(req: Request, state: &mut State) -> Result<Vec<Response>> {
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
        | OptionKV(_, _)) => Ok(handle_set_req(message, state)),
        Message => {
            // Show a message with the value of the last SETDESC
            Ok(vec![Response::Ok(None)])
        }
        Confirm => {
            // Show a confirmation dialog with the value of the last SETDESC
            Ok(vec![Response::Ok(None)])
        }
        ConfirmOneButton => {
            // Show a confirmation dialog with the value of the last SETDESC, but with only one
            // button
            Ok(vec![Response::Ok(None)])
        }
        GetInfoPid => Ok(vec![
            Response::D(format!("{}", std::process::id())),
            Response::Ok(None),
        ]),
        GetInfoVersion => Ok(vec![
            Response::D(crate::build_info::PKG_VERSION.to_string()),
            Response::Ok(None),
        ]),
        GetInfoFlavor => Ok(vec![Response::D("walker".to_string()), Response::Ok(None)]),
        GetInfoTtyinfo => {
            // TODO Get the terminal size etc
            Ok(vec![Response::Ok(None)])
        }
        GetPin => {
            use std::process::Command;
            let walker = Command::new("walker")
                .arg("--password")
                .output()
                .map_err(color_eyre::Report::new)
                .unwrap();
            if walker.status.success() {
                Ok(vec![
                    Response::D(String::from_utf8(walker.stdout)?),
                    Response::Ok(None),
                ])
            } else {
                Ok(vec![Response::Err(
                    walker.status.code().unwrap_or(1),
                    String::from_utf8(walker.stderr)?,
                )])
            }
        }
        Reset => {
            *state = State::default();
            Ok(vec![Response::Ok(None)])
        }
        Help => {
            // TODO Print all available commands
            Ok(vec![Response::Ok(None)])
        }
        Bye | End | Quit | Cancel | Auth | Nop => Ok(vec![Response::Ok(None)]),
    }
}

/// Listen for requests on the given input and respond on the given output
/// # Errors
/// If there is an error reading from or parsing the input or writing to the output
pub fn listen(input: impl BufRead, output: &mut impl Write) -> Result<()> {
    writeln!(
        output,
        "{}",
        Response::Ok(Some("Greetings from Elephantine".to_string())),
    )?;

    let mut state = State::default();
    for line in input.lines() {
        let line = line?;
        let req = parse(&line)?;

        let resps = handle_req(req, &mut state);
        for resp in resps? {
            writeln!(output, "{resp}")?;
        }
    }
    Ok(())
}
