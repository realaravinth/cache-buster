<div align="center">
  <h1>cache-buster</h1>
  <p>
    <strong>cache-buster - A library that aids in staticfile cache busting with SHA-258 hashes</strong>
  </p>

[![Documentation](https://img.shields.io/badge/docs-master-blue)](https://realaravinth.github.io/cache-buster/cache_buster/index.html)
![CI (Linux)](<https://github.com/realaravinth/cache-buster/workflows/CI%20(Linux)/badge.svg>)
[![dependency status](https://deps.rs/repo/github/realaravinth/cache-buster/status.svg)](https://deps.rs/repo/github/realaravinth/cache-buster)
<br />
[![codecov](https://codecov.io/gh/realaravinth/cache-buster/branch/master/graph/badge.svg)](https://codecov.io/gh/realaravinth/cache-buster)

</div>

## Features

- [x] `SHA-256` based name generation during compile-time
- [x] Processes files based on provided MIME filters
- [ ] Exposes modified names to program during runtime

## Usage:

Add this to your `Cargo.toml`:

```toml
cache-buster = { version = "0.2", git = "https://github.com/realaravinth/cache-buster" }
```

## Examples:

TODO
