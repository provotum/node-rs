use ::chain::block::Block;
use ::chain::chain::Chain;
use ::chain::transaction::Transaction;
use ::protocol::clique::Tally;
use serde_json;
use std::str;

/// Messages used to communicate information between nodes.
#[derive(Eq, PartialEq, Deserialize, Serialize, Clone, Debug)]
pub enum Message {
    Ping,
    Pong,
    TransactionPayload(Transaction),
    TransactionAccept,
    BlockRequest(String),
    BlockPayload(Block),
    BlockAccept,
    BlockDuplicated,
    ChainRequest,
    ChainResponse(Chain),
    ChainAccept,
    OpenVote,
    OpenVoteAccept,
    CloseVote,
    CloseVoteAccept,
    RequestTally,
    RequestTallyPayload(Tally),
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
                return message;
            }
            Err(e) => {
                warn!("Failed to decode {:?} into a message: {:?}. Will return error.", json_string, e);
                return Message::None;
            }
        }
    }
}

