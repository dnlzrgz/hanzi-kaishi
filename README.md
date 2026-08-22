# Hanzi Kaishi (汉字开始)

A simple CLI written in Rust for building a custom Chinese Anki deck from vocabulary, sentences, and audio scraped from the web.

## Motivation

There are lots of Chinese Anki decks out there, but none of them had the structure or flexibility I wanted, so I decided to build my own.

The project was put together over a weekend, so the code is really straightforward. I chose Rust because I wanted to build it in Rust, and because some crates like `genanki-rs` are quite good.

## Installation

### Prerequisites

- [Rust & Cargo](https://www.rust-lang.org/tools/install)

### Setup

1. **Clone the repository:**

```bash
git clone https://github.com/dnlzrgz/hanzi-kaishi.git
cd hanzi-kaishi
```

2. **Build the project:**

```bash
cargo build --release
```

### Usage

You can see the available options by running:

```bash
cargo run --release -- --help
```

Or just start scraping, downloading, and building the deck directly:

```bash
cargo run --release
```

## Anki Deck

The final Anki deck format is inspired by [Kaishi 1.5k](https://github.com/donkuri/Kaishi), which I quite liked when I was learning Japanese. It is designed around the idea of learning vocabulary with some context rather than just learning words in isolation.

Each note contains the word, its pinyin and meaning, an example sentence with its pinyin and meaning, audio for both the word and sentence, a picture, and some additional notes.

> [!NOTE]
> This deck is still in early development. I plan to add images and more notes for clarification in the future. The style of the flashcards is also quite ugly at the moment, but at least it is functional.

## Data Sources

> [!IMPORTANT]
> This project retrieves vocabulary, sentences, and audio from external websites. Keep in mind that the availability and structure of those websites can change, which may cause the scraper to stop working.

- [WordBrush](https://wordbrushchinese.com)

## License
 
This project is licensed under the [MIT License](LICENSE).
