mod directory;
use crate::directory::*;

#[allow(dead_code)]
fn _a_fonction_to_user_everything() -> Result<(), DirectoryError> {
  let _directory = load_directory()?;
  Ok(())
}
