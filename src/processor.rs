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
//! // note: add error checking yourself.
//! //    println!("cargo:rustc-env=GIT_process={}", git_process);
//! let types = vec![
//!     mime::IMAGE_PNG,
//!     mime::IMAGE_SVG,
//!     mime::IMAGE_JPEG,
//!     mime::IMAGE_GIF,
//! ];
//!
//! let config = BusterBuilder::default()
//!     .source("./dist")
//!     .result("./prod")
//!     .mime_types(types)
//!     .copy(true)
//!     .follow_links(true)
//!     .build()
//!     .unwrap();
//!
//! config.process().unwrap();
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

#[derive(Debug, Clone)]
/// Items to avoid hash calculation.
///
/// This is useful when serving vendor static files which are interlinked, where chaing
/// file names should mean changing how the vendor files pulls its dependencies --- which are
/// beyond the abilities of `cache_buster`.
///
/// ```rust
/// use cache_buster::NoHashCategory;
///
/// let extensions = NoHashCategory::FileExtentions(vec!["wasm"]);
/// let files = NoHashCategory::FileExtentions(vec!["swagger-ui-bundle.js", "favicon-16x16.png"]);
/// ```
pub enum NoHashCategory<'a> {
    /// vector of file extensions that should be avoided for hash processing
    FileExtentions(Vec<&'a str>),
    /// list of file paths that should be avoided for file processing
    FilePaths(Vec<&'a str>),
}

/// Configuration for setting up cache-busting
#[derive(Debug, Clone, Builder)]
#[builder(build_fn(validate = "Self::validate"))]
pub struct Buster<'a> {
    /// source directory
    #[builder(setter(into))]
    source: String,
    /// mime_types for hashing
    #[builder(setter(into, strip_option), default)]
    mime_types: Option<Vec<mime::Mime>>,
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
    no_hash: Vec<NoHashCategory<'a>>,
}

