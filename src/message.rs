use mcai_worker_sdk::{
  job::{JobResult, JobStatus},
  McaiChannel, MessageError,
};

use crate::action::copy::CopyAction;
use crate::action::list::ListAction;
use crate::action::remove::RemoveAction;
use crate::action::Action;
use crate::{FileSystemAction, FileSystemParameters};
use std::io::{Error, ErrorKind};

pub fn process(
  _channel: Option<McaiChannel>,
  parameters: FileSystemParameters,
  job_result: JobResult,
) -> Result<JobResult, MessageError> {
  let source_paths = parameters.source_paths;
  let result = match parameters.action {
    FileSystemAction::Copy => {
      if let Some(output_directory) = parameters.output_directory {
        CopyAction::new(source_paths, output_directory).execute()
      } else {
        Err(Error::new(
          ErrorKind::Other,
          "Could not copy files without output directory.",
        ))
      }
    }
    FileSystemAction::List => ListAction::new(source_paths).execute(),
    FileSystemAction::Remove => RemoveAction::new(source_paths).execute(),
  };

  result
    .map(|paths| {
      let job_result = job_result.clone().with_status(JobStatus::Completed);

      if let Some(mut paths) = paths {
        job_result.with_destination_paths(&mut paths)
      } else {
        job_result
      }
    })
    .map_err(|error| MessageError::from(error, job_result))
}

#[cfg(test)]
mod tests {
  use mcai_worker_sdk::job::Job;
  use std::fs::File;
  use std::io::Write;
  use std::path::Path;

  use super::*;

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
    let parameters: FileSystemParameters = job.get_parameters().unwrap();
    let result = process(None, parameters, job_result);

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
    let parameters: FileSystemParameters = job.get_parameters().unwrap();
    let result = process(None, parameters, job_result);

    let job_result = JobResult::new(124)
      .with_status(JobStatus::Error)
      .with_message("IO Error: No such a file or directory: \"/tmp/file_4.tmp\"");

    assert_eq!(result, Err(MessageError::ProcessingError(job_result)));
    assert!(!path1.exists(), format!("{:?} still exists", path1));
  }

  #[test]
  fn copy_test_ok() {
    let path1 = Path::new("/tmp/file_5.tmp");
    let mut file1 = File::create(path1).unwrap();
    file1.write_all(b"ABCDEF1234567890").unwrap();
    assert!(path1.exists());

    let path2 = Path::new("/tmp/file_6.tmp");
    let mut file2 = File::create(path2).unwrap();
    file2.write_all(b"ABCDEF1234567890").unwrap();
    assert!(path2.exists());

    let message = r#"{
      "parameters": [
        {
          "id": "source_paths",
          "type": "array_of_strings",
          "value": [
            "/tmp/file_5.tmp",
            "/tmp/file_6.tmp"
          ]
        },
        {
          "id": "action",
          "type": "string",
          "value": "copy"
        },
        {
          "id": "output_directory",
          "type": "array_of_strings",
          "value": "./examples"
        }
      ],
      "job_id": 123
    }"#;

    let job = Job::new(message).unwrap();
    let job_result = JobResult::new(job.job_id);
    let parameters: FileSystemParameters = job.get_parameters().unwrap();
    let result = process(None, parameters, job_result);

    let copied_path_1 = Path::new("./examples/file_5.tmp");
    let copied_path_2 = Path::new("./examples/file_6.tmp");

    assert!(result.is_ok());
    assert!(copied_path_1.exists(), format!("{:?} copy failed", copied_path_1));
    assert!(copied_path_2.exists(), format!("{:?} copy failed", copied_path_2));

    std::fs::remove_file(copied_path_1).unwrap();
    std::fs::remove_file(copied_path_2).unwrap();
  }

  #[test]
  fn copy_test_error() {
    let path1 = Path::new("/tmp/file_3.tmp");
    let mut file1 = File::create(path1).unwrap();
    file1.write_all(b"ABCDEF1234567890").unwrap();
    assert!(path1.exists());

    let message = r#"{
      "parameters": [
        {
          "id": "source_paths",
          "type": "array_of_strings",
          "value": [
            "/tmp/file_3.tmp"
          ]
        },
        {
          "id": "action",
          "type": "string",
          "value": "copy"
        }
      ],
      "job_id": 124
    }"#;

    let job = Job::new(message).unwrap();
    let job_result = JobResult::new(job.job_id);
    let parameters: FileSystemParameters = job.get_parameters().unwrap();
    let result = process(None, parameters, job_result);

    let job_result = JobResult::new(124)
      .with_status(JobStatus::Error)
      .with_message("IO Error: Could not copy files without output directory.");

    assert_eq!(result, Err(MessageError::ProcessingError(job_result)));
  }

  #[test]
  fn list_directory() {
    let message = r#"{
      "parameters": [
        {
          "id": "source_paths",
          "type": "array_of_strings",
          "value": [
            "./"
          ]
        },
        {
          "id": "action",
          "type": "string",
          "value": "list"
        }
      ],
      "job_id": 123
    }"#;

    let job = Job::new(message).unwrap();
    let job_result = JobResult::new(job.job_id);
    let parameters: FileSystemParameters = job.get_parameters().unwrap();
    let result = process(None, parameters, job_result);

    assert!(result.is_ok());
    let job_result = result.unwrap();
    assert!(job_result
      .get_destination_paths()
      .contains(&"./Cargo.toml".to_string()));
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
    let error = job.get_parameters::<FileSystemParameters>().unwrap_err();
    let expected_error = MessageError::ParameterValueError(
      "Cannot get parameters from Object({\
        \"requirements\": Object({\"paths\": Array([])}), \
        \"source_paths\": Array([String(\"/tmp/file_x.tmp\")])}): \
        Error(\"missing field `action`\", line: 0, column: 0)"
        .to_string(),
    );

    assert_eq!(error, expected_error);

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
    let error = job.get_parameters::<FileSystemParameters>().unwrap_err();
    let expected_error = MessageError::ParameterValueError(
      "Cannot get parameters from Object({\
        \"action\": String(\"bad_action\"), \
        \"requirements\": Object({\"paths\": Array([])}), \"source_paths\": Array([String(\"/tmp/file_x.tmp\")])}): \
        Error(\"unknown variant `bad_action`, expected one of `copy`, `list`, `remove`\", line: 0, column: 0)"
        .to_string()
    );

    assert_eq!(error, expected_error);
  }
}
