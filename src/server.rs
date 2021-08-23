use crate::{
    error::Error,
    models::{ClientRegisterResponse, ClientValidateRequest, ClientValidateResponse, DigestMod, UserInfo},
};
use futures_util::{FutureExt, TryFutureExt};
use hyper::{
    body::{Bytes, HttpBody},
    header,
    service::{make_service_fn, Service},
    Body, Method, Request, Response, StatusCode,
};
use rand::seq::SliceRandom;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    convert::Infallible,
    future::Future,
    net::SocketAddr,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

#[derive(Clone)]
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

    pub async fn run(self, addr: SocketAddr) -> Result<(), hyper::Error> {
        hyper::Server::bind(&addr)
            .serve(make_service_fn(move |_| {
                let svc = self.clone();
                async { Ok::<_, Infallible>(svc) }
            }))
            .await
    }

    async fn handle_register(
        client: Arc<crate::client::Client>,
        captcha_secret: String,
    ) -> Result<ClientRegisterResponse, Error> {
        log::debug!("handle register");

        let bypass_status = client.bypass_status().await?;

        if bypass_status {
            let origin_challenge = client.register(UserInfo::default()).await?;
            log::debug!("origin challenge: {}", origin_challenge);

            let challenge = match client.digestmod {
                #[cfg(feature = "digest-md5")]
                DigestMod::Md5 => {
                    let mut hasher = md5::Context::new();
                    hasher.consume(origin_challenge);
                    hasher.consume(&captcha_secret);
                    let digest = hasher.compute();
                    format!("{:x}", digest)
                },
                #[cfg(feature = "digest-sha256")]
                DigestMod::Sha256 => {
                    use sha2::Digest;
                    let mut hasher = sha2::Sha256::new();
                    hasher.update(origin_challenge);
                    hasher.update(&captcha_secret);
                    let digest = hasher.finalize();
                    format!("{:x}", digest)
                },
                #[cfg(feature = "digest-hmac-sha256")]
                DigestMod::HmacSha256 => {
                    use hmac::{Mac, NewMac};
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
        } else {
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
        }
    }

    async fn handle_validate(
        client: Arc<crate::client::Client>,
        req: ClientValidateRequest,
    ) -> Result<ClientValidateResponse, Error> {
        let is_valid_request =
            !(req.challenge.trim().is_empty() || req.validate.trim().is_empty() || req.seccode.trim().is_empty());

        if !is_valid_request {
            return Ok(ClientValidateResponse::error("Invalid request fields"));
        }

        let bypass_status = client.bypass_status().await?;

        if bypass_status {
            let seccode = client.validate(req.seccode, req.challenge, UserInfo::default()).await?;

            if let Some(_) = seccode {
                Ok(ClientValidateResponse::success())
            } else {
                Ok(ClientValidateResponse::error("Invalid security code"))
            }
        } else {
            Ok(ClientValidateResponse::success())
        }
    }

    async fn convert_reply<T: Serialize>(reply: T) -> Result<Response<Body>, Error> {
        let body = serde_json::to_vec(&reply)?;
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body))
            .map_err(Into::into)
    }

    async fn read_body(mut body: Body) -> Result<Vec<u8>, Error> {
        body.data()
            .map(|data| Ok::<_, Error>(data.unwrap_or_else(|| Ok(Bytes::new()))?.to_vec()))
            .await
    }

    async fn parse_body<T: DeserializeOwned>(body: Vec<u8>) -> Result<T, Error> {
        serde_qs::from_bytes(&body).map_err(Into::into)
    }

    async fn bad_request() -> Result<Response<Body>, Error> {
        Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::empty())
            .map_err(Into::into)
    }
}

impl Service<Request<Body>> for Server {
    type Response = Response<Body>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let route = (req.method(), req.uri().path());
        log::debug!("Route {:?}", route);
        match route {
            (&Method::GET, "/register") => {
                let (client, captcha_secret) = (self.client.clone(), self.captcha_secret.clone());
                Box::pin(Self::handle_register(client, captcha_secret).and_then(Self::convert_reply))
            },
            (&Method::POST, "/validate") => {
                let client = self.client.clone();
                Box::pin(
                    Self::read_body(req.into_body())
                        .and_then(Self::parse_body)
                        .and_then(|body| Self::handle_validate(client, body))
                        .and_then(Self::convert_reply),
                )
            },
            _ => Box::pin(Self::bad_request()),
        }
    }
}
