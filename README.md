# deeprl

Access the DeepL translation engine through a quick and reliable interface. We aim to provide the full suite of tools DeepL offers. See the [DeepL API docs](https://www.deepl.com/en/docs-api) for detailed resources.

### Note:
[This crate](https://docs.rs/deeprl/) uses a blocking http client, and as such is only suitable for use in synchronous (blocking) applications. If you intend to use the library functions in an async app, there is a [crate](https://docs.rs/deepl/latest/deepl/) for that.

### Quickstart
Create a new client with a valid API token to access the associated methods. For instance, you may wish to translate a simple text string to some target language.

```rust
use deeprl::{DeepL, Language, TextOptions};

let key = std::env::var("DEEPL_API_KEY").unwrap();
let dl = DeepL::new(&key);

// Translate 'good morning' to German
let text = vec![
    "good morning".to_string(),
];

let opt = TextOptions::new(Language::De).text(text);

let result = dl.translate(opt).unwrap();
assert!(!result.translations.is_empty());

let translation = &result.translations[0];
assert_eq!(translation.text, "Guten Morgen");
```

As a helpful sanity check, make sure you're able to return account usage statistics.

```rust
use deeprl::DeepL;

let dl = DeepL::new(
    &std::env::var("DEEPL_API_KEY").unwrap()
);

let usage = dl.usage().unwrap();
assert!(usage.character_limit > 0);

let count = usage.character_count;
let limit = usage.character_limit;
println!("Used: {count}/{limit}");
// Used: 42/500000
```

### Walkthrough
Contents:
- [Configuration](#configuration)
    - [Errors](#errors)
- [Get available languages](#get-languages)
- [Translate text options](#translate-text-options)
    - [Tag handling](#tag-handling)
- [Translate documents](#translate-documents)
- [Manage glossaries](#glossaries)

### Configuration
The library supports a number of configuration options, one of which is the ability to swap out the default client for a new instance of `reqwest::blocking::Client`.  

As before, we create a new instance of `DeepL`, but this time we declare it `mut`. Then we call `client` on the object and pass in our custom client.

```rust
let mut dl = DeepL::new(
    &std::env::var("DEEPL_API_KEY").unwrap()
);

let client = reqwest::blocking::Client::builder()
    .timeout(std::time::Duration::from_secs(21))
    .build()
    .unwrap();

dl.client(client);
``` 

We support sending a custom user agent along with requests. So for instance if you're using this library in another application, say *My App v1.2.3*, you can set the app name and version using `set_app_info`.
```rust
dl.set_app_info(
    "my-app/1.2.3".to_string()
);
```

### Errors
Errors are encapsulated in the `Error` enum whose variants may be one of:
- `Client`: A generic client-side error
- `Server`: An error sent by the server
- `Deserialize`: An error occurred while deserializing the response
- `InvalidRequest`: Error sending an http request
- `InvalidResponse`: Error parsing the response
- `InvalidLanguage`: Error matching a user-supplied string to a `Language`

The library functions we'll look at below are all methods on the `DeepL` client, and many return a `Result` type (enum) that either resolves to a value of the type we expect, or one of the above `Error`s. So for some type `T`, the return type of a function returning a result is `Result<T, Error>`. While in the examples we `unwrap` the `Result` to pull out a value, it's common to implement more robust error handling in production code.

### Get languages
Getting available `languages` requires specifying `LanguageType` as either `Source` or `Target` and returns a `Result` whose success value is a `Vec<LanguageInfo>`.

All instances of `LanguageInfo` contain `language` and `name` attributes. Target languages contain a third field, `supports_formality` which is `true` or `false`. 

```rust
let source_langs = dl.languages(LanguageType::Source).unwrap();

for lang in source_langs {
    let code = lang.language;
    let name = lang.name;
    
    println!("{code} {name}");
    // BG Bulgarian
}
```

### Translate text options
Translating text allows setting a number of options through the `TextOptions` builder, only one of which is required, namely the `target_lang` for the translation.

The list of options for text translation is as follows:

- `target_lang`: The target `Language` (required)
- `source_lang`: The source `Language`
- `split_sentences`: Decide how to split sentences from the input text. Can be one of
    - `SplitSentences::None` Do not split sentences.
    - `SplitSentences::Default` Split on punctuation and newlines (default).
    - `SplitSentences::NoNewLines` Split on punctuation only.
- `preserve_formatting`: Whether the translator should preserve the original format of the text. (default `false`)
- `formality`: The desired formality in the target language. Not all target languages support formality. Options include:
    - `Formality::Default`
    - `Formality::More`
    - `Formality::Less`
    - `Formality::PreferMore`
    - `Formality::PreferLess`
- `glossary_id`: The glossary id `String` to use for translation

### Tag handling
The following are translation options related to tag handling  

- `tag_handling`: Enable handling tags in the input text. Can be one of: 
    - `TagHandling::Xml`
    - `TagHandling::Html`
- `outline_detection`: Whether the translator should automatically detect the outline (default `true`)
- `splitting_tags`: List of tags used to split sentences, `Vec<String>`
- `non_splitting_tags`: List of tags which do not split sentences, `Vec<String>`
- `ignore_tags`: List of tags not to translate, `Vec<String>`

Below is a more complex translation where we want to specify a source language, ignore newlines in the input, preserve formatting, and set a desired formality. We'll also use a custom glossary, ensuring the given glossary matches both the source and target language of this translation.

The function `translate` expects two arguments: a `TextOptions` object, and a `Vec<String>` containing one or more texts to be translated. It returns a `Result` whose `Ok` value is a `TranslateTextResult` with a single field, `translations` that holds a `Vec<Translation>`. 

```rust
// Translate 'you are nice' to French 
let text = vec![
    "you are nice".to_string(),
];
let opt = TextOptions::new(Language::Fr) // note `new` expects the required target lang
    .source_lang(Language::En)
    .formality(Formality::PreferLess)
    .text(text);

let result = dl.translate(opt).unwrap();

let translation = &result.translations[0];
println!("{}", translation.text);
// tu es gentille
```

A `Translation` has two attributes: `text` containing the translated text string, and `detected_source_language`, a string containing the language code of the source language detected by the server.

Here's an example where the input contains xml and where we only want to translate content inside the `<p>` tags.

```rust
let xml = r"
<xml>
    <head>
        <title>My English title</title>
    </head>
    <body>
        <p>The red crab</p>
        <p>Do you speak French?</p>
    </body>
</xml>"
    .to_string();

let text = vec![xml];
let split = vec!["p".to_string()]; // split on <p> tags
let ignore = vec!["title".to_string()]; // ignore <title> tags

let opt = TextOptions::new(Language::Fr)
    .source_lang(Language::En)
    .tag_handling(TagHandling::Xml)
    .outline_detection(false)
    .splitting_tags(split)
    .ignore_tags(ignore)
    .text(text);

let result = dl.translate(opt).unwrap();

let text = &result.translations[0].text;
assert!(text.contains("My English title"));
assert!(text.contains("Le crabe rouge"));
```

### Translate documents
Translating a document consists of three steps: 1) uploading a document, 2) polling the status of a translation in progress, and 3) requesting download of the translated document.

First, we create an instance of `DocumentOptions` which requires we know the target language as well as the file path to a document stored locally and in a supported format. The list of document options is as follows:

- `target_lang`: The target `Language` (required)
- `file_path`: Path to the source file as `PathBuf` (required)
- `source_lang`: The source `Language`
- `filename`: Name of the file, `String`
- `formality`: Formality preference, can be one of:
    - `Formality::Default`
    - `Formality::More`
    - `Formality::Less`
    - `Formality::PreferMore`
    - `Formality::PreferLess`
- `glossary_id`: The id of the glossary to use for translation, `String`

```rust
// Upload a file in the current directory called 'test.txt'
let target_lang = Language::De;
let file_path = std::path::PathBuf::from("test.txt");
let opt = DocumentOptions::new(target_lang, file_path);

let doc = dl.document_upload(opt).unwrap();

println!("Document Id: {}", doc.document_id);
println!("Document Key: {}", doc.document_key);
```

`document_upload` expects an instance of `DocumentOptions` and returns a `Result` whose `Ok` value is a `Document` handle with two fields: `document_id` and `document_key` as strings.

Before we can download a finished document, we need to check the status of the translation process. We do so by calling `document_status` on the client and passing in a reference to the `Document` handle we received previously. The method returns a `Result<DocumentStatus>` where `DocumentStatus` is a struct with the following fields:  
- `document_id`: The unique document id `String`
- `status`: An enum, `DocState` in one of the following states:
    - `DocState::Queued`
    - `DocState::Translating`
    - `DocState::Done`
    - `DocState::Error`
- `seconds_remaining`: Estimated time until translation is complete `Option<u64>`
- `billed_characters`: Number of characters billed `Option<u64>`
- `error_message`: Message from the server in case of error `Option<String>`

When translation is complete, `status` will be in a state of `DocState::Done`, and calling `is_done` on our `DocumentStatus` object returns true. We may then proceed with download.

```rust
// Get the status of a document translation in progress
let status = dl.document_status(&doc).unwrap();

if status.is_done() {
    // Download translation result
    let out_file = PathBuf::from("test-translated.txt");
    let _ = dl.document_download(doc, Some(out_file.clone())).unwrap();
    let content = std::fs::read_to_string(out_file).unwrap();
    assert(!content.is_empty());
}
```

`document_download` takes as arguments the same `Document` handle we received after uploading as well as an optional `PathBuf` denoting the path to the file where the finished document will be saved. The function returns `Result<PathBuf>` where `PathBuf` is the path to the newly translated document. 

If the user-supplied file path for the outgoing file is `None`, a file will be created in the current directory whose name contains the unique `document_id`.

### Glossaries
DeepL supports creating custom glossaries for several language pairs allowing the user to specify an exact translation to use for a given word in the source text. To demonstrate, first we'll query the list of supported glossary language pairs.

The `glossary_languages` method takes no arguments and returns a `Result<GlossaryLanguagePairsResult>` whose `Ok` value has a single field, `supported_languages` holding a `Vec<GlossaryLanguagePair>`.

A `GlossaryLanguagePair` contains two fields: `source_lang` and `target_lang` as strings.

```rust
// Get supported glossary language pairs
let result = dl.glossary_languages().unwrap();

let lang_pairs = result.supported_languages;
assert!(!lang_pairs.is_empty());

for pair in lang_pairs {
    println!("{} -> {}", pair.source_lang, pair.target_lang);
    // EN -> IT
}
```

Now let's create a glossary with source language English and target language Italian. To do so, we'll create a file called *my_glossary.csv* to hold the glossary entries. The entries are formatted as a comma-separated list with two columns (source,target) with one entry per line. Thus, our csv file with two glossary entries looks like this:

> `my_glossary.csv`
```
hello,ciao
goodbye,ciao
```

Back in rust, we'll read the contents of the file to a string called `entries` and pass it to `glossary_new` together with the following parameters (note, DeepL accepts glossary entries as tab-separated values as well): 
- `name: String`
- `source_lang: Language`
- `target_lang: Language` 
- `entries: String` 
- `fmt`: The format of our entries, must be one of:
    - `GlossaryEntriesFormat::Csv`
    - `GlossaryEntriesFormat::Tsv`

`glossary_new` returns a `Result<Glossary>` where `Glossary` is a struct with the following fields:
- `glossary_id: String`
- `ready: bool`
- `name: String`
- `source_lang: String`
- `target_lang: String`
- `creation_time: String`
- `entry_count: u64`

```rust
// Create a new glossary
let name = "my_glossary".to_string();
let src = Language::En;
let trg = Language::It;
let entries = std::fs::read_to_string("my_glossary.csv").unwrap();
let fmt = GlossaryEntriesFormat::Csv;

let glossary = dl.glossary_new(name, src, trg, entries, fmt).unwrap();
assert_eq!(glossary.entry_count, 2);

let glos_id = glossary.glossary_id; // remember this!

// List glossaries
let result = dl.glossaries().unwrap();
let glossaries = result.glossaries;
assert!(!glossaries.is_empty());
```

Listing available `glossaries` returns a `Result<GlossariesResult>` whose inner value has an attribute `glossaries` that holds a `Vec<Glossary>`.

We can get information from a glossary in different ways. Calling `glossary_info` with a valid `glossary_id` returns a `Result<Glossary>` containing the attributes mentioned above. This method returns glossary metadata only. 

To retrieve the actual entries, we use the `glossary_entries` method with a valid `glossary_id`. The function returns a `Result<HashMap<String, String>>` where the `Ok` value is a collection mapping a unique source word to its target translation.

```rust
// Get glossary info
// recall `glos_id` is the glossary id we obtained earlier
let glossary = dl.glossary_info(&glos_id).unwrap();

println!("{}", glossary.name);
// my_glossary

// Get entries from a glossary
let entries = dl.glossary_entries(&glos_id).unwrap();

for (key, value) in entries {
    println!("{key} {value}");
    
    /*
    hello ciao
    goodbye ciao
    */
}

// Remove an unwanted glossary
let result = dl.glossary_delete(&glos_id);
assert!(result.is_ok());
```

To remove a glossary, call the `glossary_delete` method passing a reference to the `glossary_id`. The function returns `Result<()>` where the success value is an empty tuple.
