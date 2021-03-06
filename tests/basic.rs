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
        let change      = updates[0].as_object().unwrap();
        let change_type = change.get("change_type").unwrap().as_string().unwrap();
        assert_eq!(change_type, "Insert");
    });
}

#[test]
fn basic_delete() {
    with_slot("test_table", "id int primary key, name text", |c| {
        execute(c, "insert into test_table (id, name) values ($1, $2)", &[&1, &"Michael Baker"]);
        execute(c, "delete from test_table", &[]);
        let updates = fetch_updates(c);
        assert_eq!(updates.len(), 2);
        let change      = updates[1].as_object().unwrap();
        let change_type = change.get("change_type").unwrap().as_string().unwrap();
        assert_eq!(change_type, "Delete");
    });
}

#[test]
fn basic_update() {
    with_slot("test_table", "id int primary key, name text", |c| {
        execute(c, "insert into test_table (id, name) values ($1, $2)", &[&1, &"Michael Baker"]);
        execute(c, "update test_table set name = 'Bichael Maker'", &[]);
        let updates = fetch_updates(c);
        assert_eq!(updates.len(), 2);
        let change      = updates[1].as_object().unwrap();
        let change_type = change.get("change_type").unwrap().as_string().unwrap();
        assert_eq!(change_type, "Update");
    });
}
