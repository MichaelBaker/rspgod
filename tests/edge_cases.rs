use utils::{
    execute,
    fetch_updates,
    with_slot,
};

use rustc_serialize::json::{as_pretty_json};

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

// TODO: Add these all to the test
// -------------------------------
// date
// double precision
// inet
// integer
// interval [ fields ] [ (p) ]
// json
// jsonb
// line
// lseg
// macaddr
// money
// numeric [ (p, s) ]
// path
// pg_lsn
// point
// polygon
// real
// text
// time [ (p) ] [ without time zone ]
// time [ (p) ] with time zone
// timestamp [ (p) ] [ without time zone ]
// timestamp [ (p) ] with time zone
// tsquery
// tsvector
// txid_snapshot
// uuid
// xml
#[test]
fn works_with_all_datatypes() {
    let columns = "                        \
        id              int primary key,   \
        abool           boolean,           \
        anint           int,               \
        abigint         bigint,            \
        asmallint       smallint,          \
        aserialnint     serial,            \
        aserialbigint   bigserial,         \
        aserialsmallint smallserial,       \
        somebits        BIT(3),            \
        somemorebits    bit varying,       \
        abox            box,               \
        somebytes       bytea,             \
        somechars       character(3),      \
        morechars       character varying, \
        addr            cidr,              \
        circ            circle
    ";

    let values = "             \
        1,                     \
        true,                  \
        1,                     \
        1,                     \
        1,                     \
        1,                     \
        1,                     \
        1,                     \
        B'101',                \
        B'101',                \
        '((0,0),(1,1))',       \
        E'\x0123456789ABCDEF', \
        'abc',                 \
        'abcdef',              \
        '192.168.100.128/25',  \
        '((0,0), 1)'           \
    ";

    with_slot("datatypes", columns, |c| {
        execute(c, &format!("insert into datatypes values ({})", values)[..], &[]);
        println!("{}", as_pretty_json(&fetch_updates(c)));
        assert_eq!(fetch_updates(c).len(), 1);
    });
}