impl<'a> BusterBuilder<'a> {
    fn validate(&self) -> Result<(), String> {
        for no_hash_configs in self.no_hash.iter() {
            for no_hash in no_hash_configs.iter() {
                if let NoHashCategory::FilePaths(files) = no_hash {
                    for file in files.iter() {
                        if !Path::new(&self.source.as_ref().unwrap())
                            .join(file)
                            .exists()
                        {
                            return Err(format!("File {} doesn't exist", file));
                        }
                    }
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
        println!("cargo:rerun-if-changed={}", self.source);
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

        let mut process_worker = |path: &Path| {
            let contents = Self::read_to_string(&path).unwrap();
            let hash = Self::hasher(&contents);

            let get_name = |no_hash: bool| -> String {
                if no_hash {
                    format!(
                        "{}.{}",
                        path.file_stem().unwrap().to_str().unwrap(),
                        path.extension().unwrap().to_str().unwrap()
                    )
                } else {
                    format!(
                        "{}.{}.{}",
                        path.file_stem().unwrap().to_str().unwrap(),
                        hash,
                        path.extension().unwrap().to_str().unwrap()
                    )
                }
            };

            let no_hash_status = self.no_hash.iter().any(|no_hash| {
                match no_hash {
                    NoHashCategory::FilePaths(paths) => {
                        let no_hash_status = paths
                            .iter()
                            .any(|file_path| Path::new(&self.source).join(&file_path) == path);
                        no_hash_status
                    }
                    NoHashCategory::FileExtentions(extensions) => {
                        let mut no_hash_status = false;
                        if let Some(cur_extention) = path.extension() {
                            // .unwrap().to_str().unwrap();
                            if let Some(cur_extention) = cur_extention.to_str() {
                                no_hash_status = extensions.iter().any(|ext| &cur_extention == ext);
                            }
                        }
                        no_hash_status
                    }
                }
            });

            let new_name = get_name(no_hash_status);

            //            let new_name = if self.no_hash.iter().any(|no_hash| {
            //                let no_hash = Path::new(&self.source).join(&no_hash);
            //                no_hash == path
            //            }) {
            //                format!(
            //                    "{}.{}",
            //                    path.file_stem().unwrap().to_str().unwrap(),
            //                    path.extension().unwrap().to_str().unwrap()
            //                )
            //            } else {
            //                format!(
            //                    "{}.{}.{}",
            //                    path.file_stem().unwrap().to_str().unwrap(),
            //                    hash,
            //                    path.extension().unwrap().to_str().unwrap()
            //                )
            //            };
            self.copy(path, &new_name);
            let (source, destination) = self.gen_map(path, &&new_name);
            let _ = file_map.add(
                source.to_str().unwrap().into(),
                destination.to_str().unwrap().into(),
            );
        };

        for entry in WalkDir::new(&self.source)
            .follow_links(self.follow_links)
            .into_iter()
        {
            let entry = entry?;

            let path = entry.path();
            if !path.is_dir() {
                let path = Path::new(&path);

                match self.mime_types.as_ref() {
                    Some(mime_types) => {
                        for mime_type in mime_types.iter() {
                            let file_mime =
                                mime_guess::from_path(path).first().unwrap_or_else(|| {
                                    panic!("couldn't resolve MIME for file: {:?}", &path)
                                });
                            if &file_mime == mime_type {
                                process_worker(&path);
                            }
                        }
                    }
                    None => process_worker(&path),
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
            if result.starts_with('/') {
                result = &self.result[1..];
            }
            let destination = Path::new(prefix)
                .join(&result)
                .join(rel_location)
                .join(name);

            (source, destination)
        } else {
            let destination = Path::new(&self.result).join(rel_location).join(name);
            (source, destination)
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
            } else if entry_path.is_dir() {
                let rel_location = entry_path.strip_prefix(&self.source).unwrap();
                let destination = Path::new(&self.result).join(rel_location);
                if !destination.exists() {
                    fs::create_dir(destination)?
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
        if let std::collections::hash_map::Entry::Vacant(e) = self.map.entry(k) {
            e.insert(v);
            Ok(())
        } else {
            Err("key exists")
        }
    }

    /// This crate uses compile-time environment variables to transfer
    /// data to the main program. This funtction sets that variable
    fn to_env(&self) {
        let json = serde_json::to_string(&self).unwrap();
        let res = Path::new(CACHE_BUSTER_DATA_FILE);
        if res.exists() {
            fs::remove_file(&res).unwrap();
        }
        fs::write(CACHE_BUSTER_DATA_FILE, &json).unwrap();
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

    pub fn cleanup(config: &Buster) {
        let _ = fs::remove_dir_all(&config.result);
        delete_file();
    }

    pub fn delete_file() {
        let _ = fs::remove_file(&CACHE_BUSTER_DATA_FILE);
    }

    #[test]
    fn no_hash_validation_works() {
        let types = vec![
            mime::IMAGE_PNG,
            mime::IMAGE_SVG,
            mime::IMAGE_JPEG,
            mime::IMAGE_GIF,
        ];

        let no_hash =
            NoHashCategory::FilePaths(vec!["bbell.svg", "eye.svg", "a/b/c/d/s/d/svg/10.svg"]);

        assert!(BusterBuilder::default()
            .source("./dist")
            .result("/tmp/prod2i")
            .mime_types(types)
            .copy(true)
            .follow_links(true)
            .prefix("/test")
            .no_hash(vec![no_hash.clone()])
            .build()
            .is_err())
    }

    fn no_specific_mime() {
        delete_file();
        //use std::{thread, time};

        //let sleep = time::Duration::from_secs(4);

        //thread::sleep(sleep);

        const WASM: &str = "858fd6c482cc75111d54.module.wasm";
        let no_hash_files = vec![WASM, "bell.svg", "eye.svg", "a/b/c/d/s/d/svg/10.svg"];
        let no_hash = NoHashCategory::FilePaths(no_hash_files.clone());
        let config = BusterBuilder::default()
            .source("./dist")
            .result("/tmp/prod2ii")
            .copy(true)
            .follow_links(true)
            .no_hash(vec![no_hash.clone()])
            .build()
            .unwrap();
        config.process().unwrap();
        let files = Files::load();

        let no_hash_file = Path::new(&config.result).join(WASM);
        assert!(files.map.iter().any(|(k, v)| {
            let source = Path::new(&config.source).join(k);
            let dest = Path::new(&v);
            dest.file_name() == no_hash_file.file_name()
                && dest.exists()
                && source.file_name() == dest.file_name()
        }));

        no_hash_files.iter().for_each(|file| {
            assert!(files.map.iter().any(|(k, v)| {
                let source = Path::new(k);
                let dest = Path::new(&v);
                let no_hash = Path::new(file);
                source == Path::new(&config.source).join(file)
                    && dest.exists()
                    && no_hash.file_name() == dest.file_name()
            }));
        });

        for (k, v) in files.map.iter() {
            let src = Path::new(&k);
            let dest = Path::new(&v);

            assert_eq!(src.exists(), dest.exists());
        }

        cleanup(&config);
    }

    fn prefix_works() {
        delete_file();
        let types = vec![
            mime::IMAGE_PNG,
            mime::IMAGE_SVG,
            mime::IMAGE_JPEG,
            mime::IMAGE_GIF,
        ];

        let config = BusterBuilder::default()
            .source("./dist")
            .result("/tmp/prod2i")
            .mime_types(types)
            .copy(true)
            .follow_links(true)
            .prefix("/test")
            .build()
            .unwrap();

        config.process().unwrap();
        let mut files = Files::load();

        if let Some(prefix) = &config.prefix {
            for (k, v) in files.map.drain() {
                let src = Path::new(&k);
                let dest = Path::new(&v[prefix.len()..]);

                assert_eq!(src.exists(), dest.exists());
            }
        }

        cleanup(&config);
    }

    fn no_hash_extension_works() {
        delete_file();
        use std::{thread, time};

        let sleep = time::Duration::from_secs(4);
        const APPLICATION_WASM: &str = "wasm";
        const WASM: &str = "858fd6c482cc75111d54.module.wasm";

        thread::sleep(sleep);

        let no_hash_extensions = vec![APPLICATION_WASM];
        let no_hash_ext = NoHashCategory::FileExtentions(no_hash_extensions.clone());

        let no_hash_paths = vec!["bell.svg", "eye.svg", "a/b/c/d/s/d/svg/10.svg"];
        let no_hash_cat = NoHashCategory::FilePaths(no_hash_paths.clone());
        let no_hash = vec![no_hash_cat, no_hash_ext];

        let config = BusterBuilder::default()
            .source("./dist")
            .result("/tmp/prodnohashextension")
            .copy(true)
            .follow_links(true)
            .no_hash(no_hash.clone())
            .build()
            .unwrap();
        config.process().unwrap();
        let files = Files::load();

        assert!(files.map.iter().any(|(_k, v)| {
            let dest = Path::new(&v);
            dest.extension().unwrap().to_str().unwrap() == APPLICATION_WASM && dest.exists()
        }));

        let no_hash_file = Path::new(&config.result).join(WASM);
        assert!(files.map.iter().any(|(k, v)| {
            let source = Path::new(&config.source).join(k);
            let dest = Path::new(&v);
            dest.file_name() == no_hash_file.file_name()
                && dest.exists()
                && source.file_name() == dest.file_name()
        }));

        no_hash_paths.iter().for_each(|file| {
            assert!(files.map.iter().any(|(k, v)| {
                let source = Path::new(k);
                let dest = Path::new(&v);
                let no_hash = Path::new(file);
                source == Path::new(&config.source).join(file)
                    && dest.exists()
                    && no_hash.file_name() == dest.file_name()
            }));
        });

        for (k, v) in files.map.iter() {
            let src = Path::new(&k);
            let dest = Path::new(&v);

            assert_eq!(src.exists(), dest.exists());
        }

        cleanup(&config);
    }

    pub fn runner() {
        prefix_works();
        no_specific_mime();
        no_hash_extension_works();
    }
}
