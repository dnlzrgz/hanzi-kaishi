use std::path::Path;

use anyhow::{Context, Ok, Result};
use genanki_rs::{Deck, Field, Model, Package, Template};
use tracing::debug;

use crate::models::Flashcard;

pub fn create_vocabulary_model(model_id: i64, model_name: &str) -> Model {
    let custom_css = r#"
        .card {
            --space-1: 4px;
            --space-2: 8px;
            --space-3: 16px;
            --space-4: 24px;
            --space-5: 32px;

            --text-notes: 15px;
            --text-caption: 19px;
            --text-body: 23px;
            --text-word: 54px;

            --color-ink: #1e293b;
            --color-muted: #64748b;
            --color-faint: #94a3b8;
            --color-accent: #3b82f6;

            font-family:
                -apple-system,
                BlinkMacSystemFont,
                "Noto Sans SC",
                "Noto Sans CJK SC",
                "PingFang SC",
                "Microsoft YaHei",
                "微软雅黑",
                sans-serif;
            color: var(--color-ink);
            line-height: 1.5;
            text-align: center;
            overflow: hidden;
        }

        .card.night-mode,
        .card.nightMode {
            --color-ink: #e2e8f0;
            --color-muted: #94a3b8;
            --color-faint: #64748b;
            --color-accent: #60a5fa;
        }

        .content {
		text-alignment: center;
            gap: var(--space-3);
            margin: 0 auto;
            padding: var(--space-4) var(--space-3);
        }

        .word {
            font-size: var(--text-word);
            line-height: 1.5;
            font-weight: bold;
        }

        .pinyin,
        .sentence-pinyin {
            font-size: var(--text-caption);
            color: var(--color-muted);
        }

        .pinyin {
            font-size: var(--text-body);
        }

        .meaning {
            font-size: var(--text-body);
            margin-bottom: var(--space-4);
        }

        .sentence {
            font-size: var(--text-body);
            line-height: 1.5;
            font-weight: bold;
        }

        .notes {
            font-size: var(--text-notes);
            color: var(--color-faint);
        }

        .media {
        width: 100%;
            display: flex;
            align-items: center;
            justify-content: center;
            gap: var(--space-3);
        }
    "#;

    Model::new(
        model_id,
        model_name,
        vec![
            Field::new("Word"),
            Field::new("Word Pinyin"),
            Field::new("Word Meaning"),
            Field::new("Sentence"),
            Field::new("Sentence Pinyin"),
            Field::new("Sentence Meaning"),
            Field::new("Word Audio"),
            Field::new("Sentence Audio"),
            Field::new("Picture"),
            Field::new("Notes"),
        ],
        vec![
            Template::new(model_name)
                .qfmt(
                    r#"
                    <div lang="zh-CN" class="content">
                        <div class="word">{{Word}}</div>
                        <div class="sentence">{{Sentence}}</div>
                    </div>
                "#,
                )
                .afmt(
                    r#"
                    <div lang="zh-CN" class="content">
                        <div class="pinyin">{{Word Pinyin}}</div>
                        <div class="word">{{Word}}</div>
                        <div class="meaning">{{Word Meaning}}</div>

                        <div class="sentence-pinyin">{{Sentence Pinyin}}</div>
                        <div class="sentence">{{Sentence}}</div>
                        <div class="meaning">{{Sentence Meaning}}</div>

                        <div class="media">
                            {{Word Audio}}
                            {{Sentence Audio}}
                        </div>

                        {{#Notes}}
                            <div class="notes">Note: {{Notes}}</div>
                        {{/Notes}}
                    </div>
                "#,
                ),
        ],
    )
    .css(custom_css)
}

pub fn create_deck(deck_id: i64, deck_name: &str) -> Deck {
    Deck::new(deck_id, deck_name, "")
}

pub fn build_deck(deck: &mut Deck, flashcards: &[Flashcard], model: &Model) -> Result<()> {
    debug!(count = flashcards.len(), "adding notes to deck");

    for card in flashcards {
        deck.add_note(card.to_note(model)?);
    }

    Ok(())
}

pub fn write_apkg(deck: Deck, media_files: &[String], output: &Path) -> Result<()> {
    debug!(
        count = media_files.len(),
        "bundling media files into package"
    );

    let media_files: Vec<&str> = media_files.iter().map(String::as_str).collect();
    let output_path = output
        .to_str()
        .with_context(|| format!("output path {output:?} is not valid UTF-8"))?;

    let mut package = Package::new(vec![deck], media_files)?;
    package.write_to_file(output_path)?;

    Ok(())
}
