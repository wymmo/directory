mod directory;
use crate::directory::*;

#[allow(dead_code)]
fn _a_fonction_to_use_everything() -> Result<(), DirectoryError> {
  let _directory = load_directory()?;
  Ok(())
}

//tests
#[cfg(test)]
mod tests {
  use super::*;
  use serde::Serialize;

  #[derive(Debug, Serialize)]
  pub struct DirectoryGraphData {
    #[serde(rename = "tags")]
    pub tags: Vec<Tag>,
    #[serde(rename = "items")]
    pub items: Vec<Item>,
  }

  #[ignore]
  #[test]
  fn build_data_json() -> eyre::Result<()> {
    let directory = load_directory()?;
    let directory_data =
      DirectoryGraphData { tags: directory.tags.values().cloned().collect(), items: directory.items.values().cloned().collect() };
    let data_json_path = format!("{}/data.json", env!("CARGO_MANIFEST_DIR"));

    let data_json = serde_json::to_string_pretty(&directory_data)?;
    std::fs::write(data_json_path, data_json)?;
    Ok(())
  }
}
