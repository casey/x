use super::*;
use std::convert::TryInto;

pub(crate) struct VideoRecorder {
  encoder: ffmpeg::codec::encoder::video::Encoder,
  finished: bool,
  next_pts: i64,
  output: ffmpeg::format::context::Output,
  packet: ffmpeg::Packet,
  rgba_frame: ffmpeg::frame::Video,
  scaler: ffmpeg::software::scaling::context::Context,
  stream_index: usize,
  time_base: ffmpeg::Rational,
  yuv_frame: ffmpeg::frame::Video,
}

impl VideoRecorder {
  pub(crate) fn new(path: PathBuf, width: u32, height: u32, frame_rate: u32) -> Result<Self> {
    ffmpeg::init().context(error::VideoStart)?;

    let mut output = ffmpeg::format::output(&path).context(error::VideoStart)?;

    let codec = ffmpeg::encoder::find(ffmpeg::codec::Id::H264)
      .ok_or_else(|| Error::internal("H.264 encoder not available"))?;

    let global_header = output
      .format()
      .flags()
      .contains(ffmpeg::format::flag::Flags::GLOBAL_HEADER);

    let mut stream = output.add_stream(codec).context(error::VideoStart)?;

    let context = ffmpeg::codec::context::Context::new_with_codec(codec);
    let mut encoder = context.encoder().video().context(error::VideoStart)?;

    let frame_rate_i32 = frame_rate.try_into().unwrap_or(i32::MAX);
    let time_base = ffmpeg::Rational(1, frame_rate_i32);
    let frame_rate = ffmpeg::Rational(frame_rate_i32, 1);

    encoder.set_width(width);
    encoder.set_height(height);
    encoder.set_format(ffmpeg::format::Pixel::YUV420P);
    encoder.set_time_base(time_base);
    encoder.set_frame_rate(Some(frame_rate));
    encoder.set_bit_rate(12_000_000);
    encoder.set_gop(12);
    encoder.set_max_b_frames(2);

    if global_header {
      encoder.set_flags(ffmpeg::codec::Flags::GLOBAL_HEADER);
    }

    let opened_encoder = encoder.open().context(error::VideoStart)?;

    stream.set_time_base(time_base);
    stream.set_rate(frame_rate);
    stream.set_avg_frame_rate(frame_rate);
    stream.set_parameters(&opened_encoder);
    let stream_index = stream.index();
    drop(stream);

    output.write_header().context(error::VideoStart)?;

    let scaler = ffmpeg::software::scaling::context::Context::get(
      ffmpeg::format::Pixel::RGBA,
      width,
      height,
      ffmpeg::format::Pixel::YUV420P,
      width,
      height,
      ffmpeg::software::scaling::flag::Flags::BILINEAR,
    )
    .context(error::VideoStart)?;

    Ok(Self {
      encoder: opened_encoder,
      finished: false,
      next_pts: 0,
      output,
      packet: ffmpeg::Packet::empty(),
      rgba_frame: ffmpeg::frame::Video::new(ffmpeg::format::Pixel::RGBA, width, height),
      scaler,
      stream_index,
      time_base,
      yuv_frame: ffmpeg::frame::Video::new(ffmpeg::format::Pixel::YUV420P, width, height),
    })
  }

  pub(crate) fn encode(&mut self, image: &Image) -> Result {
    self.upload(image);

    self
      .scaler
      .run(&self.rgba_frame, &mut self.yuv_frame)
      .context(error::VideoEncode)?;

    let pts = self.next_pts;
    self.next_pts += 1;
    self.yuv_frame.set_pts(Some(pts));

    self
      .encoder
      .send_frame(&self.yuv_frame)
      .context(error::VideoEncode)?;
    self.write_pending_packets().context(error::VideoEncode)?;

    Ok(())
  }

  pub(crate) fn finish(&mut self) -> Result {
    if self.finished {
      return Ok(());
    }

    self.encoder.send_eof().context(error::VideoFinish)?;
    self.write_pending_packets().context(error::VideoFinish)?;
    self.output.write_trailer().context(error::VideoFinish)?;
    self.finished = true;

    Ok(())
  }

  fn upload(&mut self, image: &Image) {
    let row_bytes = image.width().into_usize() * CHANNELS.into_usize();
    let stride = self.rgba_frame.stride(0);

    for (src, dst) in image
      .data()
      .chunks(row_bytes)
      .zip(self.rgba_frame.data_mut(0).chunks_mut(stride))
    {
      dst[..row_bytes].copy_from_slice(src);
    }

    self.rgba_frame.set_pts(Some(self.next_pts));
  }

  fn write_pending_packets(&mut self) -> std::result::Result<(), ffmpeg::Error> {
    use ffmpeg::error::EAGAIN;

    loop {
      match self.encoder.receive_packet(&mut self.packet) {
        Ok(()) => {
          self.packet.set_stream(self.stream_index);
          self.packet.rescale_ts(self.time_base, self.time_base);
          self.packet.write_interleaved(&mut self.output)?;
        }
        Err(ffmpeg::Error::Eof) => break,
        Err(ffmpeg::Error::Other { errno }) if errno == EAGAIN => break,
        Err(err) => return Err(err),
      }
    }

    Ok(())
  }
}
