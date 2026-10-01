use std::{
    fs,
    path::{Path, PathBuf},
};

use music_corpus::{
    analyze_dcml_divergence_manifest, write_corpus_divergence_summary_tsv,
    write_piece_divergence_summaries_tsv,
};

fn temporary_directory(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("music-corpus-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}

fn copy_fixture_piece(directory: &Path, corpus: &str, piece: &str, prefix: &str) -> [PathBuf; 3] {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut paths = Vec::new();
    for name in ["notes", "measures", "expanded"] {
        let source = fs::read_to_string(fixtures.join(format!("{name}.tsv"))).unwrap();
        let destination = directory.join(format!("{prefix}-{name}.tsv"));
        fs::write(
            &destination,
            source.replace("fixture\tmovement", &format!("{corpus}\t{piece}")),
        )
        .unwrap();
        paths.push(destination);
    }
    paths.try_into().unwrap()
}

#[test]
fn batch_orders_pieces_and_preserves_piece_and_corpus_effects() {
    let directory = temporary_directory("batch-order");
    let later = copy_fixture_piece(&directory, "sample", "zeta", "zeta");
    let earlier = copy_fixture_piece(&directory, "sample", "alpha", "alpha");
    let manifest = directory.join("manifest.tsv");
    fs::write(
        &manifest,
        format!(
            "corpus\tpiece\tnotes\tmeasures\texpanded\n\
             sample\tzeta\t{}\t{}\t{}\n\
             sample\talpha\t{}\t{}\t{}\n",
            later[0].display(),
            later[1].display(),
            later[2].display(),
            earlier[0].display(),
            earlier[1].display(),
            earlier[2].display(),
        ),
    )
    .unwrap();

    let batch = analyze_dcml_divergence_manifest(&manifest).unwrap();
    assert_eq!(
        batch
            .pieces
            .iter()
            .map(|piece| piece.piece.as_str())
            .collect::<Vec<_>>(),
        ["alpha", "zeta"]
    );

    let mut per_piece = Vec::new();
    write_piece_divergence_summaries_tsv(&batch, &mut per_piece).unwrap();
    let per_piece = String::from_utf8(per_piece).unwrap();
    assert!(per_piece.find("sample\talpha").unwrap() < per_piece.find("sample\tzeta").unwrap());
    assert!(per_piece.contains("sample\talpha\tAgreement\t3\tDownbeat\tI\t1\t4"));
    assert!(per_piece.contains("sample\tzeta\tAgreement\t4\tDownbeat\tV7|PAC}\t1\t4"));

    let mut corpus = Vec::new();
    write_corpus_divergence_summary_tsv(&batch, &mut corpus).unwrap();
    let corpus = String::from_utf8(corpus).unwrap();
    assert!(corpus.contains("sample\tAgreement\t3\tDownbeat\tI\t2\t2\t8"));
    assert!(corpus.contains("sample\tAgreement\t4\tDownbeat\tV7|PAC}\t2\t2\t8"));

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn batch_reports_the_bad_piece_and_manifest_row() {
    let directory = temporary_directory("batch-error");
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let manifest = directory.join("manifest.tsv");
    fs::write(
        &manifest,
        format!(
            "corpus\tpiece\tnotes\tmeasures\texpanded\n\
             sample\tbroken\tmissing-notes.tsv\t{}\t{}\n",
            fixtures.join("measures.tsv").display(),
            fixtures.join("expanded.tsv").display(),
        ),
    )
    .unwrap();

    let error = analyze_dcml_divergence_manifest(&manifest).unwrap_err();
    let message = error.to_string();
    assert!(message.contains("sample/broken"), "{message}");
    assert!(message.contains("manifest row 2"), "{message}");

    fs::remove_dir_all(directory).unwrap();
}
