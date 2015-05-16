#include "postgres.h"
#include "access/sysattr.h"
#include "catalog/pg_class.h"
#include "catalog/pg_type.h"
#include "nodes/parsenodes.h"
#include "replication/output_plugin.h"
#include "replication/logical.h"
#include "utils/builtins.h"
#include "utils/lsyscache.h"
#include "utils/memutils.h"
#include "utils/rel.h"
#include "utils/relcache.h"
#include "utils/syscache.h"
#include "utils/typcache.h"

PG_MODULE_MAGIC;

static void
pg_decode_startup(LogicalDecodingContext *ctx, OutputPluginOptions *opt, bool is_init)
{
	opt->output_type = OUTPUT_PLUGIN_TEXTUAL_OUTPUT;
}

static void
pg_decode_shutdown(LogicalDecodingContext *ctx)
{
}

static void
pg_decode_begin_txn(LogicalDecodingContext *ctx, ReorderBufferTXN *txn)
{
}

static void
pg_decode_commit_txn(LogicalDecodingContext *ctx, ReorderBufferTXN *txn, XLogRecPtr commit_lsn)
{
}

static void
pg_decode_change(LogicalDecodingContext *ctx, ReorderBufferTXN *txn, Relation relation, ReorderBufferChange *change)
{
	OutputPluginPrepareWrite(ctx, true);
  appendStringInfoString(ctx->out, "OUTPUT FROM PLUGIN THINGY");
	OutputPluginWrite(ctx, true);
}

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
