#include "postgres.h"
#include "replication/output_plugin.h"
#include "replication/logical.h"

PG_MODULE_MAGIC;

Datum macrowrap_heap_getattr(HeapTuple tup,
                             int       attnum,
                             TupleDesc tupleDesc,
                             bool      *isnull) {
    return heap_getattr(tup, attnum, tupleDesc, isnull);
}

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
