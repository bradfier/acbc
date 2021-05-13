use nom::branch::alt;
use nom::bytes::complete::{tag, take_while};
use nom::combinator::{map, map_res, not};
use nom::error::context;
use nom::multi::{fold_many0, length_count, length_value};
use nom::number::complete::{le_f32, le_i32, le_i8, le_u16, le_u32, le_u8};
use nom::sequence::tuple;
use nom::IResult;
use nom_supreme::error::ErrorTree;
use nom_supreme::final_parser::{final_parser, ByteOffset};
use std::borrow::Cow;
use std::convert::TryFrom;
use tinyvec::ArrayVec;

use crate::protocol::acc_enum::{
    BroadcastingEventType, CarLocation, CarModel, CupCategory, DriverCategory, Nationality,
    SessionPhase, SessionType,
};
use crate::protocol::inbound::{
    BroadcastingEvent, CameraSet, Driver, EntrylistCar, EntrylistUpdate, InboundMessage, Lap,
    RealtimeCarUpdate, RealtimeUpdate, RegistrationResult, ReplayInfo, TrackData,
};

type Res<T, U> = IResult<T, U, ErrorTree<T>>;

pub(crate) fn parse(input: &[u8]) -> Result<InboundMessage, ErrorTree<ByteOffset>> {
    final_parser(context(
        "incoming_message",
        alt((
            map(registration_result, InboundMessage::RegistrationResult),
            map(realtime_update, InboundMessage::RealtimeUpdate),
            map(realtime_car_update, InboundMessage::RealtimeCarUpdate),
            map(entrylist_update, InboundMessage::EntrylistUpdate),
            map(entrylist_car, InboundMessage::EntrylistCar),
            map(track_data, InboundMessage::TrackData),
            map(broadcasting_event, InboundMessage::BroadcastingEvent),
        )),
    ))(input)
}

fn registration_result(input: &[u8]) -> Res<&[u8], RegistrationResult> {
    context(
        "registration_result",
        tuple((tag(&[0x01]), le_u32, boolean, boolean, kstring)),
    )(input)
    .map(|(next_input, res)| {
        (
            next_input,
            RegistrationResult {
                connection_id: res.1,
                connection_success: res.2,
                read_only: res.3,
                error_message: Cow::Borrowed(res.4),
            },
        )
    })
}

// Parse a 'Kunos' string, which is an int16 length marker followed by N bytes of UTF-8 string data
fn kstring(input: &[u8]) -> Res<&[u8], &str> {
    context(
        "string",
        map_res(
            length_value(le_u16, take_while(|_| true)),
            std::str::from_utf8,
        ),
    )(input)
}

fn boolean(input: &[u8]) -> Res<&[u8], bool> {
    context("boolean", map(le_u8, |i: u8| i != 0))(input)
}

// This parses just the 'header' which providers the list of existing Car IDs, the entry
// list information is contained in EntrylistCar packets.
fn entrylist_update(input: &[u8]) -> Res<&[u8], EntrylistUpdate> {
    context(
        "entrylist_update",
        tuple((
            tag(&[0x04]),                 // Packet type
            le_u32,                       // Connection ID
            length_count(le_u16, le_u16), // List of car IDs
        )),
    )(input)
    .map(|(next_input, res)| (next_input, EntrylistUpdate { car_ids: res.2 }))
}

// Replay info is not included in the datagram if it's not currently a replay context
fn replay_info(input: &[u8]) -> Res<&[u8], Option<ReplayInfo>> {
    context(
        "replay_info",
        alt((
            map(tag(&[0x00]), |_| None),
            map(
                tuple((not(tag(&[0x00])), le_f32, le_f32, le_u32)),
                |(_, session_time, remaining_time, focused_car_index)| {
                    Some(ReplayInfo {
                        session_time,
                        remaining_time,
                        focused_car_index,
                    })
                },
            ),
        )),
    )(input)
}

