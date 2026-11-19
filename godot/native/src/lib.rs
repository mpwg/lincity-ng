//! Godot's CityWorld class owns one RustCity simulation per instance.
use std::{fs::File, io, path::Path};

use godot::{
    classes::{ProjectSettings, TranslationServer},
    prelude::*,
};
use rustcity_core::{City, CityEvent, Climate, GenerationOptions, Point, VehicleTraffic};

mod actions;
mod overlay;
mod view;

struct LinCityExtension;

// SAFETY: The extension uses godot-rust's registration and owns no global Godot objects.
#[gdextension(entry_symbol = lincity_library_init)]
unsafe impl ExtensionLibrary for LinCityExtension {}

#[derive(GodotClass)]
#[class(base=RefCounted, init)]
struct CityWorld {
    city: Option<City>,
    pending_events: VarArray,
    #[init(val = VehicleTraffic::Disabled)]
    traffic: VehicleTraffic,
}

#[godot_api]
impl CityWorld {
    #[func]
    fn new_city(
        &mut self,
        #[opt(default = 0_i64)] climate: i64,
        #[opt(default = 100_i64)] side: i64,
        #[opt(default = true)] village: bool,
        #[opt(default = true)] trees: bool,
    ) -> GString {
        let climate = match climate as i32 {
            0 => Climate::River,
            1 => Climate::Desert,
            2 => Climate::Temperate,
            3 => Climate::Swamp,
            _ => return localized("Unknown climate."),
        };
        match City::generate(
            side as i32,
            GenerationOptions {
                climate,
                village,
                trees,
            },
        ) {
            Ok(city) => {
                self.replace_city(city);
                GString::new()
            }
            Err(error) => localized(&error.to_string()),
        }
    }

    #[func]
    fn load_city(&mut self, path: GString) -> GString {
        let path = ProjectSettings::singleton()
            .globalize_path(&path)
            .to_string();
        match File::open(&path).and_then(City::load) {
            Ok(city) => {
                self.replace_city(city);
                GString::new()
            }
            Err(error) => {
                godot_error!("{error}");
                localized("The city could not be loaded.")
            }
        }
    }

    #[func]
    fn save_city(&self, path: GString) -> GString {
        let Some(city) = &self.city else {
            return localized("No city loaded.");
        };
        let path = ProjectSettings::singleton()
            .globalize_path(&path)
            .to_string();
        if let Err(error) = save(city, Path::new(&path)) {
            godot_error!("{error}");
            return localized("The city could not be saved.");
        }
        GString::new()
    }

    #[func]
    fn step(&mut self) -> GString {
        let Some(city) = &mut self.city else {
            return localized("No city loaded.");
        };
        city.set_vehicle_traffic(self.traffic);
        match city.tick() {
            Ok(()) => GString::new(),
            Err(error) => localized(&error.to_string()),
        }
    }

    #[func]
    fn set_vehicle_traffic(&mut self, mode: i64) {
        if self.city.is_some() {
            self.traffic = match mode as i32 {
                2 => VehicleTraffic::Normal,
                1 => VehicleTraffic::Reduced,
                _ => VehicleTraffic::Disabled,
            };
        }
    }

    #[func]
    fn advance_animation(&mut self, millis: i64) -> PackedVector3Array {
        let Some(city) = &mut self.city else {
            return PackedVector3Array::new();
        };
        if let Err(error) = city.animate(millis as u64) {
            godot_error!("{error}");
            return PackedVector3Array::new();
        }
        city.traffic_positions()
            .map(|(x, z, heading)| Vector3::new(x, heading as f32, z))
            .collect()
    }

    #[func]
    fn advance_traffic(&mut self, millis: i64) -> PackedVector3Array {
        self.advance_animation(millis)
    }

    #[func]
    fn snapshot(&self) -> VarDictionary {
        self.city
            .as_ref()
            .map_or_else(VarDictionary::new, view::snapshot)
    }

    #[func]
    fn tools(&self) -> VarArray {
        actions::tools()
    }

    #[func]
    fn preview(&self, id: i64, position: Vector2i) -> VarDictionary {
        actions::preview(self.city.as_ref(), id as i32, position)
    }

