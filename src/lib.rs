extern crate libc;

use std::ffi::CString;
mod postgres;

use postgres::{LogicalDecodingContext, OutputPluginOptions, OUTPUT_PLUGIN_TEXTUAL_OUTPUT};

extern {
  fn OutputPluginPrepareWrite(ctx: &LogicalDecodingContext, last_write: bool);
  fn OutputPluginWrite(ctx: &LogicalDecodingContext, last_write: bool);
  fn appendStringInfoString(str: *mut postgres::Struct_StringInfoData, s: *const i8);
}

pub struct DontTouch;

#[no_mangle]
pub extern fn pg_decode_startup(ctx: &LogicalDecodingContext, opt: &mut OutputPluginOptions, is_init: bool) {
	opt.output_type = OUTPUT_PLUGIN_TEXTUAL_OUTPUT;
}

#[no_mangle]
pub extern fn pg_decode_change(ctx: &LogicalDecodingContext, a: &DontTouch, b: &DontTouch, c: &DontTouch) {
    let to_print = &b"Hello, world! -- rust"[..];
    let c_to_print = CString::new(to_print).unwrap();

    unsafe {
	  OutputPluginPrepareWrite(ctx, true);
      appendStringInfoString(ctx.out, c_to_print.as_ptr());
	  OutputPluginWrite(ctx, true);
    }
}

#[no_mangle]
pub extern fn pg_decode_commit_txn(ctx: &LogicalDecodingContext, txn: &DontTouch, commit_lsn: u64) {
}

#[no_mangle]
pub extern fn pg_decode_begin_txn(ctx: &LogicalDecodingContext, txn: &DontTouch) {
}

#[no_mangle]
pub extern fn pg_decode_shutdown(ctx: &LogicalDecodingContext) {
}
