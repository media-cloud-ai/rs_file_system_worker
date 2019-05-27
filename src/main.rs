
extern crate amqp_worker;
#[macro_use]
extern crate log;
extern crate serde;
extern crate serde_json;
extern crate simple_logger;

use amqp_worker::*;
use log::Level;
use std::env;

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
  if env::var("VERBOSE").is_ok() {
    simple_logger::init_with_level(Level::Debug).unwrap();
  } else {
    simple_logger::init_with_level(Level::Warn).unwrap();
  }

  start_worker(&FILE_SYSTEM_EVENT);
}
