use include_dir::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
  borrow::Cow,
  collections::{HashMap, HashSet},
};

static DIRECTORY_FILES: Dir = include_dir!("$DIRECTORY_DATA_FOLDER");

#[derive(thiserror::Error, Debug, Clone)]
pub enum DirectoryError {
  #[error("Directory tags non found")]
  TagsDirNotFound,
  #[error("Tag `{0}` is not unique, verify you don't have {0}.yaml and {0}.yml")]
  TagIsNotUnique(String),
  #[error("Item `{0}` is not unique, verify you don't have {0}.yaml and {0}.yml")]
  ItemIsNotUnique(String),
  #[error("Key `{0}` is not unique accross tags & items, you should not have multiple files named {0}.yaml or {0}.yml ")]
  KeyIsNotUnique(String),
  #[error("Directory file cound not be read")]
  CouldNotReadFile,
  #[error("Yaml deserialization error")]
  YamlDeserialization,
  #[error("File name and key do not match : file name : {0} / file key : {1}")]
  FileNameAndKeyDoNotMatch(String, String),
  #[error("File name or key does not match conventions (only lowercase alphanumeric characters) : {0}")]
  ShouldMatchNamingConventions(String),
}

fn is_yaml(file: &File) -> bool {
  file
    .path()
    .extension()
    .and_then(|ext| ext.to_str())
    .map(|ext| ext == "yaml" || ext == "yml")
    .unwrap_or(false)
}

static RE_KEYS: once_cell::sync::Lazy<Regex> = once_cell::sync::Lazy::new(|| Regex::new(r#"^[a-z0-9_]+$"#).unwrap());

fn get_file_stem<'a>(file: &'a File) -> Result<&'a str, DirectoryError> {
  let file_stem = file
    .path()
    .file_stem()
    .and_then(|name| name.to_str())
    .ok_or(DirectoryError::CouldNotReadFile)?;
  Ok(file_stem)
}

fn check_filename_and_key(file: &File, key: &str) -> Result<(), DirectoryError> {
  let file_stem = get_file_stem(file)?;

  if !RE_KEYS.is_match(file_stem) {
    return Err(DirectoryError::ShouldMatchNamingConventions(file_stem.to_string()));
  }
  if !RE_KEYS.is_match(key) {
    return Err(DirectoryError::ShouldMatchNamingConventions(key.to_string()));
  }

  if file_stem != key {
    return Err(DirectoryError::FileNameAndKeyDoNotMatch(file_stem.to_string(), key.to_string()));
  }
  Ok(())
}

pub fn validate_directory(directory: &Directory) -> Result<(), DirectoryError> {
  let mut keys = HashSet::new();
  for key in directory.tags.keys() {
    if !keys.insert(key.clone()) {
      return Err(DirectoryError::KeyIsNotUnique(key.clone()));
    }
  }
  for key in directory.items.keys() {
    if !keys.insert(key.clone()) {
      return Err(DirectoryError::KeyIsNotUnique(key.clone()));
    }
  }
  Ok(())
}

pub fn load_directory() -> Result<Directory, DirectoryError> {
  let tags_dir = DIRECTORY_FILES.get_dir("tags").ok_or(DirectoryError::TagsDirNotFound)?;

  let mut tags = HashMap::new();
  for tag_file in tags_dir.files() {
    if !is_yaml(tag_file) {
      continue;
    }
    let yaml = tag_file.contents_utf8().ok_or(DirectoryError::CouldNotReadFile)?;
    let mut tag: Tag = serde_yaml::from_str(yaml).map_err(|e| {
      tracing::error!("tag deserialization : `{:?}`", e);
      DirectoryError::YamlDeserialization
    })?;

    if tag.key.is_empty() {
      tag.key = get_file_stem(tag_file)?.to_string();
    }

    check_filename_and_key(tag_file, &tag.key)?;

    let key = tag.key.clone();
    if tags.insert(key.clone(), tag).is_some() {
      return Err(DirectoryError::TagIsNotUnique(key));
    }
  }

  let mut items = HashMap::new();
  for item_file in DIRECTORY_FILES.files() {
    if !is_yaml(item_file) {
      continue;
    }
    let yaml = item_file.contents_utf8().ok_or(DirectoryError::CouldNotReadFile)?;
    let mut item: Item = serde_yaml::from_str(yaml).map_err(|e| {
      tracing::error!("item deserialization : `{:?}`", e);
      DirectoryError::YamlDeserialization
    })?;

    if item.key.is_empty() {
      item.key = get_file_stem(item_file)?.to_string();
    }

    check_filename_and_key(item_file, &item.key)?;

    let key = item.key.clone();
    if items.insert(key.clone(), item).is_some() {
      return Err(DirectoryError::ItemIsNotUnique(key));
    }
  }

  let directory = Directory { tags, items };
  validate_directory(&directory)?;
  Ok(directory)
}

#[derive(Clone, Debug)]
pub struct Directory {
  pub tags: HashMap<String, Tag>,
  pub items: HashMap<String, Item>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Tag {
  #[serde(default)]
  pub key: String,
  pub title: Cow<'static, str>,
  pub description: Vec<Cow<'static, str>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Item {
  #[serde(default)]
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

  #[test]
  pub fn test_check_filename_and_key() -> anyhow::Result<()> {
    pretty_env_logger::try_init().ok();
    assert!(matches!(check_filename_and_key(&File::new("plop", &[]), "plop"), Ok(())));
    assert!(
      matches!(check_filename_and_key(&File::new("PLOP", &[]), "plop"), Err(DirectoryError::ShouldMatchNamingConventions(s)) if s == "PLOP")
    );
    assert!(
      matches!(check_filename_and_key(&File::new("plop", &[]), "PLOP"), Err(DirectoryError::ShouldMatchNamingConventions(s)) if s == "PLOP")
    );
    assert!(matches!(
      check_filename_and_key(&File::new("plop.txt", &[]), "plop.txt"),
      Err(DirectoryError::ShouldMatchNamingConventions(s)) if s == "plop.txt"
    ));
    assert!(matches!(check_filename_and_key(&File::new("pl_0_p.yaml", &[]), "pl_0_p"), Ok(())));
    assert!(matches!(check_filename_and_key(&File::new("pl_0_p.md", &[]), "pl_0_p"), Ok(())));
    assert!(matches!(check_filename_and_key(&File::new("pl_0_p.txt", &[]), "pl_0_p"), Ok(())));

    assert!(matches!(check_filename_and_key(&File::new("pl0p", &[]), "pl0p"), Ok(())));
    assert!(matches!(
      check_filename_and_key(&File::new("pl0p", &[]), "plop"),
      Err(DirectoryError::FileNameAndKeyDoNotMatch(file, key)) if file == "pl0p" && key == "plop"
    ));

    Ok(())
  }
}
