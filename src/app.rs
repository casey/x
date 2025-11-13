use super::*;

pub(crate) struct App {
  analyzer: Analyzer,
  error: Option<Error>,
  horizontal: f32,
  hub: Hub,
  makro: Vec<Key>,
  options: Options,
  #[allow(unused)]
  output_stream: OutputStream,
  recording: Option<Vec<Key>>,
  renderer: Option<Renderer>,
  scaling: f32,
  #[allow(unused)]
  sink: Sink,
  start: Instant,
  state: State,
  stream: Option<Box<dyn Stream>>,
  translation: Vec2f,
  vertical: f32,
  window: Option<Arc<Window>>,
  wrap: bool,
  zoom: f32,
}

impl App {
  fn capture(&mut self) -> Result {
    self.renderer.as_ref().unwrap().capture(|capture| {
      capture.save("capture.png".as_ref()).unwrap();
    })?;
    Ok(())
  }

  pub(crate) fn error(self) -> Option<Error> {
    self.error
  }

  fn find_song(song: &str) -> Result<PathBuf> {
    let song = RegexBuilder::new(song)
      .case_insensitive(true)
      .build()
      .context(error::SongRegex)?;

    let mut matches = Vec::<PathBuf>::new();

    let home = dirs::home_dir().context(error::Home)?;

    let music = home.join("Music/Music/Media.localized/Music");

    for entry in WalkDir::new(&music) {
      let entry = entry.context(error::SongWalk)?;

      if entry.file_type().is_dir() {
        continue;
      }

      let path = entry.path();

      let haystack = path.strip_prefix(&music).unwrap().with_extension("");

      let Some(haystack) = haystack.to_str() else {
        continue;
      };

      if song.is_match(haystack) {
        matches.push(path.into());
      }
    }

    if matches.len() > 1 {
      return Err(error::SongAmbiguous { matches }.build());
    }

    match matches.into_iter().next() {
      Some(path) => Ok(path),
      None => Err(error::SongMatch { song }.build()),
    }
  }

  pub(crate) fn new(options: Options) -> Result<Self> {
    let host = cpal::default_host();

    let output_device = host
      .default_output_device()
      .context(error::AudioDefaultOutputDevice)?;

    let stream_config = Self::stream_config(
      output_device
        .supported_output_configs()
        .context(error::AudioSupportedStreamConfigs)?,
    )?;

    let output_stream = rodio::OutputStreamBuilder::from_device(output_device)
      .context(error::AudioBuildOutputStream)?
      .with_supported_config(&stream_config)
      .open_stream()
      .context(error::AudioBuildOutputStream)?;

    let sink = Sink::connect_new(output_stream.mixer());

    if let Some(volume) = options.volume {
      sink.set_volume(volume);
    }

    let stream: Option<Box<dyn Stream>> = if let Some(track) = &options.track {
      let track = Track::new(track)?;
      sink.append(track.clone());
      Some(Box::new(track))
    } else if let Some(song) = &options.song {
      let track = Track::new(&Self::find_song(song)?)?;
      sink.append(track.clone());
      Some(Box::new(track))
    } else if options.input {
      let input_device = host
        .default_input_device()
        .context(error::AudioDefaultInputDevice)?;

      let stream_config = Self::stream_config(
        input_device
          .supported_input_configs()
          .context(error::AudioSupportedStreamConfigs)?,
      )?;

      Some(Box::new(Input::new(input_device, stream_config)?))
    } else {
      None
    };

    let mut state = options.program.map(Program::state).unwrap_or_default();

    if let Some(db) = options.db {
      state.db = db;
    }

    Ok(Self {
      analyzer: Analyzer::new(),
      error: None,
      horizontal: 0.0,
      hub: Hub::new()?,
      makro: Vec::new(),
      options,
      output_stream,
      recording: None,
      renderer: None,
      scaling: 1.0,
      sink,
      start: Instant::now(),
      state,
      stream,
      translation: Vec2f::zeros(),
      vertical: 0.0,
      window: None,
      wrap: true,
      zoom: 0.0,
    })
  }

