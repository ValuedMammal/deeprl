use serde::Deserialize;

use super::{Error, Result};
use crate::{builder, DeepL, Language};

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
    pub text: String,
}

/// Translation result
#[derive(Debug, Deserialize)]
pub struct TranslateTextResult {
    /// List of translations
    pub translations: Vec<Translation>,
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
            Self::Default => "default",
            Self::More => "more",
            Self::Less => "less",
            Self::PreferMore => "prefer_more",
            Self::PreferLess => "prefer_less",
        }
    }
}

impl std::str::FromStr for Formality {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let fm = match s {
            "more" => Self::More,
            "less" => Self::Less,
            "prefer_more" => Self::PreferMore,
            "prefer_less" => Self::PreferLess,
            _ => Self::Default,
        };

        Ok(fm)
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
    }
}

impl TextOptions {
    /// Creates a map of request params from an instance of `TextOptions`
    fn into_form(self) -> Vec<(&'static str, String)> {
        let mut form = vec![];

        form.push(("target_lang", self.target_lang.to_string()));

        if let Some(src) = self.source_lang {
            form.push(("source_lang", src.as_ref().to_string()));
        }
        if let Some(ss) = self.split_sentences {
            form.push(("split_sentences", ss.as_ref().to_string()));
        }
        if let Some(pf) = self.preserve_formatting {
            if pf {
                form.push(("preserve_formatting", "1".to_string()));
            }
        }
        if let Some(fm) = self.formality {
            form.push(("formality", fm.as_ref().to_string()));
        }
        if let Some(g) = self.glossary_id {
            form.push(("glossary_id", g));
        }
        if let Some(th) = self.tag_handling {
            form.push(("tag_handling", th.as_ref().to_string()));
        }
        if let Some(non) = self.non_splitting_tags {
            form.push(("non_splitting_tags", non));
        }
        if let Some(od) = self.outline_detection {
            if !od {
                form.push(("outline_detection", "0".to_string()));
            }
        }
        if let Some(sp) = self.splitting_tags {
            form.push(("splitting_tags", sp));
        }
        if let Some(ig) = self.ignore_tags {
            form.push(("ignore_tags", ig));
        }

        form
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
    /// let res = dl.translate(
    ///     TextOptions::new(Language::ES),
    ///     vec!["good morning".to_string()],
    /// )
    /// .unwrap();
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
    /// let opt = TextOptions::new(Language::ES)
    ///     .tag_handling(TagHandling::Html)
    ///     .outline_detection(false);
    ///
    /// let res = dl.translate(opt, text).unwrap();
    /// assert!(!res.translations.is_empty());
    /// ```
    /// ## Errors
    ///
    /// If target language and (optionally provided) source language are an invalid pair.
    pub fn translate(&self, opt: TextOptions, text: Vec<String>) -> Result<TranslateTextResult> {
        if text.is_empty() || text[0].is_empty() {
            return Err(Error::Client("empty text parameter".to_string()));
        }
        let url = format!("{}/translate", self.url);
        let mut params = opt.into_form();

        for t in text {
            params.push(("text", t));
        }

        let resp = self
            .post(url)
            .form(&params)
            .send()
            .map_err(|_| Error::InvalidRequest)?;

        if !resp.status().is_success() {
            return super::convert(resp);
        }

        resp.json().map_err(|_| Error::Deserialize)
    }
}
