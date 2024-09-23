use anyhow::Result;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use base64::Engine;
use chacha20poly1305::aead::{Aead, OsRng};
use chacha20poly1305::{AeadCore, ChaCha20Poly1305, KeyInit};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_with::{serde_as, DisplayFromStr};
use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;

const KEY: &[u8] = b"01234567890123456789012345678901";

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
struct Foo {
    name: String,
    #[serde(rename = "date_of_birth")]
    dob: DateTime<Utc>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    tags: Vec<String>,
    #[serde(serialize_with = "base64_encode", deserialize_with = "base64_decode")]
    data: Vec<u8>,
    #[serde_as(as = "DisplayFromStr")]
    sensitive_data: SensitiveData,
    bar: Bar,
    #[serde_as(as = "Vec<DisplayFromStr>")]
    urls: Vec<http::Uri>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum Bar {
    Working(String),
    OnLeave(DateTime<Utc>),
    Terminated,
}

fn base64_encode<S>(data: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let encoded = BASE64_URL_SAFE_NO_PAD.encode(data);
    serializer.serialize_str(&encoded)
}

fn base64_decode<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let encoded = String::deserialize(deserializer)?;
    let decoded = BASE64_URL_SAFE_NO_PAD
        .decode(encoded.as_bytes())
        .map_err(serde::de::Error::custom)?;

    Ok(decoded)
}

#[derive(Debug)]
struct SensitiveData(String);

impl fmt::Display for SensitiveData {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let cipher = ChaCha20Poly1305::new(KEY.into());
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);

        let cipher_text = cipher.encrypt(&nonce, self.0.as_bytes()).unwrap();
        let nonce_cipher_text: Vec<_> = nonce.iter().copied().chain(cipher_text).collect();

        let encoded = BASE64_URL_SAFE_NO_PAD.encode(nonce_cipher_text);

        write!(f, "{}", encoded)
    }
}

impl FromStr for SensitiveData {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let decoded = BASE64_URL_SAFE_NO_PAD.decode(s.as_bytes())?;
        let cipher = ChaCha20Poly1305::new(KEY.into());

        let nonce = decoded[..12].into();

        let decrypted = cipher.decrypt(nonce, &decoded[12..]).unwrap();
        let decrypted = String::from_utf8(decrypted)?;

        Ok(Self(decrypted))
    }
}

impl SensitiveData {
    fn new(data: impl Into<String>) -> Self {
        Self(data.into())
    }
}

fn main() -> Result<()> {
    let bar = Bar::OnLeave(Utc::now());
    let urls = vec![
        "https://io.jrmarcco.cn".parse()?,
        "https://siyuan.jrmarcco.cn".parse()?,
    ];

    let first_foo = Foo {
        name: "jrmarcco".to_string(),
        dob: Utc::now(),
        // tags: vec!["Rust".to_string(), "Golang".to_string()],
        tags: vec![],
        data: vec![0, 1, 2, 3, 4, 5, 6, 7],
        sensitive_data: SensitiveData::new("sensitive_data"),
        bar,
        urls,
    };

    println!("########");
    let json = serde_json::to_string(&first_foo)?;
    println!("{}", json);

    println!("########");

    let second_foo: Foo = serde_json::from_str(&json)?;
    println!("{:?}", second_foo);

    println!("########");

    println!("{:?}", second_foo.urls[0].host());

    Ok(())
}
