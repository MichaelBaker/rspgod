// These are all somewhere in /usr/local/Cellar/postgresql/9.4.1/include/
#include "postgres.h"
#include "replication/output_plugin.h"
#include "replication/logical.h"
#include "utils/lsyscache.h"
#include "fmgr.h"

Datum macrowrap_heap_getattr(
    HeapTuple tup,
    int       attnum,
    TupleDesc tupleDesc,
    bool      *isnull
);

Datum macrowrap_PointerGetDatum(void* datum);
struct varlena* macrowrap_PG_DETOAST_DATUM(Datum datum);
