use std::{collections::HashSet, path::Path};

use anyhow::Result;
use genanki_rs::{Model, Note};

pub struct Flashcard {
    pub hanzi: String,
    pub pinyin: String,
    pub meaning: String,
    pub hsk_level: Option<i8>,
    pub audio_file_url: Option<String>,
    pub audio_filename: Option<String>,
    pub sentence: Option<Sentence>,
}

#[derive(Clone)]
pub struct Sentence {
    pub hanzi: String,
    pub pinyin: String,
    pub meaning: String,
    pub hsk_level: Option<i8>,
    pub audio_file_url: Option<String>,
    pub audio_filename: Option<String>,
}

impl Flashcard {
    pub fn to_note(&self, model: &Model) -> Result<Note> {
        let sentence_hanzi = self.sentence.as_ref().map_or("", |s| s.hanzi.as_str());
        let sentence_pinyin = self.sentence.as_ref().map_or("", |s| s.pinyin.as_str());
        let sentence_meaning = self.sentence.as_ref().map_or("", |s| s.meaning.as_str());

        let word_audio = self
            .audio_filename
            .as_ref()
            .map(|f| format!("[sound:{f}]"))
            .unwrap_or_default();
        let sentence_audio = self
            .sentence
            .as_ref()
            .and_then(|s| s.audio_filename.as_deref())
            .map(|f| format!("[sound:{f}]"))
            .unwrap_or_default();

        let tag = self.hsk_level.map(|level| format!("HSK::HSK{level}"));
        let tags = tag.as_deref().map(|tag| vec![tag]);

        Note::new_with_options(
            model.clone(),
            vec![
                &self.hanzi,
                &self.pinyin,
                &self.meaning,
                sentence_hanzi,
                sentence_pinyin,
                sentence_meaning,
                &word_audio,
                &sentence_audio,
                "",
                "",
            ],
            None,
            tags,
            Some(&self.hanzi),
        )
        .map_err(Into::into)
    }
}

pub fn merge_flashcards_with_sentences(flashcards: &mut [Flashcard], sentences: &[Sentence]) {
    for card in flashcards.iter_mut() {
        let matches: Vec<&Sentence> = sentences
            .iter()
            .filter(|s| s.hanzi.contains(&card.hanzi))
            .collect();

        let same_level = matches
            .iter()
            .copied()
            .filter(|s| s.hsk_level == card.hsk_level)
            .min_by_key(|s| s.hanzi.chars().count());

        card.sentence = same_level
            .or_else(|| matches.into_iter().min_by_key(|s| s.hanzi.chars().count()))
            .cloned();
    }
}

pub fn collect_media_paths(flashcards: &[Flashcard], media_dir: &Path) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut paths = Vec::new();

    let mut add = |file_name: &str| {
        if seen.insert(file_name.to_string()) {
            paths.push(media_dir.join(file_name).to_string_lossy().into_owned());
        }
    };

    for card in flashcards {
        if let Some(file_name) = &card.audio_filename {
            add(file_name);
        }
        if let Some(sentence) = &card.sentence {
            if let Some(file_name) = &sentence.audio_filename {
                add(file_name);
            }
        }
    }

    paths
}
