pub mod pinentry;
pub mod request;
pub mod response;

pub mod build_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}
