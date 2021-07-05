/*
* Copyright (C) 2021  Aravinth Manivannan <realaravinth@batsense.net>
*
* Use of this source code is governed by the Apache 2.0 and/or the MIT
* License.
*/

//! Module describing file processor that changes filenames to setup cache-busting
//!
//! Run the following during build using `build.rs`:
//!
//! ```rust
//! use cache_buster::BusterBuilder;
//!
//! fn main() {
//!     // note: add error checking yourself.
//!     //    println!("cargo:rustc-env=GIT_process={}", git_process);
//!     let types = vec![
//!         mime::IMAGE_PNG,
//!         mime::IMAGE_SVG,
//!         mime::IMAGE_JPEG,
//!         mime::IMAGE_GIF,
//!     ];
//!
//!     let config = BusterBuilder::default()
//!         .source("./dist")
//!         .result("./prod")
//!         .mime_types(types)
//!         .copy(true)
//!         .follow_links(true)
//!         .build()
//!         .unwrap();
//!
//!     config.process().unwrap();
//! }
//! ```
//!
//! There's a runtime component to this library which will let you read modified
//! filenames from within your program. See [Files]

use std::collections::HashMap;
use std::io::Error;
use std::path::Path;
use std::{fs, path::PathBuf};

use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::*;

/// Configuration for setting up cache-busting
#[derive(Debug, Clone, Builder)]
#[builder(build_fn(validate = "Self::validate"))]
pub struct Buster<'a> {
    /// source directory
    #[builder(setter(into))]
    source: String,
    /// mime_types for hashing
    mime_types: Vec<mime::Mime>,
    /// directory for writing results
    #[builder(setter(into))]
    result: String,
    #[builder(setter(into, strip_option), default)]
    /// route prefixes
    prefix: Option<String>,
    /// copy other non-hashed files from source dire to result dir?
    copy: bool,
    /// follow symlinks?
    follow_links: bool,
    /// exclude these files for hashing.
    /// They will be copied over without including a hash in the filename
    /// Path should be relative to [self.source]
    #[builder(default)]
    no_hash: Vec<&'a str>,
}

impl<'a> BusterBuilder<'a> {
    fn validate(&self) -> Result<(), String> {
        for file in self.no_hash.iter() {
            for file in file.iter() {
                if !Path::new(&self.source.as_ref().unwrap())
                    .join(file)
                    .exists()
                {
                    return Err(format!("File {} doesn't exist", file));
                }
            }
        }
        Ok(())
    }
}

impl<'a> Buster<'a> {
    // creates base_dir to output files to
    fn init(&self) -> Result<(), Error> {
        let res = Path::new(&self.result);
        if res.exists() {
            fs::remove_dir_all(&self.result).unwrap();
        }

        fs::create_dir(&self.result).unwrap();
        self.create_dir_structure(Path::new(&self.source))?;
        Ok(())
    }

    fn hasher(payload: &[u8]) -> String {
        use data_encoding::HEXUPPER;
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(payload);
        HEXUPPER.encode(&hasher.finalize())
    }

    /// Processes files.
    ///
    /// Panics when a weird MIME is encountered.
    pub fn process(&self) -> Result<(), Error> {
        // panics when mimetypes are detected. This way you'll know which files are ignored
        // from processing

        self.init()?;
        let mut file_map: Files = Files::new(&self.result);

        for entry in WalkDir::new(&self.source)
            .follow_links(self.follow_links)
            .into_iter()
        {
            let entry = entry?;

            let path = entry.path();
            if !path.is_dir() && !self.no_hash.contains(&path.to_str().unwrap()) {
                let path = Path::new(&path);

                for mime_type in self.mime_types.iter() {
                    let file_mime = mime_guess::from_path(path)
                        .first()
                        .expect(&format!("couldn't resolve MIME for file: {:?}", &path));
                    if &file_mime == mime_type {
                        let contents = Self::read_to_string(&path).unwrap();
                        let hash = Self::hasher(&contents);
                        let new_name = format!(
                            "{}.{}.{}",
                            path.file_stem().unwrap().to_str().unwrap(),
                            hash,
                            path.extension().unwrap().to_str().unwrap()
                        );
                        self.copy(path, &new_name);
                        let (source, destination) = self.gen_map(path, &&new_name);
                        let _ = file_map.add(
                            source.to_str().unwrap().into(),
                            destination.to_str().unwrap().into(),
                        );
                    }
                }
            }
        }

        file_map.to_env();
        Ok(())
    }

    // helper fn to read file to string
    fn read_to_string(path: &Path) -> Result<Vec<u8>, Error> {
        use std::fs::File;
        use std::io::Read;

        let mut file_content = Vec::new();
        let mut file = File::open(path)?;
        file.read_to_end(&mut file_content).expect("Unable to read");
        Ok(file_content)
    }