    #[func]
    fn apply(&mut self, id: i64, position: Vector2i) -> GString {
        let Some(city) = &mut self.city else {
            return localized("No city loaded.");
        };
        let (result, warning) = actions::apply(city, id as i32, position);
        self.remember_warning(warning);
        result
    }

    #[func]
    fn preview_batch(&mut self, id: i64, positions: Array<Vector2i>) -> VarArray {
        actions::preview_batch(self.city.as_ref(), id as i32, positions)
    }

    #[func]
    fn apply_batch(&mut self, id: i64, positions: Array<Vector2i>) -> VarArray {
        actions::apply_batch(self.city.as_mut(), id as i32, positions)
    }

    #[func]
    fn inspect_tile(
        &self,
        position: Vector2i,
        #[opt(default = false)] production: bool,
    ) -> GString {
        self.city.as_ref().map_or_else(GString::new, |city| {
            view::inspect_tile(city, point(position), production)
        })
    }

    #[func]
    fn set_focus(&mut self, position: Vector2i) {
        let position = point(position);
        if let Some(city) = &mut self.city
            && city.tile(position).is_some()
        {
            city.set_focus(position);
        }
    }

    #[func]
    fn survey_coal(&mut self) -> GString {
        let Some(city) = &mut self.city else {
            return localized("No city loaded.");
        };
        let (result, warning) = actions::survey_coal(city);
        self.remember_warning(warning);
        result
    }

    #[func]
    fn launch_rocket(&mut self, position: Vector2i) -> GString {
        if let Some(city) = &mut self.city
            && let Some((_, anchor)) = city.building_at(point(position))
            && city.launch_rocket(anchor)
        {
            return GString::new();
        }
        localized("No rocket is ready to launch here.")
    }

    #[func]
    fn trade_rules(&self, position: Vector2i) -> VarArray {
        self.city.as_ref().map_or_else(VarArray::new, |city| {
            view::trade_rules(city, point(position))
        })
    }

    #[func]
    fn set_trade(&mut self, position: Vector2i, id: i64, buy: bool, sell: bool) -> GString {
        let Some(city) = &mut self.city else {
            return localized("No city loaded.");
        };
        let Some(&commodity) = rustcity_core::Commodity::ALL.get(id as i32 as usize) else {
            return localized("Unknown commodity.");
        };
        let position = point(position);
        if city.set_market_trade(position, commodity, buy, sell)
            || city.set_port_trade(position, commodity, buy, sell)
        {
            return GString::new();
        }
        localized("This building cannot trade this commodity.")
    }

    #[func]
    fn overlay(&self, id: i64, #[opt(default = 0_i64)] commodity: i64) -> PackedColorArray {
        self.city
            .as_ref()
            .map_or_else(PackedColorArray::new, |city| {
                overlay::overlay(city, id as i32, commodity as i32)
                    .into_iter()
                    .map(|rgb| {
                        let [_, r, g, b] = rgb.to_be_bytes();
                        Color::from_rgba8(r, g, b, 255)
                    })
                    .collect()
            })
    }

    #[func]
    fn finances(&self) -> VarDictionary {
        self.city
            .as_ref()
            .map_or_else(VarDictionary::new, view::finances)
    }

    #[func]
    fn history(&self) -> VarDictionary {
        self.city
            .as_ref()
            .map_or_else(VarDictionary::new, view::history)
    }

    #[func]
    fn take_events(&mut self) -> VarArray {
        self.collect_events();
        std::mem::take(&mut self.pending_events)
    }

    #[func]
    fn tile_resource(&self, position: Vector2i) -> GString {
        self.city.as_ref().map_or_else(GString::new, |city| {
            view::tile_resource(city, point(position))
        })
    }
}

impl CityWorld {
    fn replace_city(&mut self, city: City) {
        self.city = Some(city);
        self.pending_events.clear();
        self.traffic = VehicleTraffic::Disabled;
    }

    fn remember_warning(&mut self, warning: Option<actions::Warning>) {
        if let Some(warning) = warning {
            self.collect_events();
            self.pending_events.push(
                &vdict! {
                    "kind" => warning.kind(),
                    "text" => &localized(warning.message()),
                }
                .to_variant(),
            );
        }
    }

