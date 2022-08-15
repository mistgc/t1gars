use super::*;
#[test]
fn test_bits_to_bytes() {
   assert_eq!(t1gars::bits_to_bytes(16), 2);
   assert_eq!(t1gars::bits_to_bytes(17), 3);
}

#[test]
fn test_tga_new() {
    let tga = Tga::new("example/images/CBW8.TGA").unwrap();
    assert_eq!(tga.header.get_pixel_format().unwrap(), TgaPixelFormat::BW8);
}
