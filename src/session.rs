use fnv::FnvHashMap;
use log::debug;

use crate::protocol::inbound::{self, Driver, EntrylistCar, Lap, RealtimeCarUpdate, TrackData};

/// The state of a Car in the current session
///
/// At least for now, the complete state is sent with each [`RealtimeCarUpdate`](inbound::RealtimeCarUpdate)
/// packet, so this is just a type alias to the packet definition.
pub type CarState = RealtimeCarUpdate;

#[derive(Debug, Clone)]
pub struct CarContext {
    pub entry: Option<EntrylistCar<'static>>,
    pub state: Option<CarState>,
    pub laps: Vec<(u16, Lap)>,
}

impl CarContext {
    fn new_from_entry(entry: EntrylistCar<'static>) -> CarContext {
        CarContext {
            entry: Some(entry),
            state: None,
            laps: vec![],
        }
    }

    fn new_from_update(update: RealtimeCarUpdate) -> CarContext {
        CarContext {
            entry: None,
            state: Some(update),
            laps: vec![],
        }
    }

    pub fn current_driver(&self) -> Option<&Driver> {
        self.entry.as_ref().map(|e| {
            assert!(e.drivers.len() >= e.current_driver_index as usize);
            &e.drivers[e.current_driver_index as usize]
        })
    }
}

#[derive(Default)]
pub struct Context {
    track: Option<TrackData<'static>>,
    cars: FnvHashMap<u16, CarContext>,
}

impl Context {
    pub fn new() -> Context {
        Context::default()
    }

    pub fn track_data(&self) -> Option<&TrackData> {
        self.track.as_ref()
    }

    pub fn car_by_id(&self, id: u16) -> Option<&CarContext> {
        self.cars.get(&id)
    }

    pub fn car_by_race_number(&self, number: i32) -> Option<&CarContext> {
        self.cars.iter().find_map(|(_, v)| {
            v.entry.as_ref().and_then(|e| {
                if e.race_number == number {
                    Some(v)
                } else {
                    None
                }
            })
        })
    }

    pub(crate) fn update_track_data(&mut self, track_data: inbound::TrackData) {
        self.track = Some(track_data.into_owned());
    }

    /// Takes an [`EntrylistUpdate`](crate::protocol::inbound::EntrylistUpdate) and prepares the internal
    /// `HashMap` for updates which will arrive shortly afterwards.
    pub(crate) fn seed_entrylist(&mut self, update: &inbound::EntrylistUpdate) {
        // First ensure enough space is allocated for the whole set of Cars
        if self.cars.capacity() < update.car_ids.len() {
            self.cars
                .reserve(update.car_ids.len() - self.cars.capacity());
        }

        // Retain only the car IDs still in the entry list
        self.cars.retain(|&k, _| update.car_ids.contains(&k));
    }

    pub(crate) fn update_car_entry(&mut self, updated_car: EntrylistCar) {
        if let Some(mut e) = self.cars.get_mut(&updated_car.id) {
            e.entry = Some(updated_car.into_owned());
        } else {
            self.cars.insert(
                updated_car.id,
                CarContext::new_from_entry(updated_car.into_owned()),
            );
        }
    }

