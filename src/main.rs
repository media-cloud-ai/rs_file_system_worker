use mcai_worker_sdk::worker::{Parameter, ParameterType};
use mcai_worker_sdk::{
  job::{Job, JobResult},
  start_worker, McaiChannel, MessageError, MessageEvent, Version,
};

mod message;

pub mod built_info {
  include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[derive(Debug)]
struct FileSystemEvent {}

impl MessageEvent for FileSystemEvent {
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

  fn get_parameters(&self) -> Vec<Parameter> {
    vec![Parameter {
      identifier: "action".to_string(),
      label: "Action".to_string(),
      kind: vec![ParameterType::String],
      required: true,
    }]
  }

  fn process(
    &self,
    channel: Option<McaiChannel>,
    job: &Job,
    job_result: JobResult,
  ) -> Result<JobResult, MessageError> {
    message::process(channel, job, job_result)
  }
}

static FILE_SYSTEM_EVENT: FileSystemEvent = FileSystemEvent {};

fn main() {
  start_worker(&FILE_SYSTEM_EVENT);
}
