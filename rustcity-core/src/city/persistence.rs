use std::io::{self, Read, Write};

use super::{Building, City, Point, Terrain, Tile};
use crate::save::{Element, SaveGame, integer, invalid};
use crate::{BuildingKind, Commodity};

const LAKE: u32 = 0x0100_0000;
const RIVER: u32 = 0x0200_0000;
const GROUNDWATER: u32 = 0x2000_0000;
const EVACUATE: u32 = 0x200;

fn flags(node: &Element) -> io::Result<u32> {
    node.child("flags")
        .map(|v| u32::try_from(integer(&v.text())?).map_err(invalid))
        .transpose()
        .map(|flags| flags.unwrap_or(0) & !3)
}

fn location(node: &Element) -> io::Result<Point> {
    Ok(Point {
        x: node.attr_i32("map-x")?,
        y: node.attr_i32("map-y")?,
    })
}

impl Terrain {
    pub(super) fn group(self) -> u16 {
        match self {
            Self::Grass => 0,
            Self::Water => 21,
            Self::Desert => 42,
            Self::Tree => 43,
            Self::Tree2 => 44,
            Self::Tree3 => 45,
        }
    }

    fn from_group(group: i32) -> io::Result<Self> {
        match group {
            0 => Ok(Self::Grass),
            21 => Ok(Self::Water),
            42 => Ok(Self::Desert),
            43 => Ok(Self::Tree),
            44 => Ok(Self::Tree2),
            45 => Ok(Self::Tree3),
            _ => Err(invalid(format!("unknown terrain group {group}"))),
        }
    }
}

impl Tile {
    fn load(node: &Element) -> io::Result<Self> {
        let flag = flags(node)?;
        Ok(Self {
            altitude: node.child_i32("altitude", 0)?,
            water_alt: node.child_i32("water_alt", 0)?,
            lake: flag & LAKE != 0,
            diffuses: node.attr("diffusion-active") == Some("1"),
            coverage: flag & super::civic::COVER_MASK,
            river: flag & RIVER != 0,
            groundwater: flag & GROUNDWATER != 0,
            ground: Terrain::from_group(node.attr_i32("group")?)?,
            frame: node.child_i32("type", 0)? as u16,
            pollution: node.child_i32("air_pol", 0)?,
            ore: node.child_i32("ore", 0)?,
            coal: node.child_i32("coal", 0)?,
            cables: 0,
        })
    }

    fn store(self, node: &mut Element) -> io::Result<()> {
        node.set_attr("group", self.ground.group().to_string());
        if self.diffuses {
            node.set_attr("diffusion-active", "1");
        } else {
            node.remove_attr("diffusion-active");
        }
        let flag = (flags(node)? & !(LAKE | RIVER | GROUNDWATER | super::civic::COVER_MASK))
            | if self.lake { LAKE } else { 0 }
            | self.coverage
            | if self.river { RIVER } else { 0 }
            | if self.groundwater { GROUNDWATER } else { 0 };
        node.set_child("flags", format!("{flag:#x}"));
        node.set_child("altitude", self.altitude);
        node.set_child("water_alt", self.water_alt);
        node.set_child("type", self.frame);
        node.set_child("ore", self.ore);
        node.set_child("coal", self.coal);
        node.set_child("air_pol", self.pollution);
        const FIELDS: [&str; 17] = [
            "flags",
            "type",
            "air_pol",
            "ore",
            "coal",
            "altitude",
            "ecotable",
            "wastes",
            "grd_pol",
            "water_alt",
            "water_pol",
            "water_wast",
            "water_next",
            "int1",
            "int2",
            "int3",
            "int4",
        ];
        for field in &FIELDS[4..] {
            node.default_child(field, 0);
        }
        node.order_children(&FIELDS);
        Ok(())
    }
}

/// Replace historical trade fields, whose ordering changed between save versions.
pub(super) fn store_trade_rules(
    n: &mut Element,
    rules: impl IntoIterator<Item = (Commodity, bool, bool)>,
) {
    n.retain_elements(|e| {
        let commodity = e
            .name
            .strip_prefix("give_")
            .or_else(|| e.name.strip_prefix("take_"));
        commodity.and_then(Commodity::from_save_name).is_none()
    });
    for (commodity, take, give) in rules {
        let name = commodity.save_name();
        n.set_child(&format!("give_{name}"), i32::from(give));
        n.set_child(&format!("take_{name}"), i32::from(take));
    }
}

impl City {
    /// Loads the plain or gzip-compressed city files used by LinCity.
    pub fn load(reader: impl Read) -> io::Result<Self> {
        Self::load_with_decisions(reader, crate::decisions::Decisions::native(rand::random()))
    }

