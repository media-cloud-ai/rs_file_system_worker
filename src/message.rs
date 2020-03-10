use amqp_worker::job::*;
use amqp_worker::MessageError;
use amqp_worker::ParametersContainer;
use lapin_futures::Channel;
use std::fs;
use std::path::Path;

pub fn process(
  _channel: Option<&Channel>,
  job: &Job,
  job_result: JobResult,
) -> Result<JobResult, MessageError> {
  match job
    .get_string_parameter("action")
    .unwrap_or_else(|| "Undefined".to_string())
    .as_str()
  {
    "remove" => remove_files(&job, job_result),
    "copy" => copy_files(&job, job_result),
    action_label => {
      let result = job_result
        .with_status(JobStatus::Error)
        .with_message(&format!("Unknown action named {}", action_label));

      Err(MessageError::ProcessingError(result))
    }
  }
}

fn remove_files(job: &Job, job_result: JobResult) -> Result<JobResult, MessageError> {
  let source_paths = job.get_array_of_strings_parameter("source_paths");
  if source_paths.is_none() {
    let result = job_result
      .with_status(JobStatus::Error)
      .with_message("Could not remove empty source files.");
    return Err(MessageError::ProcessingError(result));
  }

  for source_path in &source_paths.unwrap() {
    let path = Path::new(&source_path);

    if path.is_file() {
      match fs::remove_file(path) {
        Ok(_) => debug!("Removed file: {:?}", path),
        Err(error) => {
          let result = job_result
            .with_status(JobStatus::Error)
            .with_message(&format!(
              "Could not remove path {:?}: {}",
              path,
              error.to_string()
            ));
          return Err(MessageError::ProcessingError(result));
        }
      }
    } else if path.is_dir() {
      match fs::remove_dir_all(path) {
        Ok(_) => debug!("Removed directory: {:?}", path),
        Err(error) => {
          let result = job_result
            .with_status(JobStatus::Error)
            .with_message(&format!(
              "Could not remove path {:?}: {}",
              path,
              error.to_string()
            ));
          return Err(MessageError::ProcessingError(result));
        }
      }
    } else {
      let result = job_result
        .with_status(JobStatus::Error)
        .with_message(&format!("No such a file or directory: {:?}", path));
      return Err(MessageError::ProcessingError(result));
    }
  }

  Ok(job_result.with_status(JobStatus::Completed))
}

