use {
  super::*,
  png::{ColorType, Compression, Encoder},
};

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

  pub(crate) fn resize(&mut self, width: u32, height: u32) {
    self.height = height;
    self.width = width;
    self.data.resize((width * height * 4).into_usize(), 0);
  }

  pub(crate) fn save(&self, path: &Path) -> Result {
    let file = File::create(path).context(error::FilesystemIo { path })?;

    let writer = BufWriter::new(file);

    let mut alpha = false;
    let mut color = false;
    let mut continuous = false;
    for chunk in self.data.chunks(4) {
      let chunk: [u8; 4] = chunk.try_into().unwrap();
      let [r, g, b, a] = chunk;

      if a != u8::MAX {
        alpha = true;
      }

      if r != g || r != b {
        color = true;
      }

      if chunk
        .iter()
        .any(|&channel| channel > 0 && channel < u8::MAX)
      {
        continuous = true;
      }
    }

    let color_type = match (color, alpha) {
      (false, false) => ColorType::Grayscale,
      (false, true) => ColorType::GrayscaleAlpha,
      (true, false) => ColorType::Rgb,
      (true, true) => ColorType::Rgba,
    };

    let data = if !continuous {
      todo!()
    } else {
      match color_type {
        ColorType::Grayscale => Cow::Owned(
          self
            .data
            .chunks(4)
            .map(|chunk| chunk[0])
            .collect::<Vec<u8>>(),
        ),
        ColorType::GrayscaleAlpha => Cow::Owned(
          self
            .data
            .chunks(4)
            .flat_map(|chunk| [chunk[0], chunk[3]])
            .collect::<Vec<u8>>(),
        ),
        ColorType::Rgb => Cow::Owned(
          self
            .data
            .chunks(4)
            .flat_map(|chunk| &chunk[0..3])
            .copied()
            .collect::<Vec<u8>>(),
        ),
        ColorType::Rgba => Cow::Borrowed(&self.data),
        ColorType::Indexed => unreachable!(),
      }
    };

    let mut encoder = Encoder::new(writer, self.width, self.height);
    encoder.set_color(color_type);
    encoder.set_compression(Compression::High);

    let mut writer = encoder.write_header().context(error::PngEncode { path })?;

    writer
      .write_image_data(&data)
      .context(error::PngEncode { path })?;

    writer.finish().context(error::PngEncode { path })?;

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn color_type_reduction() {
    #[track_caller]
    fn case(dir: &Path, data: &[u8], color_type: ColorType, expected: &[u8]) {
      let image = Image {
        data: data.into(),
        width: 2,
        height: 1,
      };

      let path = dir.join("image.png");

      image.save(&path).unwrap();

      let decoder = png::Decoder::new(BufReader::new(File::open(path).unwrap()));
      let mut reader = decoder.read_info().unwrap();
      let mut buffer = vec![0; reader.output_buffer_size().unwrap()];
      let info = reader.next_frame(&mut buffer).unwrap();
      assert_eq!(info.color_type, color_type);
      let bytes = &buffer[..info.buffer_size()];
      assert_eq!(bytes, expected);
    }

    let tempdir = tempfile::tempdir().unwrap();

    case(
      tempdir.path(),
      &[0, 0, 0, 255, 127, 127, 127, 255],
      ColorType::Grayscale,
      &[0, 127],
    );

    case(
      tempdir.path(),
      &[0, 0, 0, 255, 255, 255, 255, 127],
      ColorType::GrayscaleAlpha,
      &[0, 255, 255, 127],
    );

    case(
      tempdir.path(),
      &[0, 0, 0, 255, 0, 127, 255, 255],
      ColorType::Rgb,
      &[0, 0, 0, 0, 127, 255],
    );

    case(
      tempdir.path(),
      &[0, 0, 0, 255, 0, 127, 255, 127],
      ColorType::Rgba,
      &[0, 0, 0, 255, 0, 127, 255, 127],
    );
  }
}
