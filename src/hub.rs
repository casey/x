use super::*;

pub(crate) struct Hub {
  #[allow(unused)]
  connections: Vec<midir::MidiInputConnection<()>>,
  messages: Arc<Mutex<VecDeque<Message>>>,
}

impl Hub {
  pub(crate) fn messages(&mut self) -> &Mutex<VecDeque<Message>> {
    &self.messages
  }

  pub(crate) fn new() -> Result<Self> {
    let messages = Arc::new(Mutex::new(VecDeque::new()));

    let mut connections = Vec::new();

    let input = midir::MidiInput::new("MIDI Input").context(error::MidiInputInit)?;
    for port in input.ports() {
      let name = input.port_name(&port).context(error::MidiPortInfo)?;
      let messages = messages.clone();
      connections.push(
        midir::MidiInput::new("Port MIDI Input")
          .context(error::MidiInputInit)?
          .connect(
            &port,
            &name,
            move |timestamp, event, ()| match Message::parse(timestamp, event) {
              Ok(message) => messages.lock().unwrap().push_back(message),
              Err(err) => log::warn!("MIDI event parse error: {err}"),
            },
            (),
          )
          .context(error::MidiInputPortConnect)?,
      );
    }

    Ok(Self {
      connections,
      messages,
    })
  }
}
