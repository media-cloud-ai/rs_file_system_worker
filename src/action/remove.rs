use std::io::{Error, ErrorKind};
use std::path::Path;

use mcai_worker_sdk::debug;

use crate::action::Action;

pub struct RemoveAction {
  source_paths: Vec<String>,
}

impl RemoveAction {
  pub fn new(source_paths: Vec<String>) -> Self {
    RemoveAction { source_paths }
  }
}

impl Action for RemoveAction {
  fn execute(&self) -> Result<Option<Vec<String>>, Error> {
    for source_path in &self.source_paths {
      let path = Path::new(&source_path);

      if path.is_file() {
        std::fs::remove_file(path).map_err(|err| {
          Error::new(
            ErrorKind::Other,
            format!("Could not remove path {:?}: {}", path, err.to_string()),
          )
        })?;
        debug!("Removed file: {:?}", path);
      } else if path.is_dir() {
        std::fs::remove_dir_all(path).map_err(|err| {
          Error::new(
            ErrorKind::Other,
            format!("Could not remove directory {:?}: {}", path, err.to_string()),
          )
        })?;
        debug!("Removed directory: {:?}", path);
      } else {
        return Err(Error::new(
          ErrorKind::Other,
          format!("No such a file or directory: {:?}", path),
        ));
      }
    }

    Ok(None)
  }
}
