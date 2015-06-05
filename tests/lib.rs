extern crate postgres;

use postgres::{Connection, SslMode};

#[derive(Debug)]
struct TestRecord {
    id:   i32,
    name: String,
}

#[test]
fn sanity_test() {
    with_clean_database(|c| {
        let records = vec![
            TestRecord {
                id:   1,
                name: "Michael Baker".to_string(),
            },
            TestRecord {
                id:   2,
                name: "Josh Cheek".to_string(),
            },
        ];

        for record in records.iter() {
            create_record(c, &record);
        }
        fetch_records(c);
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
    let stmt = c.prepare("create table test (id int primary key, name text)").unwrap();
    stmt.execute(&[]).unwrap();
}

fn fetch_records(c: &Connection) -> Vec<TestRecord> {
    let stmt = c.prepare("select id, name from test").unwrap();
    let results = stmt.query(&[]).unwrap();
    results.iter().map(|r| {
        TestRecord {
            id:   r.get(0),
            name: r.get(1),
        }
    }).collect()
}

fn create_record(c: &Connection, r: &TestRecord) {
    let stmt = c.prepare("insert into test (id, name) values ($1, $2)").unwrap();
    stmt.execute(&[
        &r.id,
        &r.name,
    ]).unwrap();
}

fn drop_database(c: &Connection) {
    let stmt = c.prepare("drop table if exists test").unwrap();
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