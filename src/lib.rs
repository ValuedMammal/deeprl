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

static APP_USER_AGENT: &'static str = "deeprl/0.1.0";

/// The DeepL client struct
pub struct DeepL {
    client: reqwest::blocking::Client,
    url: String,
}

/// Alias Result<T, E> to Result<T, [`Error`]>
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

#[derive(Debug, Deserialize)]
pub struct ServerError {
    message: String,
}

/// Information about API usage & limits for this account.
#[derive(Debug, Deserialize)]
pub struct Usage {
    /// How many characters were already translated in the current billing period.
    pub character_count: u64,
    /// How many characters can be translated per billing period, based on the account settings.
    pub character_limit: u64,
}

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
            #[doc = "Builder type for `" $name "`"]
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
