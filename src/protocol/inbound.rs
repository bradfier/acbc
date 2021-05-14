//! Type definitions for messages received from the simulator.
//!
//! Incoming packets can be parsed by calling [`InboundMessage::decode`] on a byte slice obtained from
//! your socket.
//!
//! # Example
//!
//! ```
//! use acbc::protocol::{InboundMessage, RegistrationResult};
//!
//! let packet = vec![0x01, 0x01, 0x00, 0x00, 0x00, 0x01, 0x01, 0x00, 0x00];
//! let parsed = InboundMessage::decode(&packet).unwrap();
//!
//! assert!(matches!(parsed, InboundMessage::RegistrationResult(_)));
//! ```
//!
//! ## Lifetimes
//!
//! When parsed from a byte slice using [`InboundMessage::decode`] these types try to limit the
//! amount of copying required. Primitive values will be copied, but larger fields like strings are
//! zero-copy by default, and will be bound to the lifetime of the original slice.
//! ```compile_fail
//! use acbc::protocol::InboundMessage;
//!
//! // Parsed message cannot outlive `packet`!
//! let extracted = {
//!     let packet: Vec<u8> = vec![0x01, 0x01, 0x00, 0x00, 0x00, 0x01, 0x01, 0x00, 0x00];
//!     InboundMessage::decode(&packet).unwrap()
//! };
//! ```
//!
//! If you need an owned copy of the data from a packet, you can use [`.into_owned()`](InboundMessage::into_owned)
//! to obtain a copy of the message with a `'static` lifetime.
//!
//! ```
//! use acbc::protocol::InboundMessage;
//!
//! let extracted = {
//!     let packet: Vec<u8> = vec![0x01, 0x01, 0x00, 0x00, 0x00, 0x01, 0x01, 0x00, 0x00];
//!     InboundMessage::decode(&packet).unwrap().into_owned()
//! };
//! ```
//!
//! ## IDs
//!
//! The broadcasting protocol uses `u8`, `u16` and `u32` almost interchangeably for unique identifiers and
//! indexes. The structs in this module replicate the packet structure faithfully, but there appears to be
//! no obvious reason for these different widths.

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

/// An incoming message, decoded from the UDP stream sent by the simulator.
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
    /// Decode an incoming message from a UDP payload sent by the simulator.
    pub fn decode(input: &'a [u8]) -> Result<InboundMessage<'a>, ErrorTree<ByteOffset>> {
        parser::parse(input)
    }

    /// Obtain a copy of the message with a `'static` lifetime.
    pub fn into_owned(self) -> InboundMessage<'static> {
        match self {
            InboundMessage::RegistrationResult(result) => {
                InboundMessage::RegistrationResult(result.into_owned())
            }
            InboundMessage::RealtimeUpdate(update) => {
                InboundMessage::RealtimeUpdate(update.into_owned())
            }
            InboundMessage::RealtimeCarUpdate(car_update) => {
                InboundMessage::RealtimeCarUpdate(car_update)
            }
            InboundMessage::EntrylistUpdate(update) => InboundMessage::EntrylistUpdate(update),
            InboundMessage::EntrylistCar(car) => InboundMessage::EntrylistCar(car.into_owned()),
            InboundMessage::TrackData(data) => InboundMessage::TrackData(data.into_owned()),
            InboundMessage::BroadcastingEvent(event) => {
                InboundMessage::BroadcastingEvent(event.into_owned())
            }
        }
    }
}

/// Describes a response to the initial broadcast client connection request.
#[derive(Clone, Debug, PartialEq)]
pub struct RegistrationResult<'a> {
    /// The client ID of this connection, used to notify the simulator upon disconnect.
    pub connection_id: u32,
    pub connection_success: bool,
    pub read_only: bool,
    pub error_message: Cow<'a, str>,
}

impl<'a> RegistrationResult<'a> {
    /// Obtain a copy of the response with a `'static` lifetime.
    pub fn into_owned(self) -> RegistrationResult<'static> {
        RegistrationResult {
            connection_id: self.connection_id,
            connection_success: self.connection_success,
            read_only: self.read_only,
            error_message: Cow::Owned(self.error_message.into_owned()),
        }
    }
}

/// Contains the timing data for a fully or partially completed lap.
#[derive(Debug, Copy, Clone)]
pub struct Lap {
    pub lap_time_ms: i32,
    pub car_id: u16,
    /// The index of the driver who set this lap.
    pub driver_index: u16,
    /// Sector split times, in milliseconds. This field is not populated for partially completed laps.
    pub splits: ArrayVec<[i32; 3]>,
    pub is_invalid: bool,
    pub is_valid_for_best: bool,
    pub is_out_lap: bool,
    pub is_in_lap: bool,
}

/// Contains replay playback information.
#[derive(Debug, Clone)]
pub struct ReplayInfo {
    pub session_time: f32,
    pub remaining_time: f32,
    pub focused_car_index: u32,
}

