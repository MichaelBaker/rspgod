use std;
use std::ffi::CStr;
use types::{
    CFalse,
    to_bool,
};

use postgres_bindings::{
    pfree,
    getTypeOutputInfo,
    macrowrap_PointerGetDatum,
    macrowrap_PG_DETOAST_DATUM,
    Datum,
    Oid,
    OidOutputFunctionCall,
    TupleDesc,
    Struct_FormData_pg_attribute,
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

pub fn get_attribute(description:TupleDesc, attribute_number:isize) -> Struct_FormData_pg_attribute {
    let raw_desc = unsafe { *description };
    unsafe { **raw_desc.attrs.offset(attribute_number) }
}
