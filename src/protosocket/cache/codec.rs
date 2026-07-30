//! Protosocket codec for the cache command/response protocol.
//!
//! This intentionally does *not* use the `protosocket-prost` crate. That crate is built
//! against prost 0.14, while `momento-protos` generates its types with prost 0.13 (paired
//! with tonic 0.13). Depending on both would put two incompatible `prost::Message` traits
//! in the graph, and the generated `CacheCommand`/`CacheResponse` would implement the wrong
//! one. The `protosocket::Serialize`/`Decoder` traits are prost-agnostic, so we implement
//! them directly here and stay on whatever prost version `momento-protos` uses.
//!
//! The encode/decode logic mirrors `protosocket-prost` exactly: length-delimited prost,
//! byte-for-byte the same wire format.

use std::marker::PhantomData;

use protosocket::{Decoder, DeserializeError, Serialize};

/// Serializes prost messages into length-delimited buffers.
#[derive(Debug)]
pub struct ProstSerializer<Message> {
    _phantom: PhantomData<Message>,
}

impl<Message> Default for ProstSerializer<Message> {
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<Message> Serialize for ProstSerializer<Message>
where
    Message: prost::Message + std::fmt::Debug,
{
    type Message = Message;

    fn serialize_into_buffer(&mut self, message: Self::Message, buffer: &mut Vec<u8>) {
        match message.encode_length_delimited(buffer) {
            Ok(_) => {
                log::debug!("encoded {message:?}");
            }
            Err(e) => {
                log::error!("encoding error: {e:?}");
            }
        }
    }
}

/// Decodes length-delimited prost messages.
#[derive(Debug)]
pub struct ProstDecoder<Message> {
    _phantom: PhantomData<Message>,
}

impl<Message> Default for ProstDecoder<Message> {
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<Message> Decoder for ProstDecoder<Message>
where
    Message: prost::Message + Default + std::fmt::Debug,
{
    type Message = Message;

    fn decode(
        &mut self,
        mut buffer: impl bytes::Buf,
    ) -> std::result::Result<(usize, Self::Message), DeserializeError> {
        match prost::decode_length_delimiter(buffer.chunk()) {
            Ok(message_length) => {
                if buffer.remaining() < message_length + prost::length_delimiter_len(message_length)
                {
                    return Err(DeserializeError::IncompleteBuffer {
                        next_message_size: message_length,
                    });
                }
            }
            Err(e) => {
                log::trace!("can't read a length delimiter {e:?}");
                return Err(DeserializeError::IncompleteBuffer {
                    next_message_size: 10,
                });
            }
        };

        let start = buffer.remaining();
        match <Self::Message as prost::Message>::decode_length_delimited(&mut buffer) {
            Ok(message) => {
                let length = start - buffer.remaining();
                log::debug!("decoded {length}: {message:?}");
                Ok((length, message))
            }
            Err(e) => {
                log::warn!("could not decode message: {e:?}");
                Err(DeserializeError::InvalidBuffer)
            }
        }
    }
}
