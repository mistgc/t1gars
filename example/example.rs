use t1gars::*;

fn test_cbw8() -> Result<(), Error> {
    let tga = Tga::new("example/images/CBW8.TGA")?;
    assert_eq!(tga.header.get_pixel_format().unwrap(), TgaPixelFormat::BW8);
    tga.save("example/images/temp_cbw8.tga")?;

    Ok(())
}

fn test_ctc24() -> Result<(), Error> {
    let tga = Tga::new("example/images/CTC24.TGA")?;
    assert_eq!(tga.header.get_pixel_format().unwrap(), TgaPixelFormat::RGB24);
    tga.save("example/images/temp_ctc24.tga")?;

    Ok(())
}

fn test_utc24() -> Result<(), Error> {
    let tga = Tga::new("example/images/UTC24.TGA")?;
    assert_eq!(tga.header.get_pixel_format().unwrap(), TgaPixelFormat::RGB24);
    tga.save("example/images/temp_utc24.tga")?;

    Ok(())
}

fn main() {
    println!("{:?}",test_cbw8());
    println!("{:?}",test_ctc24());
    println!("{:?}",test_utc24());
}
