use std::{fs, path::Path};

use anyhow::Result;
use clap::Parser;
use hanzi_kaishi::{
    anki::{self, build_deck},
    models::{collect_media, merge_vocabulary_with_sentences},
    scraper::WordbrushScraper,
};
use reqwest::blocking::ClientBuilder;
use tracing_subscriber::EnvFilter;

use std::path::PathBuf;

/// Generate an Anki deck after scraping content from various sites with vocabulary and sentences.
#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Cli {
    /// Name of the vocabulary deck as it appears in Anki.
    #[arg(long, default_value = "Hanzi Kaishi::Vocabulary")]
    pub vocab_deck_name: String,

    /// Unique ID for the vocabulary deck.
    #[arg(long, default_value_t = 1_607_392_320)]
    pub vocab_deck_id: i64,

    /// Name of the vocabulary note type (model).
    #[arg(long, default_value = "Hanzi Kaishi Vocab")]
    pub vocab_model_name: String,

    /// Unique ID for the vocabulary note type.
    #[arg(long, default_value_t = 1_954_583_421)]
    pub vocab_model_id: i64,

    /// Name of the sentences deck as it appears in Anki.
    #[arg(long, default_value = "Hanzi Kaishi::Sentences")]
    pub sentences_deck_name: String,

    /// Unique ID for the sentences deck.
    #[arg(long, default_value_t = 1_607_392_321)]
    pub sentences_deck_id: i64,

    /// Name of the sentence note type (model).
    #[arg(long, default_value = "Hanzi Kaishi Sentences")]
    pub sentences_model_name: String,

    /// Unique ID for the sentence note type.
    #[arg(long, default_value_t = 1_954_583_422)]
    pub sentences_model_id: i64,

    /// Path to write the output .apkg package file.
    #[arg(short, long, default_value = "hanzi_chinese.apkg")]
    pub output: PathBuf,

    /// Directory to save downloaded media files into.
    #[arg(short, long, default_value = "media")]
    pub media_dir: PathBuf,

    /// Force re-downloading media files even if they exist on disk.
    #[arg(long, default_value_t = false)]
    pub overwrite: bool,

    /// User agent string used for HTTP scraping requests.
    #[arg(
        long,
        default_value = "Mozilla/5.0 AppleWebKit/537.36 (KHTML, like Gecko; compatible; Googlebot/2.1; +http://www.google.com/bot.html) Chrome/W.X.Y.Z Safari/537.36"
    )]
    pub user_agent: String,

    /// Logging verbosity (-v, -vv, -vvv).
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

/// Initializes tracing subscriber with the correct level of verbosity.
fn init_tracing(verbose: u8) {
    let directive = match verbose {
        0 => "hanzi_kaishi=info",
        1 => "hanzi_kaishi=debug",
        _ => "hanzi_kaishi=trace",
    };

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(directive));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .without_time()
        .init();
}

fn main() -> Result<()> {
    let args = Cli::parse();
    init_tracing(args.verbose);

    let media_dir = Path::new(&args.media_dir);
    fs::create_dir_all(media_dir)?;

    let client = ClientBuilder::new()
        .brotli(true)
        .user_agent(&args.user_agent)
        .build()?;
    let scraper = WordbrushScraper::new(client)?;

    let mut flashcards = scraper.get_words()?;
    let mut sentences = scraper.get_sentences()?;

    scraper.download_words_audio(&mut flashcards, media_dir, args.overwrite);
    scraper.download_sentences_audio(&mut sentences, media_dir, args.overwrite);

    merge_vocabulary_with_sentences(&mut flashcards, &sentences);

    let vocab_model = anki::create_vocabulary_model(args.vocab_model_id, &args.vocab_model_name);
    let mut vocab_deck = anki::create_deck(args.vocab_deck_id, &args.vocab_deck_name);
    build_deck(&mut vocab_deck, &flashcards, &vocab_model)?;

    let sentence_model =
        anki::create_sentence_model(args.sentences_model_id, &args.sentences_model_name);
    let mut sentence_deck = anki::create_deck(args.sentences_deck_id, &args.sentences_deck_name);
    build_deck(&mut sentence_deck, &sentences, &sentence_model)?;

    let media_files = collect_media(
        flashcards
            .iter()
            .flat_map(|c| {
                [
                    c.audio_filename.as_deref(),
                    c.sentence
                        .as_ref()
                        .and_then(|s| s.audio_filename.as_deref()),
                ]
            })
            .chain(sentences.iter().map(|s| s.audio_filename.as_deref())),
        media_dir,
    );

    anki::write_apkg(vec![vocab_deck, sentence_deck], &media_files, &args.output)?;

    Ok(())
}
