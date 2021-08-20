use crate::{
    error::Error,
    models::{DigestMod, ServerRegisterRequest, ServerRegisterResponse, ServerValidateRequest, ServerValidateResponse},
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

pub struct Client {
    client: HttpClient,
}

impl Client {
    pub async fn register(&self) -> Result<ServerRegisterResponse, Error> {
        let request = ServerRegisterRequest {
            user_id: None,
            client_type: None,
            ip_address: None,
            digestmod: DigestMod::MD5,
            json_format: "".to_string(),
            sdk: "".to_string(),
            captcha_id: "".to_string(),
        };
        let url: Uri = format!("{}?{}", GEETEST_REGISTER_URL, serde_qs::to_string(&request)?).parse()?;

        let mut reply = self.client.get(url).await?;

        let mut json = Vec::with_capacity(1024);
        while let Some(chunk) = reply.body_mut().data().await {
            let chunk = chunk?;
            json.write_all(chunk.as_ref())?;
        }

        let result = serde_json::from_slice(&json)?;
        Ok(result)
    }

    pub async fn validate(&self) -> Result<ServerValidateResponse, Error> {
        let body = ServerValidateRequest {
            user_id: None,
            client_type: None,
            ip_address: None,
            digestmod: DigestMod::MD5,
            json_format: "".to_string(),
            sdk: "".to_string(),
            captcha_id: "".to_string(),
            seccode: "".to_string(),
            challenge: "".to_string(),
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

        let result = serde_json::from_slice(&json)?;
        Ok(result)
    }
}
