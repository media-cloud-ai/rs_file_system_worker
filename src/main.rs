
extern crate amqp_worker;
extern crate env_logger;
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
  fn process(&self, message: &str) -> Result<u64, MessageError> {
    message::process(message)
  }
}

static FILE_SYSTEM_EVENT: FileSystemEvent = FileSystemEvent{};

fn main() {
  let env = env_logger::Env::default()
    .filter_or(env_logger::DEFAULT_FILTER_ENV, "info");
 
  env_logger::Builder::from_env(env).init();

  start_worker(&FILE_SYSTEM_EVENT);
}
