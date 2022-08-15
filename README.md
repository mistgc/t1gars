Lightweight library written in Rust for handling the Truevision TGA image format.

For more information about the TGA format, please refer to the [specification](http://www.dca.fee.unicamp.br/~martino/disciplinas/ea978/tgaffs.pdf).

We can't use the struct 'Vec'. It will spend a lot of time. So we need directly use raw pointer and make it as slice.

So that it could be accessed like an array.


```rust
pub struct ColorMap {
    pub first_index: u16,
    pub entry_count: u16,
    pub bytes_per_entry: u8,
    pub pixels: *mut u8,
}

pub struct Tga {
    pub header: TgaHeader,
    pub info: TgaInfo,
    pub data: *mut u8,
    pub map: Option<ColorMap>,
}
```
