use std::path::Path;

use anyhow::{Ok, Context, Result};
use genanki_rs::{Deck, Field, Model, Package, Template};

use crate::models::Flashcard;

pub fn create_model(model_id: i64, model_name: &str) -> Model {
    let custom_css = r#"
        .card {
            font-family:
                -apple-system,
                BlinkMacSystemFont,
                "Noto Sans SC",
                "Noto Sans CJK SC",
                "PingFang SC",
                "Microsoft YaHei",
                "微软雅黑",
                sans-serif;
            font-size: 44px;
            text-align: center;
            overflow: hidden;
        }

        img {
            max-width: 300px;
            max-height: 250px;
        }

        .mobile img {
            max-width: 50vw;
        }

        .sentence {
            font-size: 20px;
            padding-top: 12px;
        }

        .pinyin {
            font-size: 25px;
            padding-top: 10px;
        }

        .sentence-pinyin {
            font-size: 20px;
            padding-top: 8px;
        }

        .meaning {
            font-size: 25px;
            padding-bottom: 20px;
        }

        .notes {
            font-size: 20px;
            padding-top: 12px;
        }

        b {
            color: #5586cd;
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
            Template::new("Chinese Card")
                .qfmt(
                    r#"
                    <div lang="zh-CN">
                        {{Word}}
                        <div class="sentence">
                            {{Sentence}}
                        </div>
                    </div>
                "#,
                )
                .afmt(
                    r#"
                    <div lang="zh-CN">
                        {{Word}}

                        <div class="pinyin">
                            {{Word Pinyin}}
                        </div>

                        <div class="meaning">
                            {{Word Meaning}}
                        </div>

                        <div class="sentence">
                            {{Sentence}}
                        </div>

                        <div class="sentence-pinyin">
                            {{Sentence Pinyin}}
                        </div>

                        <div class="meaning">
                            {{Sentence Meaning}}
                        </div>

                        {{Word Audio}}
                        {{Sentence Audio}}

                        <br>
                        {{Picture}}

                        {{#Notes}}
                            <div class="notes">
                                Note: {{Notes}}
                            </div>
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
    for card in flashcards {
        deck.add_note(card.to_note(model)?);
    }

    Ok(())
}

pub fn write_apkg(deck: Deck, media_files: &[String], output: &Path) -> Result<()> {
    let media_files: Vec<&str> = media_files.iter().map(String::as_str).collect();
    let output_path = output
        .to_str()
        .with_context(|| format!("output path {output:?} is not valid UTF-8"))?;

    let mut package = Package::new(vec![deck], media_files)?;
    package.write_to_file(output_path)?;

    Ok(())
}
