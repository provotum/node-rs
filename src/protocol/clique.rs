use ::p2p::codec::Message;

pub trait ProtocolHandler {
    fn handle(message: Message) -> Message;
}

pub struct CliqueProtocol;

impl ProtocolHandler for CliqueProtocol {

    fn handle(message: Message) -> Message {
        // TODO: actually handle message

        return Message::Pong;
    }
}