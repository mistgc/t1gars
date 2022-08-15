use t1gars::*;

fn test() -> Result<(), Error> {
    let tga = Tga::new("example/images/CBW8.TGA")?;
    assert_eq!(tga.header.get_pixel_format().unwrap(), TgaPixelFormat::BW8);

    Ok(())
}

fn main() {
    println!("{:?}",test());
}
