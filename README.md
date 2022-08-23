Lightweight library written in Rust for handling the Truevision TGA image format.

For more information about the TGA format, please refer to the [specification](http://www.dca.fee.unicamp.br/~martino/disciplinas/ea978/tgaffs.pdf).

## RLE
[RLE compression algorithm](RLE.md)

## Data Structure
We can't use the struct 'Vec'. It will spend a lot of time. So we need directly use raw pointer and make it as slice.

So that it could be accessed like an array.


```rust
struct LayPtr(Layout, *mut u8);

impl Drop for LayPtr {
   fn drop(&mut self) {
       if !self.1.is_null() {
           unsafe { alloc::dealloc(self.1, self.0) }
       }
   } 
}

pub struct ColorMap {
    pub first_index: u16,
    pub entry_count: u16,
    pub bytes_per_entry: u8,
    pub pixels: LayPtr,
}

pub struct Tga {
    pub header: TgaHeader,
    pub info: TgaInfo,
    pub data: LayPtr,
    pub map: Option<ColorMap>,
}
```
