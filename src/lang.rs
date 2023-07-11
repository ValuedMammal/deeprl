//
use std::fmt;
use std::str::FromStr;
use super::Error;
use super::Result;
use super::DeepL;
use super::convert;
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq)]
pub enum LanguageType {
    Source,
    Target,
}

#[derive(Debug, Deserialize)]
pub struct LangInfo {
    /// Language code (EN, DE, etc.)
    pub language: String,
    /// English name of the language, e.g. `English (America)`
    pub name: String,
    /// 
    pub supports_formality: bool,
}

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
      /// English Great Britain (target language)
      ENGB,
      /// English USA (target language)
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
      /// Portuguese Brazil (target language)
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

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lang = match s {
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
    pub fn languages(&self, lang_type: LanguageType) -> Result<Vec<LangInfo>> {
        let url = format!("{}/languages", self.url);

        let kind = match lang_type {
            LanguageType::Source => "source",
            LanguageType::Target => "target",
        };

        // get
        // query "type"
        let q = vec![
            ("type", kind)
        ];

        let resp = self.client.get(url)
            .query(&q)
            .send()
            .map_err(|_| Error::Request)?;

        if !resp.status().is_server_error() && !resp.status().is_client_error() {
            let result: Vec<LangInfo> = resp.json()
                .map_err(|_| Error::Deserialize)?;
            return Ok(result)
        } else {
            convert(resp)
        }
    }
}


