use nom::error::VerboseError;
use nom::IResult;
use std::convert::TryFrom;
use thiserror::Error;
use tinyvec::ArrayVec;

mod parser;

pub type Res<T, U> = IResult<T, U, VerboseError<T>>;

pub enum IncomingMessage<'a> {
    RegistrationResult(RegistrationResult<'a>),
    RealtimeUpdate(RealtimeUpdate<'a>),
    RealtimeCarUpdate(RealtimeCarUpdate),
}

impl<'a> IncomingMessage<'a> {
    pub fn parse(input: &'a [u8]) -> Res<&[u8], IncomingMessage> {
        parser::parse(input)
    }
}

/// Describes a response to the initial broadcast client connection request.
#[derive(Clone, Debug, PartialEq)]
pub struct RegistrationResult<'a> {
    pub connection_id: u32,
    pub connection_success: bool,
    pub read_only: bool,
    pub error_message: &'a str,
}

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("Unrecognised session type `{0}`")]
    UnknownSessionType(u8),
    #[error("Unrecognised session phase `{0}`")]
    UnknownSessionPhase(u8),
    #[error("Unrecognised car location `{0}`")]
    UnknownCarLocation(u8),
}

#[derive(Debug, PartialEq)]
pub enum SessionType {
    Practice,
    Qualifying,
    Superpole,
    Race,
    Hotlap,
    Hotstint,
    HotlapSuperpole,
    Replay,
}

impl TryFrom<u8> for SessionType {
    type Error = DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(SessionType::Practice),
            4 => Ok(SessionType::Qualifying),
            9 => Ok(SessionType::Superpole),
            10 => Ok(SessionType::Race),
            11 => Ok(SessionType::Hotlap),
            12 => Ok(SessionType::Hotstint),
            13 => Ok(SessionType::HotlapSuperpole),
            14 => Ok(SessionType::Replay),
            x => Err(DecodeError::UnknownSessionType(x)),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum SessionPhase {
    None,
    Starting,
    PreFormation,
    FormationLap,
    PreSession,
    Session,
    SessionOver,
    PostSession,
    ResultUi,
}

impl TryFrom<u8> for SessionPhase {
    type Error = DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(SessionPhase::None),
            1 => Ok(SessionPhase::Starting),
            2 => Ok(SessionPhase::PreFormation),
            3 => Ok(SessionPhase::FormationLap),
            4 => Ok(SessionPhase::PreSession),
            5 => Ok(SessionPhase::Session),
            6 => Ok(SessionPhase::SessionOver),
            7 => Ok(SessionPhase::PostSession),
            8 => Ok(SessionPhase::ResultUi),
            x => Err(DecodeError::UnknownSessionPhase(x)),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum CarLocation {
    None,
    Track,
    Pitlane,
    PitEntry,
    PitExit,
}

impl TryFrom<u8> for CarLocation {
    type Error = DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(CarLocation::None),
            1 => Ok(CarLocation::Track),
            2 => Ok(CarLocation::Pitlane),
            3 => Ok(CarLocation::PitEntry),
            4 => Ok(CarLocation::PitExit),
            x => Err(DecodeError::UnknownCarLocation(x)),
        }
    }
}

#[derive(Debug)]
pub struct Lap {
    pub lap_time_ms: i32,
    pub car_id: u16,
    pub driver_id: u16,
    /// Sector split times, in milliseconds. This field is not populated for partially completed laps.
    pub splits: ArrayVec<[i32; 3]>,
    pub is_invalid: bool,
    pub is_valid_for_best: bool,
    pub is_out_lap: bool,
    pub is_in_lap: bool,
}

#[derive(Debug)]
pub struct ReplayInfo {
    pub session_time: f32,
    pub remaining_time: f32,
    pub focused_car_index: u32,
}

#[derive(Debug)]
pub struct RealtimeUpdate<'a> {
    /// The event index, starts at 0 when connecting, and increments with each new race weekend.
    pub event_index: u16,
    /// The session index, will start at 0 when connecting, even if a previous session has already
    /// taken place in this event.
    pub session_index: u16,
    pub session_type: SessionType,
    pub session_phase: SessionPhase,
    /// Session time in milliseconds since the green flag
    pub session_time: f32,
    /// Time in milliseconds remaining before the end of the session
    pub session_end_time: f32,
    pub focused_car_index: u32,
    pub active_camera_set: &'a str,
    pub active_camera: &'a str,
    pub current_hud_page: &'a str,
    pub replay_info: Option<ReplayInfo>,
    pub time_of_day: f32,
    pub ambient_temp: i8,
    pub track_temp: i8,
    pub clouds: u8,
    pub rain_level: u8,
    pub wetness: u8,
    pub best_session_lap: Lap,
}

#[derive(Debug)]
pub struct RealtimeCarUpdate {
    pub id: u16,
    pub driver_id: u16,
    pub driver_count: u8,
    pub gear: i8,
    pub world_pos_x: f32,
    pub world_pos_y: f32,
    pub yaw: f32,
    pub car_location: CarLocation,
    pub speed_kph: u16,
    pub position: u16,
    pub cup_position: u16,
    pub track_position: u16,
    pub spline_position: f32,
    pub laps: u16,
    pub delta: i32,
    pub best_session_lap: Lap,
    pub last_lap: Lap,
    /// Lap data for the current lap, note that sector times are not populated for this field.
    pub current_lap: Lap,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
