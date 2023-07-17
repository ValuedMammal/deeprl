use std::{env, path::PathBuf, str::FromStr, thread, time::Duration};

use super::*;
use crate::{doc::*, glos::*, lang::*, text::*};

#[test]
fn configure() {
    // test set user client + app agent
    let app = "my-app/1.2.3";
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(21))
        .build()
        .unwrap();

    let key = env::var("DEEPL_API_KEY").unwrap();
    let mut dl = DeepL::new(&key);
    dl.client(client);
    dl.app_info(app.to_owned());

    let url = format!("{}/usage", dl.url);
    let req = dl.get(url).build().unwrap();
    let headers = req.headers();

    assert_eq!(headers.get("User-Agent").unwrap(), header::HeaderValue::from_static(app));
    let auth = format!("DeepL-Auth-Key {}", key);
    assert_eq!(headers.get("Authorization").unwrap(), header::HeaderValue::from_str(&auth).unwrap());
}

#[test]
fn usage() {
    let dl = DeepL::new(&env::var("DEEPL_API_KEY").unwrap());
    
    let resp = dl.usage();
    assert!(resp.is_ok());

    let usage = resp.unwrap();
    assert!(usage.character_limit > 0);
}

#[test]
fn languages() {
    let dl = DeepL::new(&env::var("DEEPL_API_KEY").unwrap());

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
    let dl = DeepL::new(&env::var("DEEPL_API_KEY").unwrap());

    let src = Language::EN;

    let text = vec!["good morning".to_string()];
    let opt = TextOptions::new(Language::DE).source_lang(src);

    let resp = dl.translate(opt, text);
    assert!(resp.is_ok());
    let result = resp.unwrap();
    let translation = &result.translations[0];
    assert!(!translation.text.is_empty());
}

#[test]
fn translate_options() {
    let dl = DeepL::new(&env::var("DEEPL_API_KEY").unwrap());

    let opt = TextOptions::new(Language::FR)
        .source_lang(Language::EN)
        .split_sentences(SplitSentences::NoNewlines)
        .preserve_formatting(true)
        .formality(text::Formality::PreferLess);

    // newline in the text string
    // lowercase, no punctuation
    // less formal
    let text = vec!["you\nare nice".to_string()];
    let expect = "tu es gentille";

    let resp = dl.translate(opt, text).unwrap();
    let translation = &resp.translations[0];

    assert_eq!(translation.text, expect);
}

#[test]
fn translate_tags() {
    let dl = DeepL::new(&env::var("DEEPL_API_KEY").unwrap());

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
    let dl = DeepL::new(&env::var("DEEPL_API_KEY").unwrap());

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

#[test]
fn glossary_pairs() {
    // get supported glossary language pairs
    let dl = DeepL::new(&env::var("DEEPL_API_KEY").unwrap());

    let result = dl.glossary_languages().unwrap();
    let pairs = result.supported_languages;
    assert!(!pairs.is_empty());
    let first = &pairs[0];
    assert!(!first.source_lang.is_empty());
}

#[test]
fn glossaries() {
    // list available glossaries
    let dl = DeepL::new(&env::var("DEEPL_API_KEY").unwrap());

    let result = dl.glossaries().unwrap();
    let glossaries = result.glossaries;
    if !glossaries.is_empty() {
        let glos = &glossaries[0];
        assert!(glos.entry_count > 0);
    }
}

#[test]
fn glossary_all() {
    let dl = DeepL::new(&env::var("DEEPL_API_KEY").unwrap());

    // test create
    let name = "my_glossary".to_string();
    let src = Language::EN;
    let trg = Language::IT;
    let entries = "goodbye,arrivederci".to_string();
    let fmt = GlossaryEntriesFormat::Csv;

    let glossary = dl.glossary_new(name, src, trg, entries, fmt).unwrap();
    assert_eq!(glossary.entry_count, 1);

    // test fetch entries
    let glos_id = glossary.glossary_id;
    let resp = dl.glossary_entries(&glos_id);
    assert!(resp.is_ok());
    let entry = resp.unwrap();
    assert!(entry.contains("goodbye\tarrivederci"));

    // test translate with glossary
    let opts = TextOptions::new(Language::IT)
        .source_lang(Language::EN)
        .preserve_formatting(true)
        .glossary_id(glos_id.clone());

    let text = vec!["goodbye".to_string()];
    let result = dl.translate(opts, text).unwrap();
    let translations = result.translations;
    assert_eq!(translations[0].text, "arrivederci");

    // test delete
    let _: () = dl.glossary_del(&glos_id).unwrap();
    thread::sleep(Duration::from_secs(1));

    // deleted glossary id is 404
    let code = StatusCode::NOT_FOUND;
    let expect = Error::Client(code.to_string());
    let resp = dl.glossary_info(&glos_id);
    assert_eq!(resp.unwrap_err(), expect);
}