    pub(crate) fn update_car_state(&mut self, update: RealtimeCarUpdate) {
        if let Some(mut e) = self.cars.get_mut(&update.id) {
            // Check if a lap has been completed
            if let Some(ref previous) = e.state {
                if update.laps > previous.laps {
                    debug!("Storing new lap {} for car {}", update.laps, update.id);
                    e.laps.push((update.laps, update.last_lap));
                }
            }
            // Overwrite the state with the new snapshot
            e.state = Some(update);
        } else {
            self.cars
                .insert(update.id, CarContext::new_from_update(update.clone()));
            // We might be connecting mid-session with a lap already completed
            if update.laps > 0 {
                debug!("Storing new lap {} for car {}", update.laps, update.id);
                self.cars
                    .get_mut(&update.id)
                    .unwrap()
                    .laps
                    .push((update.laps, update.last_lap));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::acc_enum::{CarModel, CupCategory, DriverCategory, Nationality};
    use crate::protocol::inbound::EntrylistUpdate;
    use std::collections::HashMap;

    fn context_with_entries() -> Context {
        let mut ctx = Context::new();
        let update = EntrylistUpdate {
            car_ids: vec![1001, 1002],
        };

        let cars = vec![
            EntrylistCar {
                id: 1001,
                model: CarModel::Ferrari488Evo,
                team_name: "Team 1".into(),
                race_number: 1,
                cup_category: CupCategory::Overall,
                current_driver_index: 0,
                nationality: Nationality::GreatBritain,
                drivers: vec![Driver {
                    first_name: "John".into(),
                    last_name: "Smith".into(),
                    short_name: "SMI".into(),
                    category: DriverCategory::Gold,
                    nationality: Nationality::GreatBritain,
                }],
            },
            EntrylistCar {
                id: 1002,
                model: CarModel::Ferrari488Evo,
                team_name: "Team 2".into(),
                race_number: 2,
                cup_category: CupCategory::Overall,
                current_driver_index: 0,
                nationality: Nationality::Ireland,
                drivers: vec![Driver {
                    first_name: "Jane".into(),
                    last_name: "Bloggs".into(),
                    short_name: "BLO".into(),
                    category: DriverCategory::Platinum,
                    nationality: Nationality::Ireland,
                }],
            },
        ];

        ctx.seed_entrylist(&update);

        assert!(ctx.cars.capacity() >= 2);

        for car in cars.into_iter() {
            ctx.update_car_entry(car);
        }

        ctx
    }

    #[test]
    fn store_and_retrieve_track_data() {
        let mut ctx = Context::new();
        assert!(ctx.track_data().is_none());

        let data = TrackData {
            name: "Spa".into(),
            id: 42,
            distance: 4812,
            camera_sets: HashMap::default(),
            hud_pages: vec![],
        };

        ctx.update_track_data(data);
        assert!(ctx.track_data().is_some());
        assert_eq!(ctx.track_data().unwrap().name, "Spa");
        assert_eq!(ctx.track_data().unwrap().distance, 4812);

        // Overwrite the existing structure
        let data = TrackData {
            name: "Silverstone".into(),
            id: 36,
            distance: 3210,
            camera_sets: HashMap::default(),
            hud_pages: vec![],
        };

        ctx.update_track_data(data);
        assert!(ctx.track_data().is_some());
        assert_eq!(ctx.track_data().unwrap().name, "Silverstone");
        assert_eq!(ctx.track_data().unwrap().distance, 3210);
    }

    #[test]
    fn populates_entry_list() {
        let mut ctx = context_with_entries();
        assert!(ctx.cars.get(&1001).unwrap().entry.is_some());
        assert_eq!(
            ctx.cars
                .get(&1001)
                .unwrap()
                .entry
                .as_ref()
                .unwrap()
                .team_name,
            "Team 1"
        );
        assert!(ctx.cars.get(&1002).unwrap().entry.is_some());
        assert_eq!(
            ctx.cars
                .get(&1002)
                .unwrap()
                .entry
                .as_ref()
                .unwrap()
                .team_name,
            "Team 2"
        );
        assert!(ctx.cars.get(&1003).is_none());

        // Check that one one car gets pruned
        let update = EntrylistUpdate {
            car_ids: vec![1002],
        };
        ctx.seed_entrylist(&update);

        // ID 1002 is still present and retains its data
        assert!(ctx.cars.get(&1002).is_some());
        assert_eq!(
            ctx.cars
                .get(&1002)
                .unwrap()
                .entry
                .as_ref()
                .unwrap()
                .team_name,
            "Team 2"
        );

        // ID 1001 has been pruned
        assert!(ctx.cars.get(&1001).is_none());
    }

    #[test]
    fn car_by_race_number() {
        let ctx = context_with_entries();

        assert_eq!(
            ctx.car_by_race_number(2)
                .unwrap()
                .entry
                .as_ref()
                .unwrap()
                .id,
            1002
        );
    }

    #[test]
    fn current_drivers() {
        let ctx = context_with_entries();

        let d1 = ctx.car_by_id(1001).unwrap().current_driver();
        let d2 = ctx.car_by_id(1002).unwrap().current_driver();

        assert!(d1.is_some());
        assert!(d2.is_some());

        assert_eq!(d1.unwrap().first_name, "John");
        assert_eq!(d2.unwrap().first_name, "Jane");
    }
}
