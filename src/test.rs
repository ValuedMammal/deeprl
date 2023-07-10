//
use super::*;

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