use std::collections::HashMap;
use std::time::Duration;

use pi::provider::ModelCost;

use super::super::cache_clock::unix_now_secs;
use super::{PricedEntry, PricingCacheFile};

const TTL: Duration = Duration::from_hours(24);
const CACHE: PricingCacheFile = PricingCacheFile::new("pricing-cache-file-test.json", TTL);

fn cost(input: f64) -> ModelCost {
    ModelCost {
        input,
        output: 2.0 * input,
        cache_read: 0.0,
        cache_write: 0.0,
    }
}

fn stale_secs() -> u64 {
    unix_now_secs().saturating_sub(TTL.as_secs() + 60)
}

#[test]
fn insert_keeps_older_entry_stale() {
    crate::test_utils::with_isolated_home(|_| {
        CACHE.save_entries_for_test(HashMap::from([(
            "old/model".to_string(),
            PricedEntry {
                fetched_at_secs: stale_secs(),
                cost: cost(1.0),
            },
        )]));
        CACHE.insert("new/model", cost(3.0));
        assert_eq!(CACHE.fresh_cost("new/model"), Some(cost(3.0)));
        assert_eq!(CACHE.fresh_cost("old/model"), None);
        assert_eq!(CACHE.load().len(), 2);
        assert!(!CACHE.all_fresh());
    });
}

#[test]
fn replace_all_stamps_every_entry_fresh() {
    crate::test_utils::with_isolated_home(|_| {
        assert!(!CACHE.all_fresh());
        CACHE.replace_all(HashMap::from([
            ("a/x".to_string(), cost(1.0)),
            ("b/y".to_string(), cost(2.0)),
        ]));
        assert!(CACHE.all_fresh());
        assert_eq!(CACHE.fresh_cost("b/y"), Some(cost(2.0)));
        assert!(CACHE.path().is_file());
    });
}

#[test]
fn legacy_whole_file_timestamp_format_is_a_miss() {
    crate::test_utils::with_isolated_home(|_| {
        let legacy = format!(
            r#"{{"fetched_at_secs":{},"by_id":{{"a/x":{{"input":1.0,"output":2.0,"cacheRead":0.0,"cacheWrite":0.0}}}}}}"#,
            unix_now_secs()
        );
        let path = CACHE.path();
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        std::fs::write(path, legacy).expect("write");
        assert!(CACHE.load().is_empty());
        assert_eq!(CACHE.fresh_cost("a/x"), None);
    });
}
