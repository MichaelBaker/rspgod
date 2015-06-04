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
    Datum,
    Oid,
    OidOutputFunctionCall,
    macrowrap_PG_DETOAST_DATUM,
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
