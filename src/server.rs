use crate::{
    client::SDK,
    error::Error,
    models::{
        ClientRegisterRequest, ClientRegisterResponse, ClientValidateRequest, ClientValidateResponse, DigestMod,
        UserInfo,
    },
};
use futures_util::TryFutureExt;
use hmac::{Mac, NewMac};
use hyper::{service::Service, Method, Request, Response, StatusCode};
use rand::seq::SliceRandom;
use serde::{de::DeserializeOwned, Serialize};
use sha2::Digest;
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

pub struct Server {
    client: Arc<crate::client::Client>,
    captcha_secret: String,
}

impl Server {
    pub fn new(client: crate::client::Client, captcha_secret: String) -> Self {
        Self {
            client: Arc::new(client),
            captcha_secret,
        }
    }

    async fn handle_register(
        client: Arc<crate::client::Client>,
        captcha_secret: String,
        _req: ClientRegisterRequest,
    ) -> Result<ClientRegisterResponse, Error> {
        let origin_challenge = client.register(UserInfo::default()).await?;

        if origin_challenge.is_empty() || origin_challenge == "0" {
            let challenge = "abcdefghijklmnopqrstuvwxyz0123456789"
                .as_bytes()
                .choose_multiple(&mut rand::thread_rng(), 32)
                .copied()
                .map(|b| b as char)
                .collect();
            Ok(ClientRegisterResponse {
                success: false,
                captcha_id: client.captcha_id.clone(),
                new_captcha: true,
                challenge,
            })
        } else {
            let challenge = match client.digestmod {
                DigestMod::Md5 => {
                    let mut hasher = md5::Context::new();
                    hasher.consume(origin_challenge);
                    hasher.consume(&captcha_secret);
                    let digest = hasher.compute();
                    format!("{:x}", digest)
                },
                DigestMod::Sha256 => {
                    let mut hasher = sha2::Sha256::new();
                    hasher.update(origin_challenge);
                    hasher.update(&captcha_secret);
                    let digest = hasher.finalize();
                    format!("{:x}", digest)
                },
                DigestMod::HmacSha256 => {
                    let mut hasher = hmac::Hmac::<sha2::Sha256>::new_from_slice(origin_challenge.as_bytes())
                        .expect("HMAC can take key of any size");
                    hasher.update(captcha_secret.as_bytes());
                    let digest = hasher.finalize();
                    format!("{:x}", digest.into_bytes())
                },
            };

            Ok(ClientRegisterResponse {
                success: true,
                new_captcha: true,
                challenge,
                captcha_id: client.captcha_id.clone(),
            })
        }
    }

    async fn handle_validate(
        client: Arc<crate::client::Client>,
        req: ClientValidateRequest,
    ) -> Result<ClientValidateResponse, Error> {
        let seccode = client.validate(req.seccode, req.challenge, UserInfo::default()).await?;

        if let Some(_) = seccode {
            Ok(ClientValidateResponse {
                result: true,
                version: SDK.to_owned(),
                msg: None,
            })
        } else {
            Ok(ClientValidateResponse {
                result: false,
                version: SDK.to_owned(),
                msg: None,
            })
        }
    }

    async fn convert_reply<T: Serialize>(reply: T) -> Result<Response<Vec<u8>>, Error> {
        let body = serde_json::to_vec(&reply)?;
        Response::builder()
            .status(StatusCode::OK)
            .body(body)
            .map_err(Into::into)
    }

    async fn parse_body<T: DeserializeOwned>(body: Vec<u8>) -> Result<T, Error> {
        serde_qs::from_bytes(&body).map_err(Into::into)
    }

    async fn bad_request() -> Result<Response<Vec<u8>>, Error> {
        Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Vec::new())
            .map_err(Into::into)
    }
}

impl Service<Request<Vec<u8>>> for Server {
    type Response = Response<Vec<u8>>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Vec<u8>>) -> Self::Future {
        match (req.method(), req.uri().path()) {
            (&Method::GET, "/register") => {
                let (client, captcha_secret) = (self.client.clone(), self.captcha_secret.clone());
                Box::pin(
                    Self::parse_body(req.uri().query().unwrap_or("").as_bytes().to_vec())
                        .and_then(move |body| Self::handle_register(client, captcha_secret, body))
                        .and_then(Self::convert_reply),
                )
            },
            (&Method::POST, "/validate") => {
                let client = self.client.clone();
                Box::pin(
                    Self::parse_body(req.into_body())
                        .and_then(|body| Self::handle_validate(client, body))
                        .and_then(Self::convert_reply),
                )
            },
            _ => Box::pin(Self::bad_request()),
        }
    }
}
