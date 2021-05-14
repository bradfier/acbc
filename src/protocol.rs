use thiserror::Error;

pub mod acc_enum;
pub mod inbound;
pub mod outbound;
mod parser;

const PROTOCOL_VERSION: u8 = 4;

pub use inbound::*;
pub use outbound::*;

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("Unrecognised session type `{0}`")]
    UnknownSessionType(u8),
    #[error("Unrecognised session phase `{0}`")]
    UnknownSessionPhase(u8),
    #[error("Unrecognised car location `{0}`")]
    UnknownCarLocation(u8),
    #[error("Unrecognised driver category `{0}`")]
    UnknownDriverCategory(u8),
    #[error("Unrecognised nationality `{0}`")]
    UnknownNationality(u16),
    #[error("Unrecognised car model `{0}`")]
    UnknownCarModel(u8),
    #[error("Unrecognised cup category `{0}`")]
    UnknownCupCategory(u8),
    #[error("Unrecognised broadcasting event type `{0}`")]
    UnknownBroadcastingEvent(u8),
}
