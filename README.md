<div align="center">
<img
width="250px"
  class="greetings"
  src="./examples/actix-web/static/cachable/img/Spock_vulcan-salute.png"
  alt="logo image"
/>
<h1>
  Cache Buster
</h1>
<p>
  <strong>May your cache live long and prosper!</strong>
</p>

[![Documentation](https://img.shields.io/badge/docs-master-blue)](https://realaravinth.github.io/cache-buster/cache_buster/index.html)
![CI (Linux)](<https://github.com/realaravinth/cache-buster/workflows/CI%20(Linux)/badge.svg>)
[![dependency status](https://deps.rs/repo/github/realaravinth/cache-buster/status.svg)](https://deps.rs/repo/github/realaravinth/cache-buster)
<br />
[![codecov](https://codecov.io/gh/realaravinth/cache-buster/branch/master/graph/badge.svg)](https://codecov.io/gh/realaravinth/cache-buster)

</div>

## What is cache busting?

To optimise network load time, browsers cache static files. Caching
greatly improves performance but how do you inform browsers to
invalidate cache when your files have changed?

Cache busting is a simple but effective solution for this issue. There
are several ways to achieve this but the way this library does this is
by changing file names to include the hash of the files' contents.

So if you have `bundle.js`, it will become
`bundle.<long-sha256-hash>.js`. This lets you set a super long cache age
as, because of the file names changing, the path to the filename, too,
will change. So as far as the browser is concerned, you are trying to load
a file that it doesn't have. Pretty neat, isn't it?

## Features

-   [x] `SHA-256` based name generation during compile-time
-   [x] Processes files based on provided MIME filters
-   [x] Exclude certain files from processing based on file extensions
        and/or file paths
-   [x] Exposes modified names to program during runtime
-   [x] Route prefixes(optional)

## Usage:

Add this to your `Cargo.toml`:

```toml
cache-buster = { version = "0.2", git = "https://github.com/realaravinth/cache-buster" }
```

## Examples:

-   See [acix-example](./examples/actix-web)
-   See [mCaptcha/mcaptcha](https://github.com/mCaptcha/mcaptcha) for use
    with [Sailfish](https://crates.io/crates/sailfish) template engine.
