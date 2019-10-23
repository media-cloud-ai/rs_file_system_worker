extern crate amqp_worker;
#[macro_use]
extern crate log;
extern crate semver;
extern crate serde;
extern crate serde_json;

use amqp_worker::worker::{Parameter, ParameterType};
use amqp_worker::{job::JobResult, start_worker, MessageError, MessageEvent};
use semver::Version;

mod message;

macro_rules! crate_version {
    () => {
        env!("CARGO_PKG_VERSION")
    };
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
        semver::Version::parse(crate_version!()).expect("unable to locate Package version")
    }

    fn get_git_version(&self) -> Version {
        semver::Version::parse(crate_version!()).expect("unable to locate Package version")
    }

    fn get_parameters(&self) -> Vec<Parameter> {
        vec![Parameter {
            identifier: "action".to_string(),
            label: "Action".to_string(),
            kind: vec![ParameterType::String],
            required: true,
        }]
    }

    fn process(&self, message: &str) -> Result<JobResult, MessageError> {
        message::process(message)
    }
}

static FILE_SYSTEM_EVENT: FileSystemEvent = FileSystemEvent {};

fn main() {
    start_worker(&FILE_SYSTEM_EVENT);
}
