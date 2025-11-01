use core::fmt;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{Error, Result};
use crate::{builder, DeepL, Language};

/// Sets whether the translation engine should first split the input into sentences
#[derive(Debug, Copy, Clone, Serialize)]
pub enum SplitSentences {
    /// No splitting
    #[serde(rename = "0")]
    None,
    /// By default, split on punctuation and newlines
    #[serde(rename = "1")]
    Default,
    /// Split on punctuation only
    #[serde(rename = "lowercase")]
    NoNewlines,
}

/// Sets whether the translation engine should lean towards formal or informal language
#[derive(Debug, Copy, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Formality {
    /// Default formality
    Default,
    /// More formal
    More,
    /// Less formal
    Less,
    /// More formal if supported by target language, else default
    PreferMore,
    /// Less formal if supported by target language, else default
    PreferLess,
}

impl fmt::Display for Formality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Default => "default",
            Self::More => "more",
            Self::Less => "less",
            Self::PreferMore => "prefer_more",
            Self::PreferLess => "prefer_less",
        };
        s.fmt(f)
    }
}

/// Sets which kind of tags should be handled
#[derive(Debug, Copy, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
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
    pub text: String,
}

/// Translation result
#[derive(Debug, Deserialize)]
pub struct TranslateTextResult {
    /// List of translations
    pub translations: Vec<Translation>,
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
            non_splitting_tags: Vec<String>,
            outline_detection: bool,
            splitting_tags: Vec<String>,
            ignore_tags: Vec<String>,
            text: Vec<String>,
        };
    }
}

impl DeepL {
    /// POST /translate
    ///
    /// Translate one or more text strings.
    ///
    /// To translate text all we need is to specify a target language and a chunk of text to translate.
    /// In addition, the [`TextOptions`] type exposes a number of methods used to control formatting,
    /// set a desired formality, or tell the server how to handle HTML or XML tags.
    ///
    /// ## Example
    ///
    /// Translate text.
    ///
    /// ```rust,no_run
    /// # use deeprl::*;
    /// # let dl = DeepL::new(&std::env::var("DEEPL_API_KEY").unwrap());
    /// let text = vec!["good morning"];
    /// let options = TextOptions::new(Language::Es)
    ///     .text(vec!["good morning".to_string()]);
    /// let res = dl.translate(options).unwrap();
    /// assert!(!res.translations.is_empty());
    /// ```
    ///
    /// Translate text inside HTML. Note we can skip translation for tags with
    /// with the special "notranslate" attribute.
    ///
    /// ```rust
    /// # use deeprl::*;
    /// # let dl = DeepL::new(&std::env::var("DEEPL_API_KEY").unwrap());
    /// let raw_html = r#"
    /// <h2 class="notranslate">Good morning.</h2>
    /// <p>To be or not to be, that is the question.</p>"#;
    ///
    /// let text = vec![raw_html.to_string()];
    /// let opt = TextOptions::new(Language::Es)
    ///     .tag_handling(TagHandling::Html)
    ///     .outline_detection(false)
    ///     .text(text);
    ///
    /// let res = dl.translate(opt).unwrap();
    /// assert!(!res.translations.is_empty());
    /// ```
    /// ## Errors
    ///
    /// If target language and (optionally provided) source language are an invalid pair.
    pub fn translate(&self, opt: TextOptions) -> Result<TranslateTextResult> {
        let url = format!("{}/translate", self.url);

        let obj = match opt.text.as_ref() {
            None => return Err(Error::Api("text field must not be empty".to_string())),
            Some(text) => {
                if text.is_empty() || text.first().unwrap().is_empty() {
                    return Err(Error::Api("text field must not be empty".to_string()));
                }
                json!(opt)
            }
        };

        let resp = self.post(url).json(&obj).send().map_err(Error::Reqwest)?;

        if !resp.status().is_success() {
            return Err(Error::Response(
                resp.status(),
                resp.text().unwrap_or_default(),
            ));
        }

        Ok(resp.json()?)
    }
}
