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
