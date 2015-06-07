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

#[test]
fn works_with_all_datatypes() {
    let columns = "
        id              int primary key,
        abool           boolean,
        anint           int,
        abigint         bigint,
        asmallint       smallint,
        aserialnint     serial,
        aserialbigint   bigserial,
        aserialsmallint smallserial,
        somebits        BIT(3),
        somemorebits    bit varying,
        abox            box,
        somebytes       bytea,
        somechars       character(3),
        morechars       character varying,
        addr            cidr,
        circ            circle,
        adate           date,
        afloat          double precision,
        aninet          inet,
        aninteger       integer,
        ayear           interval year,
        somejson        json,
        somejsonb       jsonb,
        aline           line,
        alineseg        lseg,
        amac            macaddr,
        somedollers     money,
        anumber         numeric(2, 2),
        apath           path,
        alsn            pg_lsn,
        thepoint        point,
        apoly           polygon,
        thereal         real,
        sometext        text,
        thetimezone     time without time zone,
        thetimenozone   time with time zone,
        timestampzone   timestamp without time zone,
        timestampnozone timestamp with time zone,
        aquery          tsquery,
        avec            tsvector,
        txid            txid_snapshot,
        auuid           uuid,
        somexml         xml
    ";

    let values = "
        1,
        true,
        1,
        1,
        1,
        1,
        1,
        1,
        B'101',
        B'101',
        '((0,0),(1,1))',
        E'\x0123456789ABCDEF',
        'abc',
        'abcdef',
        '192.168.100.128/25',
        '((0,0), 1)',
        '1999-01-08',
        1.2,
        '192.168.100.128/25',
        1,
        '1',
        '{\"a\":123}',
        '{\"a\":123}',
        '((0,0),(1,1))',
        '((0,0),(1,1))',
        '08:00:2b:01:02:03',
        '12.23',
        0.1,
        '((0,0),(1,1))',
        '16/B374D848',
        '(0,0)',
        '((0,0),(1,1))',
        1.39284,
        'what we do in life echos in eternity',
        '04:05:06',
        '04:05:06-08:00',
        '1999-01-08 04:05:06',
        '1999-01-08 04:05:06-08:00',
        'fat & rat',
        'fat rat',
        '10:20:10,14,15',
        'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11',
        '<foo>bar</foo>'
    ";

    with_slot("datatypes", columns, |c| {
        execute(c, &format!("insert into datatypes values ({})", values)[..], &[]);
        execute(c, "update datatypes set id = 2", &[]);
        execute(c, "delete from datatypes", &[]);
        assert_eq!(fetch_updates(c).len(), 3);
    });
}
