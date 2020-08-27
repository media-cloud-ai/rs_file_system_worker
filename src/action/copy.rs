use std::io::{Error, ErrorKind};
use std::path::Path;

use mcai_worker_sdk::info;

use crate::action::Action;

pub struct CopyAction {
  source_paths: Vec<String>,
  output_directory: String,
}

impl CopyAction {
  pub fn new(source_paths: Vec<String>, output_directory: String) -> Self {
    CopyAction {
      source_paths,
      output_directory,
    }
  }
}

impl Action for CopyAction {
  fn execute(&self) -> Result<Option<Vec<String>>, Error> {
    let mut output_files = vec![];

    for source_path in &self.source_paths {
      let od = self.output_directory.clone();
      let filename = Path::new(&source_path).file_name().unwrap();
      let output_path = Path::new(&od).join(filename);
      info!("Copy {} --> {:?}", source_path, output_path);

      if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)
          .map_err(|error| Error::new(ErrorKind::Other, error.to_string()))?;
      }

      std::fs::copy(source_path, output_path.clone())
        .map_err(|error| Error::new(ErrorKind::Other, error.to_string()))?;
      output_files.push(output_path.to_str().unwrap().to_string());
    }

    Ok(Some(output_files))
  }
}
