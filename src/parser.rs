use nom::bytes::complete::{tag, take_while};
use nom::combinator::{map, map_res};
use nom::error::{context, VerboseError};
use nom::multi::{fold_many0, length_value};
use nom::number::complete::{le_f32, le_i32, le_i8, le_u16, le_u32, le_u8};
use nom::sequence::tuple;
use nom::IResult;

use crate::{Lap, RealtimeUpdate, RegistrationResult, SessionPhase, SessionType};
use std::convert::TryFrom;
use tinyvec::ArrayVec;

type Res<T, U> = IResult<T, U, VerboseError<T>>;

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
                error_message: res.4,
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

fn splits(input: &[u8]) -> Res<&[u8], ArrayVec<[i32; 3]>> {
    context(
        "splits",
        length_value(
            map(le_u8, |l: u8| l * 4),
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
            boolean,
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
                is_replay_playing,
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
                    active_camera_set,
                    active_camera,
                    current_hud_page,
                    is_replay_playing,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_kstring() {
        let input = b"\x03\x00abcefg";
        assert_eq!(kstring(input), Ok((&b"efg"[..], "abc")));
    }

    #[test]
    fn parse_empty_kstring() {
        let input = b"\x00\x00abc";
        assert_eq!(kstring(input), Ok((&b"abc"[..], "")));
    }

    #[test]
    fn parse_registration_result() {
        let input = b"\x01\x01\x00\x00\x00\x01\x01\x00\x00";
        let res = registration_result(input);

        assert_eq!(
            res,
            Ok((
                &[][..],
                RegistrationResult {
                    connection_id: 1,
                    connection_success: true,
                    read_only: true,
                    error_message: "",
                }
            ))
        );
    }

    #[test]
    fn parse_registration_fail() {
        let input = b"\x01\x01\x00\x00\x00\x00\x01\x10\x00Handshake failed";
        let res = registration_result(input);

        assert_eq!(
            res,
            Ok((
                &[][..],
                RegistrationResult {
                    connection_id: 1,
                    connection_success: false,
                    read_only: true,
                    error_message: "Handshake failed",
                }
            ))
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
        let input = include_bytes!("../docs/pcap/realtime_update.bin");
        let res = realtime_update(input).unwrap().1;

        assert_eq!(res.active_camera, "CameraPit3");
        assert_eq!(res.ambient_temp, 25);
        assert_eq!(res.session_type, SessionType::Qualifying);
    }
}
