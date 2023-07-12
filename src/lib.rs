//! # deeprl
//! 
//! My docs
//! 
//! # Usage
//!
//! # License
//!
use reqwest::{
    header,
    StatusCode,
};
use serde::Deserialize;
use thiserror::Error;

pub mod lang;
pub mod text;
pub mod doc;
pub mod glos;

// Sets the user agent request header value, e.g. 'deeprl/0.1.0'
static APP_USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
);

/// The `DeepL` client struct
/// 
/// Note: This type uses a blocking http client, and as such is only suitable for use in synchronous applications.
pub struct DeepL {
    client: reqwest::blocking::Client,
    url: String,
}

/// Crate Result type
type Result<T, E = Error> = std::result::Result<T, E>;

/// Crate error variants
#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("{0}")]
    Client(String),
    #[error("{0} {1}")]
    Server(StatusCode, String),
    #[error("error deserializing response")]
    Deserialize,
    #[error("error sending request")]
    Request,
    #[error("invalid language")]
    InvalidLanguage,
    #[error("invalid response")]
    InvalidResponse,
}

/// Server error type
#[derive(Debug, Deserialize)]
pub struct ServerError {
    message: String,
}

/// Represents API usage & account limits.
/// Currently assumes an individual developer account.
#[derive(Debug, Deserialize)]
pub struct Usage {
    /// Characters translated so far in the current billing period
    pub character_count: u64,
    /// Current maximum number of characters that can be translated per billing period
    pub character_limit: u64,
}

/// Self-implementing type builder
#[macro_export]
macro_rules! builder {
    (
        $name:ident {
            @must{
                $($must_field:ident: $must_type:ty,)+
            };
            @optional{
                $($opt_field:ident: $opt_type:ty,)+
            };
        } -> $ret:ty;
    ) => {
        use paste::paste;

        paste! {
            #[doc = "Builder type for `" [<$name Options>] "`"]
            pub struct [<$name Options>] {
                $($must_field: $must_type,)+
                $($opt_field: Option<$opt_type>,)+
            }

            impl [<$name Options>] {
                pub fn new($($must_field: $must_type,)+) -> Self {
                    Self {
                        $($must_field,)+
                        $($opt_field: None,)+
                    }
                }
                $(
                    #[doc = "Setter for `" $opt_field "`"]
                    pub fn $opt_field(mut self, $opt_field: $opt_type) -> Self {
                        self.$opt_field = Some($opt_field);
                        self
                    }
                )+
            }
        }
    };
}

impl DeepL {
    /// Create a new instance of `DeepL` from an API key.
    ///
    /// # Panics
    /// - If unable to build a `reqwest::blocking::Client`
    /// - If `key` contains invalid characters causing a failure to create a `reqwest::header::HeaderValue`
    /// 
    pub fn new(key: String) -> Self {
        let url = "https://api-free.deepl.com/v2".to_owned();
        let auth = format!("DeepL-Auth-Key {}", &key);

        let mut auth_val = header::HeaderValue::from_str(&auth)
            .expect("failed to create header value");
        auth_val.set_sensitive(true);

        let mut headers = header::HeaderMap::new();
        headers.insert(header::AUTHORIZATION, auth_val);
        
        let client = reqwest::blocking::Client::builder()
            .user_agent(APP_USER_AGENT)
            .default_headers(headers)
            .build()
            .expect("failed to build req client");
        
        DeepL { client, url }
    }

    /// GET /usage
    /// 
    /// Get account usage
    pub fn usage(&self) -> Result<Usage> {
        let url = format!("{}/usage", self.url);

        let resp = self.client.get(url)
            .send()
            .map_err(|_| Error::Request)?;

        let usage: Usage = resp.json()
            .map_err(|_| Error::Deserialize)?;

        Ok(usage)
    }
}

/// Attempt to parse an error in case of unsuccessful request
fn convert<T>(resp: reqwest::blocking::Response) -> Result<T> {
    let code = resp.status();
    if code.is_client_error() {
        return Err(Error::Client(code.to_string()))
    }
    let resp: ServerError = resp.json()
        .map_err(|_| Error::InvalidResponse)?;

    Err(Error::Server(code, resp.message))
}

#[cfg(test)]

mod test;
