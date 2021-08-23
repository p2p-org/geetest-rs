pub mod client;
pub mod error;
pub mod models;
pub mod server;

pub mod prelude {
    pub use crate::{
        client::Client,
        error::Error,
        models::{DigestMod, UserInfo},
        server::Server,
    };
}
