/*
* Copyright (C) 2021  Aravinth Manivannan <realaravinth@batsense.net>
*
* Use of this source code is governed by the Apache 2.0 and/or the MIT
* License.
*/

use std::io::Error;
use std::path::Path;
use std::{fs, path::PathBuf};

use derive_builder::Builder;
use walkdir::WalkDir;

use crate::map::Files;

#[derive(Debug, Clone, Builder)]
pub struct Buster {
    // source directory
    #[builder(setter(into))]
    source: String,
    // mime_types for hashing
    mime_types: Vec<mime::Mime>,
    // directory for writing results
    #[builder(setter(into))]
    result: String,
    // copy other non-hashed files from source dire to result dir?
    copy: bool,
    follow_links: bool,
}

impl Buster {
    pub fn init(&self) -> Result<(), Error> {
        let res = Path::new(&self.result);
        if res.exists() {
            fs::remove_dir_all(&self.result).unwrap();
        }

        fs::create_dir(&self.result).unwrap();
        self.create_dir_structure(Path::new(&self.source))?;
        Ok(())
    }

    fn hasher(payload: &str) -> String {
        use data_encoding::HEXUPPER;
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(payload);
        HEXUPPER.encode(&hasher.finalize())
    }

    // if mime types are common, then you shoud be fine using this
    // use [hash] when when they aren't
    //
    // doesn't process files for which mime is not resolved
    pub fn try_hash(&self) -> Result<Files, Error> {
        let mut file_map: Files = Files::default();
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

    // panics when mimetypes are detected. This way you'll know which files are ignored
    // from processing
    pub fn hash(&self) -> Result<Files, Error> {
        let mut file_map: Files = Files::default();

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

    fn read_to_string(path: &Path) -> Result<String, Error> {
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        let input = File::open(path)?;
        let buffered = BufReader::new(input);

        let mut res = String::new();
        for line in buffered.lines() {
            res.push_str(&line?)
        }

        Ok(res)
    }

    fn gen_map<'a>(&self, source: &'a Path, name: &str) -> (&'a Path, PathBuf) {
        let rel_location = source.strip_prefix(&self.source).unwrap().parent().unwrap();
        let destination = Path::new(&self.result).join(rel_location).join(name);
        (source, destination)
    }

    fn copy(&self, source: &Path, name: &str) {
        let rel_location = source.strip_prefix(&self.source).unwrap().parent().unwrap();
        let destination = Path::new(&self.result).join(rel_location).join(name);
        fs::copy(source, &destination).unwrap();
    }

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
            .result("./prod")
            .mime_types(types)
            .copy(true)
            .follow_links(true)
            .build()
            .unwrap();

        config.init().unwrap();
        let mut files = config.hash().unwrap();

        for (k, v) in files.map.drain() {
            let src = Path::new(&k);
            let dest = Path::new(&v);

            assert_eq!(src.exists(), dest.exists());
        }
        cleanup(&config);
    }

    #[test]
    fn try_hash_works() {
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

        config.init().unwrap();
        let mut files = config.hash().unwrap();

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
