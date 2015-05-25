#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

extern crate libc;

use std::ffi::CString;
mod postgres;

use postgres::{
    LogicalDecodingContext,
    OutputPluginOptions,
    ReorderBufferTXN,
    Relation,
    ReorderBufferChange,
    OUTPUT_PLUGIN_TEXTUAL_OUTPUT,
    REORDER_BUFFER_CHANGE_INSERT,
    REORDER_BUFFER_CHANGE_UPDATE,
    REORDER_BUFFER_CHANGE_DELETE,
};

extern {
  fn OutputPluginPrepareWrite(ctx: &LogicalDecodingContext, last_write: bool);
  fn OutputPluginWrite(ctx: &LogicalDecodingContext, last_write: bool);
  fn appendStringInfoString(str: *mut postgres::Struct_StringInfoData, s: *const i8);
}

#[no_mangle]
pub extern fn pg_decode_startup(ctx: &LogicalDecodingContext, opt: &mut OutputPluginOptions, is_init: bool) {
	opt.output_type = OUTPUT_PLUGIN_TEXTUAL_OUTPUT;
}

#[no_mangle]
pub extern fn pg_decode_change(ctx: &LogicalDecodingContext, txn: &ReorderBufferTXN, relation: &Relation, change: &ReorderBufferChange) {
    let action_string = match change.action {
        REORDER_BUFFER_CHANGE_INSERT => { "Insert" },
        REORDER_BUFFER_CHANGE_UPDATE => { "Update" },
        REORDER_BUFFER_CHANGE_DELETE => { "Delete" },
        _                            => { "Other" },
    };

    let c_to_print = CString::new(action_string).unwrap();

    unsafe {
	    OutputPluginPrepareWrite(ctx, true);
        appendStringInfoString(ctx.out, c_to_print.as_ptr());
	    OutputPluginWrite(ctx, true);
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
