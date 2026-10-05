use std::{env, path::Path};

use music_ron::{parse_path, Document};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args().nth(1).ok_or("usage: validate_score <score.ron>")?;
    match parse_path(Path::new(&path))? {
        Document::Score(score) => {
            println!(
                "valid Score: {} parts, {} measures, {} sources",
                score.parts.len(),
                score.measures.len(),
                score.sources.len()
            );
            Ok(())
        }
        other => Err(format!("expected Score, got {}", other.kind_str()).into()),
    }
}
