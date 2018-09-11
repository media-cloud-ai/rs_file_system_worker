
extern crate env_logger;
extern crate futures;
extern crate lapin_futures as lapin;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate tokio_core;

mod config;
mod emitter;
mod message;

use config::*;
use futures::future::Future;
use futures::Stream;
use lapin::types::FieldTable;
use std::{thread, time};
use std::net::ToSocketAddrs;
use tokio_core::reactor::Core;
use tokio_core::net::TcpStream;
use lapin::client::ConnectionOptions;
use lapin::channel::{BasicConsumeOptions, QueueDeclareOptions};

fn main() {
  let env = env_logger::Env::default()
    .filter_or(env_logger::DEFAULT_FILTER_ENV, "info");
 
  env_logger::Builder::from_env(env).init();

  loop {
    let amqp_hostname = get_amqp_hostname();
    let amqp_port = get_amqp_port();
    let amqp_username = get_amqp_username();
    let amqp_password = get_amqp_password();
    let amqp_vhost = get_amqp_vhost();
    let amqp_queue = get_amqp_queue();
    let amqp_completed_queue = get_amqp_completed_queue();
    let amqp_error_queue = get_amqp_error_queue();

    info!("Connecting...");
    debug!("AMQP HOSTNAME: {}", amqp_hostname);
    debug!("AMQP PORT: {}", amqp_port);
    debug!("AMQP USERNAME: {}", amqp_username);
    debug!("AMQP VHOST: {}", amqp_vhost);
    debug!("AMQP QUEUE: {}", amqp_queue);

    // create the reactor
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let address = amqp_hostname.clone() + ":" + amqp_port.as_str();
    let addr = address.to_socket_addrs().unwrap().next().unwrap();
    let channel_name = amqp_queue;

    let state = core.run(
      TcpStream::connect(&addr, &handle).and_then(|stream| {
        lapin::client::Client::connect(stream, &ConnectionOptions{
          username: amqp_username,
          password: amqp_password,
          vhost: amqp_vhost,
          ..Default::default()
        })
      }).and_then(|(client, heartbeat_future_fn)| {
        let heartbeat_client = client.clone();
        handle.spawn(heartbeat_future_fn(&heartbeat_client).map_err(|_| ()));

        client.create_channel()
      }).and_then(|channel| {
        let id = channel.id;
        debug!("created channel with id: {}", id);

        let ch = channel.clone();
        channel.queue_declare(&channel_name, &QueueDeclareOptions::default(), &FieldTable::new()).and_then(move |_| {
          debug!("channel {} declared queue {}", id, channel_name);

          channel.basic_consume(&channel_name, "my_consumer", &BasicConsumeOptions::default(), &FieldTable::new())
        }).and_then(|stream| {
          stream.for_each(move |message| {
            let data = std::str::from_utf8(&message.data).unwrap();
            trace!("got message: {}", data);

            match message::process(data) {
              Ok(job_response) => {
                emitter::publish(&amqp_completed_queue, serde_json::to_string(&job_response).unwrap());
                ch.basic_ack(message.delivery_tag);
              }
              Err(error) => {
                match error {
                  message::MessageError::FormatError(msg) => {
                    trace!("{:?}", msg);
                    ch.basic_reject(message.delivery_tag, true);
                  },
                  message::MessageError::RequirementsError(msg) => {
                    trace!("{:?}", msg);
                    ch.basic_reject(message.delivery_tag, true);
                  },
                  message::MessageError::RuntimeError(job_id, msg) => {
                    let content = json!({
                      "status": "error",
                      "message": msg,
                      "job_id": job_id,
                    });
                    emitter::publish(&amqp_error_queue, content.to_string());
                    let requeue = false;
                    ch.basic_reject(message.delivery_tag, requeue);
                  }
                }
              }
            }
            Ok(())
          })
        })
      })
    );

    debug!("{:?}", state);
    let sleep_duration = time::Duration::new(1, 0);
    thread::sleep(sleep_duration);
  }
}
