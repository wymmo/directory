use include_dir::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::BTreeMap};
use validator::Validate;

const TAGS_DIR: &str = "tags";
const ICONS_DIR: &str = "icons";

static DIRECTORY_FILES: Dir = include_dir!("$DIRECTORY_DATA_FOLDER");

#[derive(thiserror::Error, Debug, Clone)]
pub enum DirectoryError {
  #[error("Directory tags non found")]
  TagsDirNotFound,
  #[error("Directory icons non found")]
  IconsDirNotFound,
  #[error("Tag `{0}` is not unique, verify you don't have {0}.yaml and {0}.yml")]
  TagIsNotUnique(String),
  #[error("Item `{0}` is not unique, verify you don't have {0}.yaml and {0}.yml")]
  ItemIsNotUnique(String),
  #[error("Directory file cound not be read")]
  CouldNotReadFile,
  #[error("Yaml deserialization error")]
  YamlDeserialization,
  #[error("File name and key do not match : file name : {0} / file key : {1}")]
  FileNameAndKeyDoNotMatch(String, String),
  #[error("File name or key does not match conventions (only lowercase alphanumeric characters) : {0}")]
  ShouldMatchNamingConventions(String),
}

#[derive(Clone, Debug, Validate)]
pub struct Directory {
  pub tags: BTreeMap<String, Tag>,
  pub items: BTreeMap<String, Item>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Validate)]
pub struct Tag {
  #[serde(default)]
  pub key: String,
  pub title: Cow<'static, str>,
  pub description: Vec<Cow<'static, str>>,
  pub color: Option<Cow<'static, str>>,

  pub icon: Option<Cow<'static, str>>,
  #[serde(default)]
  pub resize_icon: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Validate)]
pub struct Item {
  #[serde(default)]
  pub key: String,
  pub name: Cow<'static, str>,
  pub title: Cow<'static, str>,
  pub tags: Vec<Cow<'static, str>>,
  pub created_in: usize,
  pub concluded_in: Option<usize>,
  pub description: Vec<Cow<'static, str>>,
  pub url: url::Url,
  pub backlink: Option<url::Url>,

  pub icon: Option<Cow<'static, str>>,
  #[serde(default)]
  pub resize_icon: bool,
  #[serde(default)]
  pub no_icon: bool,

  #[serde(default)]
  pub links: Vec<DirectoryLink>,

  #[serde(default)]
  pub events: Vec<DirectoryEvent>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Validate)]
