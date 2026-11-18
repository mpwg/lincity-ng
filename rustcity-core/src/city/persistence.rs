use std::io::{self, Read, Write};

use super::{Building, City, Point, Terrain, Tile};
use crate::save::{Element, SaveGame, integer, invalid};
use crate::{BuildingKind, Commodity};

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
    fn group(self) -> u16 {
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
            river: flag & RIVER != 0,
            groundwater: flag & GROUNDWATER != 0,
            ground: Terrain::from_group(node.attr_i32("group")?)?,
            frame: node.child_i32("type", 0)? as u16,
            pollution: node.child_i32("air_pol", 0)?,
            ore: node.child_i32("ore", 0)?,
        })
    }

    fn store(self, node: &mut Element) -> io::Result<()> {
        node.set_attr("group", self.ground.group().to_string());
        let flag = (flags(node)? & !(RIVER | GROUNDWATER))
            | if self.river { RIVER } else { 0 }
            | if self.groundwater { GROUNDWATER } else { 0 };
        node.set_child("flags", format!("{flag:#x}"));
        node.set_child("type", self.frame);
        node.set_child("ore", self.ore);
        node.set_child("air_pol", self.pollution);
        Ok(())
    }
}

impl City {
    /// Loads the plain or gzip-compressed city files used by LinCity.
    pub fn load(reader: impl Read) -> io::Result<Self> {
        let mut save = SaveGame::read(reader)?;
        save.upgrade()?;
        let mut city =
            Self::empty(save.map().attr_i32("size")?).map_err(|_| invalid("invalid city size"))?;
        let g = save.globals();
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
        for node in save.map().elements().filter(|n| n.name == "Construction") {
            city.load_building(node)?;
        }
        if let Some(point) = save.map().child("recentPoint") {
            city.recent_point = Point {
                x: point.child_i32("x", city.recent_point.x)?,
                y: point.child_i32("y", city.recent_point.y)?,
            };
        }
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

    fn load_building(&mut self, node: &Element) -> io::Result<()> {
        let point = location(node)?;
        let group = u16::try_from(node.attr_i32("group")?).map_err(invalid)?;
        let kind = BuildingKind::from_id(group)
            .ok_or_else(|| invalid(format!("unknown construction group {group}")))?;
        let mut building = Building::installed(kind, node.child_i32("tech", self.tech_level)?);
        let flags = flags(node)?;
        building.evacuating = flags & EVACUATE != 0;
        for (slot, c) in building.stock.iter_mut().zip(Commodity::ALL) {
            *slot = node.child_i32(c.save_name(), 0)?;
        }
        building.saved = Some(node.clone());
        let size = i32::from(kind.metadata().size);
        self.buildings.place(point, size, building).ok_or_else(|| {
            invalid(format!(
                "invalid or overlapping building footprint at {},{}",
                point.x, point.y
            ))
        })?;
        self.connect_building(point);
        self.stats.group_count[usize::from(group)] += 1;
        self.recent_point = point;
        let transparent = if node.child("flags").is_some() {
            flags & 0x0800_0000 != 0
        } else {
            kind == BuildingKind::PowerLine
        };
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
        g.set_child("total_time", self.day);
        g.set_child("people_pool", self.people_pool);
        g.set_child("total_money", self.balance);
        g.set_child("tech_level", self.tech_level);
        crate::save::accounts::store(&self.stats, g.section("stats"), self.day, self.balance);
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
            if building.evacuating || node.child("flags").is_some() {
                let flag =
                    flags(&node)? & !EVACUATE | if building.evacuating { EVACUATE } else { 0 };
                node.set_child("flags", format!("{flag:#x}"));
            }
            for (amount, commodity) in building.stock.iter().zip(Commodity::ALL) {
                if *amount != 0
                    || node.child(commodity.save_name()).is_some()
                    || building.kind.metadata().rule(commodity).max_load != 0
                {
                    node.set_child(commodity.save_name(), amount);
                }
            }
            map.push(node);
        }
        let point = map.section("recentPoint");
        point.set_child("x", self.recent_point.x);
        point.set_child("y", self.recent_point.y);
        save.write_gzip(writer)
    }
}
