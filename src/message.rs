
use serde_json;
use std::fs;
use std::error::Error;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
struct Resource {
  paths: Vec<String>
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct Requirements {
  paths: Option<Vec<String>>
}

#[derive(Debug, Serialize, Deserialize)]
enum Action {
  #[serde(rename="copy")]
  CopyFiles,
  #[serde(rename="remove")]
  Remove,
}

#[derive(Debug, Serialize, Deserialize)]
struct JobParameters {
  id: String,
  #[serde(rename="type")]
  param_type: String,
  enable: bool,
  default: String,
  value: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Parameters {
  action: Action,
  #[serde(default)]
  requirements: Requirements,
  source: Resource,
  parameters: Vec<JobParameters>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Job {
  job_id: u64,
  parameters: Parameters
}

#[derive(Debug, Serialize)]
pub struct JobResponse {
  pub job_id: u64,
  pub status: String,
  pub files: Vec<String>
}

pub enum MessageError {
  FormatError(String),
  RuntimeError(u64, String),
  RequirementsError(String)
}

fn check_requirements(requirements: Requirements) -> Result<(), MessageError> {
  if requirements.paths.is_some() {
    let required_paths :Vec<String> = requirements.paths.unwrap();
    for path in &required_paths {
      let p = Path::new(path);
      if !p.exists() {
        return Err(MessageError::RequirementsError(format!("Warning: Required file does not exists: {:?}", p)));
      }
    }
  }
  Ok(())
}

pub fn process(message: &str) -> Result<JobResponse, MessageError> {

  let parsed: Result<Job, _> = serde_json::from_str(message);

  match parsed {
    Ok(content) => {
      debug!("reveived message: {:?}", content);

      let parameters = content.parameters;

      match check_requirements(parameters.requirements) {
        Ok(_) => {},
        Err(msg) => { return Err(msg); }
      }

      match parameters.action {
        Action::Remove => return remove_files(&parameters.source.paths, content.job_id),
        Action::CopyFiles => return copy_files(&parameters.source.paths, &parameters.parameters, content.job_id),
      }

    },
    Err(msg) => {
      error!("{:?}", msg);
      return Err(MessageError::FormatError("bad input message".to_string()));
    }
  }
}

fn remove_files(files: &Vec<String>, job_id: u64) -> Result<JobResponse, MessageError> {
  for file in files {
    let path = Path::new(&file);

    if path.is_file() {
      match fs::remove_file(path) {
        Ok(_) => debug!("Removed file: {:?}", path),
        Err(error) => return Err(MessageError::RuntimeError(job_id, format!("Could not remove path {:?}: {}", path, error.description())))
      }
    } else if path.is_dir() {
      match fs::remove_dir_all(path) {
        Ok(_) => debug!("Removed directory: {:?}", path),
        Err(error) => return Err(MessageError::RuntimeError(job_id, format!("Could not remove path {:?}: {}", path, error.description())))
      }
    } else {
      return Err(MessageError::RuntimeError(job_id, format!("No such a file or directory: {:?}", path)));
    }
  }

  Ok(JobResponse{
    job_id,
    status: "completed".to_string(),
    files: vec![]
  })
}

fn copy_files(files: &Vec<String>, parameters: &Vec<JobParameters>, job_id: u64) -> Result<JobResponse, MessageError> {

  let mut output_directory = None;
  for parameter in parameters {
    if parameter.id == "output_directory" {
      output_directory = Some(parameter.value.clone());
    }
  }

  if output_directory.is_none() {
    return Err(MessageError::RuntimeError(job_id, "Could not copy files without output directory.".to_string()))
  }

  let mut output_files = vec![];

  for file in files {
    let od = output_directory.clone().unwrap();
    let filename = Path::new(&file).file_name().unwrap();
    let output_path = Path::new(&od).join(filename);
    info!("Copy {} --> {:?}", file, output_path);

    if let Some(parent) = output_path.parent() {
      if let Err(message) = fs::create_dir_all(parent) {
        return Err(MessageError::RuntimeError(job_id, format!("{:?}", message)));
      }
    }

    if let Err(message) = fs::copy(file, output_path.clone()) {
      return Err(MessageError::RuntimeError(job_id, format!("{:?}", message)));
    }
    output_files.push(output_path.to_str().unwrap().to_string());
  }

  Ok(JobResponse{
    job_id,
    status: "completed".to_string(),
    files: output_files
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::io::Read;
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


    let mut message_file = File::open("tests/message_test_remove_ok.json").unwrap();
    let mut msg = String::new();
    message_file.read_to_string(&mut msg).unwrap();

    let result = process(msg.as_str());

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

    let mut message_file = File::open("tests/message_test_remove_error.json").unwrap();
    let mut msg = String::new();
    message_file.read_to_string(&mut msg).unwrap();

    let result = process(msg.as_str());

    assert!(match result { Err(MessageError::RuntimeError(0, _)) => true, _ => false });
    assert!(!path1.exists(), format!("{:?} still exists", path1));
  }

  #[test]
  fn action_test_error() {
    let mut msg = "{\"parameters\":{\"requirements\":{},\"source\":{\"paths\":[\"/tmp/file_x.tmp\"]}},\"job_id\": 0}";
    let mut result = process(msg);
    assert!(match result { Err(MessageError::RuntimeError(0, _)) => true, _ => false });

    msg = "{\"parameters\":{\"action\": \"bad_action\",\"requirements\":{},\"source\":{\"paths\":[\"/tmp/file_x.tmp\"]}},\"job_id\": 0}";
    result = process(msg);
    assert!(match result { Err(MessageError::RuntimeError(0, _)) => true, _ => false });
  }
}
