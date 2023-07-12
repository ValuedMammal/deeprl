use serde::Deserialize;
use std::collections::HashMap;

use super::*;
use crate::lang::Language;

/// Sets whether the translation engine should first split the input into sentences
#[derive(Copy, Clone)]
pub enum SplitSentences {
    /// No splitting
    None,
    /// By default, split on punctuation and newlines
    Default,
    /// Split on punctuation only
    NoNewlines,
}

/// Sets whether the translation engine should lean towards formal or informal language
#[derive(Copy, Clone)]
pub enum Formality {
    /// More formal
    More,
    /// Less formal
    Less,
    /// More formal if supported by target language, else default
    PreferMore,
    /// Less formal if supported by target language, else default
    PreferLess,
}

/// Sets which kind of tags should be handled
#[derive(Copy, Clone)]
pub enum TagHandling {
    /// Enable XML tag handling
    Xml,
    /// Enable HTML tag handling
    Html,
}

/// An individual translation
#[derive(Debug, Deserialize)]
pub struct Translation {
    /// Detected source language
    pub detected_source_language: String,
    /// Translated text
    pub text: String
}

/// Translation result
#[derive(Debug, Deserialize)]
pub struct TranslateTextResult {
    /// List of translations
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

impl AsRef<str> for TagHandling {
    fn as_ref(&self) -> &str {
        match self {
            Self::Xml => "xml",
            Self::Html => "html",
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
            tag_handling: TagHandling,
            non_splitting_tags: String,
            outline_detection: bool,
            splitting_tags: String,
            ignore_tags: String,
        };
    } -> Self;
}

impl TextOptions {
   /// Creates a map of request params from an instance of `TextOptions`
   fn to_form(self) -> HashMap<&'static str, String> {
        let mut form = HashMap::new();
        
        form.insert("target_lang", self.target_lang.to_string());

        if let Some(src) = self.source_lang {
            form.insert("source_lang", src.as_ref().to_string());
        }
        if let Some(ss) = self.split_sentences {
            form.insert("split_sentences", ss.as_ref().to_string());
        }
        if let Some(pf) = self.preserve_formatting {
            if pf {
                form.insert("preserve_formatting", "1".to_string());
            }
        }
        if let Some(fm) = self.formality {
            form.insert("formality", fm.as_ref().to_string());
        }
        if let Some(g) = self.glossary_id {
            form.insert("glossary_id", g);
        }
        if let Some(th) = self.tag_handling {
            form.insert("tag_handling", th.as_ref().to_string());
        }
        if let Some(non) = self.non_splitting_tags {
            form.insert("non_splitting_tags", non);
        }
        if let Some(od) = self.outline_detection {
            if !od {
                form.insert("outline_detection", "0".to_string());
            }
        }
        if let Some(sp) = self.splitting_tags {
            form.insert("non_splitting_tags", sp);
        }
        if let Some(ig) = self.ignore_tags {
            form.insert("non_splitting_tags", ig);
        }

        form
   }
}

impl DeepL {
    /// POST /translate
    /// 
    /// Translate one or more text strings
    pub fn translate(&self, opt: TextOptions, text: Vec<String>) -> Result<TranslateTextResult> {
        if text.is_empty() || text[0].is_empty() {
            return Err(Error::Client("empty text parameter".to_string()))
        }
        let url = format!("{}/translate", self.url);
        let mut params = opt.to_form();

        for t in text {
            params.insert("text", t);
        }

        let resp = self.client.post(url)
            .form(&params)
            .send()
            .map_err(|_| Error::Deserialize)?;

        if !resp.status().is_success() {
            return super::convert(resp)
        }
        
        resp.json()
            .map_err(|_| Error::Deserialize)
    }
}