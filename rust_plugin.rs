use std::ffi::CString;

pub struct DontTouch;

#[repr(C)]
pub struct LogicalDecodingContext<'a> {
    memory_context:        &'a DontTouch,
    reader:                &'a DontTouch,
    slot:                  &'a DontTouch,
    reorder:               &'a DontTouch,
    snapshot_builder:      &'a DontTouch,
    callbacks:             OutputPluginCallbacks<'a>,
    options:               OutputPluginOptions,
    output_plugin_options: &'a DontTouch,
    prepare_write:         &'a DontTouch,
    write:                 &'a DontTouch,
    out:                   &'a DontTouch,
    output_plugin_private: &'a DontTouch,
    output_writer_private: &'a DontTouch,
    accept_writes:         bool,
    prepared_write:        bool,
    write_location:        u64,
    write_xid:             u32,
}

#[repr(C)]
pub struct OutputPluginCallbacks<'a> {
    startup_cb:  &'a DontTouch,
    begin_cb:    &'a DontTouch,
    change_cb:   &'a DontTouch,
    commit_cb:   &'a DontTouch,
    shutdown_cb: &'a DontTouch,
}

#[repr(C)]
pub struct OutputPluginOptions {
    output_type: OutputPluginOutputType,
}

#[repr(C)]
pub enum OutputPluginOutputType {
    OUTPUT_PLUGIN_BINARY_OUTPUT,
    OUTPUT_PLUGIN_TEXTUAL_OUTPUT,
}

extern {
  fn OutputPluginPrepareWrite(ctx: &LogicalDecodingContext, last_write: bool);
  fn OutputPluginWrite(ctx: &LogicalDecodingContext, last_write: bool);
  fn appendStringInfoString(str: &DontTouch, s: *const i8);
}

// pg_decode_change(LogicalDecodingContext *ctx, ReorderBufferTXN *txn, Relation relation, ReorderBufferChange *change)
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