// Parse the driver information supplied in the middle of EntrylistCar packets
fn driver(input: &[u8]) -> Res<&[u8], Driver> {
    context(
        "driver",
        tuple((
            kstring,
            kstring,
            kstring,
            map_res(le_u8, DriverCategory::try_from),
            map_res(le_u16, Nationality::try_from),
        )),
    )(input)
    .map(
        |(next_input, (first_name, last_name, short_name, category, nationality))| {
            (
                next_input,
                Driver {
                    first_name: Cow::Borrowed(first_name),
                    last_name: Cow::Borrowed(last_name),
                    short_name: Cow::Borrowed(short_name),
                    category,
                    nationality,
                },
            )
        },
    )
}

fn entrylist_car(input: &[u8]) -> Res<&[u8], EntrylistCar> {
    context(
        "entrylist_car",
        tuple((
            tag(&[0x06]),
            le_u16,
            map_res(le_u8, CarModel::try_from),
            kstring,
            le_i32,
            map_res(le_u8, CupCategory::try_from),
            le_u8,
            map_res(le_u16, Nationality::try_from),
            length_count(le_u8, driver),
        )),
    )(input)
    .map(
        |(
            next_input,
            (
                _,
                id,
                model,
                team_name,
                race_number,
                cup_category,
                current_driver_index,
                nationality,
                drivers,
            ),
        )| {
            (
                next_input,
                EntrylistCar {
                    id,
                    model,
                    team_name: Cow::Borrowed(team_name),
                    race_number,
                    cup_category,
                    current_driver_index,
                    nationality,
                    drivers,
                },
            )
        },
    )
}

// Split sector times are stored as <u8 number of sectors><i32 ms><i32 ms><i32 ms> etc
fn splits(input: &[u8]) -> Res<&[u8], ArrayVec<[i32; 3]>> {
    context(
        "splits",
        length_value(
            map(le_u8, |l: u8| l * 4), // le_i32s are 4 bytes wide
            fold_many0(
                le_i32,
                ArrayVec::new(),
                |mut acc: ArrayVec<[i32; 3]>, item| {
                    acc.push(item);
                    acc
                },
            ),
        ),
    )(input)
}

fn lap(input: &[u8]) -> Res<&[u8], Lap> {
    context(
        "lap",
        tuple((
            le_i32, le_u16, le_u16, splits, boolean, boolean, boolean, boolean,
        )),
    )(input)
    .map(|(next_input, res)| {
        (
            next_input,
            Lap {
                lap_time_ms: res.0,
                car_id: res.1,
                driver_id: res.2,
                splits: res.3,
                is_invalid: res.4,
                is_valid_for_best: res.5,
                is_out_lap: res.6,
                is_in_lap: res.7,
            },
        )
    })
}

fn realtime_update(input: &[u8]) -> Res<&[u8], RealtimeUpdate> {
    context(
        "realtime_update",
        tuple((
            tag(&[0x02]),
            le_u16,
            le_u16,
            map_res(le_u8, SessionType::try_from),
            map_res(le_u8, SessionPhase::try_from),
            le_f32,
            le_f32,
            le_u32,
            kstring,
            kstring,
            kstring,
            replay_info,
            le_f32,
            le_i8,
            le_i8,
            le_u8,
            le_u8,
            le_u8,
            lap,
        )),
    )(input)
    .map(
        |(
            next_input,
            (
                _,
                event_index,
                session_index,
                session_type,
                session_phase,
                session_time,
                session_end_time,
                focused_car_index,
                active_camera_set,
                active_camera,
                current_hud_page,
                replay_info,
                time_of_day,
                ambient_temp,
                track_temp,
                clouds,
                rain_level,
                wetness,
                best_session_lap,
            ),
        )| {
            (
                next_input,
                RealtimeUpdate {
                    event_index,
                    session_index,
                    session_type,
                    session_phase,
                    session_time,
                    session_end_time,
                    focused_car_index,
                    active_camera_set: Cow::Borrowed(active_camera_set),
                    active_camera: Cow::Borrowed(active_camera),
                    current_hud_page: Cow::Borrowed(current_hud_page),
                    replay_info,
                    time_of_day,
                    ambient_temp,
                    track_temp,
                    clouds,
                    rain_level,
                    wetness,
                    best_session_lap,
                },
            )
        },
    )
}

