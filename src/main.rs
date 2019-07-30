
extern crate amqp_worker;
#[macro_use]
extern crate log;
extern crate serde;
extern crate serde_json;

use amqp_worker::*;

mod message;

#[derive(Debug)]
struct FileSystemEvent {
}

impl MessageEvent for FileSystemEvent {
  fn process(&self, message: &str) -> Result<job::JobResult, MessageError> {
    message::process(message)
  }
}

static FILE_SYSTEM_EVENT: FileSystemEvent = FileSystemEvent{};

fn main() {
  start_worker(&FILE_SYSTEM_EVENT);
}
