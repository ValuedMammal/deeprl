//! # deeprl
//!
//! Access the DeepL translation engine through a quick and reliable interface. We aim to provide the full suite of tools DeepL offers.
//! See the [official docs](https://www.deepl.com/en/docs-api) for detailed resources.
//!
//! ## Note
//! This crate uses a blocking http client, and as such is only suitable for use in synchronous (blocking) applications.
//! If you intend to use the library functions in an async app, there is a [crate](https://docs.rs/deepl/latest/deepl/) for that.
//!  
//! ## Examples
//! Create a new client with a valid API token to access the associated methods. For instance, you may wish to translate a simple text string to some target language.
//! ```
//! use deeprl::{DeepL, Language, TextOptions};
//!
//! let key = std::env::var("DEEPL_API_KEY").unwrap();
//! let dl = DeepL::new(&key);
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
//!     &std::env::var("DEEPL_API_KEY").unwrap()
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
//! [`DeepL`] also allows translating documents and creating custom glossaries.
//!
//! # License
//! This project is licenced under MIT license.

#![warn(missing_docs)]

use serde::Deserialize;
use std::io;

use reqwest::header;
use reqwest::StatusCode;
use thiserror::Error;

mod doc;
mod glos;
mod lang;
mod text;

pub use {
    doc::{DocState, Document, DocumentOptions, DocumentStatus},
    glos::{
        GlossariesResult, Glossary, GlossaryEntriesFormat, GlossaryLanguagePair,
        GlossaryLanguagePairsResult,
    },
    lang::{Language, LanguageInfo, LanguageType},
    text::{Formality, SplitSentences, TagHandling, TextOptions, TranslateTextResult, Translation},
};

// Sets the user agent request header value, e.g. 'deeprl/0.1.0'
static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

/// The `DeepL` client struct
pub struct DeepL {
    client: reqwest::blocking::Client,
    url: reqwest::Url,
    user_agent: Option<String>,
    auth: String,
}

/// Crate Result type
type Result<T, E = Error> = std::result::Result<T, E>;

/// Crate error variants
#[derive(Debug, Error)]
pub enum Error {
    /// General client side error
    #[error("{0}")]
    Client(String),
    /// Error sent from the server
    #[error("{0}: {1}")]
    Server(StatusCode, String),
    /// Error deserializing response
    #[error("error deserializing response")]
    Deserialize,
    /// Invalid request
    #[error("invalid request {0}")]
    Reqwest(reqwest::Error),
    /// Io
    #[error("{0}")]
    Io(io::Error),
    /// Invalid language
    #[error("invalid language")]
    InvalidLanguage,
    /// Invalid response
    #[error("invalid response")]
    InvalidResponse,
}

/// Server error type
#[derive(Debug, Deserialize)]
struct ServerError {
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
        }
    ) => {
        use paste::paste;

        paste! {
            #[doc = "Options for `" [<$name>] "` translation"]
            pub struct [<$name Options>] {
                $($must_field: $must_type,)+
                $($opt_field: Option<$opt_type>,)+
            }

            impl [<$name Options>] {
                #[must_use]
                #[doc = "Construct a new `" [<$name Options>] "`"]
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
    pub fn new(key: &str) -> Self {
        let base = if key.ends_with(":fx") {
            "https://api-free.deepl.com/v2"
        } else {
            "https://api.deepl.com/v2"
        };

        DeepL {
            client: reqwest::blocking::Client::new(),
            url: reqwest::Url::parse(base).unwrap(),
            user_agent: None,
            auth: format!("DeepL-Auth-Key {}", &key),
        }
    }

    /// Sets a user-defined HTTP client
    pub fn client(&mut self, client: reqwest::blocking::Client) -> &mut Self {
        self.client = client;
        self
    }

    /// Sets app name and version to be used in the User-Agent header, e.g. "my-app/1.2.3"
    pub fn set_app_info(&mut self, app: String) -> &mut Self {
        self.user_agent = Some(app);
        self
    }

    /// Calls the underlying client POST method
    fn post<U>(&self, url: U) -> reqwest::blocking::RequestBuilder
    where
        U: reqwest::IntoUrl,
    {
        self.client.post(url).headers(self.default_headers())
    }

    /// Calls the underlying client GET method
    fn get<U>(&self, url: U) -> reqwest::blocking::RequestBuilder
    where
        U: reqwest::IntoUrl,
    {
        self.client.get(url).headers(self.default_headers())
    }

    /// Calls the underlying client DELETE method
    fn delete<U>(&self, url: U) -> reqwest::blocking::RequestBuilder
    where
        U: reqwest::IntoUrl,
    {
        self.client.delete(url).headers(self.default_headers())
    }

    /// Construct default headers used in the request (User-Agent, Authorization)
    fn default_headers(&self) -> header::HeaderMap {
        // user agent
        let app = if let Some(s) = &self.user_agent {
            s.clone()
        } else {
            APP_USER_AGENT.to_string()
        };
        let mut map = reqwest::header::HeaderMap::new();
        map.insert(
            header::USER_AGENT,
            header::HeaderValue::from_str(&app).unwrap(),
        );

        // auth
        map.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&self.auth).unwrap(),
        );
        map
    }

    /// GET /usage
    ///
    /// Get account usage
    pub fn usage(&self) -> Result<Usage> {
        let url = format!("{}/usage", self.url);
        let resp = self.get(url).send().map_err(Error::Reqwest)?;
        let usage: Usage = resp.json().map_err(|_| Error::Deserialize)?;

        Ok(usage)
    }
}

/// Attempt to parse an error in case of unsuccessful request
fn convert<T>(resp: reqwest::blocking::Response) -> Result<T> {
    let code = resp.status();
    let resp: ServerError = resp.json().map_err(|_| Error::InvalidResponse)?;
    Err(Error::Server(code, resp.message))
}

#[cfg(test)]
mod test;
