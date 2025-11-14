use super::*;

// todo:
// - can frames arrive out of order?
// - save audio

pub(crate) struct Recorder {
  frames: Vec<Instant>,
  tempdir: TempDir,
}

impl Recorder {
  pub(crate) fn frame(&mut self, frame: Image, time: Instant) -> Result {
    let path = self
      .tempdir
      .path()
      .join(format!("{}.png", self.frames.len()));
    log::trace!("saving frame to {}", path.display());
    frame.save(&path)?;
    self.frames.push(time);
    Ok(())
  }

  pub(crate) fn new() -> Self {
    Self {
      frames: Vec::new(),
      tempdir: TempDir::new().unwrap(),
    }
  }

  pub(crate) fn save(&self) -> Result {
    const FRAMES: &str = "frames.text";
    const RECORDING: &str = "recording.mp4";

    log::info!(
      "saving {} frame recording to {RECORDING}",
      self.frames.len(),
    );

    let mut concat = "ffconcat version 1.0\n".to_owned();
    for (i, time) in self.frames.iter().enumerate() {
      writeln!(&mut concat, "file {i}.png").unwrap();
      if let Some(next) = self.frames.get(i + 1) {
        writeln!(
          &mut concat,
          "duration {}us",
          next.duration_since(*time).as_micros()
        )
        .unwrap();
      }
    }

    let path = self.tempdir.path().join(FRAMES);
    fs::write(&path, concat).context(error::FilesystemIo { path })?;

    let output = Command::new("ffmpeg")
      .args([
        "-vsync", "vfr", "-i", FRAMES, "-c:v", "libx264", "-pix_fmt", "yuv420p", RECORDING,
      ])
      .current_dir(self.tempdir.path())
      .output()
      .context(error::RecordingInvoke)?;

    if !output.status.success() {
      eprintln!("{}", String::from_utf8_lossy(&output.stdout));
      eprintln!("{}", String::from_utf8_lossy(&output.stderr));
      return Err(
        error::RecordingStatus {
          status: output.status,
        }
        .build(),
      );
    }

    fs::rename(self.tempdir.path().join("recording.mp4"), RECORDING)
      .context(error::FilesystemIo { path: RECORDING })?;

    Ok(())
  }
}
