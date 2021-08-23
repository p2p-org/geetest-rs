# GeeTest Rust SDK

This is a Rust library for [GeeTest][1] captcha integration.

It includes two parts: client and server.

## Usage

Add this to `Cargo.toml`:

```toml
[dependencies]
geetest = "0.1"
tokio = { version = "1", features = ["full"] }
```

Client usage:

```rust
use geetest::{Client, DigestMod, Error, UserInfo, ClientType};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = Client::new("geetest-captcha-id", DigestMod::Md5);

    // ByPass status
    println!("Status: {}", client.bypass_status().await?);

    let user_info = UserInfo::new()
        .user_id("my-user-id")
        .client_type(ClientType::Web);

    // Register new captcha
    client.register(user_info).await?;

    // Validate captcha
    println!("Captcha info:", client.validate("security-code", "challenge", user_info).await?);

    Ok(())
}
```

Server usage:

```rust
use geetest::{Server, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let server = Server::new("geetest-captcha-id", "geetest-captcha-secret");
    let addr = "127.0.0.1:8000".parse()?;
    server.run(&addr).await?;
    Ok(())
}
```

See also [`examples/server.rs`][2] for full working example.

## Copyright

This is a product of [P2P Validator][3].

[1]: https://www.geetest.com/en/
[2]: examples/server.rs
[3]: https://p2p.org/