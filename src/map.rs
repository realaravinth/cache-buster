/*
* Copyright (C) 2021  Aravinth Manivannan <realaravinth@batsense.net>
*
* Use of this source code is governed by the Apache 2.0 and/or the MIT
* License.
*/

use std::collections::HashMap;
use std::env;

use serde::{Deserialize, Serialize};

const ENV_VAR_NAME: &str = "CACHE_BUSTER_FILE_MAP";

#[derive(Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
pub struct Files {
    pub map: HashMap<String, String>,
    base_dir: String,
}

impl Files {
    pub fn new(base_dir: &str) -> Self {
        Files {
            map: HashMap::default(),
            base_dir: base_dir.into(),
        }
    }

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

    pub fn to_env(&self) {
        println!(
            "cargo:rustc-env={}={}",
            ENV_VAR_NAME,
            serde_json::to_string(&self).unwrap()
        );

        // needed for testing load()
        // if the above statement fails(println), then something's broken
        // with the rust compiler. So not really worried about that.
        #[cfg(test)]
        env::set_var(ENV_VAR_NAME, serde_json::to_string(&self).unwrap());
    }

    pub fn load() -> Self {
        let env = env::var(ENV_VAR_NAME)
            .expect("unable to read env var, might be a bug in lib. Please report on GitHub");
        let res: Files = serde_json::from_str(&env).unwrap();
        res
    }
}

#[cfg(test)]
mod tests {
    use crate::hash::tests::cleanup;
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
        cleanup(&config);
    }

    fn file_exists(path: &str, files: &Files) -> bool {
        if let Some(file) = files.get(path) {
            Path::new(file).exists()
        } else {
            false
        }
    }

    #[test]
    fn load_works() {
        let types = vec![
            mime::IMAGE_PNG,
            mime::IMAGE_SVG,
            mime::IMAGE_JPEG,
            mime::IMAGE_GIF,
        ];

        let config = BusterBuilder::default()
            .source("./dist")
            .result("/tmp/prod3")
            .mime_types(types)
            .copy(true)
            .follow_links(true)
            .build()
            .unwrap();

        config.init().unwrap();
        let files = config.hash().unwrap();

        files.to_env();

        let x = Files::load();

        assert_eq!(files, x);

        cleanup(&config);
    }
}
