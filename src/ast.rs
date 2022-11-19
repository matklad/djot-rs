mod generated;

use indexmap::IndexMap;

pub use self::generated::*;

pub type Attrs = IndexMap<String, String>;

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct ReferenceDefinition {
  #[serde(skip_serializing_if = "Attrs::is_empty")]
  pub attrs: Attrs,
  pub destination: String,
}
