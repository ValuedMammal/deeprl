//
use std::collections::HashMap;
use super::{
    builder,
    convert,
    DeepL,
    Error,
    lang::*,
};
use serde::Deserialize;
use reqwest::Method;

pub enum SplitSentences {
    None,
    Default,
    NoNewlines,
}

pub enum Formality {
    More,
    Less,
    PreferMore,
    PreferLess,
}

#[derive(Debug, Deserialize)]
pub struct Translation {
    pub detected_source_language: String,
    pub text: String
}

#[derive(Debug, Deserialize)]
pub struct TranslateTextResult {
    pub translations: Vec<Translation>
}

impl AsRef<str> for SplitSentences {
    fn as_ref(&self) -> &str {
        match self {
            Self::None => "0",
            Self::Default => "1",
            Self::NoNewlines => "nonewlines",
        }
    }
}

impl AsRef<str> for Formality {
    fn as_ref(&self) -> &str {
        match self {
            Self::More => "more",
            Self::Less => "less",
            Self::PreferMore => "prefer_more",
            Self::PreferLess => "prefer_less",
        }
    }
}

// TextOptions builder
builder! {
    Text {
        @must{
            target_lang: Language,
        };
        @optional{
            source_lang: Language,
            split_sentences: SplitSentences,
            preserve_formatting: bool,
            formality: Formality,
            glossary_id: String,
        };
    } -> Self;
}

impl TextOptions {
   pub fn to_form(&self) -> HashMap<&'static str, String> {
        let mut form = HashMap::new();
        form.insert("target_lang", self.target_lang.to_string());

        if let Some(src) = &self.source_lang {
            form.insert("source_lang", src.as_ref().to_string());
        }
        if let Some(ss) = &self.split_sentences {
            form.insert("split_sentences", ss.as_ref().to_string());
        }
        if let Some(pf) = &self.preserve_formatting {
            form.insert("preserve_formatting", pf.to_string());
        }
        if let Some(fm) = &self.formality {
            form.insert("formality", fm.as_ref().to_string());
        }
        if let Some(g) = &self.glossary_id {
            form.insert("glossary_id", g.to_owned());
        }

        form
   }
}

impl DeepL {
    pub fn translate(&self, opt: TextOptions, text: Vec<String>) -> Result<TranslateTextResult, Error> {
        let url = format!("{}/translate", self.url);
        let mut params = opt.to_form();

        for t in text {
            params.insert("text", t);
        }

        let req = self.client.request(Method::POST, url)
            .form(&params);

        let resp = req.send()
            .map_err(|_| Error::Request)?;
        

        if !resp.status().is_server_error() && !resp.status().is_client_error() {
        let result: TranslateTextResult = resp.json()
            .map_err(|_| Error::Deserialize)?;
            
            return Ok(result)
        } else {
            convert(resp)
        }
    }
}