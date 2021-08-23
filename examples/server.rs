extern crate geetest;
extern crate tokio;

use geetest::prelude::*;
use std::net::{IpAddr, SocketAddr};

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();

    let captcha_id = env!("GEETEST_CAPTCHA_ID");
    let captcha_secret = env!("GEETEST_CAPTCHA_SECRET");

    let client = Client::new(captcha_id.to_owned(), DigestMod::Md5);
    let server = Server::new(client, captcha_secret.to_owned());
    let addr: SocketAddr = ("127.0.0.1".parse::<IpAddr>().unwrap(), 8000).into();
    log::info!("Running server at http://{}", addr);
    server.run(addr).await?;

    Ok(())
}