fn realtime_car_update(input: &[u8]) -> Res<&[u8], RealtimeCarUpdate> {
    context(
        "realtime_car_update",
        tuple((
            tag(&[0x03]),
            le_u16,
            le_u16,
            le_u8,
            le_i8,
            le_f32,
            le_f32,
            le_f32,
            map_res(le_u8, CarLocation::try_from),
            le_u16,
            le_u16,
            le_u16,
            le_u16,
            le_f32,
            le_u16,
            le_i32,
            lap,
            lap,
            lap,
        )),
    )(input)
    .map(
        |(
            next_input,
            (
                _,
                id,
                driver_id,
                driver_count,
                gear,
                world_pos_x,
                world_pos_y,
                yaw,
                car_location,
                speed_kph,
                position,
                cup_position,
                track_position,
                spline_position,
                laps,
                delta,
                best_session_lap,
                last_lap,
                current_lap,
            ),
        )| {
            (
                next_input,
                RealtimeCarUpdate {
                    id,
                    driver_id,
                    driver_count,
                    gear,
                    world_pos_x,
                    world_pos_y,
                    yaw,
                    car_location,
                    speed_kph,
                    position,
                    cup_position,
                    track_position,
                    spline_position,
                    laps,
                    delta,
                    best_session_lap,
                    last_lap,
                    current_lap,
                },
            )
        },
    )
}

fn camera_set<'a>(input: &'a [u8]) -> Res<&'a [u8], (Cow<'a, str>, CameraSet)> {
    context("camera_set", tuple((kstring, length_count(le_u8, kstring))))(input).map(
        |(next_input, (set_name, cameras))| {
            (
                next_input,
                (
                    Cow::Borrowed(set_name),
                    cameras.into_iter().map(|s| Cow::Borrowed(s)).collect(),
                ),
            )
        },
    )
}

fn track_data(input: &[u8]) -> Res<&[u8], TrackData> {
    context(
        "track_data",
        tuple((
            tag([0x05]),
            le_i32,
            kstring,
            le_u32,
            le_u32,
            length_count(le_u8, camera_set),
            map(length_count(le_u8, kstring), |h| {
                h.into_iter().map(|s| Cow::Borrowed(s)).collect()
            }),
        )),
    )(input)
    .map(
        |(next_input, (_, _, name, id, distance, camera_sets, hud_pages))| {
            (
                next_input,
                TrackData {
                    name: Cow::Borrowed(name),
                    id,
                    distance,
                    camera_sets: camera_sets.into_iter().collect(),
                    hud_pages,
                },
            )
        },
    )
}

