//
use std::collections::HashMap;
use super::{
    builder,
    lang::*,
};

pub enum SplitSentences {
    None,
    Default,
    NoNewlines,
}

pub enum Formality {
    PreferMore,
    PreferLess,
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
            Self::PreferMore => "prefer_more",
            Self::PreferLess => "prefer_less",
        }
    }
}

// TextOptions builder
builder! {
    Text {
        @must{
            _target_lang: Language,
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