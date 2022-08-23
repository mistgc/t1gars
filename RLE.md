# RLE compression algorithm
The idea of RLE is classifying the linear sequence into two situations:

1. Continuous repetitive data block
2. Continuous non-repetitive data block

## Encode
Omit for now.

## Decode
Conditions:
1. the highest bit of the identification byte is `1`:

    Let $n$ equal from the lowest bit of the identification byte to the second highest bit of it.

    Just repeat the data byte $n$ times and write them to the buffer.

2. the highest bit of the identification byte is `0`

    Let $n$ equal from the lowest bit of the identification byte to the second highest bit of it.

    Write directly data bytes of length $n$ to the buffer.

```rust
pub fn RLE_decode(inbuf: &[u8], outbuf: &mut [u8]) -> Result<(), &str> {
    let idx: usize = 0;
    let decoded_length: usize = 0;
    while idx < inbuf.len() {
        let sign = inbuf[idx];
        let n = (sign & 0x3f) as usize;
        if (n as usize + decoded_length) > outbuf.len() {
            return Err("Error: the length of outbuf is'n enough.");
        }
        if (sign & 0x80) == 0x80 {
            for i in 0..n {
                outbuf[idx+i] = inbuf[idx+1];
            }
        } else {
            for i in 0..n {
                outbuf[idx+i] = inbuf[idx+1+i];
            }
        }
        idx += n + 1;
    }
    Ok(())
}
```
