// C callback signatures
// ---------------------
// static void pg_decode_startup(LogicalDecodingContext *ctx, OutputPluginOptions *opt, bool is_init);
// static void pg_decode_shutdown(LogicalDecodingContext *ctx);
// static void pg_decode_begin_txn(LogicalDecodingContext *ctx, ReorderBufferTXN *txn);
// static void pg_output_begin(LogicalDecodingContext *ctx, TestDecodingData *data, ReorderBufferTXN *txn, bool last_write);
// static void pg_decode_commit_txn(LogicalDecodingContext *ctx, ReorderBufferTXN *txn, XLogRecPtr commit_lsn);
// static void pg_decode_change(LogicalDecodingContext *ctx, ReorderBufferTXN *txn, Relation rel, ReorderBufferChange *change);

#[no_mangle]
pub extern fn add_two(a: u32) -> u32 {
    a + 2
}

#[repr(C)]
pub struct Data {
  field_one: u32,
  field_two: f32,
}

#[no_mangle]
pub extern fn rs_print_struct(a: &mut Data) {
    a.field_one = 234;
    a.field_two = 9.23;
}
