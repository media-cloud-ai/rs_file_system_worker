
use serde_json;
use std::fs::*;
use std::error::Error;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
struct Resource {
  paths: Vec<String>
}

#[derive(Debug, Serialize, Deserialize)]
struct Requirements {
  paths: Option<Vec<String>>
}

#[derive(Debug, Serialize, Deserialize)]
struct Parameters {
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

      // Clean files
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

      Ok(content.job_id)
    },
    Err(msg) => {
      println!("ERROR {:?}", msg);
      return Err(MessageError::RuntimeError("bad input message".to_string()));
    }
  }
}
