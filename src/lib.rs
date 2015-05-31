#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

extern crate libc;
extern crate rustc_serialize;

use rustc_serialize::json;
use std::ffi::CString;

pub mod automatic_postgres;
pub mod manual_postgres;

use automatic_postgres::{
    LogicalDecodingContext,
    OutputPluginOptions,
    ReorderBufferTXN,
    Relation,
    ReorderBufferChange,
    TupleDesc,
    HeapTuple,
    OUTPUT_PLUGIN_TEXTUAL_OUTPUT,
    REORDER_BUFFER_CHANGE_INSERT,
    REORDER_BUFFER_CHANGE_UPDATE,
    REORDER_BUFFER_CHANGE_DELETE,
};

use manual_postgres::{
    heap_getattr,
};

#[derive(RustcDecodable, RustcEncodable)]
pub struct WrappedPG {
    pub num_attributes: u32,
    pub fields:         Vec<Field>,
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct Field {
    pub name:  String,
    pub value: Option<String>,
}

extern {
  fn OutputPluginPrepareWrite(ctx: &LogicalDecodingContext, last_write: bool);
  fn OutputPluginWrite(ctx: &LogicalDecodingContext, last_write: bool);
  fn appendStringInfoString(str: *mut automatic_postgres::Struct_StringInfoData, s: *const i8);
}

#[no_mangle]
pub extern fn pg_decode_startup(ctx: &LogicalDecodingContext, opt: &mut OutputPluginOptions, is_init: bool) {
	opt.output_type = OUTPUT_PLUGIN_TEXTUAL_OUTPUT;
}

#[no_mangle]
pub extern fn pg_decode_change(ctx:      &LogicalDecodingContext,
                               txn:      &ReorderBufferTXN,
                               relation: Relation,
                               change:   &mut ReorderBufferChange) {

    let action_string = match change.action {
        REORDER_BUFFER_CHANGE_INSERT => { "Insert " },
        REORDER_BUFFER_CHANGE_UPDATE => { "Update " },
        REORDER_BUFFER_CHANGE_DELETE => { "Delete " },
        _                            => { "Other " },
    };

    let c_action_string = CString::new(action_string).unwrap();

    unsafe {
        OutputPluginPrepareWrite(ctx, true);
        appendStringInfoString(ctx.out, c_action_string.as_ptr());
    }

    match change.action {
        REORDER_BUFFER_CHANGE_INSERT => {
            let tuple = pg_tuple_to_rspgod_tuple(
                unsafe { (*relation).rd_att },
                unsafe { &mut(*(*change.data.tp()).newtuple).tuple }
            );

            let output         = format!("{}", json::encode(&tuple).unwrap());
            let c_tuple_string = CString::new(&output[..]).unwrap();

            unsafe {
                appendStringInfoString(ctx.out, c_tuple_string.as_ptr());
            }
        },
        // REORDER_BUFFER_CHANGE_UPDATE => { "Update".to_string() },
        // REORDER_BUFFER_CHANGE_DELETE => { "Delete".to_string() },
        _                            => {},
    };

    unsafe { OutputPluginWrite(ctx, true); }
}


pub fn extract_string(i8str:[::libc::c_char; 64usize]) -> String {
    let u8str:[u8; 64usize] = unsafe { std::mem::transmute(i8str) };
    let str = String::from_utf8(u8str.to_vec()).unwrap(); // unwrap = danger!
    str.chars().take_while(|c| *c != '\u{0}').collect()
}


// type TupleDesc = *mut Struct_tupleDesc
//   which is: struct Struct_tupleDesc {
//                      natts: ::libc::c_int,
//                      attrs: *mut Form_pg_attribute,
//                      tdtypeid: Oid,

// pub struct Struct_HeapTupleData {
//     pub t_len: uint32,
//     pub t_self: ItemPointerData,
//     pub t_tableOid: Oid,
//     pub t_data: HeapTupleHeader,
// }

// pub fn heap_getsysattr(tup: HeapTuple, attnum: ::libc::c_int, tupleDesc: TupleDesc, isnull: *mut _bool) -> Datum;
// pub fn nocachegetattr(tup: HeapTuple, attnum: ::libc::c_int, att: TupleDesc) -> Datum;
// #define TextDatumGetCString(d) text_to_cstring((text *) DatumGetPointer(d))
// src/include/access/htup_details.h
pub fn pg_tuple_to_rspgod_tuple(description:TupleDesc, tuple:HeapTuple) -> WrappedPG {
    let raw_desc         = unsafe { *description };
    let num_attributes   = raw_desc.natts as u32;
    let mut fields       = vec![];

    for n in 0..num_attributes {
        let pg_attribute = unsafe { **raw_desc.attrs.offset(n as isize) };
        let name         = extract_string(pg_attribute.attname.data);

        match heap_getattr(tuple, n + 1, description) {
            Some(datum) => {
                fields.push(Field { name: name.clone(), value: Some(name.clone()) });
            },
            None => {
                fields.push(Field { name: name.clone(), value: None });
            },
        }
    }

    WrappedPG {
        num_attributes: num_attributes,
        fields:         fields,
    }
}
    // tuple_to_avro_row (from bottledwater)
    //   this works by modifying a pointer to the (avro_value_t *output_value)
    //   or returning (at the end, or via the `check` macro) an error int
    // -----------------------------------------------------
    // int err = 0, field = 0;
    // check(err, avro_value_reset(output_val));
    // for (int i = 0; i < tupdesc->natts; i++) {
    //     avro_value_t field_val;
    //     bool isnull;
    //     Datum datum;
    //     Form_pg_attribute attr = tupdesc->attrs[i];
    //     if (attr->attisdropped) continue; /* skip dropped columns */
    //     check(err, avro_value_get_by_index(output_val, field, &field_val, NULL));
    //     datum = heap_getattr(tuple, i + 1, tupdesc, &isnull);
    //     if (isnull) {
    //         check(err, avro_value_set_branch(&field_val, 0, NULL));
    //     } else {
    //         check(err, update_avro_with_datum(&field_val, attr->atttypid, datum));
    //     }
    //     field++;
    // }
    // return err;






#[no_mangle]
pub extern fn pg_decode_commit_txn(ctx: &LogicalDecodingContext, txn: &ReorderBufferTXN, commit_lsn: u64) {
}

#[no_mangle]
pub extern fn pg_decode_begin_txn(ctx: &LogicalDecodingContext, txn: &ReorderBufferTXN) {
}

#[no_mangle]
pub extern fn pg_decode_shutdown(ctx: &LogicalDecodingContext) {
}

