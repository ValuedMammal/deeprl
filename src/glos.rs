use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt};

use reqwest::header;

use super::{Error, Result};
use crate::{DeepL, Language};

/// A glossary language pair
#[derive(Debug, Deserialize, Serialize)]
pub struct GlossaryLanguagePair {
    /// Source language
    pub source_lang: String,
    /// Target language
    pub target_lang: String,
}

/// Defines the set of supported language pairs for a glossary
#[derive(Debug, Deserialize, Serialize)]
pub struct GlossaryLanguagePairsResult {
    /// List of supported glossary language pairs
    pub supported_languages: Vec<GlossaryLanguagePair>,
}

/// Format in which glossary entries are provided
#[derive(Clone, Copy, Debug)]
pub enum GlossaryEntriesFormat {
    /// Tab-separated values
    Tsv,
    /// Comma-separated values
    Csv,
}

/// Information that uniquely identifies a glossary
#[derive(Debug, Deserialize, Serialize)]
pub struct Glossary {
    /// A unique ID assigned to a glossary
    pub glossary_id: String,
    /// Indicates if the newly created glossary can already be used in translate requests.
    /// If the created glossary is not yet ready, you have to wait and check the ready status
    /// of the glossary before using it in a translate request.
    pub ready: bool,
    /// Name associated with the glossary
    pub name: String,
    /// The language in which the source texts in the glossary are specified
    pub source_lang: String,
    /// The language in which the target texts in the glossary are specified
    pub target_lang: String,
    /// The creation time of the glossary in ISO 8601-1:2019 format (e.g. 2021-08-03T14:16:18.329Z)
    pub creation_time: String,
    /// The number of entries in the glossary
    pub entry_count: u64,
}

/// The result of getting available glossaries
#[derive(Debug, Deserialize, Serialize)]
pub struct GlossariesResult {
    /// List of glossaries
    pub glossaries: Vec<Glossary>,
}

impl AsRef<str> for GlossaryEntriesFormat {
    fn as_ref(&self) -> &str {
        match self {
            Self::Tsv => "tsv",
            Self::Csv => "csv",
        }
    }
}

impl fmt::Display for GlossaryEntriesFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

impl DeepL {
    /// GET /glossary-language-pairs
    ///
    /// Get supported glossary language pairs
    pub fn glossary_languages(&self) -> Result<GlossaryLanguagePairsResult> {
        let url = format!("{}/glossary-language-pairs", self.url);

        let resp = self.get(url).send().map_err(Error::Reqwest)?;

        if !resp.status().is_success() {
            return super::convert(resp);
        }

        resp.json().map_err(|_| Error::Deserialize)
    }

    /// POST /glossaries
    ///
    /// Create a new glossary.
    ///
    /// [`DeepL`] supports creating custom glossaries, i.e. a collection of entries which when
    /// used in a translation ensures that a given word from the source language always maps to
    /// the same target word in the glossary, giving a user more control in cases where
    /// translation might otherwise be unreliable or ambiguous. A given glossary is defined by one
    /// source language and one target language where the source word in each entry is unique.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # use deeprl::*;
    /// # let dl = DeepL::new(&std::env::var("DEEPL_API_KEY").unwrap());
    /// let name = "my_glossary".to_string();
    /// let source_lang = Language::EN;
    /// let target_lang = Language::IT;
    /// let entries = "hello,ciao".to_string();
    /// let fmt = GlossaryEntriesFormat::Csv;
    ///
    /// let glossary = dl.glossary_new(
    ///     name,
    ///     source_lang,
    ///     target_lang,
    ///     entries,
    ///     fmt
    /// )
    /// .unwrap();
    /// assert!(!glossary.glossary_id.is_empty());
    /// ```
    pub fn glossary_new(
        &self,
        name: String,
        source_lang: Language,
        target_lang: Language,
        entries: String,
        fmt: GlossaryEntriesFormat,
    ) -> Result<Glossary> {
        let url = format!("{}/glossaries", self.url);

        let params = HashMap::from([
            ("name", name),
            ("source_lang", source_lang.to_string()),
            ("target_lang", target_lang.to_string()),
            ("entries", entries),
            ("entries_format", fmt.to_string()),
        ]);

        let resp = self
            .post(url)
            .form(&params)
            .send()
            .map_err(Error::Reqwest)?;

        if !resp.status().is_success() {
            return super::convert(resp);
        }

        resp.json().map_err(|_| Error::Deserialize)
    }

    /// GET /glossaries
    ///
    /// List current active glossaries
    pub fn glossaries(&self) -> Result<GlossariesResult> {
        let url = format!("{}/glossaries", self.url);

        let resp = self.get(url).send().map_err(Error::Reqwest)?;

        if !resp.status().is_success() {
            return super::convert(resp);
        }

        resp.json().map_err(|_| Error::Deserialize)
    }

    /// GET /glossaries/`{glossary_id}`
    ///
    /// Get meta information for a specified glossary (excluding entries)
    pub fn glossary_info(&self, glossary_id: &str) -> Result<Glossary> {
        let url = format!("{}/glossaries/{}", self.url, glossary_id);

        let resp = self.get(url).send().map_err(Error::Reqwest)?;

        if !resp.status().is_success() {
            return super::convert(resp);
        }

        resp.json().map_err(|_| Error::Deserialize)
    }

    /// GET /glossaries/`{glossary_id}`/entries
    ///
    /// Retrieve entries for a specified glossary.
    // Currently supports receiving entries in TSV format.
    pub fn glossary_entries(&self, glossary_id: &str) -> Result<HashMap<String, String>> {
        let url = format!("{}/glossaries/{}/entries", self.url, glossary_id);
        let accept = header::HeaderValue::from_static("text/tab-separated-values");

        let resp = self
            .get(url)
            .header(header::ACCEPT, accept)
            .send()
            .map_err(Error::Reqwest)?;

        if !resp.status().is_success() {
            return super::convert(resp);
        }

        let t = resp.text().map_err(|_| Error::InvalidResponse).unwrap();
        // The response text contains newline-separated entries
        // where each entry contains two strings separated by a tab.
        // First we split entries on '\n', then for each entry, split words
        // on '\t' and build a map of source to target words
        let raw_entries: Vec<&str> = t.split('\n').collect();

        let mut map = HashMap::new();
        for entry in raw_entries {
            let words: Vec<&str> = entry.split('\t').collect();
            if words.len() != 2 {
                continue;
            }
            map.insert(words[0].to_string(), words[1].to_string());
        }

        Ok(map)
    }

    /// DELETE /glossaries/`{glossary_id}`
    ///
    /// Destroy a glossary
    pub fn glossary_delete(&self, glossary_id: &str) -> Result<()> {
        let url = format!("{}/glossaries/{}", self.url, glossary_id);

        let _ = self.delete(url).send().map_err(Error::Reqwest);

        Ok(())
    }
}