    pub fn load_with_decisions(
        reader: impl Read,
        decisions: crate::decisions::Decisions,
    ) -> io::Result<Self> {
        let mut save = SaveGame::read(reader)?;
        save.upgrade()?;
        let mut city = Self::empty_with_decisions(save.map().attr_i32("size")?, decisions)
            .map_err(|_| invalid("invalid city size"))?;
        let g = save.globals();
        city.economy.load(g)?;
        city.rockets_launched = g.child_i32("rockets_launched", 0)?;
        city.rockets_successful = g.child_i32("rockets_launched_success", 0)?;
        city.scenario_name = g
            .child("given_scene")
            .map(|node| node.text().into_owned())
            .unwrap_or_default();
        city.game_end = g
            .child("game_end")
            .map(Element::boolean)
            .transpose()?
            .unwrap_or_default();
        city.balance = g.child_i32("total_money", 0)?;
        city.day = g.child_i32("total_time", 0)?;
        city.people_pool = g.child_i32("people_pool", 100)?;
        city.stats.total_money = city.balance;
        city.tech_level = g.child_i32("tech_level", 0)?;
        city.stats.tech_level = city.tech_level;
        city.stats.sustainability.old_population = 100;
        if let Some(stats) = g.child("stats") {
            crate::save::accounts::load(&mut city.stats, stats)?;
        }
        for node in save.map().elements().filter(|n| n.name == "MapTile") {
            let idx = city
                .index(location(node)?)
                .ok_or_else(|| invalid("tile outside map"))?;
            city.tiles[idx] = Tile::load(node)?;
        }
        // C++ constructs every building before placing any of them. Farm constructor
        // draws must therefore precede the first placement in the decision tape.
        let mut pending = Vec::new();
        for node in save.map().elements().filter(|n| n.name == "Construction") {
            let kind = u16::try_from(node.attr_i32("group")?)
                .ok()
                .and_then(BuildingKind::from_id)
                .ok_or_else(|| invalid("unknown construction group"))?;
            let building = city.create_building(kind, node.child_i32("tech", city.tech_level)?)?;
            pending.push((node, building));
        }
        for (node, building) in pending {
            city.load_building(node, building)?;
        }
        if let Some(point) = save.map().child("recentPoint") {
            city.recent_point = Point {
                x: point.child_i32("x", city.recent_point.x)?,
                y: point.child_i32("y", city.recent_point.y)?,
            };
        }
        city.refresh_transport(
            Point { x: 1, y: 1 },
            Point {
                x: city.side - 2,
                y: city.side - 2,
            },
        );
        city.update_frontiers(
            Point { x: 1, y: 1 },
            Point {
                x: city.side - 1,
                y: city.side - 1,
            },
        );
        city.saved = Some(save);
        Ok(city)
    }

    fn load_building(&mut self, node: &Element, mut building: Building) -> io::Result<()> {
        let point = location(node)?;
        if building.kind.is_transport() {
            building.kind = self.actual_kind(building.kind, point);
        }
        let kind = building.kind;
        let group = kind.id();
        let flags = flags(node)?;
        building.evacuating = flags & EVACUATE != 0;
        if node.child("flags").is_some() {
            building.transparent = flags & 0x0800_0000 != 0;
        }
        let transparent = building.transparent;
        if let super::Operation::Civic(c) = &mut building.operation {
            c.load(node, flags)?;
        }
        for (slot, c) in building.stock.iter_mut().zip(Commodity::ALL) {
            *slot = node.child_i32(c.save_name(), 0)?;
        }
        if let super::Operation::Port(port) = &mut building.operation {
            port.load(node)?;
        }
        if let super::Operation::Market(m) = &mut building.operation {
            m.load(node)?;
        }
        if let super::Operation::Rocket(r) = &mut building.operation {
            r.load(node)?;
        }
        building.saved = Some(node.clone());
        let size = i32::from(kind.metadata().size);
        self.place_building(point, size, building)?;
        self.stats.group_count[usize::from(group)] += 1;
        self.recent_point = point;
        for y in point.y..point.y + size {
            for x in point.x..point.x + size {
                let tile = &mut self.tiles[(y * self.side + x) as usize];
                if tile.ground != Terrain::Water {
                    if !transparent {
                        tile.ground = Terrain::Desert;
                        tile.frame = 0;
                    } else if tile.ground != Terrain::Desert {
                        tile.ground = Terrain::Grass;
                        tile.frame = 0;
                    }
                }
            }
        }
        Ok(())
    }

