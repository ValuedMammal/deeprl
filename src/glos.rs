use reqwest::header;
use serde::Deserialize;
use std::{fmt, collections::HashMap};

use super::*;
use crate::lang::Language;

#[derive(Debug, Deserialize)]
pub struct GlossaryLanguagePair {
    pub source_lang: String,
    pub target_lang: String,
}

#[derive(Debug, Deserialize)]
pub struct GlossaryLanguagePairsResult {
    pub supported_languages: Vec<GlossaryLanguagePair>,
}

#[derive(Debug)]
pub enum GlossaryEntriesFormat {
    Tsv,
    Csv,
}

#[derive(Debug, Deserialize)]
pub struct Glossary {
    /// A unique ID assigned to a glossary.
    pub glossary_id: String,
    /// Indicates if the newly created glossary can already be used in translate requests. If the created glossary is not yet ready, you have to wait and check the ready status of the glossary before using it in a translate request.
    pub ready: bool,
    /// Name associated with the glossary.
    pub name: String,
    /// The language in which the source texts in the glossary are specified.
    pub source_lang: String,
    /// The language in which the target texts in the glossary are specified.
    pub target_lang: String,
    /// The creation time of the glossary.
    pub creation_time: String,
    /// The number of entries in the glossary.
    pub entry_count: u64,
}

#[derive(Debug, Deserialize)]
pub struct GlossariesResult {
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
    pub fn glossary_languages(&self) -> Result<GlossaryLanguagePairsResult> {
        let url = format!("{}/glossary-language-pairs", self.url);

        let resp = self.client.get(url)
            .send()
            .map_err(|_| Error::Request)?;
        
        if !resp.status().is_success() {
            return super::convert(resp)
        }
        
        resp.json()
            .map_err(|_| Error::Deserialize)
    }

    /// POST /glossaries
    pub fn glossary_new(
        &self, 
        name: String, 
        source_lang: Language, 
        target_lang: Language, 
        entries: String, 
        fmt: GlossaryEntriesFormat
    ) -> Result<Glossary>
    {
        let url = format!("{}/glossaries", self.url);

        let params = HashMap::from([
            ("name", name),
            ("source_lang", source_lang.to_string()),
            ("target_lang", target_lang.to_string()),
            ("entries", entries),
            ("entries_format", fmt.to_string()),
        ]);

        let resp = self.client.post(url)
            .form(&params)
            .send()
            .map_err(|_| Error::Request)?;

        if !resp.status().is_success() {
            return super::convert(resp)
        }
            
        resp.json()
            .map_err(|_| Error::Deserialize)
    }

    /// GET /glossaries
    pub fn glossaries(&self) -> Result<GlossariesResult> {
        let url = format!("{}/glossaries", self.url);

        let resp = self.client.get(url)
            .send()
            .map_err(|_| Error::Request)?;

        if !resp.status().is_success() {
            return super::convert(resp)
        }
        
        resp.json()
            .map_err(|_| Error::Deserialize)
    }
    
    /// GET /glossaries/{glossary_id}
    pub fn glossary_info(&self, glossary_id: &str) -> Result<Glossary> {
        let url = format!("{}/glossaries/{}", self.url, glossary_id);

        let resp = self.client.get(url)
            .send()
            .map_err(|_| Error::Request)?;

        if !resp.status().is_success() {
            return super::convert(resp)
        }

        resp.json()
            .map_err(|_| Error::Deserialize)
    }
    
    /// GET /glossaries/{glossary_id}/entries
    pub fn glossary_entries(&self, glossary_id: &str) -> Result<String> {
        let url = format!("{}/glossaries/{}/entries", self.url, glossary_id);
        let accept = header::HeaderValue::from_static("text/tab-separated-values");
        
        let resp = self.client.get(url)
            .header(header::ACCEPT, accept)
            .send()
            .map_err(|_| Error::Request)?;
    
        if !resp.status().is_success() {
            return super::convert(resp)
        }
        
        resp.text()
            .map_err(|_| Error::InvalidResponse)
    }
    
    /// DELETE /glossaries/{glossary_id}
    pub fn glossary_del(&self, glossary_id: &str) -> Result<()> {
        let url = format!("{}/glossaries/{}", self.url, glossary_id);

        let _ = self.client.delete(url)
            .send()
            .map_err(|_| Error::Request);

        Ok(())
    }
}
