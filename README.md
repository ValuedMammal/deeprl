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
let opt = TextOptions::new(Language::DE);

let text = vec![
    "good morning".to_string(),
];

let result = dl.translate(opt, text).unwrap();
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
- [Get all languages](#get-languages)
- [Translate text options](#translate-text-options)
    - [Tag handling](#tag-handling)
- translate documents
    - upload
    - get status
    - download
- working with glossaries
    - get supported pairs
    - list glossaries
    - create new
    - get glos
    - get glos entries
    - remove glos

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
// snip --

dl.set_app_info(
    "my-app/1.2.3".to_string()
);
```

### Get languages
Getting available `languages` requires specifying either `Source` or `Target` language type and returns a `Vec` of `LanguageInfo` objects.
```rust
// snip --

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

The list of remaining options for text translation is as follows:  

- `source_lang`: The source `Language`
- `split_sentences`: Decide how to split sentences from the input text. Can be one of
    - `SplitSentences::None` Do not split sentences.
    - `SplitSentences::Default` Split on punctuation and newlines (default).
    - `SplitSentences::NoNewLines` Split on punctuation only.
- `preserve_formatting`: Whether the translator should preserve the original format of the text. Either `true` or `false`.
- `formality`: The desired formality in the target language. Not all target languages support formality. Options include:
    - `Formality::Default`
    - `Formality::More`
    - `Formality::Less`
    - `Formality::PreferMore`
    - `Formality::PreferLess`
- `glossary_id`: The glossary id as `String` to use for translation

### Tag handling
The following are translation options related to tag handling  

- `tag_handling`: Enable handling tags in the input text. Can be one of: 
    - `TagHandling::Xml`
    - `TagHandling::Html`
- `outline_detection`: Whether the translator should automatically detect the outline (default `true`)
- `splitting_tags`: A comma-separated list of tags that are used to split sentences e.g. "head,title,body" (`String`)
- `non_splitting_tags`: A comma-separated list of tags which do not split sentences, (`String`)
- `ignore_tags`: A comma-separated list of tags not to translate (`String`),

Here's a normal text translation where we want to specify a source language, ignore newlines in the input, preserve formatting, and set a desired formality. We'll also use a custom glossary, ensuring the glossary entries match both the source and target language of this translation.
```rust
// snip --

// Translate 'you are nice' to French (containing a '\n')
let text = vec![
    "you\nare nice".to_string(),
];

let glossary = String::from("abc-123"); // your glossary id

let opt = TextOptions::new(Language::FR) // note `new` expects the required target lang
    .source_lang(Language::EN)
    .split_sentences(SplitSentences::NoNewlines)
    .preserve_formatting(true)
    .formality(text::Formality::PreferLess)
    .glossary_id(glossary);

let result = dl.translate(opt, text).unwrap();

let translation = &result.translations[0];
println!("{}", translation.text);
// tu es gentille
```

And here's an example where the input contains xml and where we only want to translate content inside the `<p>` tags.

```rust
// snip --

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
let split = "p".to_string(); // split on <p> tags
let ignore = "title".to_string(); // ignore <title> tags

let opt = TextOptions::new(Language::FR)
    .source_lang(Language::EN)
    .tag_handling(TagHandling::Xml)
    .outline_detection(false)
    .splitting_tags(split)
    .ignore_tags(ignore);

let result = dl.translate(opt, text).unwrap();

let text = &result.translations[0].text;
assert!(text.contains("My English title"));
assert!(text.contains("Parlez-vous français ?"));
```