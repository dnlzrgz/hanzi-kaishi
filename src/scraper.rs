use std::path::Path;

use anyhow::Result;
use reqwest::blocking::{Client};
use scraper::{Html, Selector};
use url::Url;

use crate::{models::{Flashcard, Sentence}, utils::download_media};

pub struct WordbrushScraper {
    client: Client,
    base_url: Url,
}

impl WordbrushScraper {
    pub fn new(client: Client) -> Result<Self> {
        Ok(Self {
            client: client,
            base_url: Url::parse("https://wordbrushchinese.com")?,
        })
    }

    pub fn get_words(&self) -> Result<Vec<Flashcard>> {
        let mut flashcards: Vec<Flashcard> = Vec::new();

        for level in [1i8, 2, 3] {
            let url = self.base_url.join(&format!("hsk-{}-word-list", level))?;
            let body = self.client.get(url).send()?.error_for_status()?.text()?;

            let document = Html::parse_document(&body);
            let table_selector = Selector::parse("table").unwrap();
            let row_selector = Selector::parse("tr").unwrap();
            let cell_selector = Selector::parse("td").unwrap();
            let button_selector = Selector::parse("button.play").unwrap();

            let table = document.select(&table_selector).next().unwrap();
            for row in table.select(&row_selector) {
                let cells: Vec<_> = row.select(&cell_selector).collect();

                let (Some(han), Some(py), Some(meaning), Some(audio_cell)) =
                    (cells.get(1), cells.get(2), cells.get(3), cells.get(4))
                else {
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

                flashcards.push(Flashcard { hanzi, pinyin, meaning, audio_file_url, hsk_level: Some(level), audio_filename: None, sentence: None });
            }
        }

        Ok(flashcards)
    }

    pub fn get_sentences(&self) -> Result<Vec<Sentence>> {
        let mut sentences: Vec<Sentence> = Vec::new();

        for level in [1i8, 2, 3] {
            let url = self
                .base_url
                .join(&format!("hsk-{}-example-sentences", level))?;
            let body = self.client.get(url).send()?.error_for_status()?.text()?;

            let document = Html::parse_document(&body);
            let sent_container_sel = Selector::parse("div.sent").unwrap();
            let text_sel = Selector::parse("div.sent-hz span").unwrap();
            let button_sel = Selector::parse("button.play").unwrap();

            let py_sel = Selector::parse("div.sent-py").unwrap();
            let meaning_sel = Selector::parse("div.sent-en").unwrap();

            for sent_node in document.select(&sent_container_sel) {
                let Some(span_node) = sent_node.select(&text_sel).next() else {
                    continue;
                };

                let hanzi = span_node.text().collect::<String>().trim().to_string();

                let audio_file_url = sent_node
                    .select(&button_sel)
                    .next()
                    .and_then(|btn| btn.value().attr("data-src"))
                    .and_then(|src| self.base_url.join(src).ok())
                    .map(|url| url.to_string());

                let pinyin = sent_node
                    .select(&py_sel)
                    .next()
                    .map(|el| el.text().collect::<String>().trim().to_string())
                    .unwrap();

                let meaning = sent_node
                    .select(&meaning_sel)
                    .next()
                    .map(|el| el.text().collect::<String>().trim().to_string())
                    .unwrap();

                sentences.push(Sentence { hanzi, pinyin, meaning, audio_file_url, hsk_level: Some(level), audio_filename: None });
            }
        }

        Ok(sentences)
    }

    pub fn download_words_audio(&self, flashcards: &mut [Flashcard], media_dir: &Path, overwrite: bool) {
        for card in flashcards.iter_mut() {
            let Some(url) = &card.audio_file_url else { continue };
            let file_name = format!("{}.mp3", card.hanzi);
            let path = media_dir.join(&file_name);

            if path.exists() && !overwrite {
                card.audio_filename = Some(file_name);
                continue;
            }

            match download_media(&self.client, url, &path) {
                Ok(()) => card.audio_filename = Some(file_name),
                Err(e) => eprintln!("word audio failed for {}: {e}", card.hanzi),
            }
        }
    }

    pub fn download_sentences_audio(&self, sentences: &mut [Sentence], media_dir: &Path, overwrite: bool) {
        for sentence in sentences.iter_mut() {
            let Some(url) = &sentence.audio_file_url else { continue };
            let file_name = format!("{}.mp3", sentence.hanzi);
            let path = media_dir.join(&file_name);

            if path.exists() && !overwrite {
                sentence.audio_filename = Some(file_name);
                continue;
            }

            match download_media(&self.client, url, &path) {
                Ok(()) => sentence.audio_filename = Some(file_name),
                Err(e) => eprintln!("sentence audio failed for {}: {e}", sentence.hanzi),
            }
        }
    }
}