  fn press(&mut self, event_loop: &ActiveEventLoop, key: Key) {
    let mut capture = true;

    match key {
      Key::Character(ref c) => match c.as_str() {
        "+" => {
          self.state.db += 1.0;
        }
        "-" => {
          self.state.db -= 1.0;
        }
        ">" => {
          if let Err(err) = self.capture() {
            self.error = Some(err);
            event_loop.exit();
          }
        }
        "@" => {
          for key in self.makro.clone() {
            self.press(event_loop, key);
          }
          capture = false;
        }
        "a" => self.state.filters.push(Filter {
          color: invert_color(),
          field: Field::All,
          wrap: self.wrap,
          ..default()
        }),
        "c" => self.state.filters.push(Filter {
          color: invert_color(),
          field: Field::Circle,
          wrap: self.wrap,
          ..default()
        }),
        "d" => self.state.filters.push(Filter {
          coordinates: true,
          wrap: self.wrap,
          ..default()
        }),
        "f" => {
          self.options.fit = !self.options.fit;
        }
        "l" => self.state.filters.push(Filter {
          color: invert_color(),
          field: Field::Frequencies,
          wrap: self.wrap,
          ..default()
        }),
        "n" => self.state.filters.push(Filter {
          field: Field::None,
          wrap: self.wrap,
          ..default()
        }),
        "q" => {
          if let Some(recording) = self.recording.take() {
            self.makro = recording;
          } else {
            self.recording = Some(Vec::new());
          }
          capture = false;
        }
        "r" => {
          self.options.repeat = !self.options.repeat;
        }
        "s" => self.state.filters.push(Filter {
          color: invert_color(),
          field: Field::Samples,
          wrap: self.wrap,
          ..default()
        }),
        "t" => {
          self.options.tile = !self.options.tile;
        }
        "w" => {
          self.wrap = !self.wrap;
        }
        "x" => self.state.filters.push(Filter {
          color: invert_color(),
          field: Field::X,
          wrap: self.wrap,
          ..default()
        }),
        "z" => self.state.filters.push(Filter {
          position: Mat3f::new_scaling(2.0),
          wrap: self.wrap,
          ..default()
        }),
        _ => {}
      },
      Key::Named(key) => match key {
        NamedKey::Backspace => {
          self.state.filters.pop();
        }
        NamedKey::ArrowLeft => {
          self.state.filters.push(Filter {
            position: Mat3f::new_rotation(-0.01),
            ..default()
          });
        }
        NamedKey::ArrowRight => {
          self.state.filters.push(Filter {
            position: Mat3f::new_rotation(0.01),
            ..default()
          });
        }
        _ => {}
      },
      _ => {}
    }

    if capture && let Some(recording) = &mut self.recording {
      recording.push(key);
    }
  }

