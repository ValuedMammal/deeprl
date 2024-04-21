use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_formality: Option<bool>,
}

impl Default for LanguageInfo {
    /// Provides serde with a default value for `supports_formality`, since
    /// the field is only returned for target lang (not source).
    /// [deepl-openapi docs](https://docs.rs/deepl-openapi/2.7.1/src/deepl_openapi/models/get_languages_200_response_inner.rs.html)
    //
    // note: can we just derive Default?
    fn default() -> Self {
        Self {
            language: String::default(),
            name: String::default(),
            supports_formality: None,
        }
    }
}

/// Language variants.
///
/// Please note that while many [`Language`] variants are interchangeable as both source and
/// target languages, there are exceptions. For example when translating text and documents,
/// the following may only be used as source languages:
/// - `EN`
/// - `PT`
///
/// and the following may only be used as target languages (representing regional variants):
/// - `ENUS`
/// - `ENGB`
/// - `PTBR`
/// - `PTPT`
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Language {
    /// Bulgarian
    BG,
    /// Czech
    CS,
    /// Danish
    DA,
    /// German
    DE,
    /// Greek
    EL,
    /// English (source language)
    EN,
    /// English British (target language)
    ENGB,
    /// English American (target language)
    ENUS,
    /// Spanish
    ES,
    /// Estonian
    ET,
    /// Finish
    FI,
    /// French
    FR,
    /// Hungarian
    HU,
    /// Indonesian
    ID,
    /// Italian
    IT,
    /// Japanese
    JA,
    /// Korean
    KO,
    /// Lithuanian
    LT,
    /// Latvian
    LV,
    /// Norwegian
    NB,
    /// Dutch
    NL,
    /// Polish
    PL,
    /// Portuguese (source language)
    PT,
    /// Portuguese Brazilian (target language)
    PTBR,
    /// Portuguese European (target language)
    PTPT,
    /// Romanian
    RO,
    /// Russian
    RU,
    /// Slovak
    SK,
    /// Slovenian
    SL,
    /// Swedish
    SV,
    /// Turkish
    TR,
    /// Ukranian
    UK,
    /// Chinese simplified
    ZH,
}

impl FromStr for Language {
    type Err = Error;

    /// # Errors
    ///
    /// If a [`Language`] cannot be parsed from the input `s`
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lang = match s.to_uppercase().as_str() {
            "BG" => Language::BG,
            "CS" => Language::CS,
            "DA" => Language::DA,
            "DE" => Language::DE,
            "EL" => Language::EL,
            "EN" => Language::EN,
            "EN-GB" => Language::ENGB,
            "EN-US" => Language::ENUS,
            "ES" => Language::ES,
            "ET" => Language::ET,
            "FI" => Language::FI,
            "FR" => Language::FR,
            "HU" => Language::HU,
            "ID" => Language::ID,
            "IT" => Language::IT,
            "JA" => Language::JA,
            "KO" => Language::KO,
            "LT" => Language::LT,
            "LV" => Language::LV,
            "NB" => Language::NB,
            "NL" => Language::NL,
            "PL" => Language::PL,
            "PT" => Language::PT,
            "PT-BR" => Language::PTBR,
            "PT-PT" => Language::PTPT,
            "RO" => Language::RO,
            "RU" => Language::RU,
            "SK" => Language::SK,
            "SL" => Language::SL,
            "SV" => Language::SV,
            "TR" => Language::TR,
            "UK" => Language::UK,
            "ZH" => Language::ZH,
            _ => return Err(Error::InvalidLanguage),
        };

        Ok(lang)
    }
}

impl AsRef<str> for Language {
    fn as_ref(&self) -> &str {
        match self {
            Self::BG => "BG",
            Self::CS => "CS",
            Self::DA => "DA",
            Self::DE => "DE",
            Self::EL => "EL",
            Self::EN => "EN",
            Self::ENGB => "EN-GB",
            Self::ENUS => "EN-US",
            Self::ES => "ES",
            Self::ET => "ET",
            Self::FI => "FI",
            Self::FR => "FR",
            Self::HU => "HU",
            Self::ID => "ID",
            Self::IT => "IT",
            Self::JA => "JA",
            Self::KO => "KO",
            Self::LT => "LT",
            Self::LV => "LV",
            Self::NB => "NB",
            Self::NL => "NL",
            Self::PL => "PL",
            Self::PT => "PT",
            Self::PTBR => "PT-BR",
            Self::PTPT => "PT-PT",
            Self::RO => "RO",
            Self::RU => "RU",
            Self::SK => "SK",
            Self::SL => "SL",
            Self::SV => "SV",
            Self::TR => "TR",
            Self::UK => "UK",
            Self::ZH => "ZH",
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

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
    /// println!("{}", language.language); // BG
    /// println!("{}", language.name); // Bulgarian
    ///```
    pub fn languages(&self, lang_type: LanguageType) -> Result<Vec<LanguageInfo>> {
        let url = format!("{}/languages", self.url);

        let kind = match lang_type {
            LanguageType::Source => "source",
            LanguageType::Target => "target",
        };

        // get, query "type"
        let q = vec![("type", kind)];

        let resp = self
            .get(url)
            .query(&q)
            .send()
            .map_err(|_| Error::InvalidRequest)?;

        if !resp.status().is_success() {
            return super::convert(resp);
        }

        resp.json().map_err(|_| Error::Deserialize)
    }
}
