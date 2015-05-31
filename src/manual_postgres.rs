use automatic_postgres::{
    heap_getsysattr,
    TupleDesc,
    HeapTuple,
    HeapTupleHeader,
    Datum,
};

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

// FROM_POSTGRES: Originally the macro heap_getattr
// on my system: /usr/local/Cellar/postgresql/9.4.1/include/server/access/htup_details.h
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
        if *isnull == (0 as ::libc::c_char) {
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
// FROM_POSTGRES: Originally the macro fastgetattr
pub fn fastgetattr(tuple: HeapTuple, attnum: u32, tuple_desc: TupleDesc) -> Option<Datum> {
    if HeapTupleNoNulls(tuple) {
    //     if /* (tupleDesc)->attrs[(attnum)-1]->attcacheoff >= 0 ? */ {
    //         // fetchatt((tupleDesc)->attrs[(attnum)-1],
    //         // 	(char *) (tup)->t_data + (tup)->t_data->t_hoff +
    //         // 		(tupleDesc)->attrs[(attnum)-1]->attcacheoff)
    //     } else {
    //         Some(nocachegetattr(tuple, attnum, tuple_desc))
    //     }
        None
    } else {
    //     if att_isnull((attnum)-1, (tup)->t_data->t_bits) {
    //         None
    //     } else {
    //         Some(nocachegetattr(tuple, attnum, tuple_desc))
    //     }
        None
    }
}