    fn collect_events(&mut self) {
        if let Some(city) = &mut self.city {
            while let Some(event) = city.pop_event() {
                self.pending_events
                    .push(&event_dictionary(event).to_variant());
            }
        }
    }
}

fn event_dictionary(event: CityEvent) -> VarDictionary {
    let (kind, message) = match event {
        CityEvent::SustainableEconomy(_) => ("economy", "The economy is sustainable."),
        CityEvent::NoPeopleLeft(_) => ("loss", "The city has lost all its inhabitants."),
        CityEvent::RocketReady(_) => ("rocket_ready", "A rocket is ready to launch."),
        CityEvent::RocketFailed(_) => ("rocket_crash", "The rocket crashed."),
        CityEvent::RocketSuccess(_) => ("rocket_success", "The rocket launched successfully."),
        CityEvent::RocketEvacuated(_) => ("rocket_evacuation", "1,000 people were evacuated."),
        CityEvent::RocketWon(_) => ("evacuation", "Everyone has been evacuated from the city."),
        CityEvent::OutOfMoney => ("out_of_money", "Out of Money"),
        CityEvent::FireStarted(cell, group) => {
            let name = localized(group.metadata().name).to_string();
            let position = format!("({},{})", cell.x, cell.y);
            return vdict! {
                "kind" => "fire",
                "cell" => Vector2i::new(cell.x, cell.y),
                "group" => i32::from(group.id()),
                "text" => &localize_format("A fire has started: {} {}.", &[&name, &position]),
            };
        }
    };
    vdict! { "kind" => kind, "text" => &localized(message) }
}

fn save(city: &City, path: &Path) -> io::Result<()> {
    let directory = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut temporary = tempfile::NamedTempFile::new_in(directory)?;
    city.save(temporary.as_file_mut())?;
    temporary.persist(path)?;
    Ok(())
}

fn point(position: Vector2i) -> Point {
    Point {
        x: position.x,
        y: position.y,
    }
}

fn localized(key: &str) -> GString {
    TranslationServer::singleton()
        .translate(&StringName::from(key))
        .to_gstring()
}

// The existing catalogs use {} for text and {:.1f} for a preformatted percentage.
fn localize_format(key: &str, arguments: &[&str]) -> GString {
    let translation = localized(key).to_string();
    let mut remaining = translation.as_str();
    let mut text = String::new();
    for argument in arguments {
        let Some((start, length)) = remaining.match_indices('{').find_map(|(i, _)| {
            let placeholder = &remaining[i..];
            if placeholder.starts_with("{}") {
                Some((i, 2))
            } else if placeholder.starts_with("{:.1f}") {
                Some((i, 6))
            } else {
                None
            }
        }) else {
            break;
        };
        text.push_str(&remaining[..start]);
        text.push_str(argument);
        remaining = &remaining[start + length..];
    }
    text.push_str(remaining);
    GString::from(&text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saving_preserves_the_resume_position_and_city_state() {
        let fixture = include_bytes!("../../../rustcity-core/tests/fixtures/blacksmith-2140.xml");
        let mut city = City::load(fixture.as_slice()).unwrap();
        let focus = Point { x: 3, y: 4 };
        city.set_focus(focus);
        city.tick().unwrap();

        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("city.gz");
        save(&city, &path).unwrap();
        let resumed = City::load(File::open(path).unwrap()).unwrap();

        assert_eq!(resumed.side(), city.side());
        assert_eq!(resumed.focus(), focus);
        assert_eq!(resumed.report().day, city.report().day);
        assert_eq!(resumed.balance(), city.balance());
        city.tick().unwrap();
        assert_eq!(resumed.report().day + 1, city.report().day);
    }

    #[test]
    fn replacing_a_directory_with_a_save_fails_without_removing_its_contents() {
        let city = City::empty(32).unwrap();
        let directory = tempfile::tempdir().unwrap();
        let marker = directory.path().join("existing.txt");
        std::fs::write(&marker, b"existing content").unwrap();

        assert!(save(&city, directory.path()).is_err());
        assert_eq!(std::fs::read(marker).unwrap(), b"existing content");
    }
}
