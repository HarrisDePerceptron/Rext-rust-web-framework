use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::ops::{Deref, DerefMut};

use bson::serde_helpers::{
    deserialize_hex_string_from_object_id, serialize_hex_string_as_object_id,
};

fn convert_opt_id_to_object_id<S>(v: &Option<String>, s: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match v {
        None => Err(serde::ser::Error::custom(
            "Value is none, therefore cannot be converted",
        )),
        Some(v) => serialize_hex_string_as_object_id(&v, s),
    }
}

fn convert_opt_hex_string_to_string<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let val = deserialize_hex_string_from_object_id(deserializer)?;
    Ok(Some(val))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DTO<T> {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "convert_opt_id_to_object_id")]
    #[serde(deserialize_with = "convert_opt_hex_string_to_string")]
    pub id: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,

    #[serde(flatten)]
    pub data: T,
}

impl<T> Deref for DTO<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for DTO<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<T> DTO<T> {
    pub fn new(data: T) -> Self {
        Self {
            id: None,
            created_at: Some(chrono::Utc::now()),
            updated_at: Some(chrono::Utc::now()),
            data,
        }
    }
}
