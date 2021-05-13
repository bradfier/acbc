use nom_supreme::error::ErrorTree;
use nom_supreme::final_parser::ByteOffset;
use std::borrow::Cow;
use std::collections::HashMap;
use tinyvec::ArrayVec;

use crate::protocol::acc_enum::{
    BroadcastingEventType, CarLocation, CarModel, CupCategory, DriverCategory, Nationality,
    SessionPhase, SessionType,
};
use crate::protocol::parser;

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
    pub error_message: Cow<'a, str>,
}

#[derive(Debug, Copy, Clone)]
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

#[derive(Debug, Clone)]
pub struct ReplayInfo {
    pub session_time: f32,
    pub remaining_time: f32,
    pub focused_car_index: u32,
}

#[derive(Debug, Clone)]
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
    pub active_camera_set: Cow<'a, str>,
    pub active_camera: Cow<'a, str>,
    pub current_hud_page: Cow<'a, str>,
    pub replay_info: Option<ReplayInfo>,
    pub time_of_day: f32,
    pub ambient_temp: i8,
    pub track_temp: i8,
    pub clouds: u8,
    pub rain_level: u8,
    pub wetness: u8,
    pub best_session_lap: Lap,
}

impl<'a> RealtimeUpdate<'a> {
    pub fn into_owned(self) -> RealtimeUpdate<'static> {
        RealtimeUpdate {
            event_index: self.event_index,
            session_index: self.session_index,
            session_type: self.session_type,
            session_phase: self.session_phase,
            session_time: self.session_time,
            session_end_time: self.session_end_time,
            focused_car_index: self.focused_car_index,
            active_camera_set: Cow::Owned(self.active_camera_set.into_owned()),
            active_camera: Cow::Owned(self.active_camera.into_owned()),
            current_hud_page: Cow::Owned(self.current_hud_page.into_owned()),
            replay_info: self.replay_info,
            time_of_day: self.time_of_day,
            ambient_temp: self.ambient_temp,
            track_temp: self.track_temp,
            clouds: self.clouds,
            rain_level: self.rain_level,
            wetness: self.wetness,
            best_session_lap: self.best_session_lap,
        }
    }
}

#[derive(Debug, Clone)]
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
    pub first_name: Cow<'a, str>,
    pub last_name: Cow<'a, str>,
    pub short_name: Cow<'a, str>,
    pub category: DriverCategory,
    pub nationality: Nationality,
}

impl<'a> Driver<'a> {
    pub fn into_owned(self) -> Driver<'static> {
        Driver {
            first_name: Cow::Owned(self.first_name.into_owned()),
            last_name: Cow::Owned(self.last_name.into_owned()),
            short_name: Cow::Owned(self.short_name.into_owned()),
            category: self.category,
            nationality: self.nationality,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EntrylistCar<'a> {
    pub id: u16,
    pub model: CarModel,
    pub team_name: Cow<'a, str>,
    pub race_number: i32,
    pub cup_category: CupCategory,
    pub current_driver_index: u8,
    pub nationality: Nationality,
    pub drivers: Vec<Driver<'a>>,
}

impl<'a> EntrylistCar<'a> {
    pub fn into_owned(self) -> EntrylistCar<'static> {
        EntrylistCar {
            id: self.id,
            model: self.model,
            team_name: Cow::Owned(self.team_name.into_owned()),
            race_number: self.race_number,
            cup_category: self.cup_category,
            current_driver_index: self.current_driver_index,
            nationality: self.nationality,
            drivers: self.drivers.into_iter().map(|d| d.into_owned()).collect(),
        }
    }
}

pub type CameraSet<'a> = Vec<Cow<'a, str>>;
pub type HudPages<'a> = Vec<Cow<'a, str>>;

#[derive(Debug, Clone)]
pub struct TrackData<'a> {
    pub name: Cow<'a, str>,
    pub id: u32,
    pub distance: u32,
    pub camera_sets: HashMap<Cow<'a, str>, CameraSet<'a>>,
    pub hud_pages: HudPages<'a>,
}

impl<'a> TrackData<'a> {
    pub fn into_owned(self) -> TrackData<'static> {
        TrackData {
            name: Cow::Owned(self.name.into_owned()),
            id: self.id,
            distance: self.distance,
            camera_sets: self
                .camera_sets
                .into_iter()
                .map(|(k, v)| {
                    (
                        Cow::Owned(k.into_owned()),
                        v.into_iter().map(|s| Cow::Owned(s.into_owned())).collect(),
                    )
                })
                .collect(),
            hud_pages: self
                .hud_pages
                .into_iter()
                .map(|h| Cow::Owned(h.into_owned()))
                .collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BroadcastingEvent<'a> {
    pub event_type: BroadcastingEventType,
    pub message: Cow<'a, str>,
    pub time_ms: i32,
    pub car_id: u16,
}
