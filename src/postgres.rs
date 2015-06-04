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

        /* This looks up the output function by OID on every call. Might be a bit faster
         * to do cache the output function info (like how printtup() does it). */
        let cstr = OidOutputFunctionCall(output_func, real_datum);

        let slice   = CStr::from_ptr(cstr);
        let to_free = std::mem::transmute(cstr);
        pfree(to_free);
        std::str::from_utf8(slice.to_bytes()).unwrap().to_string()
    }
}