fn copy_files(job: &Job, job_result: JobResult) -> Result<JobResult, MessageError> {
  let output_directory = job.get_string_parameter("output_directory");
  let source_paths = job.get_array_of_strings_parameter("source_paths");

  if output_directory.is_none() {
    let result = job_result
      .with_status(JobStatus::Error)
      .with_message("Could not copy files without output directory.");
    return Err(MessageError::ProcessingError(result));
  }

  if source_paths.is_none() {
    let result = job_result
      .with_status(JobStatus::Error)
      .with_message("Could not copy files without input sources.");
    return Err(MessageError::ProcessingError(result));
  }

  let mut output_files = vec![];

  for source_path in &source_paths.unwrap() {
    let od = output_directory.clone().unwrap();
    let filename = Path::new(&source_path).file_name().unwrap();
    let output_path = Path::new(&od).join(filename);
    info!("Copy {} --> {:?}", source_path, output_path);

    if let Some(parent) = output_path.parent() {
      if let Err(message) = fs::create_dir_all(parent) {
        let result = job_result
          .with_status(JobStatus::Error)
          .with_message(&message.to_string());
        return Err(MessageError::ProcessingError(result));
      }
    }

    if let Err(message) = fs::copy(source_path, output_path.clone()) {
      let result = job_result
        .with_status(JobStatus::Error)
        .with_message(&message.to_string());
      return Err(MessageError::ProcessingError(result));
    }
    output_files.push(output_path.to_str().unwrap().to_string());
  }

  Ok(job_result.with_status(JobStatus::Completed))
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs::File;
  use std::io::Write;

  #[test]
  fn remove_test_ok() {
    let path1 = Path::new("/tmp/file_1.tmp");
    let mut file1 = File::create(path1).unwrap();
    file1.write_all(b"ABCDEF1234567890").unwrap();
    assert!(path1.exists());

    let path2 = Path::new("/tmp/file_2.tmp");
    let mut file2 = File::create(path2).unwrap();
    file2.write_all(b"ABCDEF1234567890").unwrap();
    assert!(path2.exists());

    let message = r#"{
      "parameters": [
        {
          "id": "requirements",
          "type": "requirements",
          "value": {"paths": [
            "/tmp/file_1.tmp",
            "/tmp/file_2.tmp"
          ]}
        },
        {
          "id": "source_paths",
          "type": "array_of_strings",
          "value": [
            "/tmp/file_1.tmp",
            "/tmp/file_2.tmp"
          ]
        },
        {
          "id": "action",
          "type": "string",
          "value": "remove"
        }
      ],
      "job_id": 123
    }"#;

    let job = Job::new(message).unwrap();
    let job_result = JobResult::new(job.job_id);
    let result = process(None, &job, job_result);

    assert!(result.is_ok());
    assert!(!path1.exists(), format!("{:?} still exists", path1));
    assert!(!path2.exists(), format!("{:?} still exists", path2));
  }

  #[test]
  fn remove_test_error() {
    let path1 = Path::new("/tmp/file_3.tmp");
    let mut file1 = File::create(path1).unwrap();
    file1.write_all(b"ABCDEF1234567890").unwrap();
    assert!(path1.exists());

    let message = r#"{
      "parameters": [
        {
          "id": "requirements",
          "type": "requirements",
          "value": {"paths": [
          ]}
        },
        {
          "id": "source_paths",
          "type": "array_of_strings",
          "value": [
            "/tmp/file_3.tmp",
            "/tmp/file_4.tmp"
          ]
        },
        {
          "id": "action",
          "type": "string",
          "value": "remove"
        }
      ],
      "job_id": 124
    }"#;

    let job = Job::new(message).unwrap();
    let job_result = JobResult::new(job.job_id);
    let result = process(None, &job, job_result);

    let job_result = JobResult::new(124)
      .with_status(JobStatus::Error)
      .with_message("No such a file or directory: \"/tmp/file_4.tmp\"");

    assert_eq!(result, Err(MessageError::ProcessingError(job_result)));
    assert!(!path1.exists(), format!("{:?} still exists", path1));
  }

  #[test]
  fn action_test_error() {
    let mut message = r#"{
      "parameters": [
        {
          "id": "requirements",
          "type": "requirements",
          "value": {"paths": []}
        },
        {
          "id": "source_paths",
          "type": "array_of_strings",
          "value": ["/tmp/file_x.tmp"]
        }
      ],
      "job_id": 0
    }"#;

    let job = Job::new(message).unwrap();
    let job_result = JobResult::new(job.job_id);
    let result = process(None, &job, job_result);

    let job_result = JobResult::new(0)
      .with_status(JobStatus::Error)
      .with_message("Unknown action named Undefined");

    assert_eq!(result, Err(MessageError::ProcessingError(job_result)));

    message = r#"{
      "parameters": [
        {
          "id": "action",
          "type": "string",
          "value": "bad_action"
        },
        {
          "id": "requirements",
          "type": "requirements",
          "value": {"paths": []}
        },
        {
          "id": "source_paths",
          "type": "array_of_strings",
          "value": ["/tmp/file_x.tmp"]
        }
      ],
      "job_id": 0
    }"#;

    let job = Job::new(message).unwrap();
    let job_result = JobResult::new(job.job_id);
    let result = process(None, &job, job_result);

    let job_result = JobResult::new(0)
      .with_status(JobStatus::Error)
      .with_message("Unknown action named bad_action");

    assert_eq!(result, Err(MessageError::ProcessingError(job_result)));
  }
}
