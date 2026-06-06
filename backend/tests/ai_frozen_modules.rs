//! AC4: US-0008 must not modify frozen AI tool-layer modules.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

const FROZEN: &[&str] = &[
    "src/ai/registry.rs",
    "src/ai/privacy.rs",
];

fn file_hash(path: &PathBuf) -> u64 {
    let content = std::fs::read_to_string(path).expect("read frozen file");
    let mut h = DefaultHasher::new();
    content.hash(&mut h);
    h.finish()
}

/// Update only when TL explicitly changes frozen modules outside US-0008.
const EXPECTED: &[(&str, u64)] = &[];

#[test]
fn frozen_modules_unchanged_or_allowlisted() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for rel in FROZEN {
        let path = root.join(rel);
        assert!(path.exists(), "missing frozen file {rel}");
        let hash = file_hash(&path);
        if let Some((_, expected)) = EXPECTED.iter().find(|(p, _)| *p == *rel) {
            assert_eq!(
                hash, *expected,
                "{rel} hash changed — US-0008 must not modify registry/privacy"
            );
        }
    }
}

#[test]
fn frozen_modules_no_us0008_marker() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for rel in FROZEN {
        let content = std::fs::read_to_string(root.join(rel)).unwrap();
        assert!(
            !content.contains("US-0008-MODIFIED"),
            "{rel} must not contain US-0008 edit marker"
        );
    }
    let tools_dir = root.join("src/ai/tools");
    for entry in std::fs::read_dir(tools_dir).unwrap() {
        let content = std::fs::read_to_string(entry.unwrap().path()).unwrap();
        assert!(!content.contains("US-0008-MODIFIED"));
    }
}
