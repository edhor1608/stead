use std::sync::Arc;

use rusqlite::Connection;
use stead_contracts::{Contract, ContractStatus, SqliteContractStore};

#[test]
fn concurrent_writers_persist_all_contracts_without_loss() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("contracts.db");
    let store = Arc::new(SqliteContractStore::open(&db_path).unwrap());

    let mut handles = Vec::new();
    for worker in 0..4 {
        let store = Arc::clone(&store);
        handles.push(std::thread::spawn(move || {
            for idx in 0..25 {
                let contract = Contract::new(format!("writer-{worker}-{idx}"), vec![]);
                store.save_contract(&contract).unwrap();
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let conn = Connection::open(db_path).unwrap();
    let total: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM contracts WHERE id LIKE 'writer-%'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(total, 100);
}

#[test]
fn readers_can_load_while_writer_updates_snapshot() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("contracts.db");
    let store = Arc::new(SqliteContractStore::open(&db_path).unwrap());

    let seed = Contract::new("shared-contract", vec![]);
    store.save_contract(&seed).unwrap();

    let writer_store = Arc::clone(&store);
    let writer = std::thread::spawn(move || {
        for i in 0..100 {
            let mut contract = Contract::new("shared-contract", vec![]);
            contract.status = if i % 2 == 0 {
                ContractStatus::Ready
            } else {
                ContractStatus::Claimed
            };
            writer_store.save_contract(&contract).unwrap();
        }
    });

    let mut readers = Vec::new();
    for _ in 0..4 {
        let reader_store = Arc::clone(&store);
        readers.push(std::thread::spawn(move || {
            for _ in 0..100 {
                let loaded = reader_store.load_contract("shared-contract").unwrap();
                assert!(loaded.is_some());
            }
        }));
    }

    writer.join().unwrap();
    for reader in readers {
        reader.join().unwrap();
    }
}
