use arrayvec::ArrayString;
use circular_buffer::CircularBuffer;
use log::error;
use serde_json::Error;
use crate::message::message::Message;

const CIRCULAR_BUFFER_SIZE: usize = 15;

fn serialize_messages(messages: &CircularBuffer<CIRCULAR_BUFFER_SIZE, Message>) -> Result<String, Error> {
    match serde_json::to_string(&messages.iter().collect::<Vec<&Message>>()) {
        Ok(serialized) => Ok(serialized),
        Err(err) => Err(err)
    }
}

#[derive(Default)]
pub struct MessageBuffer {
    _buffer: CircularBuffer<CIRCULAR_BUFFER_SIZE, Message>,

    // Array string is stack allocated and provides higher performance
    _cache: ArrayString<{ 512 * CIRCULAR_BUFFER_SIZE }>
}

impl MessageBuffer {
    pub const fn new() -> Self {
        MessageBuffer {
            _buffer: CircularBuffer::<CIRCULAR_BUFFER_SIZE, Message>::new(),
            _cache: ArrayString::new_const()
        }
    }

    pub fn add_message(&mut self, author: String, message: String) {
        let id = self._buffer.back().map(|message| message.id + 1).unwrap_or(0);

        self._buffer.push_back(Message { id, author, message });

        match serialize_messages(&self._buffer) {
            Ok(json) => {
                self._cache.clear();
                self._cache.push_str(json.as_str());
            }
            Err(err) => error!("Could not serialize messages: {:?}", err)
        }
    }

    pub fn cached(&self) -> String {
        self._cache.to_string()
    }
}