/// Contains a snapshot of the current simulator state, the complete state is sent
/// for each update.
///
/// This type of update is sent approximately once per update interval.
#[derive(Debug, Clone)]
pub struct RealtimeUpdate<'a> {
    /// The event index, starts at 0 when connecting, and increments with each new race weekend.
    pub event_index: u16,
    /// The session index, will start at 0 when connecting, even if a previous session has already
    /// taken place in this event.
    pub session_index: u16,
    pub session_type: SessionType,
    pub session_phase: SessionPhase,
    /// Session time in milliseconds since the green flag.
    pub session_time: f32,
    /// Time in milliseconds remaining before the end of the session.
    pub session_end_time: f32,
    /// Index into the entry list of the car currently focused by the simulator.
    pub focused_car_index: u32, // TODO: Implement .focused_car() on Context
    /// Active camera set, this string will be one of those returned in [`TrackData`]
    pub active_camera_set: Cow<'a, str>,
    /// Active camera, this string will be one of those returned in [`TrackData`]
    pub active_camera: Cow<'a, str>,
    /// Current HUD page shown, this string will be one of those returned in [`TrackData`]
    pub current_hud_page: Cow<'a, str>,
    /// `None` if the current session is not a replay.
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
    /// Obtain a copy of the update with a `'static` lifetime.
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

/// Contains a snapshot of the state of a single car within the session.
///
/// This type of update is sent approximately once per update interval.
#[derive(Debug, Clone)]
pub struct RealtimeCarUpdate {
    /// Unique Car ID
    pub id: u16,
    /// Driver index within this Car's entry
    pub driver_index: u16,
    /// Total number of drivers in this Car's entry
    pub driver_count: u8,
    /// Selected gear, 0 == N, -1 == R
    pub gear: i8,
    /// World position X, frequently sent as 0.0
    pub world_pos_x: f32,
    /// World position Y, frequently sent as 0.0
    pub world_pos_y: f32,
    pub yaw: f32,
    /// Location, indicates if a car is Entering, Exiting or In the Pit Lane.
    pub car_location: CarLocation,
    pub speed_kph: u16,
    /// Overall position in the session.
    pub position: u16,
    /// Position within class.
    pub cup_position: u16,
    /// Position 'on the road', for drawing relative position displays.
    pub track_position: u16,
    /// Fractional representation of how 'far around' the lap the car is.
    pub spline_position: f32,
    /// The number of completed laps.
    pub laps: u16,
    /// The improvement or otherwise of _this_ lap.
    pub delta: i32,
    /// The best lap of the current driver. May be full of zeros.
    pub best_session_lap: Lap,
    /// The previous lap of the current driver. May be full of zeros.
    pub last_lap: Lap,
    /// Lap data for the current lap, note that sector times are not populated for this field.
    pub current_lap: Lap,
}

/// A message sent by the simulator to indicate that the session entry list has changed.
///
/// This packet is sent ahead of a stream of [`EntrylistCar`] packets to give the client an opportunity
/// to pre-allocate space for the updated information. This type of packet is sent upon initial connection,
/// when a change to the entry list occurs, or when the client explicitly requests an update.
#[derive(Debug, Clone)]
pub struct EntrylistUpdate {
    /// The list of Car IDs in the session.
    pub car_ids: Vec<u16>,
}

/// Basic driver information.
#[derive(Debug, Clone)]
pub struct Driver<'a> {
    pub first_name: Cow<'a, str>,
    pub last_name: Cow<'a, str>,
    pub short_name: Cow<'a, str>,
    pub category: DriverCategory,
    pub nationality: Nationality,
}

impl<'a> Driver<'a> {
    /// Obtain a copy of the driver details with a `'static` lifetime.
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

/// Updated entry information for a single Car.
///
/// This packet will typically have been preceded by an [`EntrylistUpdate`] containing its ID.
/// `nationality` and `cup_category` appear to reflect those of the current driver.
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
    /// Obtain a copy of the Car data with a `'static` lifetime.
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

/// Camera Sets available for selection.
pub type CameraSet<'a> = Vec<Cow<'a, str>>;
/// HUD pages available for selection.
pub type HudPages<'a> = Vec<Cow<'a, str>>;

/// Information about the current track.
///
/// There is no definitive list of Track IDs, determining such a mapping is left as an exercise for the
/// reader. `name` is typically human readable rather than the `spa_2020` format used in the config
/// files.
#[derive(Debug, Clone)]
pub struct TrackData<'a> {
    pub name: Cow<'a, str>,
    pub id: u32,
    /// Distance given in meters.
    pub distance: u32,
    pub camera_sets: HashMap<Cow<'a, str>, CameraSet<'a>>,
    pub hud_pages: HudPages<'a>,
}

impl<'a> TrackData<'a> {
    /// Obtain a copy of the track data with a `'static` lifetime.
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

/// A message indicating a relevant event has occurred in the session.
#[derive(Debug, Clone)]
pub struct BroadcastingEvent<'a> {
    pub event_type: BroadcastingEventType,
    pub message: Cow<'a, str>,
    pub time_ms: i32,
    /// ID of the car, for global events, `car_id` will be zero.
    pub car_id: u16,
}

impl<'a> BroadcastingEvent<'a> {
    pub fn into_owned(self) -> BroadcastingEvent<'static> {
        BroadcastingEvent {
            event_type: self.event_type,
            message: Cow::Owned(self.message.into_owned()),
            time_ms: self.time_ms,
            car_id: self.car_id,
        }
    }
}
