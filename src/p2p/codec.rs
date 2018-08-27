use serde_json;
use std::str;

// macro Deserialize, Serialize from serde_derive
#[derive(Deserialize, Serialize, Debug)]
pub enum Message {
    Ping,
    Pong,
    Error,
    Payload,
    None
}

pub trait Codec {
    fn encode(message: Message) -> String;
    fn decode(message: String) -> Message;
}

pub struct JsonCodec;

impl Codec for JsonCodec {

    fn encode(message: Message) -> String {
        let result = serde_json::to_string(&message);

        match result {
            Ok(json_message) => {
                trace!("Encoded message {:?} into json {:?}", message, json_message);
                return json_message;
            },
            Err(e) => {
                warn!("Failed to encode {:?} to json: {:?}. Will return an empty message", message, e);
                return String::new();
            }
        }
    }

    fn decode(json_string: String) -> Message {
        let result = serde_json::from_str(&json_string.as_str());

        match result {
            Ok(message) => {
                trace!("Decoded json message {:?} into {:?}", json_string, message);
                return message;
            },
            Err(e) => {
                warn!("Failed to decode {:?} into a message: {:?}. Will return error.", json_string, e);
                return Message::Error;
            }
        }
    }
}

