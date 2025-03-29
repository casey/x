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
      println!("{name}");
      let midi_messages = messages.clone();
      connections.push(
        midir::MidiInput::new("Port MIDI Input")
          .context(error::MidiInputInit)?
          .connect(
            &port,
            &name,
            move |timestamp, event, _| {
              let event = midly::live::LiveEvent::parse(event).unwrap();
              let (channel, key, velocity, on) = match event {
                midly::live::LiveEvent::Midi { channel, message } => match message {
                  midly::MidiMessage::NoteOn { key, vel } => (channel, key, vel, true),
                  midly::MidiMessage::NoteOff { key, vel } => (channel, key, vel, false),
                  _ => return,
                },
                _ => return,
              };

              midi_messages.lock().unwrap().push_back(Message {
                on,
                timestamp,
                channel,
                key,
                velocity,
              });
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
