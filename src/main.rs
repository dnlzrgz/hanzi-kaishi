use std::{fs, path::Path};

use anyhow::Result;
use hanzi_kaishi::{anki::{self, build_deck}, models::{collect_media_paths, merge_flashcards_with_sentences}, scraper::WordbrushScraper};
use reqwest::blocking::ClientBuilder;
use clap::Parser;

use std::path::PathBuf;

/// Generate an Anki deck after scraping content from various sites with vocabulary and sentences.
#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Cli {
    /// Name of the Anki deck, as it will appear in Anki.
    #[arg(long, default_value = "Kaishi Chinese")]
    pub deck_name: String,

    /// Name of the Anki note type (model), as it will appear in Anki.
    #[arg(long, default_value = "Kaishi Chinese")]
    pub model_name: String,

    /// Anki deck's id. Changing it makes Anki to treat re-imports as a brand-new deck.
    #[arg(long, default_value_t = 1_607_392_320)]
    pub deck_id: i64,

    /// Anki note type (model) id. Changing it makes Anki to treat re-imports as a brand-new note type.
    #[arg(long, default_value_t = 1_954_583_421)]
    pub model_id: i64,

    /// User agent used  for scraping.
    #[arg(
        long,
        default_value = "Mozilla/5.0 AppleWebKit/537.36 (KHTML, like Gecko; compatible; Googlebot/2.1; +http://www.google.com/bot.html) Chrome/W.X.Y.Z Safari/537.36"
    )]
    pub user_agent: String,

    /// Path to write the generated .apkg file to.
    #[arg(long, default_value = "kaishi_chinese.apkg")]
    pub output: PathBuf,

    /// Directory to save downloaded media files into.
    #[arg(long, default_value = "media")]
    pub media_dir: PathBuf,

    /// Force re-downloading media files even if they already exist on disk.
    #[arg(long, default_value_t = false)]
    pub overwrite: bool,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let media_dir = Path::new(&args.media_dir);
    fs::create_dir_all(media_dir)?;

    let client = ClientBuilder::new().brotli(true).user_agent("Mozilla/5.0 AppleWebKit/537.36 (KHTML, like Gecko; compatible; Googlebot/2.1; +http://www.google.com/bot.html) Chrome/W.X.Y.Z Safari/537.36").build()?;
    let scraper = WordbrushScraper::new(client.clone())?;
    let mut flashcards = scraper.get_words()?;
    let mut sentences = scraper.get_sentences()?;
    scraper.download_words_audio(&mut flashcards, media_dir, args.overwrite);
    scraper.download_sentences_audio(&mut sentences, media_dir, args.overwrite);

    merge_flashcards_with_sentences(&mut flashcards, &sentences);

    let model = anki::create_model(args.model_id, &args.model_name);
    let mut deck = anki::create_deck(args.deck_id, &args.deck_name);
    build_deck(&mut deck, &flashcards, &model)?;

    let media_files = collect_media_paths(&flashcards, media_dir);
    anki::write_apkg(deck, &media_files, &args.output)?;

    Ok(())
}
