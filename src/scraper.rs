use std::path::Path;

use anyhow::{Context, Result};
use rayon::prelude::*;
use reqwest::blocking::Client;
use scraper::{Html, Selector};
use tracing::{debug, warn};
use url::Url;

use crate::{
    models::{Sentence, VocabFlashcard},
    utils::download_media,
};

pub struct WordbrushScraper {
    client: Client,
    base_url: Url,
}

impl WordbrushScraper {
    pub fn new(client: Client) -> Result<Self> {
        Ok(Self {
            client,
            base_url: Url::parse("https://wordbrushchinese.com")?,
        })
    }

    pub fn get_words(&self) -> Result<Vec<VocabFlashcard>> {
        let mut flashcards: Vec<VocabFlashcard> = Vec::new();

        let table_selector = Selector::parse("table").expect("hardcored selector is valid");
        let row_selector = Selector::parse("tr:has(td)").expect("hardcored selector is valid");
        let cell_selector = Selector::parse("td").expect("hardcoded selector is valid");
        let button_selector = Selector::parse("button.play").expect("hardcored selector is valid");

        for level in [1i8, 2, 3] {
            debug!(level, "fetching word list");

            let url = self.base_url.join(&format!("hsk-{}-word-list", level))?;
            let body = self.client.get(url).send()?.error_for_status()?.text()?;

            let document = Html::parse_document(&body);
            let table = document.select(&table_selector).next().with_context(|| {
                format!("no <table> element found on the HSK {level} word list page")
            })?;
            for row in table.select(&row_selector) {
                let cells: Vec<_> = row.select(&cell_selector).collect();

                let (Some(han), Some(py), Some(meaning), Some(audio_cell)) =
                    (cells.get(1), cells.get(2), cells.get(3), cells.get(4))
                else {
                    warn!(level, "skipping malformed word row: missing expected cells");
                    continue;
                };

                let hanzi: String = han.text().collect();
                let pinyin: String = py.text().collect();
                let meaning: String = meaning.text().collect();
                let audio_file_url = audio_cell
                    .select(&button_selector)
                    .next()
                    .and_then(|btn| btn.value().attr("data-src"))
                    .and_then(|src| self.base_url.join(src).ok())
                    .map(|url| url.to_string());

                flashcards.push(VocabFlashcard {
                    hanzi,
                    pinyin,
                    meaning,
                    audio_file_url,
                    hsk_level: Some(level),
                    audio_filename: None,
                    sentence: None,
                });
            }
        }

        Ok(flashcards)
    }

    pub fn get_sentences(&self) -> Result<Vec<Sentence>> {
        let mut sentences: Vec<Sentence> = Vec::new();

        let sent_container_sel = Selector::parse("div.sent").expect("hardcoded selector is valid");
        let text_sel = Selector::parse("div.sent-hz span").expect("hardcoded selecotr is valid");
        let button_sel = Selector::parse("button.play").expect("hardcoded selector is valid");

        let py_sel = Selector::parse("div.sent-py").expect("hardcoded selector is valid");
        let meaning_sel = Selector::parse("div.sent-en").expect("hardcoded selector is valid");

        for level in [1i8, 2, 3] {
            debug!(level, "fetching example sentences");

            let url = self
                .base_url
                .join(&format!("hsk-{}-example-sentences", level))?;
            let body = self.client.get(url).send()?.error_for_status()?.text()?;

            let document = Html::parse_document(&body);
            for sent_node in document.select(&sent_container_sel) {
                let Some(span_node) = sent_node.select(&text_sel).next() else {
                    warn!(level, "skipping sentence with no hanzi text found");
                    continue;
                };

                let hanzi = span_node.text().collect::<String>().trim().to_string();

                let audio_file_url = sent_node
                    .select(&button_sel)
                    .next()
                    .and_then(|btn| btn.value().attr("data-src"))
                    .and_then(|src| self.base_url.join(src).ok())
                    .map(|url| url.to_string());

                let Some(pinyin) = sent_node
                    .select(&py_sel)
                    .next()
                    .map(|el| el.text().collect::<String>().trim().to_string())
                else {
                    warn!(level, hanzi, "skipping sentence with no pinyin found");
                    continue;
                };

                let Some(meaning) = sent_node
                    .select(&meaning_sel)
                    .next()
                    .map(|el| el.text().collect::<String>().trim().to_string())
                else {
                    warn!(level, hanzi, "skipping sentence with no meaning found");
                    continue;
                };

                sentences.push(Sentence {
                    hanzi,
                    pinyin,
                    meaning,
                    audio_file_url,
                    hsk_level: Some(level),
                    audio_filename: None,
                });
            }
        }

        Ok(sentences)
    }

    pub fn download_words_audio(
        &self,
        flashcards: &mut [VocabFlashcard],
        media_dir: &Path,
        overwrite: bool,
    ) {
        debug!(count = flashcards.len(), "downloading word audios");

        flashcards.par_iter_mut().for_each(|card| {
            let Some(url) = &card.audio_file_url else {
                return;
            };
            let file_name = format!("{}.mp3", card.hanzi);
            let path = media_dir.join(&file_name);

            if path.exists() && !overwrite {
                card.audio_filename = Some(file_name);
                return;
            }

            match download_media(&self.client, url, &path) {
                Ok(()) => card.audio_filename = Some(file_name),
                Err(e) => warn!(word = %card.hanzi, error = %e, "word audio download failed"),
            }
        });
    }

    pub fn download_sentences_audio(
        &self,
        sentences: &mut [Sentence],
        media_dir: &Path,
        overwrite: bool,
    ) {
        debug!(
            count = sentences.len(),
            "downloading example sentence audios"
        );

        sentences.par_iter_mut().for_each(|sentence| {
            let Some(url) = &sentence.audio_file_url else {
                return;
            };
            let file_name = format!("{}.mp3", sentence.hanzi);
            let path = media_dir.join(&file_name);

            if path.exists() && !overwrite {
                sentence.audio_filename = Some(file_name);
                return;
            }

            match download_media(&self.client, url, &path) {
                Ok(()) => sentence.audio_filename = Some(file_name),
                Err(e) => {
                    warn!(sentence = %sentence.hanzi, error = %e, "sentence audio download failed")
                }
            }
        });
    }
}
