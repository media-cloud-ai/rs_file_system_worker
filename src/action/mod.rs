use schemars::JsonSchema;
use std::io::Error;

pub mod copy;
pub mod list;
pub mod remove;

pub trait Action {
  fn execute(&self) -> Result<Option<Vec<String>>, Error>;
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
pub enum FileSystemAction {
  #[serde(rename = "copy")]
  Copy,
  #[serde(rename = "list")]
  List,
  #[serde(rename = "remove")]
  Remove,
}
