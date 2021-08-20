use serde_derive::{Deserialize, Serialize};
use std::{net::IpAddr, time::SystemTime};

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
    pub gt: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ClientRegisterRequest {
    #[serde(rename = "t", with = "system_time_as_timestamp")]
    pub timestamp: SystemTime,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ClientValidateRequest {
    #[serde(with = "bool_as_string")]
    pub result: bool,
    pub version: String,
    pub msg: Option<String>,
}

pub type ClientValidateResponse = ClientValidateRequest;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ServerRegisterRequest {
    pub user_id: Option<String>,
    pub client_type: Option<ClientType>,
    #[serde(with = "maybe_ipaddr_as_string")]
    pub ip_address: Option<IpAddr>,
    pub digestmod: DigestMod,
    pub json_format: String,
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
    pub user_id: Option<String>,
    pub client_type: Option<ClientType>,
    pub ip_address: Option<IpAddr>,
    pub digestmod: DigestMod,
    pub json_format: String,
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
    MD5,
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug)]
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

mod system_time_as_timestamp {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::{Duration, SystemTime};

    pub(crate) fn serialize<S: Serializer>(value: &SystemTime, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_u64(
            value
                .duration_since(SystemTime::UNIX_EPOCH)
                .map_or_else(|_| 0, |duration| duration.as_secs()),
        )
    }

    pub(crate) fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<SystemTime, D::Error> {
        u64::deserialize(d).map(|value| SystemTime::UNIX_EPOCH + Duration::from_secs(value))
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
