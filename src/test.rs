use crate::{lang::Language, text::TextOptions};

//
use super::*;
use std::env;

#[test]
fn usage() {
    let dl = DeepL::new(
        env::var("DEEPL_API_KEY").unwrap()
    );

    let resp = dl.usage();
    assert!(resp.is_ok());

    let usage = resp.unwrap();
    assert!(usage.character_limit > 0);
}

#[test]
fn translate_text() {
    let dl = DeepL::new(
        env::var("DEEPL_API_KEY").unwrap()
    );

    let src = Language::EN;

    let text = vec!["good morning".to_string()];
    let opt = TextOptions::new(Language::DE)
        .source_lang(src);

    let resp = dl.translate(opt, text);
    assert!(resp.is_ok());
    let result = resp.unwrap();
    let translation = &result.translations[0];
    assert!(!translation.text.is_empty());
}

#[test]
fn languages() {
    todo!()
}
