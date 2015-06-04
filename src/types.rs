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
