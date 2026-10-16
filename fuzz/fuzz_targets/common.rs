//! Shared by the targets: categories of divergence that are already
//! documented in `fuzz-test.md` can be skipped, so a campaign gets past them.
//! `ANPA_FUZZ_SKIP=cat1,cat2` (or `all`) names the ones to skip.

use std::sync::OnceLock;

#[allow(dead_code)]
pub fn known(category: &str) -> bool {
    static SKIP: OnceLock<Vec<String>> = OnceLock::new();
    let skip = SKIP.get_or_init(|| {
        std::env::var("ANPA_FUZZ_SKIP")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())
            .collect()
    });
    skip.iter().any(|s| s == "all" || s == category)
}
