# Goxoy File Chunker

[![Version](https://img.shields.io/crates/v/goxoy-file-chunker)](https://crates.io/crates/tempmail)
[![Downloads](https://img.shields.io/crates/d/goxoy-file-chunker)](https://crates.io/crates/goxoy-file-chunker)
[![License](https://img.shields.io/crates/l/goxoy-file-chunker)](https://crates.io/crates/goxoy-file-chunker)
[![Docs](https://docs.rs/goxoy-file-chunker/badge.svg)](https://docs.rs/goxoy-file-chunker)

This library was written to split large files into pieces of certain sizes.

## Split Example

```rust
    // create FileChunk object
    let mut file_obj=FileChunk::new();
    file_obj.set_storage_path(DefaultStoragePath::TempPath);
    // set target file name
    file_obj.assign_file("file_name.extension");
    if file_obj.is_exist==true{
        file_obj.set_size(256,FileChunkType::KiloByte);
        let split_result=file_obj.split();
        if split_result==true{
            println!("chunks ready");
        }else{
            println!("error accoured");
        }
    }else{
        println!("file does not exist");
    }

```


## Merge Example

```rust
    // create FileChunk object
    let mut file_obj=FileChunk::new();
    let merge_result=file_obj.merge("path_name");
    if merge_result==true{
        println!("file merged");
    }else{
        println!("error accoured");
    }

```
  
## Lisans

[MIT](https://choosealicense.com/licenses/mit/)