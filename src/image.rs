use super::*;

#[derive(Default, Debug, PartialEq)]
pub(crate) struct Image {
  data: Vec<u8>,
  height: u32,
  width: u32,
}

impl Image {
  pub(crate) fn data_mut(&mut self) -> &mut [u8] {
    &mut self.data
  }

  pub(crate) fn load(path: &Path) -> Result<Self> {
    let decoder = png::Decoder::new(BufReader::new(
      File::open(path).context(error::FilesystemIo { path })?,
    ));
    let mut reader = decoder.read_info().context(error::PngDecode { path })?;
    let mut buf = vec![
      0;
      reader
        .output_buffer_size()
        .context(error::PngOutputBufferSize { path })?
    ];
    let info = reader
      .next_frame(&mut buf)
      .context(error::PngDecode { path })?;
    let data = &buf[..info.buffer_size()];
    let info = reader.info();

    Ok(Self {
      data: data.into(),
      height: info.height,
      width: info.width,
    })
  }

  pub(crate) fn resize(&mut self, width: u32, height: u32) {
    self.height = height;
    self.width = width;
    self.data.resize((width * height * 4).into_usize(), 0);
  }

  pub(crate) fn save(&self, path: &Path) -> Result {
    let file = File::create(path).context(error::FilesystemIo { path })?;

    let writer = BufWriter::new(file);

    let mut encoder = png::Encoder::new(writer, self.width, self.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header().context(error::PngEncode { path })?;

    writer
      .write_image_data(&self.data)
      .context(error::PngEncode { path })?;

    writer.finish().context(error::PngEncode { path })?;

    Ok(())
  }
}
