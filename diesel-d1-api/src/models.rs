use serde::{Deserialize, Serialize};

/// Body for `POST /accounts/{account_id}/d1/database/{database_id}/raw`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RawQueryRequest {
    Single(D1SingleQuery),
    Multiple(MultipleQueries),
}

/// A single query with or without parameters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct D1SingleQuery {
    pub sql: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<Vec<String>>,
}

/// A batch of queries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MultipleQueries {
    pub batch: Vec<D1SingleQuery>,
}

/// Cloudflare API v4 envelope for the D1 `/raw` endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawQueryResponse {
    pub errors: Vec<ResponseInfo>,
    pub messages: Vec<ResponseInfo>,
    pub result: Vec<RawQueryResult>,
    /// Whether the API call was successful.
    pub success: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponseInfo {
    pub code: u64,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub documentation_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<ResponseSource>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponseSource {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pointer: Option<String>,
}

/// One statement result in the `/raw` `result` array.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawQueryResult {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<QueryMeta>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub results: Option<RawResults>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub success: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryMeta {
    /// Denotes if the database has been altered in some way, like deleting rows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub changed_db: Option<bool>,
    /// Rough indication of how many rows were modified, from SQLite's `sqlite3_total_changes()`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub changes: Option<f64>,
    /// SQL execution duration inside the database, excluding network time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,
    /// Row ID of the last insert into a table with an `INTEGER PRIMARY KEY`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_row_id: Option<f64>,
    /// Rows read during execution, including indices.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rows_read: Option<f64>,
    /// Rows written during execution, including indices.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rows_written: Option<f64>,
    /// Three-letter airport code of the colo that handled the query.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub served_by_colo: Option<String>,
    /// Whether the query was handled by the database primary instance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub served_by_primary: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub served_by_region: Option<ServedByRegion>,
    /// Size of the database after the query committed, in bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size_after: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timings: Option<QueryTimings>,
}

/// Region location hint of the database instance that handled the query.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ServedByRegion {
    WNAM,
    ENAM,
    WEUR,
    EEUR,
    APAC,
    OC,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryTimings {
    /// SQL execution duration inside the database, excluding network time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sql_duration_ms: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawResults {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub columns: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rows: Option<Vec<Vec<RawValue>>>,
}

/// A cell in a `/raw` result row: JSON number, string, or any other JSON value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RawValue {
    Number(f64),
    String(String),
    Null,
    Unknown(serde_json::Value),
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn serializes_single_query() {
        let request = RawQueryRequest::Single(D1SingleQuery {
            sql: "SELECT * FROM myTable WHERE field = ? OR field = ?;".into(),
            params: Some(vec!["firstParam".into(), "secondParam".into()]),
        });

        assert_eq!(
            serde_json::to_value(&request).unwrap(),
            json!({
                "sql": "SELECT * FROM myTable WHERE field = ? OR field = ?;",
                "params": ["firstParam", "secondParam"]
            })
        );
    }

    #[test]
    fn omits_missing_params() {
        let request = RawQueryRequest::Single(D1SingleQuery {
            sql: "SELECT 1".into(),
            params: None,
        });

        assert_eq!(
            serde_json::to_value(&request).unwrap(),
            json!({ "sql": "SELECT 1" })
        );
    }

    #[test]
    fn serializes_batch_query() {
        let request = RawQueryRequest::Multiple(MultipleQueries {
            batch: vec![
                D1SingleQuery {
                    sql: "SELECT 1".into(),
                    params: None,
                },
                D1SingleQuery {
                    sql: "SELECT ?".into(),
                    params: Some(vec!["2".into()]),
                },
            ],
        });

        assert_eq!(
            serde_json::to_value(&request).unwrap(),
            json!({
                "batch": [
                    { "sql": "SELECT 1" },
                    { "sql": "SELECT ?", "params": ["2"] }
                ]
            })
        );
    }

    #[test]
    fn deserializes_documented_200_example() {
        let parsed: RawQueryResponse = serde_json::from_value(json!({
            "errors": [
                {
                    "code": 1000,
                    "message": "message",
                    "documentation_url": "documentation_url",
                    "source": { "pointer": "pointer" }
                }
            ],
            "messages": [
                {
                    "code": 1000,
                    "message": "message",
                    "documentation_url": "documentation_url",
                    "source": { "pointer": "pointer" }
                }
            ],
            "result": [
                {
                    "meta": {
                        "changed_db": true,
                        "changes": 0,
                        "duration": 0,
                        "last_row_id": 0,
                        "rows_read": 0,
                        "rows_written": 0,
                        "served_by_colo": "LHR",
                        "served_by_primary": true,
                        "served_by_region": "EEUR",
                        "size_after": 0,
                        "timings": { "sql_duration_ms": 0 }
                    },
                    "results": {
                        "columns": ["string"],
                        "rows": [[0]]
                    },
                    "success": true
                }
            ],
            "success": true
        }))
        .unwrap();

        assert!(parsed.success);
        assert_eq!(parsed.errors.len(), 1);
        assert_eq!(parsed.errors[0].code, 1000);
        assert_eq!(parsed.errors[0].message, "message");
        assert_eq!(
            parsed.errors[0].documentation_url.as_deref(),
            Some("documentation_url")
        );
        assert_eq!(
            parsed.errors[0]
                .source
                .as_ref()
                .and_then(|source| source.pointer.as_deref()),
            Some("pointer")
        );
        assert_eq!(parsed.messages, parsed.errors);

        assert_eq!(parsed.result.len(), 1);
        let statement = &parsed.result[0];
        assert_eq!(statement.success, Some(true));

        let meta = statement.meta.as_ref().unwrap();
        assert_eq!(meta.changed_db, Some(true));
        assert_eq!(meta.changes, Some(0.0));
        assert_eq!(meta.duration, Some(0.0));
        assert_eq!(meta.last_row_id, Some(0.0));
        assert_eq!(meta.rows_read, Some(0.0));
        assert_eq!(meta.rows_written, Some(0.0));
        assert_eq!(meta.served_by_colo.as_deref(), Some("LHR"));
        assert_eq!(meta.served_by_primary, Some(true));
        assert_eq!(meta.served_by_region, Some(ServedByRegion::EEUR));
        assert_eq!(meta.size_after, Some(0.0));
        assert_eq!(meta.timings.as_ref().unwrap().sql_duration_ms, Some(0.0));

        let results = statement.results.as_ref().unwrap();
        assert_eq!(results.columns, Some(vec!["string".into()]));
        assert_eq!(results.rows.as_ref().unwrap()[0], [RawValue::Number(0.0)]);
    }

    #[test]
    fn deserializes_null_string_and_unknown_cells() {
        let rows: Vec<Vec<RawValue>> =
            serde_json::from_value(json!([[1, "Ada", null, true, [65, 66]]])).unwrap();

        assert_eq!(
            rows[0],
            [
                RawValue::Number(1.0),
                RawValue::String("Ada".into()),
                RawValue::Null,
                RawValue::Unknown(json!(true)),
                RawValue::Unknown(json!([65, 66])),
            ]
        );
    }
}
