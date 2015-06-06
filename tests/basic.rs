use utils::{
    execute,
    fetch_updates,
    with_slot,
    with_table,
};

#[test]
fn sanity_test() {
    with_table("test_table", "id int primary key, name text", |c| {
        execute(c, "insert into test_table (id, name) values ($1, $2)", &[&1, &"Michael Baker"]);
        execute(c, "insert into test_table (id, name) values ($1, $2)", &[&2, &"Josh Cheek"]);
        execute(c, "select * from test_table", &[]);
    });
}

#[test]
fn basic_insert() {
    with_slot("test_table", "id int primary key, name text", |c| {
        execute(c, "insert into test_table (id, name) values ($1, $2)", &[&1, &"Michael Baker"]);
        let updates = fetch_updates(c);
        assert_eq!(updates.len(), 1);
        let change = updates[0].as_object().unwrap();
        let variant = change.get("variant").unwrap().as_string().unwrap();
        assert_eq!(variant, "Insert");
    });
}

#[test]
fn basic_delete() {
    with_slot("test_table", "id int primary key, name text", |c| {
        execute(c, "insert into test_table (id, name) values ($1, $2)", &[&1, &"Michael Baker"]);
        execute(c, "delete from test_table", &[]);
        let updates = fetch_updates(c);
        assert_eq!(updates.len(), 2);
        let change = updates[1].as_object().unwrap();
        let variant = change.get("variant").unwrap().as_string().unwrap();
        assert_eq!(variant, "Delete");
    });
}

#[test]
fn basic_update() {
    with_slot("test_table", "id int primary key, name text", |c| {
        execute(c, "insert into test_table (id, name) values ($1, $2)", &[&1, &"Michael Baker"]);
        execute(c, "update test_table set name = 'Bichael Maker'", &[]);
        let updates = fetch_updates(c);
        assert_eq!(updates.len(), 2);
        let change = updates[1].as_object().unwrap();
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
