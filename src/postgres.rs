use std;
use std::ffi::CStr;
use std::str::{Utf8Error};
use types::{
    to_bool,
    CFalse,
    Field,
    Tuple,
};

use postgres_bindings::{
    format_type_be,
    getTypeOutputInfo,
    get_namespace_name,
    macrowrap_PG_DETOAST_DATUM,
    macrowrap_PointerGetDatum,
    macrowrap_RelationGetNamespace,
    macrowrap_RelationGetRelationName,
    macrowrap_heap_getattr,
    pfree,
    list_free,
    relation_close,
    relation_open,
    Datum,
    HeapTuple,
    Oid,
    OidOutputFunctionCall,
    Struct_FormData_pg_attribute,
    TupleDesc,
    ReorderBufferTupleBuf,
    Relation,
    RelationGetReplicaIndex,
    RelationGetIndexList,
};

const NoLock:                   ::libc::c_int = 0;
const AccessShareLock:          ::libc::c_int = 1;
const RowShareLock:             ::libc::c_int = 2;
const RowExclusiveLock:         ::libc::c_int = 3;
const ShareUpdateExclusiveLock: ::libc::c_int = 4;
const ShareLock:                ::libc::c_int = 5;
const ShareRowExclusiveLock:    ::libc::c_int = 6;
const ExclusiveLock:            ::libc::c_int = 7;
const AccessExclusiveLock:      ::libc::c_int = 8;

const REPLICA_IDENTITY_DEFAULT: ::libc::c_char = 'd' as i8;
const REPLICA_IDENTITY_NOTHING: ::libc::c_char = 'n' as i8;
const REPLICA_IDENTITY_FULL:    ::libc::c_char = 'f' as i8;
const REPLICA_IDENTITY_INDEX:   ::libc::c_char = 'i' as i8;

pub fn pg_tuple_to_rspgod_tuple(description:TupleDesc, heap:*mut ReorderBufferTupleBuf) -> Result<Tuple, String> {
    if heap.is_null() {
        return Err("pg_typel_to_rspgod_tuple: Tuple buffer is null".to_string());
    }

    let tuple            = unsafe { &mut (*heap).tuple };
    let raw_desc         = unsafe { *description };
    let num_attributes   = raw_desc.natts as u32;
    let mut fields       = vec![];

    for n in 0..num_attributes {
        let pg_attribute = attribute(description, n as isize);
        let name         = parse_attname(pg_attribute.attname.data);
        let type_name    = match type_name(pg_attribute) {
            Ok(name) => { name },
            Err(e)   => { return Err(e.to_string()); }
        };

        match datum(tuple, description, n as i32) {
            None => {
              fields.push(Field {
                  name:     name.clone(),
                  value:    None,
                  datatype: type_name,
              });
            },
            Some(d) => {
                match datum_to_string(pg_attribute.atttypid, d) {
                    Err(e)    => { return Err(e.to_string()); },
                    Ok(value) => {
                        fields.push(Field {
                            name:     name.clone(),
                            value:    Some(value),
                            datatype: type_name,
                        });
                    },
                }
            }
        }
    }

    Ok(fields)
}


pub fn get_namespace(relation:Relation) -> String {
    let c_namespace = unsafe { get_namespace_name(macrowrap_RelationGetNamespace(relation)) };
    pg_str_to_rs_str(c_namespace)
}

pub fn get_relation_name(relation:Relation) -> String {
    let c_relation_name = unsafe { macrowrap_RelationGetRelationName(relation) };
    pg_str_to_rs_str(c_relation_name)
}

pub fn get_primary_key(relation:Relation) -> Option<String> {
    match get_primary_key_relation(relation) {
        None => { None },
        Some(primary_key_relation) => {
            // TODO: Get the data here
            None
        },
    }
}

fn get_primary_key_relation(relation:Relation) -> Option<Relation> {
    unsafe {
        let replica_identity_type = (*(*relation).rd_rel).relreplident;

        if replica_identity_type == REPLICA_IDENTITY_NOTHING {
            return None;
        }

        if replica_identity_type == REPLICA_IDENTITY_INDEX {
            let replica_index_oid = RelationGetReplicaIndex(relation);
            if replica_index_oid != (0 as Oid) {
                return Some(relation_open(replica_index_oid, AccessShareLock));
            }
        }

        let indexes       = RelationGetIndexList(relation);
        let mut index_oid = if indexes.is_null() {
            return None;
        } else {
            (*indexes).head
        };


        loop {
            if index_oid.is_null() {
                list_free(indexes);
                return None;
            }

            let index_relation = relation_open(*(*index_oid).data.oid_value(), AccessShareLock);
            let index          = *(*index_relation).rd_index;

            if to_bool(index.indisvalid) && to_bool(index.indisready) && to_bool(index.indisprimary) {
                list_free(indexes);
                return Some(index_relation);
            }

            relation_close(index_relation, AccessShareLock);
            index_oid = (*index_oid).next;
        }
    }
}

// For any datatypes that we don't know, this function converts them into a string
// representation (which is always required by a datatype).
fn datum_to_string(typid:Oid, datum:Datum) -> Result<String, Utf8Error> {
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

        pg_str_to_rs_str_and_free(OidOutputFunctionCall(output_func, real_datum))
    }
}

// You cannot use the input string after you call this function because its memory will have been freed.
fn pg_str_to_rs_str_and_free(pg_str: *mut i8) -> Result<String, Utf8Error> {
    unsafe {
      let slice   = CStr::from_ptr(pg_str);
      let to_free = std::mem::transmute(pg_str);
      pfree(to_free);
      let slice = try!(std::str::from_utf8(slice.to_bytes()));
      Ok(slice.to_string())
    }
}

fn pg_str_to_rs_str(pg_str: *mut i8) -> String {
    unsafe {
      let slice = CStr::from_ptr(pg_str);
      std::str::from_utf8(slice.to_bytes()).unwrap().to_string()
    }
}

fn parse_attname(i8str:[::libc::c_char; 64usize]) -> String {
    let u8str:[u8; 64usize] = unsafe { std::mem::transmute(i8str) };
    let str = String::from_utf8(u8str.to_vec()).unwrap(); // unwrap = danger!
    str.chars().take_while(|c| *c != '\u{0}').collect()
}

fn attribute(description:TupleDesc, attribute_number:isize) -> Struct_FormData_pg_attribute {
    let raw_desc = unsafe { *description };
    unsafe { **raw_desc.attrs.offset(attribute_number) }
}

fn type_name(pg_attribute:Struct_FormData_pg_attribute) -> Result<String, Utf8Error> {
    unsafe { pg_str_to_rs_str_and_free(format_type_be(pg_attribute.atttypid)) }
}

fn datum(tuple:HeapTuple, description:TupleDesc, attribute_number: i32) -> Option<Datum> {
    let isnull = &mut CFalse;
    let datum  = unsafe { macrowrap_heap_getattr(tuple, attribute_number + 1, description, isnull) };

    if to_bool(*isnull) {
        None
    } else {
        Some(datum)
    }
}
