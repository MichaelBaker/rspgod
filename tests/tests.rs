extern crate postgres;
extern crate rustc_serialize;

use postgres::{Connection, SslMode};
use rustc_serialize::json::Json;

#[derive(Debug)]
struct TestRecord {
    id:   i32,
    name: String,
}

#[test]
fn sanity_test() {
    with_clean_database(|c| {
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
    with_slot(|c| {
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
    with_slot(|c| {
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
    with_slot(|c| {
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

//
// [TODO] I want to move a lot of these into a utility module when I can figure out how to do that
//

fn fetch_updates(c: &Connection) -> Vec<String> {
    let stmt = c.prepare("SELECT * FROM pg_logical_slot_peek_changes('slot', NULL, NULL)").unwrap();
    let mut result = vec![];
    for r in stmt.query(&[]).unwrap() {
        result.push(r.get(2));
    }
    result
}

fn create_slot(c: &Connection) {
    let stmt = c.prepare("select * from pg_create_logical_replication_slot('slot', 'thingy')").unwrap();
    stmt.execute(&[]).unwrap();
}

fn drop_slot(c: &Connection) {
    let stmt = c.prepare("select pg_drop_replication_slot('slot')").unwrap();
    match stmt.execute(&[]) {
        _ => {},
    }
}

fn with_slot<F>(f: F) where F:Fn(&Connection) -> () {
    with_clean_database(|c| {
        drop_slot(c);
        create_slot(c);
        f(c);
        drop_slot(c);
    });
}

fn with_clean_database<F>(f: F) where F:Fn(&Connection) -> () {
    let c = connection();
    reset_database(&c);
    f(&c);
    drop_database(&c);
}

fn reset_database(c: &Connection) {
    drop_database(&c);
    create_database(&c);
}

fn create_database(c: &Connection) {
    let stmt = c.prepare("create table test_table (id int primary key, name text)").unwrap();
    stmt.execute(&[]).unwrap();
}

fn fetch_records(c: &Connection) -> Vec<TestRecord> {
    let stmt = c.prepare("select id, name from test_table").unwrap();
    let results = stmt.query(&[]).unwrap();
    results.iter().map(|r| {
        TestRecord {
            id:   r.get(0),
            name: r.get(1),
        }
    }).collect()
}

fn create_record(c: &Connection, r: &TestRecord) {
    let stmt = c.prepare("insert into test_table (id, name) values ($1, $2)").unwrap();
    stmt.execute(&[
        &r.id,
        &r.name,
    ]).unwrap();
}

fn delete_record(c: &Connection, id: i32) {
    let stmt = c.prepare("delete from test_table where id = $1").unwrap();
    stmt.execute(&[&id]).unwrap();
}

fn update_record(c: &Connection, new_record: TestRecord) {
    let stmt = c.prepare("update test_table set name = $2 where id = $1").unwrap();
    stmt.execute(&[&new_record.id, &new_record.name]).unwrap();
}

fn drop_database(c: &Connection) {
    let stmt = c.prepare("drop table if exists test_table").unwrap();
    stmt.execute(&[]).unwrap();
}

fn connection_string() -> String {
    match std::env::var("POSTGRES_URL") {
        Ok(val) => val,
        Err(e)  => panic!(format!("You must set the POSTGRES_URL environment variable to point to a running Postgres test database when running automated tests: {}", e)),
    }
}

fn connection() -> Connection {
    match Connection::connect(&connection_string()[..], &SslMode::None) {
        Ok(c)  => c,
        Err(e) => panic!(format!("Could not connect to the test database: {}", e)),
    }
}
