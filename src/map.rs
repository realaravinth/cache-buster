/*
* Copyright (C) 2021  Aravinth Manivannan <realaravinth@batsense.net>
*
* Use of this source code is governed by the Apache 2.0 and/or the MIT
* License.
*/

use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct Files {
    pub map: HashMap<String, String>,
    base_dir: String,
}

impl Files {
    pub fn get<'a>(&'a self, path: &'a str) -> Option<&'a String> {
        self.map.get(path)
    }

    pub fn add(&mut self, k: String, v: String) -> Result<(), &'static str> {
        if self.map.contains_key(&k) {
            Err("key exists")
        } else {
            self.map.insert(k, v);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::hash::*;

    use super::*;
    use std::path::Path;

    #[test]
    fn get_works() {
        let types = vec![
            mime::IMAGE_PNG,
            mime::IMAGE_SVG,
            mime::IMAGE_JPEG,
            mime::IMAGE_GIF,
        ];

        let config = BusterBuilder::default()
            .source("./dist")
            .result("/tmp/prod2")
            .mime_types(types)
            .copy(true)
            .follow_links(true)
            .build()
            .unwrap();

        config.init().unwrap();
        let files = config.hash().unwrap();

        assert!(file_exists("./dist/log-out.svg", &files));
        assert!(file_exists(
            "./dist/a/b/c/d/s/d/svg/credit-card.svg",
            &files
        ));

        assert!(!file_exists("dist/log-out.svg", &files));
        assert!(!file_exists("dist/a/b/c/d/s/d/svg/credit-card.svg", &files));
    }

    fn file_exists(path: &str, files: &Files) -> bool {
        if let Some(file) = files.get(path) {
            Path::new(file).exists()
        } else {
            false
        }
    }
}
