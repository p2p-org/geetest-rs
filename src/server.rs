use crate::{
    error::Error,
    models::{ClientRegisterRequest, ClientRegisterResponse, ClientValidateRequest, ClientValidateResponse},
};
use futures_util::TryFutureExt;
use hyper::{service::Service, Method, Request, Response, StatusCode};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

pub struct Server {
    client: crate::client::Client,
}

impl Server {
    async fn handle_register(req: ClientRegisterRequest) -> Result<ClientRegisterResponse, Error> {
        Ok(ClientRegisterResponse {
            success: false,
            new_captcha: false,
            challenge: "".to_string(),
            gt: "".to_string(),
        })
    }

    async fn handle_validate(req: ClientValidateRequest) -> Result<ClientValidateResponse, Error> {
        Ok(ClientValidateResponse {
            result: false,
            version: "".to_string(),
            msg: None,
        })
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
            (&Method::GET, "/register") => Box::pin(
                Self::parse_body(req.uri().query().unwrap_or("").as_bytes().to_vec())
                    .and_then(Self::handle_register)
                    .and_then(Self::convert_reply),
            ),
            (&Method::POST, "/validate") => Box::pin(
                Self::parse_body(req.into_body())
                    .and_then(Self::handle_register)
                    .and_then(Self::convert_reply),
            ),
            _ => Box::pin(Self::bad_request()),
        }
    }
}
