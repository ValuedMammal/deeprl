//! # deeprl
//!
//! Access the DeepL translation engine through a quick and reliable interface. We aim to provide the full suite of tools DeepL offers.
//! See the [official docs](https://www.deepl.com/en/docs-api) for detailed resources.
//!
//! # Note
//! This crate uses a blocking http client, and as such is only suitable for use in synchronous (blocking) applications.
//! If you intend to use the library functions in an async app, there is a [crate](https://docs.rs/deepl/latest/deepl/) for that.
//!  
//! # Examples
//! Create a new client with a valid API token to access the associated methods. For instance, you may wish to translate a simple text string to some target language.
//! ```
//! use deeprl::{DeepL, Language, TextOptions};
//!
//! let key = std::env::var("DEEPL_API_KEY").unwrap();
//! let dl = DeepL::new(key);
//!
//! // Translate 'good morning' to German
//! let opt = TextOptions::new(Language::DE);
//!
//! let text = vec![
//!     "good morning".to_string(),
//! ];
//!
//! let result = dl.translate(opt, text).unwrap();
//! assert!(!result.translations.is_empty());
//!
//! let translation = &result.translations[0];
//! assert_eq!(translation.text, "Guten Morgen");
//! ```
//!
//! As a helpful sanity check, make sure you're able to return account usage statistics.
//! ```
//! use deeprl::DeepL;
//!
//! let dl = DeepL::new(
//!     std::env::var("DEEPL_API_KEY").unwrap()
//! );
//!
//! let usage = dl.usage().unwrap();
//! assert!(usage.character_limit > 0);
//!
//! let count = usage.character_count;
//! let limit = usage.character_limit;
//! println!("Used: {count}/{limit}");
//! // Used: 42/500000
//! ```
//!
//! `DeepL` also allows translating documents and creating custom glossaries.
//!
//! # License
//! This project is licenced under MIT license.
use reqwest::{header, StatusCode};
use serde::Deserialize;
use thiserror::Error;

pub mod doc;
pub mod glos;
pub mod lang;
pub mod text;
pub use self::doc::{Document, DocumentOptions};
pub use self::glos::Glossary;
pub use self::lang::Language;
pub use self::text::TextOptions;

// Sets the user agent request header value, e.g. 'deeprl/0.1.0'
static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

/// The `DeepL` client struct
pub struct DeepL {
    client: reqwest::blocking::Client,
    url: reqwest::Url,
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

/// API usage & account limits. Currently assumes an individual developer account.
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
                #[must_use]
                pub fn new($($must_field: $must_type,)+) -> Self {
                    Self {
                        $($must_field,)+
                        $($opt_field: None,)+
                    }
                }
                $(
                    #[must_use]
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
    /// - If `key` contains invalid characters causing a failure to create a `reqwest::header::HeaderValue`
    /// - If unable to build a `reqwest::blocking::Client` such as when called from an async runtime
    pub fn new(key: &str) -> Self {
        let base = if key.ends_with(":fx") {
            "https://api-free.deepl.com/v2"
        } else {
            "https://api.deepl.com/v2"
        };
        let url = reqwest::Url::parse(base).unwrap();

        let auth = format!("DeepL-Auth-Key {}", &key);

        let mut auth_val =
            header::HeaderValue::from_str(&auth).expect("failed to create header value");
        auth_val.set_sensitive(true);

        let mut headers = header::HeaderMap::new();
        headers.insert(header::AUTHORIZATION, auth_val);

        let client = reqwest::blocking::Client::builder()
            .user_agent(APP_USER_AGENT)
            .default_headers(headers)
            .build()
            .expect("failed to build request client");

        DeepL { client, url }
    }

    /// GET /usage
    ///
    /// Get account usage
    pub fn usage(&self) -> Result<Usage> {
        let url = format!("{}/usage", self.url);

        let resp = self.client.get(url).send().map_err(|_| Error::Request)?;

        let usage: Usage = resp.json().map_err(|_| Error::Deserialize)?;

        Ok(usage)
    }
}

/// Attempt to parse an error in case of unsuccessful request
fn convert<T>(resp: reqwest::blocking::Response) -> Result<T> {
    let code = resp.status();
    if code.is_client_error() {
        return Err(Error::Client(code.to_string()));
    }
    let resp: ServerError = resp.json().map_err(|_| Error::InvalidResponse)?;

    Err(Error::Server(code, resp.message))
}

#[cfg(test)]
mod test;
