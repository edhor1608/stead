use rusqlite::Connection;
use stead_contracts::{CURRENT_SCHEMA_VERSION, SqliteContractStore};

fn has_table(conn: &Connection, table: &str) -> bool {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1)",
        [table],
        |row| row.get::<_, i64>(0),
    )
    .map(|v| v == 1)
    .unwrap_or(false)
}

#[test]
fn bootstraps_schema_and_sets_version() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("contracts.db");

    let store = SqliteContractStore::open(&db_path).expect("store should initialize schema");

    assert_eq!(
        store
            .schema_version()
            .expect("schema version should be readable"),
        CURRENT_SCHEMA_VERSION
    );

    let conn = Connection::open(db_path).unwrap();
    assert!(has_table(&conn, "schema_meta"));
    assert!(has_table(&conn, "contracts"));
    assert!(has_table(&conn, "contract_events"));
    assert!(has_table(&conn, "decision_items"));
}

#[test]
fn schema_bootstrap_is_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("contracts.db");

    let first = SqliteContractStore::open(&db_path).unwrap();
    assert_eq!(first.schema_version().unwrap(), CURRENT_SCHEMA_VERSION);

    let second = SqliteContractStore::open(&db_path).unwrap();
    assert_eq!(second.schema_version().unwrap(), CURRENT_SCHEMA_VERSION);

    let conn = Connection::open(db_path).unwrap();
    let row_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM schema_meta WHERE key = 'schema_version'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(row_count, 1, "schema version row should not duplicate");
}
