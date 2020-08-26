#[macro_use]
extern crate serde_derive;

use crate::action::FileSystemAction;
use mcai_worker_sdk::{
  job::JobResult, start_worker, McaiChannel, MessageError, MessageEvent, Version,
};
use schemars::JsonSchema;

mod action;
mod message;

pub mod built_info {
  include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[derive(Debug, Default)]
struct FileSystemEvent {}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
pub struct FileSystemParameters {
  source_paths: Vec<String>,
  action: FileSystemAction,
  output_directory: Option<String>,
}

impl MessageEvent<FileSystemParameters> for FileSystemEvent {
  fn get_name(&self) -> String {
    "File System".to_string()
  }

  fn get_short_description(&self) -> String {
    "Interact with File System: list, copy, delete".to_string()
  }

  fn get_description(&self) -> String {
    r#"Manipulate files on the storage linked to the worker
    ."#
      .to_string()
  }

  fn get_version(&self) -> Version {
    Version::parse(built_info::PKG_VERSION).expect("unable to locate Package version")
  }

  fn process(
    &self,
    channel: Option<McaiChannel>,
    parameters: FileSystemParameters,
    job_result: JobResult,
  ) -> Result<JobResult, MessageError> {
    message::process(channel, parameters, job_result)
  }
}

fn main() {
  let message_event = FileSystemEvent::default();
  start_worker(message_event);
}
