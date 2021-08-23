use crate::{
    error::Error,
    models::{
        DigestMod, ServerRegisterRequest, ServerRegisterResponse, ServerValidateRequest, ServerValidateResponse,
        StatusRequest, StatusResponse, UserInfo,
    },
};
use hyper::{
    body::HttpBody,
    client::{Client as HyperClient, HttpConnector},
    header, Body, Method, Request, Response, Uri,
};
use hyper_tls::HttpsConnector;
use serde::{de::DeserializeOwned, Deserialize};
use std::io::Write;

type HttpClient = HyperClient<HttpsConnector<HttpConnector>>;

pub static GEETEST_REGISTER_URL: &str = "https://api.geetest.com/register.php";
pub static GEETEST_VALIDATE_URL: &str = "https://api.geetest.com/validate.php";
pub static GEETEST_STATUS_URL: &str = "https://api.geetest.com/v1/bypass_status.php";

pub static SDK: &str = "geetest rust sdk 1.0";

pub struct Client {
    pub(crate) captcha_id: String,
    pub(crate) digestmod: DigestMod,
    client: HttpClient,
}

impl Client {
    pub fn new(captcha_id: String, digestmod: DigestMod) -> Self {
        Self {
            captcha_id,
            digestmod,
            client: HyperClient::builder().build(HttpsConnector::new()),
        }
    }

    pub async fn bypass_status(&self) -> Result<bool, Error> {
        let request = StatusRequest {
            captcha_id: self.captcha_id.clone(),
        };
        let url: Uri = format!("{}?{}", GEETEST_STATUS_URL, serde_qs::to_string(&request)?).parse()?;
        let mut reply = self.client.get(url).await?;
        let result: StatusResponse = Self::read_body(reply).await?;
        Ok(result.status)
    }

    pub async fn register(&self, user_info: UserInfo) -> Result<String, Error> {
        let request = ServerRegisterRequest {
            user_info,
            digestmod: self.digestmod,
            json_format: 1,
            sdk: SDK.to_owned(),
            captcha_id: self.captcha_id.clone(),
        };
        let url: Uri = format!("{}?{}", GEETEST_REGISTER_URL, serde_qs::to_string(&request)?).parse()?;

        log::debug!("geetest request: {}", url);
        let mut reply = self.client.get(url).await?;
        let result: ServerRegisterResponse = Self::read_body(reply).await?;
        Ok(result.challenge)
    }

    pub async fn validate(
        &self,
        seccode: String,
        challenge: String,
        user_info: UserInfo,
    ) -> Result<Option<String>, Error> {
        let body = ServerValidateRequest {
            user_info,
            digestmod: self.digestmod,
            json_format: 1,
            sdk: SDK.to_owned(),
            captcha_id: self.captcha_id.clone(),
            seccode,
            challenge,
        };

        let request = Request::builder()
            .method(Method::POST)
            .uri(GEETEST_VALIDATE_URL)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(serde_json::to_vec(&body)?))?;

        let mut reply = self.client.request(request).await?;
        let result: ServerValidateResponse = Self::read_body(reply).await?;
        Ok(result.seccode)
    }

    async fn read_body<T: DeserializeOwned>(mut reply: Response<Body>) -> Result<T, Error> {
        let mut json = Vec::with_capacity(1024);
        while let Some(chunk) = reply.body_mut().data().await {
            let chunk = chunk?;
            json.write_all(chunk.as_ref())?;
        }
        Ok(serde_json::from_slice(&json)?)
    }
}