pub struct DirectoryLink {
  pub target_key: String,
  pub begin_in: usize,
  pub end_in: Option<usize>,
  #[validate(length(min = 20))]
  pub description: Cow<'static, str>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Validate)]
pub struct DirectoryEvent {
  pub happened_in: usize,
  #[validate(length(min = 20))]
  pub description: Cow<'static, str>,
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

pub fn load_directory() -> Result<Directory, DirectoryError> {
  let tags_dir = DIRECTORY_FILES.get_dir(TAGS_DIR).ok_or(DirectoryError::TagsDirNotFound)?;
  let _icons_dir = DIRECTORY_FILES.get_dir(ICONS_DIR).ok_or(DirectoryError::IconsDirNotFound)?;

  let mut tags = BTreeMap::new();
  for tag_file in tags_dir.files() {
    if !is_yaml(tag_file) {
      continue;
    }
    let yaml = tag_file.contents_utf8().ok_or(DirectoryError::CouldNotReadFile)?;
    let mut tag: Tag = serde_yaml::from_str(yaml).map_err(|e| {
      let file_name = tag_file.path().to_string_lossy();
      tracing::error!("{file_name} tag deserialization error : `{e:?}`");
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

  let mut items = BTreeMap::new();
  for item_file in DIRECTORY_FILES.files() {
    if !is_yaml(item_file) {
      continue;
    }
    let yaml = item_file.contents_utf8().ok_or(DirectoryError::CouldNotReadFile)?;
    let mut item: Item = serde_yaml::from_str(yaml).map_err(|e| {
      let file_name = item_file.path().to_string_lossy();
      tracing::error!("{file_name} item deserialization error : `{e:?}`");
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

  Ok(directory)
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::collections::HashSet;

  #[test]
  pub fn validate_current_directory_files() -> anyhow::Result<()> {
    pretty_env_logger::try_init().ok();
    let directory = load_directory()?;
    validate_directory(&directory)
  }

  #[test]
  pub fn test_check_filename_and_key() -> anyhow::Result<()> {
    pretty_env_logger::try_init().ok();
    assert!(matches!(check_filename_and_key(&File::new("plop", &[]), "plop"), Ok(())));
    assert!(
      matches!(check_filename_and_key(&File::new("PLOP", &[]), "plop"), Err(DirectoryError::ShouldMatchNamingConventions(s)) if s == "PLOP")
    );
    assert!(
      matches!(check_filename_and_key(&File::new("plop .txt", &[]), "plop"), Err(DirectoryError::ShouldMatchNamingConventions(s)) if s == "plop ")
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

  pub fn validate_directory(directory: &Directory) -> anyhow::Result<()> {
    let icons_dir = DIRECTORY_FILES.get_dir(ICONS_DIR).ok_or(DirectoryError::IconsDirNotFound)?;

    let mut fail = false;

    let mut keys = HashSet::new();
    for key in directory.tags.keys() {
      if !keys.insert(key.clone()) {
        tracing::error!("Key `{key}` is not unique accross tags & items, you should not have multiple {key}.yaml or {key}.yml files");
        fail = true;
      }
    }
    for key in directory.items.keys() {
      if !keys.insert(key.clone()) {
        tracing::error!("Key `{key}` is not unique accross tags & items, you should not have multiple {key}.yaml or {key}.yml files");
        fail = true;
      }
    }

    for (tag_key, tag) in &directory.tags {
      tracing::info!("- tag : `{}`", tag_key);
      let found = directory
        .items
        .values()
        .flat_map(|x| x.tags.iter())
        .any(|x| x.as_ref() == tag_key.as_str());

      if !found {
        tracing::error!("tag `{}` not found in any item", tag_key);
        fail = true;
      }

      if tag.description.is_empty() {
        tracing::error!("tag `{}` has no description", tag_key);
        fail = true;
      }

      if let Some(file_name) = &tag.icon {
        let file_name = format!("{ICONS_DIR}/{file_name}");
        if icons_dir.get_file(&file_name).is_none() {
          tracing::error!("icon file `{file_name}` for tag `{tag_key}` not found");
          fail = true;
        }
      }
    }

    for (item_key, item) in &directory.items {
      tracing::info!("- item : `{}`", item_key);
      for tag in &item.tags {
        if !directory.tags.contains_key(tag.as_ref()) {
          tracing::error!("tag not found : `{}` in item `{}`", tag, item_key);
          fail = true;
        }
      }

      if item.description.is_empty() {
        tracing::error!("item `{}` has no description", item_key);
        fail = true;
      }

      for link in &item.links {
        if !directory.items.contains_key(&link.target_key) {
          tracing::error!("link target not found : `{}` in item `{}`", link.target_key, item_key);
          fail = true;
        }
        if link.target_key == item.key {
          tracing::error!("link target is the same as the item : `{}` in item `{}`", link.target_key, item_key);
          fail = true;
        }
        if link.description.is_empty() {
          tracing::error!("link description is empty : `{}` in item `{}`", link.target_key, item_key);
          fail = true;
        }
      }

      for event in &item.events {
        if event.description.is_empty() {
          tracing::error!("event description is empty : `{}` in item `{}`", event.description, item_key);
          fail = true;
        }
      }

      if let Some(file_name) = &item.icon {
        let file_name = format!("{ICONS_DIR}/{file_name}");
        if icons_dir.get_file(&file_name).is_none() {
          tracing::error!("icon file `{file_name}` for item `{item_key}` not found");
          fail = true;
        }
      }
    }

    if fail {
      anyhow::bail!("Directory did not validate, see logs !")
    }

    Ok(())
  }

  #[test]
  pub fn assert_validations() -> anyhow::Result<()> {
    let mut directory = Directory { items: Default::default(), tags: Default::default() };
    assert!(matches!(validate_directory(&directory), Ok(())));

    directory.items.insert(
      "wymmo".to_string(),
      Item {
        key: "wymmo".into(),
        name: "Wymmo".into(),
        title: "Où voudriez-vous habiter ?".into(),
        tags: vec!["b2b".into()],
        created_in: 2005,
        concluded_in: None,
        description: vec!["Où !! Mais où ! Parles !".into()],
        url: "https://wymmo.com".parse()?,
        backlink: None,
        icon: None,
        resize_icon: false,
        no_icon: true,
        links: vec![],
        events: vec![],
      },
    );
    directory.items.insert(
      "lbc".to_string(),
      Item {
        key: "lbc".into(),
        name: "Vendre n'importe quoi, n'importe comment !".into(),
        title: "LBC".into(),
        tags: vec!["b2b".into()],
        created_in: 2006,
        concluded_in: None,
        description: vec!["Le Bon Canard".into()],
        url: "https://wymmo.com".parse()?,
        backlink: None,
        icon: None,
        resize_icon: false,
        no_icon: false,
        links: vec![DirectoryLink {
          target_key: "wymmo".into(),
          begin_in: 2026,
          end_in: None,
          description: "Wymmo rachète LBC, ils supportaient pas la compétition".into(),
        }],
        events: vec![],
      },
    );
    directory.tags.insert(
      "b2b".to_string(),
      Tag {
        key: "b2c".into(),
        title: "Business to consumer".into(),
        description: vec!["Le B2B,".into(), "c'est la vie".into()],
        color: None,
        icon: None,
        resize_icon: false,
      },
    );

    assert!(matches!(validate_directory(&directory), Ok(())));

    // empty item description is forbidden
    let mut wrong_directory = directory.clone();
    wrong_directory.tags.entry("b2b".into()).and_modify(|x| {
      x.description = vec![];
    });
    assert!(validate_directory(&wrong_directory).is_err());

    // empty tag description is forbidden
    let mut wrong_directory = directory.clone();
    wrong_directory.items.entry("wymmo".into()).and_modify(|x| {
      x.description = vec![];
    });
    assert!(validate_directory(&wrong_directory).is_err());

    // adds a tag with the same key than an item
    let mut wrong_directory = directory.clone();
    wrong_directory.tags.insert(
      "wymmo".into(),
      Tag {
        key: "wymmo".into(),
        title: "Wymmo".into(),
        description: vec!["description".into()],
        color: None,
        icon: None,
        resize_icon: false,
      },
    );
    assert!(validate_directory(&wrong_directory).is_err());

    // adds a tag with a key that is use by no item
    let mut wrong_directory = directory.clone();
    wrong_directory.tags.insert(
      "not_used".to_string(),
      Tag {
        key: "not_used".into(),
        title: "This tag is not used".into(),
        description: vec!["description".into()],
        color: None,
        icon: None,
        resize_icon: false,
      },
    );
    assert!(validate_directory(&wrong_directory).is_err());

    // adds a non exising tag to an item
    let mut wrong_directory = directory.clone();
    wrong_directory
      .items
      .entry("wymmo".into())
      .and_modify(|x| x.tags.push("not_existing_tag".into()));
    assert!(validate_directory(&wrong_directory).is_err());

    // adds a link to itself
    let mut wrong_directory = directory.clone();
    wrong_directory.items.entry("wymmo".into()).and_modify(|x| {
      x.links
        .push(DirectoryLink { target_key: "wymmo".into(), begin_in: 2020, end_in: None, description: "description".into() })
    });
    assert!(validate_directory(&wrong_directory).is_err());

    // adds a link to an item that does not exist
    let mut wrong_directory = directory.clone();
    wrong_directory.items.entry("wymmo".into()).and_modify(|x| {
      x.links
        .push(DirectoryLink { target_key: "does_not_exists".into(), begin_in: 2020, end_in: None, description: "description".into() })
    });
    assert!(validate_directory(&wrong_directory).is_err());

    Ok(())
  }
}
