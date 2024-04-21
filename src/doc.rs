use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use reqwest::blocking::multipart;

use super::{Error, Result};
use crate::{builder, DeepL, Formality, Language};

/// Document handle
#[derive(Debug, Deserialize, Serialize)]
pub struct Document {
    /// A unique ID assigned to the uploaded document
    pub document_id: String,
    /// Document encryption key
    pub document_key: String,
}

/// Document translation status
#[derive(Debug, Deserialize)]
pub struct DocumentStatus {
    /// A unique ID assigned to the uploaded document
    pub document_id: String,
    /// A short description of the current state of the document translation process
    pub status: DocState,
    /// Estimated number of seconds until the translation is done.
    /// This parameter is only included while status is "translating".
    pub seconds_remaining: Option<u64>,
    /// The number of characters billed to your account
    pub billed_characters: Option<u64>,
    /// Description of the error, if available.
    /// This parameter may be included if an error occurred during translation.
    pub error_message: Option<String>,
}

/// Document state
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DocState {
    /// The translation job is waiting in line to be processed
    Queued,
    /// The translation is currently ongoing
    Translating,
    /// The translation is done and the document is ready for download
    Done,
    /// An irrecoverable error occurred while translating the document
    Error,
}

// DocumentOptions builder
builder! {
    Document {
        @must{
            target_lang: Language,
            file_path: PathBuf,
        };
        @optional{
            source_lang: Language,
            filename: String,
            formality: Formality,
            glossary_id: String,
        };
    }
}

impl DocumentStatus {
    /// Whether the document is done translating and ready to be downloaded
    pub fn is_done(&self) -> bool {
        matches!(self.status, DocState::Done)
    }
}

impl DocumentOptions {
    /// Creates a multipart request form from an instance of `DocumentOptions`
    fn into_multipart(self) -> Result<multipart::Form> {
        let mut form = multipart::Form::new()
            .file("file", self.file_path)
            .map_err(|_| Error::Client("failed to attach file".to_string()))?
            .text("target_lang", self.target_lang.to_string());

        if let Some(src) = self.source_lang {
            form = form.text("source_lang", src.to_string());
        }
        if let Some(name) = self.filename {
            form = form.text("filename", name);
        }
        if let Some(fm) = self.formality {
            form = form.text("formality", fm.as_ref().to_string());
        }
        if let Some(glos) = self.glossary_id {
            form = form.text("glossary_id", glos);
        }

        Ok(form)
    }
}

impl DeepL {
    /// POST /document
    ///
    /// Upload a document.
    ///
    /// Translating a document consists of three steps: upload, polling translation status,
    /// and download. `document_upload` returns a document handle that we need in order to
    /// complete the remaining steps: getting the [`document_status`](Self::document_status)
    /// and finally fetching the translation result with
    /// [`document_download`](Self::document_download).
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # use deeprl::*;
    /// # use std::{env, fs, path::PathBuf, thread};
    /// # let dl = DeepL::new(&env::var("DEEPL_API_KEY").unwrap());
    /// # use std::time::Duration;
    /// let file_path = PathBuf::from("test.txt");
    /// let target_lang = Language::DE;
    /// let opt = DocumentOptions::new(target_lang, file_path);
    /// let doc = dl.document_upload(opt).unwrap();
    ///
    /// while !dl.document_status(&doc).unwrap().is_done() {
    ///     // wait a second
    ///     thread::sleep(Duration::from_secs(1));
    /// }
    ///
    /// let out_file = PathBuf::from("test-translated.txt");
    /// let _ = dl.document_download(doc, Some(out_file.clone())).unwrap();
    /// let content = fs::read_to_string(out_file).unwrap();
    /// assert!(!content.is_empty());
    /// ```
    pub fn document_upload(&self, opt: DocumentOptions) -> Result<Document> {
        let url = format!("{}/document", self.url);

        let form = opt.into_multipart()?;

        let resp = self
            .post(url)
            .multipart(form)
            .send()
            .map_err(|_| Error::InvalidRequest)?;

        if !resp.status().is_success() {
            return super::convert(resp);
        }

        resp.json().map_err(|_| Error::Deserialize)
    }

    /// POST /document/`{document_id}`
    ///
    /// Get document translation status. In case there's an issue with translation,
    /// [`DocumentStatus`] contains a field `error_message` that may provide context
    /// for the cause of the error.
    pub fn document_status(&self, doc: &Document) -> Result<DocumentStatus> {
        let doc_id = doc.document_id.clone();
        let url = format!("{}/document/{}", self.url, doc_id);

        let key = doc.document_key.clone();
        let params = vec![("document_key", key)];

        let resp = self
            .post(url)
            .form(&params)
            .send()
            .map_err(|_| Error::InvalidRequest)?;

        if !resp.status().is_success() {
            return super::convert(resp);
        }

        resp.json().map_err(|_| Error::Deserialize)
    }

    /// POST /document/`{document_id}`/result
    ///
    /// Download translated document
    pub fn document_download(&self, doc: Document, out_file: Option<PathBuf>) -> Result<PathBuf> {
        let doc_id = doc.document_id;
        let url = format!("{}/document/{}/result", self.url, doc_id);

        let params = vec![("document_key", doc.document_key)];

        let mut resp = self
            .post(url)
            .form(&params)
            .send()
            .map_err(|_| Error::InvalidRequest)?;

        if !resp.status().is_success() {
            return super::convert(resp);
        }

        // write out file
        let mut buf: Vec<u8> = Vec::with_capacity(100 * 1024);
        resp.copy_to(&mut buf)
            .map_err(|_| Error::Client("could not copy response data".to_string()))?;

        let path = out_file.unwrap_or(PathBuf::from(doc_id));

        std::fs::write(&path, buf)
            .map_err(|_| Error::Client("failed to write out file".to_string()))?;

        Ok(path)
    }
}