    // helper fn to generate filemap
    fn gen_map<'b>(&self, source: &'b Path, name: &str) -> (&'b Path, PathBuf) {
        let rel_location = source.strip_prefix(&self.source).unwrap().parent().unwrap();
        if let Some(prefix) = &self.prefix {
            //panic!("{}", &prefix);
            let mut result = self.result.as_str();
            if result.chars().nth(0) == Some('/') {
                result = &self.result[1..];
            }
            let destination = Path::new(prefix)
                .join(&result)
                .join(rel_location)
                .join(name);

            (source, destination.into())
        } else {
            let destination = Path::new(&self.result).join(rel_location).join(name);
            (source, destination.into())
        }
    }

    // helper fn to copy files
    fn copy(&self, source: &Path, name: &str) {
        let rel_location = source.strip_prefix(&self.source).unwrap().parent().unwrap();
        let destination = Path::new(&self.result).join(rel_location).join(name);
        fs::copy(source, &destination).unwrap();
    }

    // helper fn to create directory structure in self.base_dir
    fn create_dir_structure(&self, path: &Path) -> Result<(), Error> {
        for entry in WalkDir::new(&path)
            .follow_links(self.follow_links)
            .into_iter()
        {
            let entry = entry?;
            let entry_path = entry.path();
            let entry_path = Path::new(&entry_path);

            if entry_path.is_dir() && path != entry_path {
                Self::create_dir_structure(&self, entry_path)?;
            } else {
                if entry_path.is_dir() {
                    let rel_location = entry_path.strip_prefix(&self.source).unwrap();
                    let destination = Path::new(&self.result).join(rel_location);
                    if !destination.exists() {
                        fs::create_dir(destination)?
                    }
                }
            }
        }
        Ok(())
    }
}
/// Filemap struct
///
/// maps original names to generated names
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
struct Files {
    /// filemap<original-path, modified-path>
    pub map: HashMap<String, String>,
    base_dir: String,
}

impl Files {
    /// Initialize map
    fn new(base_dir: &str) -> Self {
        Files {
            map: HashMap::default(),
            base_dir: base_dir.into(),
        }
    }

    /// Create file map: map original path to modified paths
    fn add(&mut self, k: String, v: String) -> Result<(), &'static str> {
        if self.map.contains_key(&k) {
            Err("key exists")
        } else {
            self.map.insert(k, v);
            Ok(())
        }
    }

    /// This crate uses compile-time environment variables to transfer
    /// data to the main program. This funtction sets that variable
    fn to_env(&self) {
        let json = serde_json::to_string(&self).unwrap();
        //        println!("cargo:rustc-env={}={}", ENV_VAR_NAME, json);
        let res = Path::new(CACHE_BUSTER_DATA_FILE);
        if res.exists() {
            fs::remove_file(&res).unwrap();
        }

        //       const PREFIX: &str = r##"pub const FILE_MAP: &str = r#" "##;
        //       const POSTFIX: &str = r##""#;"##;

        //       let content = format!("#[allow(dead_code)]\n{}{}{}", &PREFIX, &json, &POSTFIX);

        //        fs::write(CACHE_BUSTER_DATA_FILE, content).unwrap();
        fs::write(CACHE_BUSTER_DATA_FILE, &json).unwrap();

        // needed for testing load()
        // if the above statement fails(println), then something's broken
        // with the rust compiler. So not really worried about that.
        //        #[cfg(test)]
        //        std::env::set_var(ENV_VAR_NAME, serde_json::to_string(&self).unwrap());
    }

    #[cfg(test)]
    /// Load filemap in main program. Should be called from main program
    fn load() -> Self {
        let map = fs::read_to_string(CACHE_BUSTER_DATA_FILE).unwrap();
        let res: Files = serde_json::from_str(&map).unwrap();
        res
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    fn hasher_works() {
        delete_file();
        let types = vec![
            mime::IMAGE_PNG,
            mime::IMAGE_SVG,
            mime::IMAGE_JPEG,
            mime::IMAGE_GIF,
        ];

        let config = BusterBuilder::default()
            .source("./dist")
            .result("./prod56")
            .mime_types(types)
            .copy(true)
            .follow_links(true)
            .build()
            .unwrap();

        config.process().unwrap();
        let mut files = Files::load();

        for (k, v) in files.map.drain() {
            let src = Path::new(&k);
            let dest = Path::new(&v);

            assert_eq!(src.exists(), dest.exists());
        }

        cleanup(&config);
    }

    pub fn cleanup(config: &Buster) {
        let _ = fs::remove_dir_all(&config.result);
        delete_file();
    }

    pub fn delete_file() {
        let _ = fs::remove_file(&CACHE_BUSTER_DATA_FILE);
    }

    fn prefix_works() {
        delete_file();
        let types = vec![
            mime::IMAGE_PNG,
            mime::IMAGE_SVG,
            mime::IMAGE_JPEG,
            mime::IMAGE_GIF,
        ];

        let no_hash = vec!["bell.svg", "eye.svg", "a/b/c/d/s/d/svg/10.svg"];

        let config = BusterBuilder::default()
            .source("./dist")
            .result("/tmp/prod2i")
            .mime_types(types)
            .copy(true)
            .follow_links(true)
            .prefix("/test")
            .no_hash(no_hash.clone())
            .build()
            .unwrap();

        config.process().unwrap();
        let mut files = Files::load();

        if let Some(prefix) = &config.prefix {
            no_hash.iter().for_each(|file| {
                files.map.iter().any(|(k, v)| {
                    let dest = Path::new(&v[prefix.len()..]);
                    let no_hash = Path::new(file);
                    k == file && dest.exists() && no_hash.file_name() == dest.file_name()
                });
            });

            for (k, v) in files.map.drain() {
                let src = Path::new(&k);
                let dest = Path::new(&v[prefix.len()..]);

                assert_eq!(src.exists(), dest.exists());
            }
        }

        cleanup(&config);
    }

    #[test]
    fn no_hash_validation_works() {
        let types = vec![
            mime::IMAGE_PNG,
            mime::IMAGE_SVG,
            mime::IMAGE_JPEG,
            mime::IMAGE_GIF,
        ];

        let no_hash = vec!["bbell.svg", "eye.svg", "a/b/c/d/s/d/svg/10.svg"];
        assert!(BusterBuilder::default()
            .source("./dist")
            .result("/tmp/prod2i")
            .mime_types(types)
            .copy(true)
            .follow_links(true)
            .prefix("/test")
            .no_hash(no_hash.clone())
            .build()
            .is_err())
    }

    pub fn runner() {
        prefix_works();
        hasher_works();
    }
}
