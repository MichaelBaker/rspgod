#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

extern crate libc;
extern crate rustc_serialize;

use rustc_serialize::json;
use std::ffi::CString;

pub mod postgres;
pub mod postgres_bindings;
pub mod types;

use types::{
    Change,
    Tuple,
    Field,
    CFalse,
    to_bool,
};

use postgres::{
    datum_to_string,
};

use postgres_bindings::{
    macrowrap_heap_getattr,
    LogicalDecodingContext,
    OutputPluginOptions,
    ReorderBufferTXN,
    Relation,
    ReorderBufferChange,
    TupleDesc,
    HeapTuple,
    Struct_StringInfoData,
    OUTPUT_PLUGIN_TEXTUAL_OUTPUT,
    REORDER_BUFFER_CHANGE_INSERT,
    REORDER_BUFFER_CHANGE_UPDATE,
    REORDER_BUFFER_CHANGE_DELETE,
};

extern {
  fn OutputPluginPrepareWrite(ctx: &LogicalDecodingContext, last_write: bool);
  fn OutputPluginWrite(ctx: &LogicalDecodingContext, last_write: bool);
  fn appendStringInfoString(str: *mut Struct_StringInfoData, s: *const i8);
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

    let change = match change.action {
        REORDER_BUFFER_CHANGE_INSERT => {
            let tuple = pg_tuple_to_rspgod_tuple(
                unsafe { (*relation).rd_att },
                unsafe { &mut(*(*change.data.tp()).newtuple).tuple }
            );

            Change::Insert { new_row: tuple }
        },
        REORDER_BUFFER_CHANGE_UPDATE => { Change::Update  { whatever: "".to_string() } },
        REORDER_BUFFER_CHANGE_DELETE => { Change::Delete  { whatever: "".to_string() } },
        _                            => { Change::Unknown { whatever: "".to_string() } },
    };

    let output         = format!("{}", json::encode(&change).unwrap());
    let c_tuple_string = CString::new(&output[..]).unwrap();

    unsafe {
        OutputPluginPrepareWrite(ctx, true);
        appendStringInfoString(ctx.out, c_tuple_string.as_ptr());
        OutputPluginWrite(ctx, true);
    }
}


pub fn extract_string(i8str:[::libc::c_char; 64usize]) -> String {
    let u8str:[u8; 64usize] = unsafe { std::mem::transmute(i8str) };
    let str = String::from_utf8(u8str.to_vec()).unwrap(); // unwrap = danger!
    str.chars().take_while(|c| *c != '\u{0}').collect()
}

pub fn pg_tuple_to_rspgod_tuple(description:TupleDesc, tuple:HeapTuple) -> Tuple {
    let raw_desc         = unsafe { *description };
    let num_attributes   = raw_desc.natts as u32;
    let mut fields       = vec![];

    for n in 0..num_attributes {
        let pg_attribute = unsafe { **raw_desc.attrs.offset(n as isize) };
        let name         = extract_string(pg_attribute.attname.data);

        let isnull = &mut CFalse;
        let datum  = unsafe { macrowrap_heap_getattr(tuple, (n as i32) + 1, description, isnull) };
        if to_bool(*isnull) {
            fields.push(Field { name: name.clone(), value: None });
        } else {
            fields.push(Field {
                name:  name.clone(),
                value: Some(datum_to_string(pg_attribute.atttypid, datum)),
            });
        }
    }

    fields
}

#[no_mangle]
pub extern fn pg_decode_commit_txn(ctx: &LogicalDecodingContext, txn: &ReorderBufferTXN, commit_lsn: u64) {
}

#[no_mangle]
pub extern fn pg_decode_begin_txn(ctx: &LogicalDecodingContext, txn: &ReorderBufferTXN) {
}

#[no_mangle]
pub extern fn pg_decode_shutdown(ctx: &LogicalDecodingContext) {
}