  fn redraw(&mut self, event_loop: &ActiveEventLoop) {
    for message in self.hub.messages().lock().unwrap().drain(..) {
      match message.tuple() {
        (Device::Spectra, 0, Event::Button(true)) => self.state.filters.push(Filter {
          color: invert_color(),
          field: Field::Top,
          wrap: self.wrap,
          ..default()
        }),
        (Device::Spectra, 1, Event::Button(true)) => self.state.filters.push(Filter {
          color: invert_color(),
          field: Field::Bottom,
          wrap: self.wrap,
          ..default()
        }),
        (Device::Spectra, 2, Event::Button(true)) => self.state.filters.push(Filter {
          color: invert_color(),
          field: Field::X,
          wrap: self.wrap,
          ..default()
        }),
        (Device::Spectra, 3, Event::Button(true)) => self.state.filters.push(Filter {
          color: invert_color(),
          field: Field::Circle,
          wrap: self.wrap,
          ..default()
        }),
        (Device::Spectra, 4, Event::Button(true)) => self.state.filters.push(Filter {
          position: Mat3f::new_scaling(2.0),
          wrap: self.wrap,
          ..default()
        }),
        (Device::Spectra, 5, Event::Button(true)) => self.state.filters.push(Filter {
          position: Mat3f::new_scaling(0.5),
          wrap: self.wrap,
          ..default()
        }),
        (Device::Spectra, 6, Event::Button(true)) => self.state.filters.push(Filter {
          position: Mat3f::new_translation(&Vec2f::new(-0.1, 0.0)),
          wrap: self.wrap,
          ..default()
        }),
        (Device::Spectra, 7, Event::Button(true)) => self.state.filters.push(Filter {
          position: Mat3f::new_translation(&Vec2f::new(0.1, 0.0)),
          wrap: self.wrap,
          ..default()
        }),
        (Device::Spectra, 8, Event::Button(true)) => {
          self.state.filters.pop();
        }
        (Device::Twister, control, Event::Button(true)) => match control {
          4 => self.translation.x = 0.0,
          5 => self.translation.y = 0.0,
          6 => self.scaling = 1.0,
          _ => {}
        },
        (Device::Twister, control, Event::Encoder(parameter)) => {
          self.state.parameter = parameter;
          match control {
            0 => self.state.alpha = parameter,
            1 => self.state.db = parameter.value() as f32,
            4 => self.horizontal = parameter.bipolar(),
            5 => self.vertical = parameter.bipolar(),
            6 => self.zoom = parameter.bipolar(),
            _ => {}
          }
        }
        _ => {}
      }
    }

    if let Some(stream) = self.stream.as_mut() {
      self.analyzer.update(stream.as_mut(), &self.state);
    }

    let now = Instant::now();
    let elapsed = (now - self.start).as_secs_f32();
    self.start = now;

    self.scaling -= self.zoom * elapsed;
    self.translation.x -= self.horizontal * 4.0 * elapsed;
    self.translation.y -= self.vertical * 4.0 * elapsed;

    self.state.filters.push(Filter {
      position: Mat3f::new_translation(&self.translation).prepend_scaling(self.scaling),
      wrap: self.wrap,
      ..default()
    });

    if let Err(err) =
      self
        .renderer
        .as_mut()
        .unwrap()
        .render(&self.options, &self.analyzer, &self.state)
    {
      self.error = Some(err);
      event_loop.exit();
      return;
    }

    self.state.filters.pop();

    self.window().request_redraw();
  }

  fn stream_config(
    configs: impl Iterator<Item = SupportedStreamConfigRange>,
  ) -> Result<SupportedStreamConfig> {
    let config = configs
      .max_by_key(SupportedStreamConfigRange::max_sample_rate)
      .context(error::AudioSupportedStreamConfig)?;

    Ok(SupportedStreamConfig::new(
      config.channels(),
      config.max_sample_rate(),
      match config.buffer_size() {
        SupportedBufferSize::Range { min, .. } => SupportedBufferSize::Range {
          min: *min,
          max: *min,
        },
        SupportedBufferSize::Unknown => SupportedBufferSize::Unknown,
      },
      config.sample_format(),
    ))
  }

  fn window(&self) -> &Window {
    self.window.as_ref().unwrap()
  }
}

impl ApplicationHandler for App {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    if self.window.is_none() {
      assert!(self.renderer.is_none());

      let window = match event_loop
        .create_window(
          WindowAttributes::default()
            .with_inner_size(PhysicalSize {
              width: 1024,
              height: 1024,
            })
            .with_min_inner_size(PhysicalSize {
              width: 256,
              height: 256,
            })
            .with_title("x"),
        )
        .context(error::CreateWindow)
      {
        Ok(window) => Arc::new(window),
        Err(err) => {
          self.error = Some(err);
          event_loop.exit();
          return;
        }
      };

      self.window = Some(window.clone());

      let renderer = match pollster::block_on(Renderer::new(&self.options, window)) {
        Ok(renderer) => renderer,
        Err(err) => {
          self.error = Some(err);
          event_loop.exit();
          return;
        }
      };

      self.renderer = Some(renderer);
    }
  }

  fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
    if self.renderer.is_none() {
      event_loop.exit();
      return;
    }

    match event {
      WindowEvent::CloseRequested => {
        event_loop.exit();
      }
      WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
        self.press(event_loop, event.logical_key);
      }
      WindowEvent::RedrawRequested => {
        self.redraw(event_loop);
      }
      WindowEvent::Resized(size) => {
        self.renderer.as_mut().unwrap().resize(&self.options, size);
        self.window().request_redraw();
      }
      _ => {}
    }
  }
}
