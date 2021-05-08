use tinyvec::ArrayVec;

use crate::protocol::acc_enum::{
    BroadcastingEventType, CarLocation, CarModel, CupCategory, DriverCategory, Nationality,
    SessionPhase, SessionType,
};
use crate::protocol::parser;
use nom_supreme::error::ErrorTree;
use nom_supreme::final_parser::ByteOffset;
use std::collections::HashMap;

#[derive(Debug)]
pub enum InboundMessage<'a> {
    RegistrationResult(RegistrationResult<'a>),
    RealtimeUpdate(RealtimeUpdate<'a>),
    RealtimeCarUpdate(RealtimeCarUpdate),
    EntrylistUpdate(EntrylistUpdate),
    EntrylistCar(EntrylistCar<'a>),
    TrackData(TrackData<'a>),
    BroadcastingEvent(BroadcastingEvent<'a>),
}

impl<'a> InboundMessage<'a> {
    pub fn decode(input: &'a [u8]) -> Result<InboundMessage<'a>, ErrorTree<ByteOffset>> {
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

#[derive(Debug, Clone)]
pub struct EntrylistUpdate {
    pub car_ids: Vec<u16>,
}

#[derive(Debug, Clone)]
pub struct Driver<'a> {
    pub first_name: &'a str,
    pub last_name: &'a str,
    pub short_name: &'a str,
    pub category: DriverCategory,
    pub nationality: Nationality,
}

#[derive(Debug, Clone)]
pub struct EntrylistCar<'a> {
    pub id: u16,
    pub model: CarModel,
    pub team_name: &'a str,
    pub race_number: i32,
    pub cup_category: CupCategory,
    pub current_driver_index: u8,
    pub nationality: Nationality,
    pub drivers: Vec<Driver<'a>>,
}

pub type CameraSet<'a> = Vec<&'a str>;
pub type HudPages<'a> = Vec<&'a str>;

#[derive(Debug, Clone)]
pub struct TrackData<'a> {
    pub name: &'a str,
    pub id: u32,
    pub distance: u32,
    pub camera_sets: HashMap<&'a str, CameraSet<'a>>,
    pub hud_pages: HudPages<'a>,
}

#[derive(Debug, Clone)]
pub struct BroadcastingEvent<'a> {
    pub event_type: BroadcastingEventType,
    pub message: &'a str,
    pub time_ms: i32,
    pub car_id: u16,
}
