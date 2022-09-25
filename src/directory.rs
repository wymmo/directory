#![allow(dead_code)]

use include_dir::*;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap};

static DIRECTORY_FILES: Dir = include_dir!("$CARGO_MANIFEST_DIR/directory");

#[derive(thiserror::Error, Debug, Clone)]
pub enum DirectoryError {
  #[error("Directory tags non found")]
  TagsDirNotFound,
  #[error("Directory file cound not be read")]
  CouldNotReadFile,
  #[error("Yaml deserialization error")]
  YamlDeserialization,
}

fn get_file_stem(file: &File) -> Result<String, DirectoryError> {
  Ok(
    file
      .path()
      .file_stem()
      .ok_or(DirectoryError::CouldNotReadFile)?
      .to_string_lossy()
      .to_string(),
  )
}

pub fn load_directory() -> Result<Directory, DirectoryError> {
  let tags_dir = DIRECTORY_FILES.get_dir("tags").ok_or(DirectoryError::TagsDirNotFound)?;
  let mut tags = HashMap::new();
  for tag_file in tags_dir.files() {
    let key = get_file_stem(tag_file)?;
    let yaml = tag_file.contents_utf8().ok_or(DirectoryError::CouldNotReadFile)?;
    let tag: Tag = serde_yaml::from_str(yaml).map_err(|e| {
      tracing::error!("tag deserialization : `{:?}`", e);
      DirectoryError::YamlDeserialization
    })?;
    tags.insert(key.to_string(), tag);
  }
  let mut items = HashMap::new();
  for item_file in DIRECTORY_FILES.files() {
    let key = get_file_stem(item_file)?;
    let yaml = item_file.contents_utf8().ok_or(DirectoryError::CouldNotReadFile)?;
    let item: Item = serde_yaml::from_str(yaml).map_err(|e| {
      tracing::error!("item deserialization : `{:?}`", e);
      DirectoryError::YamlDeserialization
    })?;
    items.insert(key.to_string(), item);
  }
  Ok(Directory { tags, items })
}

#[derive(Clone, Debug)]
pub struct Directory {
  pub tags: HashMap<String, Tag>,
  pub items: HashMap<String, Item>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Tag {
  pub key: String,
  pub title: Cow<'static, str>,
  pub description: Vec<Cow<'static, str>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Item {
  pub key: String,
  pub name: Cow<'static, str>,
  pub title: Cow<'static, str>,
  pub tags: Vec<Cow<'static, str>>,
  pub created_in: Option<usize>,
  pub description: Vec<Cow<'static, str>>,
  pub url: url::Url,
  pub backlink: Option<url::Url>,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  pub fn read_all_directory() -> anyhow::Result<()> {
    pretty_env_logger::try_init().ok();
    let directory = load_directory()?;

    let mut fail = false;

    #[allow(clippy::for_kv_map)]
    for (key, _tag) in &directory.tags {
      tracing::info!("- tag : `{}`", key);
      let mut found = false;
      for (_item_key, item) in &directory.items {
        if item.tags.iter().any(|x| x.as_ref() == key.as_str()) {
          found = true;
          break;
        }
      }
      if !found {
        tracing::error!("tag `{}` not found in any item", key);
        fail = true;
      }
    }

    for (key, item) in &directory.items {
      tracing::info!("- item : `{}`", key);
      for tag in &item.tags {
        if !directory.tags.contains_key(tag.as_ref()) {
          tracing::error!("tag not found : `{}` in item `{}`", tag, key);
          fail = true;
        }
      }
    }

    if fail {
      Err(anyhow::anyhow!("Some errors found in data"))
    } else {
      Ok(())
    }
  }
}
