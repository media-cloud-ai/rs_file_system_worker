use std::io::Error;
use std::path::Path;

use mcai_worker_sdk::{info, warn};

use crate::action::Action;

pub struct ListAction {
  source_paths: Vec<String>,
}

impl ListAction {
  pub fn new(source_paths: Vec<String>) -> Self {
    ListAction { source_paths }
  }
}

impl Action for ListAction {
  fn execute(&self) -> Result<Option<Vec<String>>, Error> {
    let mut listing = vec![];

    for source_path in &self.source_paths {
      info!("List {}", source_path);
      let path = Path::new(&source_path);
      if !path.is_dir() {
        warn!("{} is not a directory", source_path);
        continue;
      }
      let entries = std::fs::read_dir(path)?
        .map(|res| {
          res.map(|entry| {
            entry
              .path()
              .to_str()
              .map(|p| p.to_string())
              .unwrap_or_else(|| "".to_string())
          })
        })
        .collect::<Result<Vec<_>, Error>>()?;

      listing.extend(entries);
    }

    Ok(Some(listing))
  }
}
