use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u32)]
pub enum DefinitionVersion {
   Version1 = 1,
}
