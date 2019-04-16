
use amqp_worker::MessageError;
use amqp_worker::job::*;
use std::fs;
use std::error::Error;
use std::path::Path;

pub fn process(message: &str) -> Result<u64, MessageError> {
  let job = Job::new(message)?;
  debug!("reveived message: {:?}", job);

  match job.check_requirements() {
    Ok(_) => {},
    Err(message) => { return Err(message); }
  }

  match job.get_string_parameter("action").unwrap_or("Undefined".to_string()).as_str() {
    "remove" => {
      remove_files(&job)
    },
    "copy" => {
      copy_files(&job)
    },
    action_label => {
      Err(MessageError::ProcessingError(job.job_id, format!("Unknown action named {}", action_label)))
    }
  }
}

fn remove_files(job: &Job) -> Result<u64, MessageError> {
  let source_paths = job.get_array_of_strings_parameter("source_paths");
  if source_paths.is_none() {
    return Err(MessageError::ProcessingError(job.job_id, "Could not remove empty source files.".to_string()))
  }

  for source_path in &source_paths.unwrap() {
    let path = Path::new(&source_path);

    if path.is_file() {
      match fs::remove_file(path) {
        Ok(_) => debug!("Removed file: {:?}", path),
        Err(error) => return Err(MessageError::ProcessingError(job.job_id, format!("Could not remove path {:?}: {}", path, error.description())))
      }
    } else if path.is_dir() {
      match fs::remove_dir_all(path) {
        Ok(_) => debug!("Removed directory: {:?}", path),
        Err(error) => return Err(MessageError::ProcessingError(job.job_id, format!("Could not remove path {:?}: {}", path, error.description())))
      }
    } else {
      return Err(MessageError::ProcessingError(job.job_id, format!("No such a file or directory: {:?}", path)));
    }
  }

  Ok(job.job_id)
}

fn copy_files(job: &Job) -> Result<u64, MessageError> {
  let output_directory = job.get_string_parameter("output_directory");
  let source_paths = job.get_array_of_strings_parameter("source_paths");

  if output_directory.is_none() {
    return Err(MessageError::ProcessingError(job.job_id, "Could not copy files without output directory.".to_string()))
  }

  if source_paths.is_none() {
    return Err(MessageError::ProcessingError(job.job_id, "Could not copy files without input sources.".to_string()))
  }

  let mut output_files = vec![];

  for source_path in &source_paths.unwrap() {
    let od = output_directory.clone().unwrap();
    let filename = Path::new(&source_path).file_name().unwrap();
    let output_path = Path::new(&od).join(filename);
    info!("Copy {} --> {:?}", source_path, output_path);

    if let Some(parent) = output_path.parent() {
      if let Err(message) = fs::create_dir_all(parent) {
        return Err(MessageError::ProcessingError(job.job_id, format!("{:?}", message)));
      }
    }

    if let Err(message) = fs::copy(source_path, output_path.clone()) {
      return Err(MessageError::ProcessingError(job.job_id, format!("{:?}", message)));
    }
    output_files.push(output_path.to_str().unwrap().to_string());
  }

  Ok(job.job_id)
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

    let msg = r#"{
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

    let result = process(msg);

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

    let msg = r#"{
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

    let result = process(msg);

    assert_eq!(result, Err(MessageError::ProcessingError(124, "No such a file or directory: \"/tmp/file_4.tmp\"".to_string())));
    assert!(!path1.exists(), format!("{:?} still exists", path1));
  }

  #[test]
  fn action_test_error() {
    let mut msg = r#"{
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

    let mut result = process(msg);
    assert_eq!(result, Err(MessageError::ProcessingError(0, "Unknown action named Undefined".to_string())));

    msg = r#"{
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
    result = process(msg);
    assert_eq!(result, Err(MessageError::ProcessingError(0, "Unknown action named bad_action".to_string())));
  }
}
