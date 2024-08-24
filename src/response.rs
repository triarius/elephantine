#[derive(Debug, PartialEq, Eq)]
pub enum Response {
    Ok,
    Err(String),
    D(String),
    S(String, String),
    Inquire(String, String),
}
