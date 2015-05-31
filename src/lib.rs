#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

extern crate libc;
extern crate rustc_serialize;

use rustc_serialize::json;

use std::ffi::CString;
mod postgres;

use postgres::{
    heap_getsysattr,
    LogicalDecodingContext,
    OutputPluginOptions,
    ReorderBufferTXN,
    Relation,
    ReorderBufferChange,
    TupleDesc,
    HeapTuple,
    HeapTupleHeader,
    Datum,
    Struct_HeapTupleData,
    OUTPUT_PLUGIN_TEXTUAL_OUTPUT,
    REORDER_BUFFER_CHANGE_INSERT,
    REORDER_BUFFER_CHANGE_UPDATE,
    REORDER_BUFFER_CHANGE_DELETE,
};

#[derive(RustcDecodable, RustcEncodable)]
pub struct WrappedPG {
    pub num_attributes: u32,
    pub fields:         Vec<Field>,
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct Field {
    pub name:  String,
    pub value: String,
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

    let c_action_string = CString::new(action_string).unwrap();

    unsafe {
        OutputPluginPrepareWrite(ctx, true);
        appendStringInfoString(ctx.out, c_action_string.as_ptr());
    }

    match change.action {
        REORDER_BUFFER_CHANGE_INSERT => {
            let tuple = pg_tuple_to_rspgod_tuple(
                unsafe { (*relation).rd_att },
                unsafe { &mut(*(*change.data.tp()).newtuple).tuple }
            );

            let output         = format!("{}", json::encode(&tuple).unwrap());
            let c_tuple_string = CString::new(&output[..]).unwrap();

            unsafe {
                appendStringInfoString(ctx.out, c_tuple_string.as_ptr());
            }
        },
        // REORDER_BUFFER_CHANGE_UPDATE => { "Update".to_string() },
        // REORDER_BUFFER_CHANGE_DELETE => { "Delete".to_string() },
        _                            => {},
    };

    unsafe { OutputPluginWrite(ctx, true); }
}


pub fn extract_string(i8str:[::libc::c_char; 64usize]) -> String {
    let u8str:[u8; 64usize] = unsafe { std::mem::transmute(i8str) };
    let mut str = String::from_utf8(u8str.to_vec()).unwrap(); // unwrap = danger!
    str.chars().take_while(|c| *c != '\u{0}').collect()
}


// type TupleDesc = *mut Struct_tupleDesc
//   which is: struct Struct_tupleDesc {
//                      natts: ::libc::c_int,
//                      attrs: *mut Form_pg_attribute,
//                      tdtypeid: Oid,

// pub struct Struct_HeapTupleData {
//     pub t_len: uint32,
//     pub t_self: ItemPointerData,
//     pub t_tableOid: Oid,
//     pub t_data: HeapTupleHeader,
// }

// pub fn heap_getsysattr(tup: HeapTuple, attnum: ::libc::c_int, tupleDesc: TupleDesc, isnull: *mut _bool) -> Datum;
// pub fn nocachegetattr(tup: HeapTuple, attnum: ::libc::c_int, att: TupleDesc) -> Datum;
// #define TextDatumGetCString(d) text_to_cstring((text *) DatumGetPointer(d))
// src/include/access/htup_details.h
pub fn pg_tuple_to_rspgod_tuple(description:TupleDesc, tuple:HeapTuple) -> WrappedPG {
    let raw_desc         = unsafe { *description };
    let num_attributes   = raw_desc.natts as u32;
    let mut fields       = vec![];

    for n in 0..num_attributes {
        let pg_attribute = unsafe { **raw_desc.attrs.offset(n as isize) };
        let name         = extract_string(pg_attribute.attname.data);

        match heap_getattr(tuple, n + 1, description) {
            Some(datum) => {},
            None        => {},
        }
        fields.push(Field { name: name.clone(), value: name.clone() });
    }

    WrappedPG {
        num_attributes: num_attributes,
        fields:         fields,
    }
}
    // tuple_to_avro_row (from bottledwater)
    //   this works by modifying a pointer to the (avro_value_t *output_value)
    //   or returning (at the end, or via the `check` macro) an error int
    // -----------------------------------------------------
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



// THESE ARE TRANSLATED FROM PG MACROS MOSTLY
// on my system: /usr/local/Cellar/postgresql/9.4.1/include/server/access/htup_details.h
/* ----------------
 *		heap_getattr
 *
 *		Extract an attribute of a heap tuple and return it as a Datum.
 *		This works for either system or user attributes.  The given attnum
 *		is properly range-checked.
 *
 *		If the field in question has a NULL value, we return a zero Datum
 *		and set *isnull == true.  Otherwise, we set *isnull == false.
 *
 *		<tup> is the pointer to the heap tuple.  <attnum> is the attribute
 *		number of the column (field) caller wants.  <tupleDesc> is a
 *		pointer to the structure describing the row and all its fields.
 * ----------------
 */

// 11 bits for number of attributes
const HEAP_NATTS_MASK   : u16 = 0x07FF;
// const HEAP_KEYS_UPDATED : u16 = 0x2000; // tuple was updated and key cols modified, or tuple deleted
// const HEAP_HOT_UPDATED  : u16 = 0x4000; // tuple was HOT-updated
// const HEAP_ONLY_TUPLE   : u16 = 0x8000; // this is heap-only tuple
// const HEAP2_XACT_MASK   : u16 = 0xE000; // visibility-related bits

pub fn HeapTupleHeaderGetNatts(header:HeapTupleHeader) -> u16  {
    unsafe { *header }.t_infomask2 & HEAP_NATTS_MASK
}


// information stored in t_infomask:
const HEAP_HASNULL          : u16 = 0x0001; // has null attribute(s)
// const HEAP_HASVARWIDTH      : u16 = 0x0002; // has variable-width attribute(s)
// const HEAP_HASEXTERNAL      : u16 = 0x0004; // has external stored attribute(s)
// const HEAP_HASOID           : u16 = 0x0008; // has an object-id field
// const HEAP_XMAX_KEYSHR_LOCK : u16 = 0x0010; // xmax is a key-shared locker
// const HEAP_COMBOCID         : u16 = 0x0020; // t_cid is a combo cid
// const HEAP_XMAX_EXCL_LOCK   : u16 = 0x0040; // xmax is exclusive locker
// const HEAP_XMAX_LOCK_ONLY   : u16 = 0x0080; // xmax, if valid, is only a locker
pub fn HeapTupleNoNulls(tuple:HeapTuple) -> bool {
    0 == (HEAP_HASNULL & unsafe { (*(*tuple).t_data).t_infomask })
}

// check to see if the ATT'th bit of an array of 8-bit bytes is set.
// pub fn att_isnull(attnum:u32, pub t_bits: [bits8; 1usize],) {
//     let low_bitmask  = 1 << (attnum & 0x07); // set the attnumth bit
//     let high_bitmask = (BITS)[attnum >> 3];
//     0 == (low_bitmask & high_bitmask)
// }

// pub type HeapTuple = *mut HeapTupleData;
// pub type HeapTupleData = Struct_HeapTupleData;
// pub t_data: HeapTupleHeader,

// pub type HeapTupleHeader = *mut Struct_HeapTupleHeaderData;
// pub struct Struct_HeapTupleHeaderData {
//     pub t_choice: Union_Unnamed26,
//     pub t_ctid: ItemPointerData,
//     pub t_infomask2: uint16,
//     pub t_infomask: uint16,
//     pub t_hoff: uint8,
//     pub t_bits: [bits8; 1usize],
// }

// Originally the macro heap_getattr
pub fn heap_getattr(tuple: HeapTuple, attnum: u32, tuple_desc: TupleDesc) -> Option<Datum> {
    if attnum > 0 {
        if attnum > HeapTupleHeaderGetNatts(unsafe { *tuple }.t_data) as u32 {
            None
        } else {
            fastgetattr(tuple, attnum, tuple_desc)
        }
    } else {
        let isnull = &mut (0 as ::libc::c_char);
        let datum  = unsafe { heap_getsysattr(tuple, attnum as i32, tuple_desc, isnull) };
        if(*isnull == (0 as ::libc::c_char)) {
            None
        } else {
            Some(datum)
        }
    }
}

/* ----------------
 *		fastgetattr
 *
 *		Fetch a user attribute's value as a Datum (might be either a
 *		value, or a pointer into the data area of the tuple).
 *
 *		This must not be used when a system attribute might be requested.
 *		Furthermore, the passed attnum MUST be valid.  Use heap_getattr()
 *		instead, if in doubt.
 *
 *		This gets called many times, so we macro the cacheable and NULL
 *		lookups, and call nocachegetattr() for the rest.
 * ----------------
 */
pub fn fastgetattr(tuple: HeapTuple, attnum: u32, tuple_desc: TupleDesc) -> Option<Datum> {
    None
    // if HeapTupleNoNulls(tuple) {
    //     if /* (tupleDesc)->attrs[(attnum)-1]->attcacheoff >= 0 ? */ {
    //         // fetchatt((tupleDesc)->attrs[(attnum)-1],
    //         // 	(char *) (tup)->t_data + (tup)->t_data->t_hoff +
    //         // 		(tupleDesc)->attrs[(attnum)-1]->attcacheoff)
    //     } else {
    //         Some(nocachegetattr(tuple, attnum, tuple_desc))
    //     }
    // } else {
    //     if att_isnull((attnum)-1, (tup)->t_data->t_bits) {
    //         None
    //     } else {
    //         Some(nocachegetattr(tuple, attnum, tuple_desc))
    //     }
    // }
}


#[no_mangle]
pub extern fn pg_decode_commit_txn(ctx: &LogicalDecodingContext, txn: &ReorderBufferTXN, commit_lsn: u64) {
}

#[no_mangle]
pub extern fn pg_decode_begin_txn(ctx: &LogicalDecodingContext, txn: &ReorderBufferTXN) {
}

#[no_mangle]
pub extern fn pg_decode_shutdown(ctx: &LogicalDecodingContext) {
}

