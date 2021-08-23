use serde_derive::{Deserialize, Serialize};
use std::net::IpAddr;

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct UserInfo {
    pub user_id: Option<String>,
    pub client_type: Option<ClientType>,
    #[serde(with = "maybe_ipaddr_as_string")]
    pub ip_address: Option<IpAddr>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct StatusRequest {
    #[serde(rename = "gt")]
    pub captcha_id: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct StatusResponse {
    #[serde(with = "bool_as_string")]
    pub status: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ClientRegisterResponse {
    #[serde(with = "bool_as_u8")]
    pub success: bool,
    pub new_captcha: bool,
    pub challenge: String,
    #[serde(rename = "gt")]
    pub captcha_id: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ClientValidateResponse {
    #[serde(with = "bool_as_string")]
    pub result: bool,
    pub version: String,
    pub msg: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ClientValidateRequest {
    #[serde(rename = "geetest_challenge")]
    pub challenge: String,
    #[serde(rename = "geetest_validate")]
    pub validate: String,
    #[serde(rename = "geetest_seccode")]
    pub seccode: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ServerRegisterRequest {
    #[serde(flatten)]
    pub user_info: UserInfo,
    pub digestmod: DigestMod,
    pub json_format: u32,
    pub sdk: String,
    #[serde(rename = "gt")]
    pub captcha_id: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ServerRegisterResponse {
    pub challenge: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ServerValidateRequest {
    #[serde(flatten)]
    pub user_info: UserInfo,
    pub digestmod: DigestMod,
    pub json_format: u32,
    pub sdk: String,
    #[serde(rename = "captchaid")]
    pub captcha_id: String,
    pub seccode: String,
    pub challenge: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ServerValidateResponse {
    #[serde(with = "maybe_seccode")]
    pub seccode: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug)]
#[serde(rename_all = "lowercase")]
pub enum DigestMod {
    Md5,
    Sha256,
    #[serde(rename = "hmac-sha256")]
    HmacSha256,
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ClientType {
    Web,
    #[serde(rename = "h5")]
    Mobile,
    Native,
    Unknown,
}

mod bool_as_u8 {
    use serde::{Deserialize, Deserializer, Serializer};

    pub(crate) fn serialize<S: Serializer>(value: &bool, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_u8(*value as u8)
    }

    pub(crate) fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<bool, D::Error> {
        u8::deserialize(d).map(|value| if value == 0 { false } else { true })
    }
}

mod bool_as_string {
    use serde::{de::Unexpected, Deserialize, Deserializer, Serializer};

    pub(crate) fn serialize<S: Serializer>(value: &bool, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(if *value { "success" } else { "fail" })
    }

    pub(crate) fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<bool, D::Error> {
        String::deserialize(d).and_then(|value| match &*value {
            "success" => Ok(true),
            "fail" => Ok(false),
            unexpected => Err(serde::de::Error::invalid_value(
                Unexpected::Str(unexpected),
                &"fail or success",
            )),
        })
    }
}

mod maybe_seccode {
    use serde::{Deserialize, Deserializer, Serializer};

    pub(crate) fn serialize<S: Serializer>(value: &Option<String>, s: S) -> Result<S::Ok, S::Error> {
        match value {
            None => s.serialize_str("false"),
            Some(value) => s.serialize_str(&**value),
        }
    }

    pub(crate) fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
        String::deserialize(d).map(|value| if value == "false" { None } else { Some(value) })
    }
}

mod maybe_ipaddr_as_string {
    use serde::{de::Unexpected, Deserialize, Deserializer, Serializer};
    use std::net::IpAddr;

    pub(crate) fn serialize<S: Serializer>(value: &Option<IpAddr>, s: S) -> Result<S::Ok, S::Error> {
        match value {
            Some(value) => s.serialize_some(&value.to_string()),
            None => s.serialize_none(),
        }
    }

    pub(crate) fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<IpAddr>, D::Error> {
        Option::<String>::deserialize(d).and_then(|value| {
            value
                .map(|value| {
                    value
                        .parse()
                        .map_err(|_| serde::de::Error::invalid_value(Unexpected::Str(&value), &"IP address"))
                })
                .transpose()
        })
    }
}
