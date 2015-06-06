use utils::{
    execute,
    fetch_updates,
    with_slot,
};

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
