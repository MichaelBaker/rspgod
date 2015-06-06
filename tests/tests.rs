extern crate postgres;
extern crate rustc_serialize;

mod utils;

use rustc_serialize::json::Json;
use utils::{
    fetch_updates,
    fetch_records,
    with_slot,
    with_clean_database,
    execute,
    create_record,
    update_record,
    delete_record,
    TestRecord,
};

#[test]
fn sanity_test() {
    with_clean_database("test_table", "id int primary key, name text", |c| {
        let records = vec![
            TestRecord { id: 1, name: "Michael Baker".to_string() },
            TestRecord { id: 2, name: "Josh Cheek".to_string()   },
        ];

        for record in records.iter() {
            create_record(c, &record);
        }

        fetch_records(c);
    });
}

#[test]
fn basic_insert() {
    with_slot("test_table", "id int primary key, name text", |c| {
        let record = TestRecord { id: 1, name: "Michael Baker".to_string(), };
        create_record(c, &record);
        let updates = fetch_updates(c);
        assert_eq!(updates.len(), 1);
        let data = Json::from_str(&updates[0][..]).unwrap();
        let change = data.as_object().unwrap();
        let variant = change.get("variant").unwrap().as_string().unwrap();
        assert_eq!(variant, "Insert");
    });
}

#[test]
fn basic_delete() {
    with_slot("test_table", "id int primary key, name text", |c| {
        let record = TestRecord { id: 1, name: "Michael Baker".to_string() };
        create_record(c, &record);
        delete_record(c, 1);
        let updates = fetch_updates(c);
        assert_eq!(updates.len(), 2);
        let data = Json::from_str(&updates[1][..]).unwrap();
        let change = data.as_object().unwrap();
        let variant = change.get("variant").unwrap().as_string().unwrap();
        assert_eq!(variant, "Delete");
    });
}

#[test]
fn basic_update() {
    with_slot("test_table", "id int primary key, name text", |c| {
        let record = TestRecord { id: 1, name: "Michael Baker".to_string() };
        create_record(c, &record);
        update_record(c, TestRecord { id: 1, name: "Bichael Maker".to_string() });
        let updates = fetch_updates(c);
        assert_eq!(updates.len(), 2);
        let data = Json::from_str(&updates[1][..]).unwrap();
        let change = data.as_object().unwrap();
        let variant = change.get("variant").unwrap().as_string().unwrap();
        assert_eq!(variant, "Update");
    });
}

#[test]
// The output decoder does not emit any changes for deletions from a table without a primary key.
// This is because Postgres doesn't give the decoder any tuple information in this scenario.
fn delete_without_primary_key() {
    with_slot("no_primary_table", "id int, name text, whatever float", |c| {
        execute(c, "insert into no_primary_table (id, name, whatever) values ($1, $2, $3)", &[&1, &"hello", &3.2]);
        execute(c, "delete from no_primary_table where id = 1", &[]);
        assert_eq!(fetch_updates(c).len(), 1);
    });
}
