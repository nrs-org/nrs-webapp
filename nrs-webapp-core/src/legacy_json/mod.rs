use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::legacy_json::math::{Matrix, Vector};

pub mod factors;
pub mod math;

type Map<K, V> = BTreeMap<K, V>;

// Module for importing legacy JSON NRS data
#[derive(Deserialize, Serialize)]
pub struct Bulk {
    pub entries: Map<String, Entry>,
    pub impacts: Vec<Impact>,
    pub relations: Vec<Relation>,
    pub scores: Map<String, ScoreResult>,
}

pub type DAHMeta = serde_json::Value;

pub fn empty_meta() -> DAHMeta {
    serde_json::json!({})
}

#[derive(Deserialize, Serialize)]
pub struct Entry {
    pub id: String,
    #[serde(default, rename = "DAH_meta")]
    pub meta: DAHMeta,
}

#[derive(Deserialize, Serialize)]
pub struct Impact {
    pub contributors: Map<String, Matrix>,
    #[serde(default, rename = "DAH_meta")]
    pub meta: DAHMeta,
}

#[derive(Deserialize, Serialize)]
pub struct Relation {
    pub contributors: Map<String, Matrix>,
    pub references: Map<String, Matrix>,
    #[serde(default, rename = "DAH_meta")]
    pub meta: DAHMeta,
}

#[derive(Deserialize, Serialize)]
pub struct ScoreResult {
    #[serde(rename = "positiveScore")]
    pub positive_score: Vector,
    #[serde(rename = "negativeScore")]
    pub negative_score: Vector,
    #[serde(default, rename = "DAH_meta")]
    pub meta: DAHMeta,
}

#[test]
fn test_deserialize_bulk() {
    let json_data = r#"
    {
        "entries": {
            "entry1": { "id": "entry1", "meta": {} }
        },
        "impacts": [
            { "contributors": {}, "meta": {} }
        ],
        "relations": [
            { "contributors": {}, "references": {}, "meta": {} }
        ],
        "scores": {
            "entry1": {
                "positiveScore": {},
                "negativeScore": {},
                "meta": {}
            }
        }
    }
    "#;

    let bulk: Bulk = serde_json::from_str(json_data).unwrap();
    assert_eq!(bulk.entries.len(), 1);
    assert_eq!(bulk.impacts.len(), 1);
    assert_eq!(bulk.relations.len(), 1);
    assert_eq!(bulk.scores.len(), 1);
}

#[test]
fn test_deserialize_dah_anime_normalize_bulk() {
    let json_data = include_str!("./DAH_anime_normalize_bulk.json");

    let bulk: Bulk = serde_json::from_str(json_data).unwrap();
    assert_eq!(bulk.entries.len(), 14);
}
