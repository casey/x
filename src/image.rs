use super::*;

pub(crate) struct Image {
  data: Vec<u8>,
  height: u32,
  width: u32,
}

impl Image {
  pub(crate) fn new(width: u32, height: u32, data: Vec<u8>) -> Self {
    assert_eq!(width * height * 4, data.len().try_into().unwrap());
    Self {
      data,
      height,
      width,
    }
  }

  pub(crate) fn write(&self, path: &Utf8Path) -> Result {
    let file = File::create(path)?;
    let mut encoder = png::Encoder::new(file, self.width, self.height);
    encoder.set_color(png::ColorType::Rgba);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&self.data)?;
    writer.finish()?;
    Ok(())
  }
}
