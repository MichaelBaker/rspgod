#include "postgres.h"
#include "replication/output_plugin.h"
#include "replication/logical.h"

Datum macrowrap_heap_getattr(
    HeapTuple tup,
    int       attnum,
    TupleDesc tupleDesc,
    bool      *isnull
);
