use crate::lang::*;
use crate::text::*;
use crate::doc::*;
use std::path::PathBuf;

//
use super::*;
use std::{env, str::FromStr};
use std::thread;
use std::time::Duration;

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
fn languages() {
    let dl = DeepL::new(
        env::var("DEEPL_API_KEY").unwrap()
    );

    let source_langs = dl.languages(LanguageType::Source).unwrap();
    let target_langs = dl.languages(LanguageType::Target).unwrap();

    // collect language codes ['EN', ...]
    let mut v: Vec<String> = vec![];
    for lang in source_langs {
        v.push(lang.language);
    }
    for lang in target_langs {
        v.push(lang.language);
    }

    // test we have modeled all available langs
    // need to run with --nocapture ?
    let l: Vec<Language> = v.iter()
        .map(|s| Language::from_str(&s).map_err(|_| dbg!(s)).unwrap())
        .collect();
    
    assert_eq!(v.len(), l.len());
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
fn translate_options() {
    let dl = DeepL::new(
        env::var("DEEPL_API_KEY").unwrap()
    );

    let opt = TextOptions::new(Language::FR)
        .source_lang(Language::EN)
        .split_sentences(SplitSentences::NoNewlines)
        .preserve_formatting(true)
        .formality(text::Formality::PreferLess);

    // newline in the text string
    // lowercase, no punctuation
    // less formal
    let text = vec![
        "you\nare nice".to_string()
    ];
    let expect = "tu es gentille";

    let resp = dl.translate(opt, text).unwrap();
    let translation = &resp.translations[0];
    
    assert_eq!(translation.text, expect);
}

#[test]
fn translate_tags() {
    let dl = DeepL::new(
        env::var("DEEPL_API_KEY").unwrap()
    );

    let xml = r"
<xml>
    <head>
        <title>My English title</title>
    </head>
    <body>
        <p>Do you speak French?</p>
        <p>The red crab</p>
    </body>
</xml>"
    .to_string();
    
    let text = vec![xml];
    let split = "p".to_string();
    let ignore = "title".to_string();
    
    let opt = TextOptions::new(Language::FR)
        .source_lang(Language::EN)
        .tag_handling(TagHandling::Xml)
        .outline_detection(false)
        .splitting_tags(split)
        .ignore_tags(ignore);

    let resp = dl.translate(opt, text).unwrap();
    let text = &resp.translations[0].text;
    assert!(text.contains("<title>My English title</title>"));
    assert!(text.contains("<p>Parlez-vous français ?</p>"));
    assert!(text.contains("<p>Le crabe rouge</p>"));
}

#[test]
fn document() {
    let dl = DeepL::new(
        env::var("DEEPL_API_KEY").unwrap()
    );
    
    // create file
    let text = "good morning".to_string();
    let path = PathBuf::from("gm.txt");
    std::fs::write(&path, text).unwrap();
    
    // test upload
    let lang = Language::DE;
    let opt = DocumentOptions::new(lang, path);
    let doc_resp = dl.document_upload(opt);
    assert!(doc_resp.is_ok());
    let doc = doc_resp.unwrap();
    assert!(!doc.document_id.is_empty());
    
    // test status
    thread::sleep(Duration::from_secs(3));
    let mut doc_status = dl.document_status(&doc).unwrap();
    
    while !doc_status.status.is_done() {
        // try again after 3 sec
        thread::sleep(Duration::from_secs(3));
        doc_status = dl.document_status(&doc).unwrap();
    }
    assert!(doc_status.status.is_done());
    
    // test download
    let out_file = PathBuf::from("de.txt");
    let result = dl.document_download(doc, Some(out_file.clone()));
    assert!(result.is_ok());
    
    let content = std::fs::read_to_string(out_file).unwrap();
    assert_eq!(content, "Guten Morgen");
}
    