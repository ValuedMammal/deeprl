use super::*;
use std::{env, str::FromStr, thread, time::Duration};

const KEY: &'static str = env!("DEEPL_API_KEY");

#[test]
fn configure() {
    // test set user client + app agent
    let app = "my-app/1.2.3";
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(21))
        .build()
        .unwrap();

    let mut dl = DeepL::new(KEY);
    dl.client(client);
    dl.set_app_info(app.to_owned());

    let url = format!("{}/usage", dl.url);
    let req = dl.get(url).build().unwrap();
    let headers = req.headers();

    assert_eq!(
        headers.get("User-Agent").unwrap(),
        header::HeaderValue::from_static(app)
    );
    let auth = format!("DeepL-Auth-Key {}", KEY);
    assert_eq!(
        headers.get("Authorization").unwrap(),
        header::HeaderValue::from_str(&auth).unwrap()
    );
}

#[test]
fn usage() {
    let dl = DeepL::new(KEY);

    let resp = dl.usage();
    assert!(resp.is_ok());

    let usage = resp.unwrap();
    assert!(usage.character_limit > 0);
}

#[test]
fn languages() {
    let dl = DeepL::new(KEY);

    // fetch source langs
    let langs = dl.languages(LanguageType::Source).unwrap();

    // fetch target langs, filtering duplicates
    let target_langs = dl.languages(LanguageType::Target).unwrap();
    let target_langs: Vec<LanguageInfo> = target_langs
        .into_iter()
        .filter(|l| {
            let code = &l.language;
            for src_lang in &langs {
                if code == &src_lang.language {
                    return false;
                }
            }
            true
        })
        .collect();

    // test we have modeled all available langs
    // should run with --show-output
    let _: Vec<Language> = langs
        .iter()
        .chain(target_langs.iter())
        .map(|info| {
            let code = &info.language;
            let name = &info.name;
            Language::from_str(code)
                .map_err(|_| println!("Failed to convert lang: {code} {name}"))
                .unwrap()
        })
        .collect();
}

#[test]
fn translate_text() {
    let dl = DeepL::new(KEY);

    let src = Language::En;
    let text = vec!["good morning".to_string()];
    let opt = TextOptions::new(Language::De).source_lang(src).text(text);

    let res = dl.translate(opt).unwrap();
    let translation = &res.translations[0];
    assert!(!translation.text.is_empty());
}

#[test]
fn translate_error_empty_text() {
    let dl = DeepL::new(KEY);

    let cases: Vec<Option<Vec<String>>> = vec![None, Some(vec![]), Some(vec!["".to_string()])];
    for text in cases {
        let mut opt = TextOptions::new(Language::De);
        if let Some(text) = text {
            opt = opt.text(text);
        }
        let res = dl.translate(opt);
        assert!(matches!(
            res,
            Err(Error::Api(s))
            if s.contains("empty")
        ));
    }
}

#[test]
fn translate_options() {
    let dl = DeepL::new(KEY);

    let text = vec!["you\nare nice".to_string()];

    let opt = TextOptions::new(Language::Fr)
        .source_lang(Language::En)
        .split_sentences(SplitSentences::None)
        .preserve_formatting(true)
        .formality(Formality::PreferLess)
        .text(text);

    // newline in the text string
    // lowercase, no punctuation
    // less formal
    let expect = "tu es gentille";

    let resp = dl.translate(opt).unwrap();
    let translation = &resp.translations[0];

    assert_eq!(translation.text, expect);
}

#[test]
fn translate_tags() {
    let dl = DeepL::new(KEY);

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
    let split = vec!["p".to_string()];
    let ignore = vec!["title".to_string()];

    let opt = TextOptions::new(Language::Fr)
        .source_lang(Language::En)
        .tag_handling(TagHandling::Xml)
        .outline_detection(false)
        .splitting_tags(split)
        .ignore_tags(ignore)
        .text(text);

    let resp = dl.translate(opt).unwrap();
    let text = &resp.translations[0].text;
    assert!(text.contains("<title>My English title</title>"));
    assert!(text.contains("<p>Parlez-vous français ?</p>"));
    assert!(text.contains("<p>Le crabe rouge</p>"));
}

