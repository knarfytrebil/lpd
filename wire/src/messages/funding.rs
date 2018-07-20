use super::channel::ChannelId;
use super::types::Signature;
use super::types::PublicKey;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct FundingTxid {
    data: [u8; 32],
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct OutputIndex {
    index: u16,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct FundingCreated {
    temporary_channel_id: ChannelId,
    funding_txid: FundingTxid,
    output_index: OutputIndex,
    signature: Signature,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct FundingSigned {
    channel_id: ChannelId,
    signature: Signature,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct FundingLocked {
    channel_id: ChannelId,
    next_per_commitment_point: PublicKey,
}