    /// Writes a gzip-compressed 2140 save with the current accounts, map and buildings.
    pub fn save(&self, writer: impl Write) -> io::Result<()> {
        let mut save = self
            .saved
            .clone()
            .unwrap_or_else(|| SaveGame::new(self.side));
        let g = save.globals_mut();
        self.economy.store(g);
        g.set_child("rockets_launched", self.rockets_launched);
        g.set_child("rockets_launched_success", self.rockets_successful);
        g.set_child("given_scene", &self.scenario_name);
        g.set_child("game_end", self.game_end);
        g.set_child("total_time", self.day);
        g.set_child("people_pool", self.people_pool);
        g.set_child("total_money", self.balance);
        g.set_child("tech_level", self.tech_level);
        crate::save::accounts::store(&self.stats, g.section("stats"), self.day, self.balance);
        crate::save::accounts::complete_globals(g);
        let map = save.map_mut();
        map.retain_elements(|e| e.name != "Construction");
        let mut covered = vec![false; self.tiles.len()];
        for node in map.elements_mut().filter(|e| e.name == "MapTile") {
            let idx = self
                .index(location(node)?)
                .ok_or_else(|| invalid("tile outside map"))?;
            self.tiles[idx].store(node)?;
            covered[idx] = true;
        }
        for (i, tile) in self.tiles.iter().enumerate().filter(|(i, _)| !covered[*i]) {
            let mut node = Element::new("MapTile");
            node.set_attr("map-x", (i % self.side as usize).to_string());
            node.set_attr("map-y", (i / self.side as usize).to_string());
            tile.store(&mut node)?;
            map.push(node);
        }
        for (origin, building) in self.buildings.iter() {
            let mut node = building
                .saved
                .clone()
                .unwrap_or_else(|| Element::new("Construction"));
            if let super::Operation::Port(port) = &building.operation {
                port.store(&mut node);
            }
            if let super::Operation::Industry(ref industry) = building.operation {
                industry.store(&mut node);
            }
            if let super::Operation::Market(m) = &building.operation {
                m.store(&mut node);
            }
            // Older saves contain cached output and efficiency values recomputed at placement.
            let obsolete: &[&str] = match building.kind {
                BuildingKind::OrganicFarm => &["tech_bonus"],
                BuildingKind::Windmill => &["kwh_output"],
                BuildingKind::CoalPower | BuildingKind::WindPower | BuildingKind::SolarPower => {
                    &["mwh_output"]
                }
                BuildingKind::LightIndustry | BuildingKind::HeavyIndustry => {
                    &["bonus", "extra_bonus"]
                }
                BuildingKind::Recycle => &["make_ore", "make_steel"],
                _ => &[],
            };
            node.retain_elements(|e| !obsolete.contains(&e.name.as_str()));
            if building.kind == BuildingKind::RocketPad {
                node.default_child("steps", 0);
                node.default_child("stage", "building");
            }
            if let super::Operation::Rocket(r) = &building.operation {
                r.store(&mut node);
            }
            node.set_attr("group", building.kind.id().to_string());
            node.set_attr("map-x", origin.x.to_string());
            node.set_attr("map-y", origin.y.to_string());
            if matches!(
                building.kind,
                BuildingKind::CoalPower
                    | BuildingKind::Windmill
                    | BuildingKind::WindPower
                    | BuildingKind::SolarPower
                    | BuildingKind::HeavyIndustry
                    | BuildingKind::LightIndustry
                    | BuildingKind::OrganicFarm
                    | BuildingKind::RocketPad
                    | BuildingKind::Recycle
            ) {
                node.set_child("tech", building.installed_tech);
            }
            if building.kind == BuildingKind::RocketPad {
                node.order_children(&["tech", "steps", "stage"]);
            }
            let default_flags = match building.kind {
                BuildingKind::PowerLine => 0x08000100,
                BuildingKind::Track | BuildingKind::Rail | BuildingKind::Road => 0x180,
                BuildingKind::TrackBridge | BuildingKind::RoadBridge | BuildingKind::RailBridge => {
                    0x08000180
                }
                BuildingKind::ParkPond
                | BuildingKind::Parkland
                | BuildingKind::Tip
                | BuildingKind::Shanty => 0x100,
                _ => 0,
            };
            let mut flag = if node.child("flags").is_some() {
                flags(&node)?
            } else {
                default_flags
            };
            flag = flag & !EVACUATE | if building.evacuating { EVACUATE } else { 0 };
            if let super::Operation::Civic(c) = &building.operation {
                c.store(&mut node, &mut flag);
                node.order_children(c.ordered_fields());
            }
            if let super::Operation::Transport(t) = &building.operation {
                flag = flag & !0x0800_0000 | if t.transparent { 0x0800_0000 } else { 0 };
            }
            node.set_child("flags", format!("{flag:#x}"));
            for (amount, commodity) in building.stock.iter().zip(Commodity::ALL) {
                if *amount != 0
                    || node.child(commodity.save_name()).is_some()
                    || building.kind.metadata().rule(commodity).max_load != 0
                {
                    node.set_child(commodity.save_name(), amount);
                }
            }
            // The C++ base writer appends flags and commodity stocks after module-specific data.
            let mut names: Vec<_> = node
                .elements()
                .filter(|e| e.name != "flags" && Commodity::from_save_name(&e.name).is_none())
                .map(|e| e.name.clone())
                .collect();
            names.push("flags".into());
            names.extend(Commodity::ALL.into_iter().map(|c| c.save_name().to_owned()));
            node.order_children(&names.iter().map(String::as_str).collect::<Vec<_>>());
            map.push(node);
        }
        let point = map.section("recentPoint");
        point.set_child("x", self.recent_point.x);
        point.set_child("y", self.recent_point.y);
        save.write_gzip(writer)
    }
}
