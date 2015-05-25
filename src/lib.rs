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
    TupleDesc,
    HeapTuple,
    Struct_HeapTupleData,
    OUTPUT_PLUGIN_TEXTUAL_OUTPUT,
    REORDER_BUFFER_CHANGE_INSERT,
    REORDER_BUFFER_CHANGE_UPDATE,
    REORDER_BUFFER_CHANGE_DELETE,
};

pub struct WrappedPG {
    pub num_attributes:i32,
}

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
pub extern fn pg_decode_change(ctx:      &LogicalDecodingContext,
                               txn:      &ReorderBufferTXN,
                               relation: Relation,
                               change:   &mut ReorderBufferChange) {
    let action_string = match change.action {
        REORDER_BUFFER_CHANGE_INSERT => { "Insert " },
        REORDER_BUFFER_CHANGE_UPDATE => { "Update " },
        REORDER_BUFFER_CHANGE_DELETE => { "Delete " },
        _                            => { "Other " },
    };

    let tuple_string = match change.action {
        REORDER_BUFFER_CHANGE_INSERT => {
            let t = pg_tuple_to_rspgod_tuple(
                unsafe { (*relation).rd_att },
                unsafe { (*(*change.data.tp()).newtuple).tuple }
            );
            t.num_attributes.to_string()
        },
        REORDER_BUFFER_CHANGE_UPDATE => { "Update".to_string() },
        REORDER_BUFFER_CHANGE_DELETE => { "Delete".to_string() },
        _                            => { "No Data".to_string() },
    };

    let c_action_string = CString::new(action_string).unwrap();
    let c_tuple_string  = CString::new(&tuple_string[..]).unwrap();

    unsafe {
	    OutputPluginPrepareWrite(ctx, true);
        appendStringInfoString(ctx.out, c_action_string.as_ptr());
        appendStringInfoString(ctx.out, c_tuple_string.as_ptr());
	    OutputPluginWrite(ctx, true);
    }
}

pub fn pg_tuple_to_rspgod_tuple(description:TupleDesc, tuple:Struct_HeapTupleData) -> WrappedPG {
    let num_attributes = unsafe { (*description).natts };
    WrappedPG { num_attributes: num_attributes }
}



// NOTE code copied from tuple_to_avro_row, which
//   modifies a pointer to the (avro_value_t *output_value)
//   or returns (at the end, or via the `check` macro) an error int

// type TupleDesc = *mut Struct_tupleDesc
//   which is: struct Struct_tupleDesc {
//                      natts: ::libc::c_int,
//                      attrs: *mut Form_pg_attribute,
//                      tdtypeid: Oid,

fn tuple_to_string(description:TupleDesc, tuple:Struct_HeapTupleData) -> String {
    let num_attributes = unsafe { (*description).natts };
    let data_structure = WrappedPG { num_attributes: num_attributes };
    return data_structure.num_attributes.to_string();

    // int err = 0, field = 0;
    // check(err, avro_value_reset(output_val));

    // for (int i = 0; i < tupdesc->natts; i++) {
    //     avro_value_t field_val;
    //     bool isnull;
    //     Datum datum;

    //     Form_pg_attribute attr = tupdesc->attrs[i];
    //     if (attr->attisdropped) continue; /* skip dropped columns */

    //     check(err, avro_value_get_by_index(output_val, field, &field_val, NULL));

    //     datum = heap_getattr(tuple, i + 1, tupdesc, &isnull);

    //     if (isnull) {
    //         check(err, avro_value_set_branch(&field_val, 0, NULL));
    //     } else {
    //         check(err, update_avro_with_datum(&field_val, attr->atttypid, datum));
    //     }

    //     field++;
    // }

    // return err;
}

    // /* ----------------
    //  *		heap_getattr
    //  *
    //  *		Extract an attribute of a heap tuple and return it as a Datum.
    //  *		This works for either system or user attributes.  The given attnum
    //  *		is properly range-checked.
    //  *
    //  *		If the field in question has a NULL value, we return a zero Datum
    //  *		and set *isnull == true.  Otherwise, we set *isnull == false.
    //  *
    //  *		<tup> is the pointer to the heap tuple.  <attnum> is the attribute
    //  *		number of the column (field) caller wants.  <tupleDesc> is a
    //  *		pointer to the structure describing the row and all its fields.
    //  * ----------------
    //  */
    // #define heap_getattr(tup, attnum, tupleDesc, isnull) \
    // 	( \
    // 		((attnum) > 0) ? \
    // 		( \
    // 			((attnum) > (int) HeapTupleHeaderGetNatts((tup)->t_data)) ? \
    // 			( \
    // 				(*(isnull) = true), \
    // 				(Datum)NULL \
    // 			) \
    // 			: \
    // 				fastgetattr((tup), (attnum), (tupleDesc), (isnull)) \
    // 		) \
    // 		: \
    // 			heap_getsysattr((tup), (attnum), (tupleDesc), (isnull)) \
    // 	)
//
    // /* ----------------
    //  *		fastgetattr
    //  *
    //  *		Fetch a user attribute's value as a Datum (might be either a
    //  *		value, or a pointer into the data area of the tuple).
    //  *
    //  *		This must not be used when a system attribute might be requested.
    //  *		Furthermore, the passed attnum MUST be valid.  Use heap_getattr()
    //  *		instead, if in doubt.
    //  *
    //  *		This gets called many times, so we macro the cacheable and NULL
    //  *		lookups, and call nocachegetattr() for the rest.
    //  * ----------------
    //  */
    //
    // #if !defined(DISABLE_COMPLEX_MACRO)
    //
    // #define fastgetattr(tup, attnum, tupleDesc, isnull)					\
    // (																	\
    // 	AssertMacro((attnum) > 0),										\
    // 	(*(isnull) = false),											\
    // 	HeapTupleNoNulls(tup) ?											\
    // 	(																\
    // 		(tupleDesc)->attrs[(attnum)-1]->attcacheoff >= 0 ?			\
    // 		(															\
    // 			fetchatt((tupleDesc)->attrs[(attnum)-1],				\
    // 				(char *) (tup)->t_data + (tup)->t_data->t_hoff +	\
    // 					(tupleDesc)->attrs[(attnum)-1]->attcacheoff)	\
    // 		)															\
    // 		:															\
    // 			nocachegetattr((tup), (attnum), (tupleDesc))			\
    // 	)																\
    // 	:																\
    // 	(																\
    // 		att_isnull((attnum)-1, (tup)->t_data->t_bits) ?				\
    // 		(															\
    // 			(*(isnull) = true),										\
    // 			(Datum)NULL												\
    // 		)															\
    // 		:															\
    // 		(															\
    // 			nocachegetattr((tup), (attnum), (tupleDesc))			\
    // 		)															\
    // 	)																\
    // )
    // #else							/* defined(DISABLE_COMPLEX_MACRO) */
    //
    // extern Datum fastgetattr(HeapTuple tup, int attnum, TupleDesc tupleDesc,
    // 			bool *isnull);
    // #endif   /* defined(DISABLE_COMPLEX_MACRO) */


#[no_mangle]
pub extern fn pg_decode_commit_txn(ctx: &LogicalDecodingContext, txn: &ReorderBufferTXN, commit_lsn: u64) {
}

#[no_mangle]
pub extern fn pg_decode_begin_txn(ctx: &LogicalDecodingContext, txn: &ReorderBufferTXN) {
}

#[no_mangle]
pub extern fn pg_decode_shutdown(ctx: &LogicalDecodingContext) {
}

