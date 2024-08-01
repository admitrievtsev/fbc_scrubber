## Frequency Based Chunking Scrubber
FBC Scrubber is a scrubber that can be used to implement different FBC algorithms with ChunkFS

FBC Scrubber is currently under active development, breaking changes can always happen.

## Scrubber

To use Scrubber whith ChunkFS you need to create it:
```rust
extern crate fbc-chunker;

use fbc-chunker::{FBCMap, FBCScrubber};

fn main() -> io::Result<()> {
    let mut fs = FileSystem::new(HashMap::default(), Box::new(FBCMap::new()), Box::new(FBCScrubber::new()), Sha256Hasher::default());
}
```

Comments for each FBC method and optimization are provided in [analyser.rs](src/analyser.rs).

## Usage

Add the following dependency to your `Cargo.toml`:

```toml
[dependencies]
chunking = { git = "[https://github.com/Piletskii-Oleg/chunkfs.git](https://github.com/admitrievtsev/fbc-chunker.git)" }
```
## Example

```rust
extern crate fbc-chunker;
extern crate chunkfs;

use fbc-chunker::{FBCMap, FBCScrubber};
use std::io;
use chunkfs::FileSystem;
use std::collections::HashMap;
use chunkfs::hashers::Sha256Hasher;
use chunkfs::chunkers::SuperChunker;

fn main() -> io::Result<()> {
    let mut fs = FileSystem::new(HashMap::default(), Box::new(FBCMap::new()), Box::new(FBCScrubber::new()), Sha256Hasher::default());
    let mut handle = fs.create_file("file".to_string(), SuperChunker::new(), true)?;
    let data = vec![10; 1024 * 1024];
    fs.write_to_file(&mut handle, &data)?;
    fs.close_file(handle)?;

    let res = fs.scrub().unwrap();
    println!("{res:?}");

    let mut handle = fs.open_file("file", SuperChunker::new())?;
    let read = fs.read_file_complete(&mut handle)?;
    assert_eq!(read.len(), data.len());
    Ok(())
}
```