#[test]
#[ignore]
fn document() {
    let dl = DeepL::new(KEY);

    // create file
    let text = "good morning".to_string();
    let tempdir = tempfile::tempdir().unwrap();
    let path = tempdir.path().join("gm.txt");
    std::fs::write(&path, text).unwrap();

    // test upload
    let lang = Language::De;
    let opt = DocumentOptions::new(lang, path);
    let doc_resp = dl.document_upload(opt);
    assert!(doc_resp.is_ok());
    let doc = doc_resp.unwrap();
    assert!(!doc.document_id.is_empty());

    // test status
    let mut delay = Duration::from_millis(64);
    while !dl.document_status(&doc).unwrap().is_done() {
        thread::sleep(delay);
        delay *= 2;
    }

    // test download
    let path = tempdir.path().join("de.txt");
    let result = dl.document_download(doc, Some(path.clone()));
    assert!(result.is_ok());

    let content = std::fs::read_to_string(path).unwrap();
    assert!(!content.is_empty());
}

#[test]
fn glossary_pairs() {
    // get supported glossary language pairs
    let dl = DeepL::new(KEY);

    let result = dl.glossary_languages().unwrap();
    let pairs = result.supported_languages;
    assert!(!pairs.is_empty());
    let first = &pairs[0];
    assert!(!first.source_lang.is_empty());
}

#[test]
fn glossaries() {
    // list available glossaries
    let dl = DeepL::new(KEY);

    let result = dl.glossaries().unwrap();
    let glossaries = result.glossaries;
    if !glossaries.is_empty() {
        let glos = &glossaries[0];
        assert!(glos.entry_count > 0);
    }
}

#[test]
#[ignore]
fn glossary_all() {
    // create csv file with two glossary entries
    let dl = DeepL::new(KEY);

    // test create glossary
    let name = "my_glossary".to_string();
    let src = Language::En;
    let trg = Language::It;
    let entries = "hello,ciao\ngoodbye,ciao".to_string();
    let fmt = GlossaryEntriesFormat::Csv;

    let glossary = dl.glossary_new(name, src, trg, entries, fmt).unwrap();
    assert_eq!(glossary.entry_count, 2);
    assert!(glossary.ready);

    // test fetch entries
    let glos_id = glossary.glossary_id;
    let resp = dl.glossary_entries(&glos_id);
    assert!(resp.is_ok());
    let entries = resp.unwrap();
    assert_eq!(entries.len(), 2);

    // test translate with glossary
    let text = vec!["goodbye".to_string()];
    let opts = TextOptions::new(Language::It)
        .source_lang(Language::En)
        .preserve_formatting(true)
        .glossary_id(glos_id.clone())
        .text(text);

    let result = dl.translate(opts).unwrap();
    let translations = result.translations;
    assert_eq!(translations[0].text, "ciao");

    // test delete
    dl.glossary_delete(&glos_id).unwrap();
    thread::sleep(Duration::from_secs(1));

    // deleted glossary id is 404
    let resp = dl.glossary_info(&glos_id);
    assert!(matches!(
        resp,
        Err(Error::Response(code, ..)) if code == StatusCode::NOT_FOUND,
    ));
}

#[test]
fn test_error() {
    let dl = DeepL::new(KEY);

    // translate using an invalid match
    let res = dl.translate(
        TextOptions::new(
            // bad target lang
            Language::Pt,
        )
        // bad source lang
        .source_lang(Language::EnGb)
        .text(vec!["good morning".to_string()]),
    );
    assert!(res.is_err());
}

// Doc tests
#[test]
fn doc_text_options() {
    let dl = DeepL::new(KEY);

    let text = vec!["you are nice \nthe red crab".to_string()];
    let target_lang = Language::Fr;

    let opt = TextOptions::new(target_lang)
        .split_sentences(SplitSentences::None)
        .preserve_formatting(true)
        .formality(Formality::PreferLess)
        .text(text);

    let translations = dl.translate(opt).unwrap().translations;

    assert_eq!(&translations[0].text, "tu es gentil le crabe rouge");
}

#[test]
fn doc_text_html() {
    let dl = DeepL::new(KEY);

    let html = r#"
<h2 class="notranslate">good morning</h2>
<p>good morning</p>"#
        .to_string();

    let text = vec![html];
    let opt = TextOptions::new(Language::Es)
        .tag_handling(TagHandling::Html)
        .outline_detection(false)
        .text(text);
    let trans = dl.translate(opt).unwrap().translations;

    let text = &trans[0].text;
    // dbg!(text);
    assert!(text.contains("good morning"));
    assert!(text.contains("buenos días"));
}
