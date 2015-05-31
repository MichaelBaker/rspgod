#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

extern crate libc;
extern crate rustc_serialize;

use rustc_serialize::json;
use std::ffi::CString;
use std::ffi::CStr;

pub mod postgres;

use postgres::{
    macrowrap_heap_getattr,
    pfree,
    getTypeOutputInfo,
    LogicalDecodingContext,
    macrowrap_PointerGetDatum,
    Datum,
    Oid,
    OidOutputFunctionCall,
    OutputPluginOptions,
    ReorderBufferTXN,
    Relation,
    ReorderBufferChange,
    TupleDesc,
    HeapTuple,
    macrowrap_PG_DETOAST_DATUM,
    OUTPUT_PLUGIN_TEXTUAL_OUTPUT,
    REORDER_BUFFER_CHANGE_INSERT,
    REORDER_BUFFER_CHANGE_UPDATE,
    REORDER_BUFFER_CHANGE_DELETE,
};


#[derive(RustcDecodable, RustcEncodable)]
pub enum Change {
    Insert  { new_row: WrappedPG },
    Delete  { whatever: String },
    Update  { whatever: String },
    Unknown { whatever: String },
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct WrappedPG {
    pub num_attributes: u32,
    pub cells:          Vec<Field>,
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct Field {
    pub name:  String,
    pub value: Option<String>,
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

    let change = match change.action {
        REORDER_BUFFER_CHANGE_INSERT => {
            let tuple = pg_tuple_to_rspgod_tuple(
                unsafe { (*relation).rd_att },
                unsafe { &mut(*(*change.data.tp()).newtuple).tuple }
            );

            Change::Insert { new_row: tuple }
        },
        REORDER_BUFFER_CHANGE_UPDATE => { Change::Update  { whatever: "".to_string() } },
        REORDER_BUFFER_CHANGE_DELETE => { Change::Delete  { whatever: "".to_string() } },
        _                            => { Change::Unknown { whatever: "".to_string() } },
    };

    let output         = format!("{}", json::encode(&change).unwrap());
    let c_tuple_string = CString::new(&output[..]).unwrap();

    unsafe {
        OutputPluginPrepareWrite(ctx, true);
        appendStringInfoString(ctx.out, c_tuple_string.as_ptr());
        OutputPluginWrite(ctx, true);
    }
}


pub fn extract_string(i8str:[::libc::c_char; 64usize]) -> String {
    let u8str:[u8; 64usize] = unsafe { std::mem::transmute(i8str) };
    let str = String::from_utf8(u8str.to_vec()).unwrap(); // unwrap = danger!
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

        let isnull = &mut (0 as ::libc::c_char);
        let datum  = unsafe { macrowrap_heap_getattr(tuple, (n as i32) + 1, description, isnull) };
        if *isnull == (0 as ::libc::c_char) {
            fields.push(Field { name: name.clone(), value: None });
        } else {
            fields.push(Field { name: name.clone(), value: Some(name.clone()) });
        }
    }

    WrappedPG {
        num_attributes: num_attributes,
        cells:          fields,
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






#[no_mangle]
pub extern fn pg_decode_commit_txn(ctx: &LogicalDecodingContext, txn: &ReorderBufferTXN, commit_lsn: u64) {
}

#[no_mangle]
pub extern fn pg_decode_begin_txn(ctx: &LogicalDecodingContext, txn: &ReorderBufferTXN) {
}

#[no_mangle]
pub extern fn pg_decode_shutdown(ctx: &LogicalDecodingContext) {
}


// For any datatypes that we don't know, this function converts them into a string
// representation (which is always required by a datatype).
fn datum_to_string(typid:Oid, datum:Datum) -> String {
    unsafe {
        let mut output_func = 0;
        let mut is_varlena  = 0 as ::libc::c_char;

        getTypeOutputInfo(typid, &mut output_func, &mut is_varlena);

        let real_datum = if is_varlena == (0 as ::libc::c_char) {
            let toasty = std::mem::transmute(macrowrap_PG_DETOAST_DATUM(datum));
            macrowrap_PointerGetDatum(toasty)
        } else {
            datum
        };

        /* This looks up the output function by OID on every call. Might be a bit faster
         * to do cache the output function info (like how printtup() does it). */
        let cstr = OidOutputFunctionCall(output_func, real_datum);

        let slice   = CStr::from_ptr(cstr);
        let to_free = std::mem::transmute(cstr);
        pfree(to_free);
        std::str::from_utf8(slice.to_bytes()).unwrap().to_string()
    }
}
