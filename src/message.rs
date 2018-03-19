
use serde_json;
use std::fs::*;
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
struct Parameters {
  action: String,
  #[serde(default)]
  requirements: Requirements,
  source: Resource
}

#[derive(Debug, Serialize, Deserialize)]
struct Job {
  job_id: u64,
  parameters: Parameters
}

pub enum MessageError {
  RuntimeError(String),
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

pub fn process(message: &str) -> Result<u64, MessageError> {

  let parsed: Result<Job, _> = serde_json::from_str(message);

  match parsed {
    Ok(content) => {
      println!("{:?}", content);

      let parameters = content.parameters;

      match check_requirements(parameters.requirements) {
        Ok(_) => {},
        Err(msg) => { return Err(msg); }
      }

      if parameters.action == "remove" {
        let files = parameters.source.paths;
        for file in files {
          let path = Path::new(&file);

          if path.is_file() {
            match remove_file(path) {
              Ok(_) => println!("Removed file: {:?}", path),
              Err(error) => return Err(MessageError::RuntimeError(format!("Could not remove path {:?}: {}", path, error.description())))
            }
          } else if path.is_dir() {
            match remove_dir_all(path) {
              Ok(_) => println!("Removed directory: {:?}", path),
              Err(error) => return Err(MessageError::RuntimeError(format!("Could not remove path {:?}: {}", path, error.description())))
            }
          } else {
            return Err(MessageError::RuntimeError(format!("No such a file or directory: {:?}", path)));
          }
        }
      } else {
        return Err(MessageError::RuntimeError(format!("Unsupported action: {:?}", parameters.action)));
      }
      Ok(content.job_id)
    },
    Err(msg) => {
      println!("ERROR {:?}", msg);
      return Err(MessageError::RuntimeError("bad input message".to_string()));
    }
  }
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

    assert!(match result { Err(MessageError::RuntimeError(_)) => true, _ => false });
    assert!(!path1.exists(), format!("{:?} still exists", path1));
  }

  #[test]
  fn action_test_error() {
    let mut msg = "{\"parameters\":{\"requirements\":{},\"source\":{\"paths\":[\"/tmp/file_x.tmp\"]}},\"job_id\": 0}";
    let mut result = process(msg);
    assert!(match result { Err(MessageError::RuntimeError(_)) => true, _ => false });

    msg = "{\"parameters\":{\"action\": \"bad_action\",\"requirements\":{},\"source\":{\"paths\":[\"/tmp/file_x.tmp\"]}},\"job_id\": 0}";
    result = process(msg);
    assert!(match result { Err(MessageError::RuntimeError(_)) => true, _ => false });
  }
}
