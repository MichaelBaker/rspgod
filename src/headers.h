#include "postgres.h"
#include "fmgr.h"
#include "utils/builtins.h"
#include "utils/lsyscache.h"
#include "replication/logical.h"
#include "replication/output_plugin.h"

Datum macrowrap_heap_getattr(
    HeapTuple tup,
    int       attnum,
    TupleDesc tupleDesc,
    bool      *isnull
);

Datum macrowrap_PointerGetDatum(void* datum);
struct varlena* macrowrap_PG_DETOAST_DATUM(Datum datum);
Oid macrowrap_RelationGetNamespace(Relation relation);
char *macrowrap_RelationGetRelationName(Relation relation);
