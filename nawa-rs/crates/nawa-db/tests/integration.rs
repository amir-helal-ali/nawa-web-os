//! Integration tests for nawa-db.

use nawa_db::{DbConfig, DbEngine, Value};
use std::collections::BTreeMap;

#[test]
fn put_get_roundtrip() {
    let db = DbEngine::open_in_memory();
    db.put("user:1", Value::from_str("Ahmed")).unwrap();
    let v = db.get("user:1").unwrap();
    assert_eq!(v.display(), "Ahmed");
}

#[test]
fn json_document_roundtrip() {
    let db = DbEngine::open_in_memory();
    let doc = Value::from_json_str(r#"{"name":"Sara","age":28,"roles":["admin"]}"#).unwrap();
    db.put("user:2", doc).unwrap();

    let v = db.get("user:2").unwrap();
    assert!(matches!(v, Value::Json(_)));
    let s = v.display();
    assert!(s.contains("Sara"));
    assert!(s.contains("28"));
}

#[test]
fn delete_returns_existence() {
    let db = DbEngine::open_in_memory();
    db.put("k1", Value::from_str("v1")).unwrap();
    assert!(db.delete("k1").unwrap());
    assert!(!db.delete("k1").unwrap());
}

#[test]
fn scan_prefix_returns_sorted() {
    let db = DbEngine::open_in_memory();
    db.put("user:3", Value::from_str("C")).unwrap();
    db.put("user:1", Value::from_str("A")).unwrap();
    db.put("user:2", Value::from_str("B")).unwrap();
    db.put("post:1", Value::from_str("P1")).unwrap();

    let users = db.scan_prefix("user:", 100);
    assert_eq!(users.len(), 3);
    // Should be sorted lexicographically.
    assert_eq!(users[0].0, b"user:1");
    assert_eq!(users[1].0, b"user:2");
    assert_eq!(users[2].0, b"user:3");
}

#[test]
fn stats_track_operations() {
    let db = DbEngine::open_in_memory();
    db.put("k1", Value::from_str("v1")).unwrap();
    db.put("k2", Value::from_str("v2")).unwrap();
    let _ = db.get("k1");
    let _ = db.get("k2");
    let _ = db.get("missing");
    db.delete("k1").unwrap();
    let _ = db.scan_prefix("k", 10);

    let stats = db.stats();
    assert_eq!(stats.puts, 2);
    assert_eq!(stats.gets, 3);
    assert_eq!(stats.deletes, 1);
    assert_eq!(stats.scans, 1);
}

#[test]
fn persistence_across_reopen() {
    let tmp = tempfile::tempdir().unwrap();
    let config = DbConfig {
        data_dir: tmp.path().to_path_buf(),
        memtable_max_size: 4 * 1024 * 1024,
        wal_sync: true,
    };

    // Write data.
    {
        let db = DbEngine::open(config.clone()).unwrap();
        db.put("user:1", Value::from_str("Ahmed")).unwrap();
        db.put("user:2", Value::from_str("Sara")).unwrap();
        db.put("counter:visits", Value::from_i64(14823)).unwrap();
    }

    // Reopen and verify.
    {
        let db = DbEngine::open(config).unwrap();
        assert_eq!(db.get("user:1").map(|v| v.display()), Some("Ahmed".into()));
        assert_eq!(db.get("user:2").map(|v| v.display()), Some("Sara".into()));
        assert_eq!(db.get("counter:visits").map(|v| v.display()), Some("14823".into()));
    }
}

#[test]
fn delete_persists_across_reopen() {
    let tmp = tempfile::tempdir().unwrap();
    let config = DbConfig {
        data_dir: tmp.path().to_path_buf(),
        ..Default::default()
    };

    {
        let db = DbEngine::open(config.clone()).unwrap();
        db.put("user:1", Value::from_str("Ahmed")).unwrap();
        db.delete("user:1").unwrap();
    }

    let db = DbEngine::open(config).unwrap();
    assert!(db.get("user:1").is_none());
}

#[test]
fn large_number_of_keys() {
    let db = DbEngine::open_in_memory();
    for i in 0..1000u32 {
        db.put(format!("key:{i:04}"), Value::from_i64(i as i64)).unwrap();
    }
    assert_eq!(db.len(), 1000);

    // All should be retrievable.
    for i in 0..1000u32 {
        let v = db.get(format!("key:{i:04}")).unwrap();
        assert_eq!(v.display(), i.to_string());
    }

    // Scan all keys.
    let all = db.scan_prefix("key:", 10_000);
    assert_eq!(all.len(), 1000);
}

#[test]
fn skip_list_operations() {
    use nawa_db::SkipList;
    let sl: SkipList<Vec<u8>, String> = SkipList::new();
    sl.insert(b"key:1".to_vec(), "value1".into());
    sl.insert(b"key:2".to_vec(), "value2".into());
    sl.insert(b"key:3".to_vec(), "value3".into());

    assert_eq!(sl.len(), 3);
    assert_eq!(sl.get(&b"key:2".to_vec()), Some("value2".into()));

    let scanned = sl.scan_prefix(&b"key:".to_vec());
    assert_eq!(scanned.len(), 3);

    sl.remove(&b"key:2".to_vec());
    assert_eq!(sl.len(), 2);
    assert!(sl.get(&b"key:2".to_vec()).is_none());
}

#[test]
fn bloom_filter_false_positive_rate() {
    use nawa_db::BloomFilter;
    let mut bf = BloomFilter::new(10_000, 0.01);

    // Insert 10k keys.
    for i in 0..10_000u32 {
        bf.insert(&i.to_le_bytes());
    }

    // All inserted keys must be present.
    for i in 0..10_000u32 {
        assert!(bf.might_contain(&i.to_le_bytes()));
    }

    // Check false positive rate on 10k non-inserted keys.
    let mut false_positives = 0;
    for i in 10_000..20_000u32 {
        if bf.might_contain(&i.to_le_bytes()) {
            false_positives += 1;
        }
    }
    let rate = false_positives as f64 / 10_000.0;
    assert!(rate < 0.02, "false positive rate too high: {rate}");
}

#[test]
fn compactor_creates_sstable() {
    use nawa_db::Compactor;
    let tmp = tempfile::tempdir().unwrap();
    let compactor = Compactor::new(tmp.path(), 1);

    let mut entries = BTreeMap::new();
    entries.insert(b"user:1".to_vec(), Value::from_str("Ahmed"));
    entries.insert(b"user:2".to_vec(), Value::from_str("Sara"));

    let result = compactor.compact_from_map(entries).unwrap();
    assert_eq!(result.entries, 2);
    assert!(result.output.exists());
    assert!(result.output_bytes > 0);
}
