#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct Change {
    pub namespace:     String,
    pub change_type:   ChangeType,
    pub new_row:       Option<Tuple>,
    pub old_row:       Option<Tuple>,
    pub debug_message: Option<String>,
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
pub enum ChangeType {
    Insert,
    Delete,
    Update,
    Error
}

pub type Tuple = Vec<Field>;

#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct Field {
    pub name:     String,
    pub datatype: String,
    pub value:    Option<String>,
}

pub type CBool = ::libc::c_char;
pub const CFalse:CBool = 0 as CBool;
pub const CTrue:CBool  = 1 as CBool;
pub fn to_bool(cbool:CBool) -> bool {
    cbool != CFalse
}
