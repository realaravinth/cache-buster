## 0.2.0

### Added:

- `Buster.no_hash` Option to exclude select files from processing. These
  files will be copied over without any processing for cache
  busting(i.e, no renaming)

- `Buster.source` is tracked by cargo

### Changed:

- `Files::new()` takes a `&str`: Earlier versions were using
  environment variables to pass filemap information from `build.rs`
  component to program code but this proved to be unreliable. Starting
  with `0.2.0`, `cache_buster` will write filemap to
  `CACHE_BUSTER_DATA_FILE`(`./src/cache_buster_data.json`) and the user
  is requested to read and pass the value to `File::new()`

- `Files.mime_types` now accepts an `Option<Vec<mime::Mime>>`. When it
  is unset(i.e `None`), no mime based filtering is done and all files
  inside source directory is considered for processing.

- `Files::get_full_path()` and `Files::get()` now accept `impl
  Into<&str>` for path argument.

### Fixed:

- `Files::get()` now behaves as it is described in the documentation

## 0.1.1

### Added:

- Optional route prefix to `Processor`

### Changed:

- `Files::load()` became `Files::new()`

### Removed:

- Some methods on `Files` were for internal use only but they had a
  public API, they were modified to private.

## 0.1.0

### Added:

- `SHA-256`-based cache-buster
- runtime filemap loading