fn broadcasting_event(input: &[u8]) -> Res<&[u8], BroadcastingEvent> {
    context(
        "broadcasting_event",
        tuple((
            tag([0x07]),
            map_res(le_u8, BroadcastingEventType::try_from),
            kstring,
            le_i32,
            map(le_u32, |i| {
                // For some reason, the car ID which is u16 everywhere else, is sent in this packet type
                // as a 4-byte wide integer. Here we just drop the 2 most significant bytes
                i as u16
            }),
        )),
    )(input)
    .map(|(next_input, (_, event_type, message, time_ms, car_id))| {
        (
            next_input,
            BroadcastingEvent {
                event_type,
                message: Cow::Borrowed(message),
                time_ms,
                car_id,
            },
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_kstring() {
        let input = b"\x03\x00abcefg";
        assert_eq!(kstring(input).unwrap(), (&b"efg"[..], "abc"));
    }

    #[test]
    fn parse_empty_kstring() {
        let input = b"\x00\x00abc";
        assert_eq!(kstring(input).unwrap(), (&b"abc"[..], ""));
    }

    #[test]
    fn parse_registration_result() {
        let input = b"\x01\x01\x00\x00\x00\x01\x01\x00\x00";
        let res = registration_result(input).unwrap();

        assert_eq!(
            res,
            (
                &[][..],
                RegistrationResult {
                    connection_id: 1,
                    connection_success: true,
                    read_only: true,
                    error_message: Cow::Borrowed(""),
                }
            )
        );
    }

    #[test]
    fn parse_registration_fail() {
        let input = b"\x01\x01\x00\x00\x00\x00\x01\x10\x00Handshake failed";
        let res = registration_result(input).unwrap();

        assert_eq!(
            res,
            (
                &[][..],
                RegistrationResult {
                    connection_id: 1,
                    connection_success: false,
                    read_only: true,
                    error_message: Cow::Borrowed("Handshake failed"),
                }
            )
        );
    }

    #[test]
    fn parse_lap_data() {
        let input = b"\x1b\x62\x01\x00\xe9\x03\x00\x00\x03\x5e\x77\x00\x00\xfb\x73\x00\x00\xc2\x76\x00\x00\x00\x01\x00\x00";
        let res = lap(input).unwrap().1;

        assert_eq!(res.car_id, 1001);
        assert_eq!(res.splits.len(), 3);
    }

    #[test]
    fn parse_realtime_update() {
        let input = include_bytes!("../../docs/pcap/realtime_update.bin");
        let res = realtime_update(input).unwrap().1;

        assert_eq!(res.active_camera, "CameraPit3");
        assert_eq!(res.ambient_temp, 25);
        assert_eq!(res.session_type, SessionType::Qualifying);
    }

    #[test]
    fn parse_realtime_car_update() {
        let input = include_bytes!("../../docs/pcap/realtime_car_update.bin");
        let res = realtime_car_update(input).unwrap();

        assert_eq!(res.0.len(), 0);
        assert_eq!(res.1.current_lap.splits.len(), 0);
    }

    #[test]
    fn parse_entrylist_update() {
        let input = &[0x04, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0xe9, 0x03];
        let res = entrylist_update(input).unwrap().1;

        assert_eq!(res.car_ids.len(), 1);
    }

    #[test]
    fn parse_entrylist_car() {
        let input: &[u8] = &[
            0x06, 0xe9, 0x03, 0x18, 0x00, 0x00, 0x4b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x01, 0x06, 0x00, 0x4d, 0x61, 0x72, 0x74, 0x69, 0x6e, 0x08, 0x00, 0x52, 0x6f, 0x77,
            0x6e, 0x74, 0x72, 0x65, 0x65, 0x03, 0x00, 0x52, 0x4f, 0x57, 0x03, 0x05, 0x00,
        ];
        let res = entrylist_car(input).unwrap().1;

        assert_eq!(res.id, 1001);
        assert_eq!(res.race_number, 75);
        assert_eq!(res.drivers.len(), 1);
        assert_eq!(res.drivers[0].first_name, "Martin");
        assert_eq!(res.drivers[0].last_name, "Rowntree");
        assert_eq!(res.drivers[0].short_name, "ROW");
        assert_eq!(res.drivers[0].nationality, Nationality::GreatBritain);
        assert_eq!(res.model, CarModel::Ferrari488Evo);
    }

    #[test]
    fn parse_track_data() {
        let input = include_bytes!("../../docs/pcap/track_data.bin");
        let res = track_data(input).unwrap().1;

        assert_eq!(res.distance, 4011);
        assert_eq!(res.name, "Circuit Zolder");
        assert_eq!(res.id, 10);
        assert_eq!(res.camera_sets.len(), 7);
        assert_eq!(res.hud_pages.len(), 7);
    }

    #[test]
    fn parse_broadcasting_event() {
        // Handwritten as we don't yet have a pcap of a Broadcast Event
        let input = b"\x07\x05\x0d\x00Lap completed\x2c\x4a\x00\x00\xe9\x03\x00\x00";
        let res = broadcasting_event(input).unwrap().1;

        assert_eq!(res.car_id, 1001);
        assert_eq!(res.message, "Lap completed");
        assert_eq!(res.event_type, BroadcastingEventType::LapCompleted);
    }

    #[test]
    fn parse_bogus_data() {
        // random64 starts with a 0x03 so it looks like a RealtimeCarUpdate and
        // should proceed down that parse tree and unwind without a panic.
        let input = include_bytes!("../../docs/pcap/random64.bin");
        let res = InboundMessage::decode(input);
        assert!(res.is_err());
    }
}
