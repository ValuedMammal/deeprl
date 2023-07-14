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
TODO!
- advanced configuration, e.g. app user agent (not implemented)
- get available languages 
- translate documents 
- working with glossaries 