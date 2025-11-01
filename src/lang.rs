use serde::{Deserialize, Serialize};

use super::{Error, Result};
use crate::DeepL;

/// Language type. Note: this is currently only used when fetching language meta information.
#[derive(Copy, Clone, Debug)]
pub enum LanguageType {
    /// Source language
    Source,
    /// Target language
    Target,
}

/// Information about a supported language
#[derive(Debug, Deserialize, Serialize)]
pub struct LanguageInfo {
    /// Language code (EN, DE, etc.)
    pub language: String,
    /// Name of the language in English
    pub name: String,
    /// Denotes formality support in case of target language
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_formality: Option<bool>,
}

/// Generate all languages.
macro_rules! impl_language {
    ( $($lang:ident, $upper:literal, $name:literal,)* ) => {
        /// Language variants.
        ///
        /// Please note that while many [`Language`] variants are interchangeable as both source and
        /// target languages, there are exceptions. For example when translating text and documents,
        /// the following may only be used as source languages:
        /// - [`En`](Self::En)
        /// - [`Pt`](Self::Pt)
        ///
        /// and the following may only be used as target languages (representing regional variants):
        /// - [`EnUs`](Self::EnUs)
        /// - [`EnGb`](Self::EnGb)
        /// - [`PtBr`](Self::PtBr)
        /// - [`PtPt`](Self::PtPt)
        #[derive(Copy, Clone, Debug, PartialEq, serde::Serialize)]
        #[serde(rename_all = "SCREAMING-KEBAB-CASE")]
        pub enum Language {
            $(
                #[doc = $name]
                $lang,
            )*
        }

        impl core::str::FromStr for Language {
            type Err = crate::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.to_uppercase().as_str() {
                    $(
                       $upper => Ok(Self::$lang),
                    )*
                    _ => Err(crate::lang::ParseLanguageError(s.to_string()))?,
                }
            }
        }

        #[allow(unused)]
        impl Language {
            /// Get this [`Language`] as a `&str`.
            fn as_str(&self) -> &str {
                match self {
                    $(
                        Self::$lang => $upper,
                    )*
                }
            }
        }

        impl AsRef<str> for Language {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl core::fmt::Display for Language {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}", self.as_ref())
            }
        }
    }
}

#[rustfmt::skip]
impl_language!(
    Ar, "AR", " Arabic",
    Bg, "BG", " Bulgarian",
    Cs, "CS", " Czech",
    Da, "DA", " Danish",
    De, "DE", " German",
    El, "EL", " Greek",
    En, "EN", " English",
    EnGb, "EN-GB", " English Britain",
    EnUs, "EN-US", " English US",
    Es, "ES", " Spanish",
    Es419, "ES-419", " Spanish Latin America",
    Et, "ET", " Estonian",
    Fi, "FI", " Finish",
    Fr, "FR", " French",
    Hu, "HU", " Hungarian",
    Id, "ID", " Indonesian",
    It, "IT", " Italian",
    Ja, "JA", " Japanese",
    Ko, "KO", " Korean",
    Lt, "LT", " Lithuanian",
    Lv, "LV", " Latvian",
    Nb, "NB", " Norwegian",
    Nl, "NL", " Dutch",
    Pl, "PL", " Polish",
    Pt, "PT", " Portuguese",
    PtBr, "PT-BR", " Portuguese Brazil",
    PtPt, "PT-PT", " Portuguese Europe",
    Ro, "RO", " Romanian",
    Ru, "RU", " Russian",
    Sk, "SK", " Slovak",
    Sl, "SL", " Slovenian",
    Sv, "SV", " Swedish",
    Tr, "TR", " Turkish",
    Uk, "UK", " Ukranian",
    Zh, "ZH", " Chinese",
    ZhHans, "ZH-HANS", " Chinese simplified",
    ZhHant, "ZH-HANT", " Chinese traditional",
);

/// Error attempting to parse a [`Language`] from a string.
#[derive(Debug)]
pub struct ParseLanguageError(String);

impl core::fmt::Display for ParseLanguageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid language: {}", self.0)
    }
}

impl std::error::Error for ParseLanguageError {}

impl DeepL {
    /// GET /languages
    ///
    /// Get information on supported languages.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # use deeprl::{DeepL, LanguageType};
    /// # let dl = DeepL::new(&std::env::var("DEEPL_API_KEY").unwrap());
    /// let source_langs = dl.languages(LanguageType::Source).unwrap();
    /// assert!(!source_langs.is_empty());
    ///
    /// let language = &source_langs[0];
    /// println!("{}", language.language);
    /// println!("{}", language.name);
    ///```
    pub fn languages(&self, lang_type: LanguageType) -> Result<Vec<LanguageInfo>> {
        let url = format!("{}/languages", self.url);

        let kind = match lang_type {
            LanguageType::Source => "source",
            LanguageType::Target => "target",
        };

        // get, query "type"
        let q = vec![("type", kind)];

        let resp = self.get(url).query(&q).send().map_err(Error::Reqwest)?;

        if !resp.status().is_success() {
            return super::convert_error(resp);
        }

        resp.json().map_err(|_| Error::Deserialize)
    }
}
