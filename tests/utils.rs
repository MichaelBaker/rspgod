use std::env;
use postgres::{Connection, SslMode};
use postgres::types::{ToSql};

#[derive(Debug)]
pub struct TestRecord {
    pub id:   i32,
    pub name: String,
}

pub fn execute(c: &Connection, command: &str, args: &[&ToSql]) {
    let stmt = c.prepare(command).unwrap();
    stmt.execute(args).unwrap();
}

pub fn execute_silent(c: &Connection, command: &str, args: &[&ToSql]) {
    let stmt = c.prepare(command).unwrap();
    match stmt.execute(args) { _ => {} };
}

pub fn fetch_updates(c: &Connection) -> Vec<String> {
    let stmt = c.prepare("SELECT * FROM pg_logical_slot_peek_changes('slot', NULL, NULL)").unwrap();
    stmt.query(&[]).unwrap().iter().map(|r| { r.get(2) }).collect()
}

pub fn create_slot(c: &Connection) {
    execute(c, "select * from pg_create_logical_replication_slot('slot', 'thingy')", &[]);
}

pub fn drop_slot(c: &Connection) {
    execute_silent(c, "select pg_drop_replication_slot('slot')", &[]);
}

pub fn with_slot<F>(table_name: &str, columns: &str, f: F) where F:Fn(&Connection) -> () {
    with_table(table_name, columns, |c| {
        drop_slot(c);
        create_slot(c);
        f(c);
        drop_slot(c);
    });
}

pub fn with_table<F>(table_name: &str, columns: &str, f: F) where F:Fn(&Connection) -> () {
    let c = connection();
    reset_database(&c, table_name, columns);
    f(&c);
    drop_database(&c, table_name);
}

pub fn reset_database(c: &Connection, table_name: &str, columns: &str) {
    drop_database(&c, table_name);
    create_database(&c, table_name, columns);
}

pub fn create_database(c: &Connection, table_name: &str, columns: &str) {
    execute(c, &format!("create table {} ({})", table_name, columns), &[]);
}

pub fn fetch_records(c: &Connection) -> Vec<TestRecord> {
    let stmt = c.prepare("select id, name from test_table").unwrap();
    let results = stmt.query(&[]).unwrap();
    results.iter().map(|r| {
        TestRecord {
            id:   r.get(0),
            name: r.get(1),
        }
    }).collect()
}

pub fn create_record(c: &Connection, r: &TestRecord) {
    execute(c, "insert into test_table (id, name) values ($1, $2)", &[
        &r.id,
        &r.name,
    ]);
}

pub fn delete_record(c: &Connection, id: i32) {
    execute(c, "delete from test_table where id = $1", &[&id]);
}

pub fn update_record(c: &Connection, new_record: TestRecord) {
    execute(c, "update test_table set name = $2 where id = $1", &[
        &new_record.id,
        &new_record.name
    ]);
}

pub fn drop_database(c: &Connection, table_name: &str) {
    execute(c, &format!("drop table if exists {}", table_name), &[]);
}

pub fn connection_string() -> String {
    match env::var("POSTGRES_URL") {
        Ok(val) => val,
        Err(e)  => panic!(format!("You must set the POSTGRES_URL environment variable to point to a running Postgres test database when running automated tests: {}", e)),
    }
}

pub fn connection() -> Connection {
    match Connection::connect(&connection_string()[..], &SslMode::None) {
        Ok(c)  => c,
        Err(e) => panic!(format!("Could not connect to the test database: {}", e)),
    }
}
