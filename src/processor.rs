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

use std::io::Error;
use std::path::Path;
use std::{fs, path::PathBuf};

use derive_builder::Builder;
use walkdir::WalkDir;

use crate::Files;

/// Configuration for setting up cache-busting
#[derive(Debug, Clone, Builder)]
pub struct Buster {
    /// source directory
    #[builder(setter(into))]
    source: String,
    /// mime_types for hashing
    mime_types: Vec<mime::Mime>,
    /// directory for writing results
    #[builder(setter(into))]
    result: String,
    /// copy other non-hashed files from source dire to result dir?
    copy: bool,
    /// follow symlinks?
    follow_links: bool,
}

impl Buster {
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
    /// If MIME types are uncommon, then use this funtion
    /// as it won't panic when a weird MIM is encountered.
    ///
    /// Otherwise, use [process][Self::process]
    ///
    /// Note: it omits processing uncommon MIME types
    pub fn try_process(&self) -> Result<Files, Error> {
        self.init()?;
        let mut file_map: Files = Files::new(&self.result);
        for entry in WalkDir::new(&self.source)
            .follow_links(self.follow_links)
            .into_iter()
        {
            let entry = entry?;
            let path = entry.path();
            let path = Path::new(&path);

            for mime_type in self.mime_types.iter() {
                if let Some(file_mime) = mime_guess::from_path(path).first() {
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

        Ok(file_map)
    }

    /// Processes files.
    ///
    /// If MIME types are common, then use this funtion
    /// as it will panic when a weird MIM is encountered.
    pub fn process(&self) -> Result<Files, Error> {
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
            if !path.is_dir() {
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

        Ok(file_map)
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
    fn gen_map<'a>(&self, source: &'a Path, name: &str) -> (&'a Path, PathBuf) {
        let rel_location = source.strip_prefix(&self.source).unwrap().parent().unwrap();
        let destination = Path::new(&self.result).join(rel_location).join(name);
        (source, destination)
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

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn hasher_works() {
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

        let mut files = config.process().unwrap();

        for (k, v) in files.map.drain() {
            let src = Path::new(&k);
            let dest = Path::new(&v);

            assert_eq!(src.exists(), dest.exists());
        }
        cleanup(&config);
    }

    #[test]
    fn try_process_works() {
        let types = vec![
            mime::IMAGE_PNG,
            mime::IMAGE_SVG,
            mime::IMAGE_JPEG,
            mime::IMAGE_GIF,
        ];

        let config = BusterBuilder::default()
            .source("./dist")
            .result("/tmp/prod")
            .mime_types(types)
            .copy(true)
            .follow_links(true)
            .build()
            .unwrap();

        let mut files = config.try_process().unwrap();

        for (k, v) in files.map.drain() {
            let src = Path::new(&k);
            let dest = Path::new(&v);

            assert_eq!(src.exists(), dest.exists());
        }

        cleanup(&config);
    }

    pub fn cleanup(config: &Buster) {
        let _ = fs::remove_dir_all(&config.result);
    }
}
