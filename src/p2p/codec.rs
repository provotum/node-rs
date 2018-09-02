use ::chain::block::Block;
use ::chain::transaction::Transaction;
use serde_json;
use std::str;

// macro Deserialize, Serialize from serde_derive
#[derive(Deserialize, Serialize, Debug)]
pub enum Message {
    Ping,
    Pong,
    TransactionPayload(Transaction),
    TransactionAccept,
    BlockRequest(String),
    BlockPayload(Block),
    BlockAccept,
    None,
}


/// A codec is able to encode as well decode a particular `Message`
/// into a corresponding `String` representation.
pub trait Codec {
    /// Encode the given message into a string.
    fn encode(message: Message) -> String;
    /// Decode the given string into a message.
    fn decode(message: String) -> Message;
}

/// JsonCodec is able to encode and decode a particular
/// `Message` as a json `String` and vice-versa, respectively.
pub struct JsonCodec;

impl Codec for JsonCodec {
    /// Encode the given message into a JSON string.
    /// If the message cannot be encoded, an empty string will be returned.
    fn encode(message: Message) -> String {
        let result = serde_json::to_string(&message);

        match result {
            Ok(json_message) => {
                trace!("Encoded message {:?} into json {:?}", message, json_message);
                return json_message;
            }
            Err(e) => {
                warn!("Failed to encode {:?} to json: {:?}. Will return an empty message", message, e);
                return String::new();
            }
        }
    }

    /// Decode the given JSON string into a corresponding Message.
    /// Will return a `Message::None` if the string cannot be decoded.
    fn decode(json_string: String) -> Message {
        let result = serde_json::from_str(&json_string.as_str());

        match result {
            Ok(message) => {
                trace!("Decoded json message {:?} into {:?}", json_string, message);
                return message;
            }
            Err(e) => {
                warn!("Failed to decode {:?} into a message: {:?}. Will return error.", json_string, e);
                return Message::None;
            }
        }
    }
}

