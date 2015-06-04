#[derive(RustcDecodable, RustcEncodable)]
pub enum Change {
    Insert  { new_row:  Tuple },
    Delete  { whatever: String },
    Update  { whatever: String },
    Unknown { whatever: String },
}

pub type Tuple = Vec<Field>;

#[derive(RustcDecodable, RustcEncodable)]
pub struct Field {
    pub name:  String,
    pub value: Option<String>,
}

pub type CBool = ::libc::c_char;
pub const CFalse:CBool = 0 as CBool;
pub const CTrue:CBool  = 1 as CBool;
pub fn to_bool(cbool:CBool) -> bool {
    cbool != CFalse
}
