use crate::{
    error::Error,
    models::{
        DigestMod, ServerRegisterRequest, ServerRegisterResponse, ServerValidateRequest, ServerValidateResponse,
        UserInfo,
    },
};
use hyper::{
    body::HttpBody,
    client::{Client as HyperClient, HttpConnector},
    header, Body, Method, Request, Uri,
};
use std::io::Write;

type HttpClient = HyperClient<HttpConnector>;

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
            client: HttpClient::new(),
        }
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

        let mut reply = self.client.get(url).await?;

        let mut json = Vec::with_capacity(1024);
        while let Some(chunk) = reply.body_mut().data().await {
            let chunk = chunk?;
            json.write_all(chunk.as_ref())?;
        }

        let result: ServerRegisterResponse = serde_json::from_slice(&json)?;
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

        let mut json = Vec::with_capacity(1024);
        while let Some(chunk) = reply.body_mut().data().await {
            let chunk = chunk?;
            json.write_all(chunk.as_ref())?;
        }

        let result: ServerValidateResponse = serde_json::from_slice(&json)?;
        Ok(result.seccode)
    }
}
