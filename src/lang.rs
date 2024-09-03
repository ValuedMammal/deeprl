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
#[derive(Copy, Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum Language {
    /// Bulgarian
    Bg,
    /// Czech
    Cs,
    /// Danish
    Da,
    /// German
    De,
    /// Greek
    El,
    /// English (source language)
    En,
    /// English British (target language)
    EnGb,
    /// English American (target language)
    EnUs,
    /// Spanish
    Es,
    /// Estonian
    Et,
    /// Finish
    Fi,
    /// French
    Fr,
    /// Hungarian
    Hu,
    /// Indonesian
    Id,
    /// Italian
    It,
    /// Japanese
    Ja,
    /// Korean
    Ko,
    /// Lithuanian
    Lt,
    /// Latvian
    Lv,
    /// Norwegian
    Nb,
    /// Dutch
    Nl,
    /// Polish
    Pl,
    /// Portuguese (source language)
    Pt,
    /// Portuguese Brazilian (target language)
    PtBr,
    /// Portuguese European (target language)
    PtPt,
    /// Romanian
    Ro,
    /// Russian
    Ru,
    /// Slovak
    Sk,
    /// Slovenian
    Sl,
    /// Swedish
    Sv,
    /// Turkish
    Tr,
    /// Ukranian
    Uk,
    /// Chinese simplified
    Zh,
    /// Chinese simplified
    ZhHans,
}

impl FromStr for Language {
    type Err = Error;

    /// # Errors
    ///
    /// If a [`Language`] cannot be parsed from the input `s`
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lang = match s.to_uppercase().as_str() {
            "BG" => Language::Bg,
            "CS" => Language::Cs,
            "DA" => Language::Da,
            "DE" => Language::De,
            "EL" => Language::El,
            "EN" => Language::En,
            "EN-GB" => Language::EnGb,
            "EN-US" => Language::EnUs,
            "ES" => Language::Es,
            "ET" => Language::Et,
            "FI" => Language::Fi,
            "FR" => Language::Fr,
            "HU" => Language::Hu,
            "ID" => Language::Id,
            "IT" => Language::It,
            "JA" => Language::Ja,
            "KO" => Language::Ko,
            "LT" => Language::Lt,
            "LV" => Language::Lv,
            "NB" => Language::Nb,
            "NL" => Language::Nl,
            "PL" => Language::Pl,
            "PT" => Language::Pt,
            "PT-BR" => Language::PtBr,
            "PT-PT" => Language::PtPt,
            "RO" => Language::Ro,
            "RU" => Language::Ru,
            "SK" => Language::Sk,
            "SL" => Language::Sl,
            "SV" => Language::Sv,
            "TR" => Language::Tr,
            "UK" => Language::Uk,
            "ZH" => Language::Zh,
            "ZH-HANS" => Language::ZhHans,
            _ => return Err(Error::InvalidLanguage),
        };

        Ok(lang)
    }
}

impl AsRef<str> for Language {
    fn as_ref(&self) -> &str {
        match self {
            Self::Bg => "BG",
            Self::Cs => "CS",
            Self::Da => "DA",
            Self::De => "DE",
            Self::El => "EL",
            Self::En => "EN",
            Self::EnGb => "EN-GB",
            Self::EnUs => "EN-US",
            Self::Es => "ES",
            Self::Et => "ET",
            Self::Fi => "FI",
            Self::Fr => "FR",
            Self::Hu => "HU",
            Self::Id => "ID",
            Self::It => "IT",
            Self::Ja => "JA",
            Self::Ko => "KO",
            Self::Lt => "LT",
            Self::Lv => "LV",
            Self::Nb => "NB",
            Self::Nl => "NL",
            Self::Pl => "PL",
            Self::Pt => "PT",
            Self::PtBr => "PT-BR",
            Self::PtPt => "PT-PT",
            Self::Ro => "RO",
            Self::Ru => "RU",
            Self::Sk => "SK",
            Self::Sl => "SL",
            Self::Sv => "SV",
            Self::Tr => "TR",
            Self::Uk => "UK",
            Self::Zh => "ZH",
            Self::ZhHans => "ZH-HANS",
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

        let resp = self.get(url).query(&q).send().map_err(Error::Reqwest)?;

        if !resp.status().is_success() {
            return super::convert_error(resp);
        }

        resp.json().map_err(|_| Error::Deserialize)
    }
}
