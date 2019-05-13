use serde_derive::{Serialize, Deserialize};
use std::time::SystemTime;
use std::io::Cursor;
use binformat::BinarySD;
use std::error::Error;

use super::Message;

pub enum MessageDirection {
    Received,
    Sent,
}

// contains info about message
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MessageInfoJSON {
    // message serialized in hex format
    #[serde(rename="msg_raw")]
    pub msg_raw: String,

    // public key of peer (compressed, hex) of peer
    #[serde(rename="peer_pubkey")]
    pub peer_pub_key: String,

    // `sent` or `received`
    #[serde(rename="direction")]
    pub direction: String,

    // message type
    #[serde(rename="type")]
    pub type_: String,

    // Unix timestamp when this message was received or sent
    #[serde(rename="time")]
    pub time: String,
}

impl MessageInfoJSON {
    pub fn new(msg: &Message, direction: MessageDirection, peer_pubkey: String) -> Result<MessageInfoJSON, Box<Error>> {
        let msg_raw = message_to_hex(msg)
            .map_err(|err| format!("cannot encode message: {:?}", err))?;

        let direction_str = match direction {
            MessageDirection::Received => "received".to_owned(),
            MessageDirection::Sent => "sent".to_owned(),
        };

        let since_epoch = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|err| format!("cannot get unix time: {:?}", err))?;
        let time_str = format!("{}", since_epoch.as_secs());

        let type_str = msg.get_type();

        Ok(MessageInfoJSON{
            msg_raw,
            peer_pub_key: peer_pubkey,
            direction: direction_str,
            type_: type_str,
            time: time_str,
        })
    }
}

pub fn message_from_hex(msg_hex: &str) -> Result<Message, Box<Error>> {
    let msg_bytes = hex::decode(msg_hex)
        .map_err(|err| format!("cannot decode message from hex: {:?}", err))?;

    let mut cursor = Cursor::new(msg_bytes);
    let msg = BinarySD::deserialize::<Message, _>(&mut cursor)
        .map_err(|err| format!("cannot deserialize message: {:?}", err))?;
    Ok(msg)
}

pub fn message_to_hex(m: &Message) -> Result<String, Box<Error>> {
    let mut new_msg_bytes = vec![];
    BinarySD::serialize(&mut new_msg_bytes, m)
        .map_err(|err| format!("cannot serialize message: {:?}", err))?;

    Ok(hex::encode(&new_msg_bytes))
}