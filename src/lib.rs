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
    ChangeType,
    Tuple,
};

use postgres::{
    pg_tuple_to_rspgod_tuple,
    get_namespace,
    get_relation_name,
};

use postgres_bindings::{
    LogicalDecodingContext,
    OutputPluginOptions,
    Relation,
    ReorderBufferChange,
    ReorderBufferTXN,
    Struct_StringInfoData,
    OUTPUT_PLUGIN_TEXTUAL_OUTPUT,
    REORDER_BUFFER_CHANGE_DELETE,
    REORDER_BUFFER_CHANGE_INSERT,
    REORDER_BUFFER_CHANGE_UPDATE,
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

    let namespace     = get_namespace(relation);
    let relation_name = get_relation_name(relation);

    let change = match change.action {
        REORDER_BUFFER_CHANGE_INSERT => {
            let tuple = pg_tuple_to_rspgod_tuple(
                unsafe { (*relation).rd_att },
                unsafe { (*change.data.tp()).newtuple }
            );

            match tuple {
                Ok(n) => { make_change(namespace, relation_name, ChangeType::Insert, None, Some(n), None) },
                _     => { None },
            }
        },
        REORDER_BUFFER_CHANGE_UPDATE => {
            let new_tuple = pg_tuple_to_rspgod_tuple(
                unsafe { (*relation).rd_att },
                unsafe { (*change.data.tp()).newtuple }
            );

            let old_tuple = pg_tuple_to_rspgod_tuple(
                unsafe { (*relation).rd_att },
                unsafe { (*change.data.tp()).newtuple }
            );

            match (new_tuple, old_tuple) {
                (Ok(n), Ok(o)) => { make_change(namespace, relation_name, ChangeType::Update, Some(o), Some(n), None) },
                _ => { None },
            }
        },
        REORDER_BUFFER_CHANGE_DELETE => {
            let tuple = pg_tuple_to_rspgod_tuple(
                unsafe { (*relation).rd_att },
                unsafe { (*change.data.tp()).oldtuple }
            );

            match tuple {
                Ok(o) => { make_change(namespace, relation_name, ChangeType::Delete, Some(o), None, None) },
                _     => { None },
            }
        },
        _ => { None },
    };

    if let Some(c) = change {
        let output         = format!("{}", json::encode(&c).unwrap());
        let c_tuple_string = CString::new(&output[..]).unwrap();

        unsafe {
            OutputPluginPrepareWrite(ctx, true);
            appendStringInfoString(ctx.out, c_tuple_string.as_ptr());
            OutputPluginWrite(ctx, true);
        }
    }
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

fn make_change(namespace:     String,
               relation_name: String,
               change_type:   ChangeType,
               old_row:       Option<Tuple>,
               new_row:       Option<Tuple>,
               debug:         Option<String>) -> Option<Change> {
    Some(Change {
        namespace:     namespace,
        relation_name: relation_name,
        change_type:   change_type,
        old_row:       old_row,
        new_row:       new_row,
        debug_message: debug,
    })
}
