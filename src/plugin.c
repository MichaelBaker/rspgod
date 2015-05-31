#include "postgres.h"
#include "replication/output_plugin.h"
#include "replication/logical.h"

PG_MODULE_MAGIC;

// Wrap Macros
Datum macrowrap_heap_getattr(HeapTuple tup,
                             int       attnum,
                             TupleDesc tupleDesc,
                             bool      *isnull) {
  return heap_getattr(tup, attnum, tupleDesc, isnull);
}

Datum macrowrap_PointerGetDatum(void* datum) {
  return PointerGetDatum(datum);
}

// 196 #define PG_DETOAST_DATUM(datum) \
// 197   pg_detoast_datum((struct varlena *)
//         DatumGetPointer(datum))
//
// extern struct varlena *pg_detoast_datum(struct varlena * datum);
struct varlena* macrowrap_PG_DETOAST_DATUM(Datum datum) {
  return PG_DETOAST_DATUM(datum);
}

// The C interface PG calls, just calls into our Rust
void pg_decode_startup(LogicalDecodingContext *ctx, OutputPluginOptions *opt, bool is_init);
void pg_decode_shutdown(LogicalDecodingContext *ctx);
void pg_decode_begin_txn(LogicalDecodingContext *ctx, ReorderBufferTXN *txn);
void pg_decode_commit_txn(LogicalDecodingContext *ctx, ReorderBufferTXN *txn, XLogRecPtr commit_lsn);
void pg_decode_change(LogicalDecodingContext *ctx, ReorderBufferTXN *txn, Relation relation, ReorderBufferChange *change);

void
_PG_output_plugin_init(OutputPluginCallbacks *cb)
{
	AssertVariableIsOfType(&_PG_output_plugin_init, LogicalOutputPluginInit);

	cb->startup_cb  = pg_decode_startup;
	cb->begin_cb    = pg_decode_begin_txn;
	cb->change_cb   = pg_decode_change;
	cb->commit_cb   = pg_decode_commit_txn;
	cb->shutdown_cb = pg_decode_shutdown;
}
