use std;
use std::ffi::CStr;
use types::{
    to_bool,
    CFalse,
    Field,
    Tuple,
};

use postgres_bindings::{
    format_type_be,
    getTypeOutputInfo,
    macrowrap_PG_DETOAST_DATUM,
    macrowrap_PointerGetDatum,
    macrowrap_heap_getattr,
    pfree,
    Datum,
    HeapTuple,
    Oid,
    OidOutputFunctionCall,
    Struct_FormData_pg_attribute,
    TupleDesc,
};

// For any datatypes that we don't know, this function converts them into a string
// representation (which is always required by a datatype).
pub fn datum_to_string(typid:Oid, datum:Datum) -> String {
    unsafe {
        let mut output_func = 0;
        let mut is_varlena  = CFalse;

        getTypeOutputInfo(typid, &mut output_func, &mut is_varlena);

        let real_datum = if to_bool(is_varlena) {
            let toasty = std::mem::transmute(macrowrap_PG_DETOAST_DATUM(datum));
            macrowrap_PointerGetDatum(toasty)
        } else {
            datum
        };

        pg_str_to_rs_str(OidOutputFunctionCall(output_func, real_datum))
    }
}

// You cannot use the input string after you call this function because its memory will have been freed.
pub fn pg_str_to_rs_str(pg_str: *mut i8) -> String {
    unsafe {
      let slice   = CStr::from_ptr(pg_str);
      let to_free = std::mem::transmute(pg_str);
      pfree(to_free);
      std::str::from_utf8(slice.to_bytes()).unwrap().to_string()
    }
}

pub fn parse_attname(i8str:[::libc::c_char; 64usize]) -> String {
    let u8str:[u8; 64usize] = unsafe { std::mem::transmute(i8str) };
    let str = String::from_utf8(u8str.to_vec()).unwrap(); // unwrap = danger!
    str.chars().take_while(|c| *c != '\u{0}').collect()
}

pub fn attribute(description:TupleDesc, attribute_number:isize) -> Struct_FormData_pg_attribute {
    let raw_desc = unsafe { *description };
    unsafe { **raw_desc.attrs.offset(attribute_number) }
}

pub fn type_name(pg_attribute:Struct_FormData_pg_attribute) -> String {
    unsafe { pg_str_to_rs_str(format_type_be(pg_attribute.atttypid)) }
}

pub fn datum(tuple:HeapTuple, description:TupleDesc, attribute_number: i32) -> Option<Datum> {
    let isnull = &mut CFalse;
    let datum  = unsafe { macrowrap_heap_getattr(tuple, attribute_number + 1, description, isnull) };

    if to_bool(*isnull) {
        None
    } else {
        Some(datum)
    }
}

pub fn pg_tuple_to_rspgod_tuple(description:TupleDesc, tuple:HeapTuple) -> Tuple {
    let raw_desc         = unsafe { *description };
    let num_attributes   = raw_desc.natts as u32;
    let mut fields       = vec![];

    for n in 0..num_attributes {
        let pg_attribute = attribute(description, n as isize);
        let name         = parse_attname(pg_attribute.attname.data);
        let type_name    = type_name(pg_attribute);

        match datum(tuple, description, n as i32) {
            None => {
              fields.push(Field {
                  name:     name.clone(),
                  value:    None,
                  datatype: type_name,
              });
            },
            Some(d) => {
                fields.push(Field {
                    name:     name.clone(),
                    value:    Some(datum_to_string(pg_attribute.atttypid, d)),
                    datatype: type_name,
                });
            }
        }
    }

    fields
}
