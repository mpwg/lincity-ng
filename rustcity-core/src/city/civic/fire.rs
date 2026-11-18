use super::*;

pub(in crate::city) struct FireState {
    pub(in crate::city) burning: i32,
    pub(in crate::city) smoking: i32,
    spread_in: i32,
    pub(in crate::city) frame: i32,
    next_frame: u64,
}

impl Default for FireState {
    fn default() -> Self {
        Self {
            burning: 0,
            smoking: 0,
            spread_in: FIRE_SPREAD_DAYS,
            frame: 0,
            next_frame: 0,
        }
    }
}

impl FireState {
    pub(super) fn load(&mut self, node: &Element) -> io::Result<()> {
        self.burning = node.child_i32("burning_days", 0)?;
        self.smoking = node.child_i32("smoking_days", 0)?;
        self.spread_in = node.child_i32("days_before_spread", FIRE_SPREAD_DAYS)?;
        Ok(())
    }
    pub(super) fn store(&self, node: &mut Element) {
        node.set_child("burning_days", self.burning);
        node.set_child("smoking_days", self.smoking);
        node.set_child("days_before_spread", self.spread_in);
    }
    pub(super) fn animate(&mut self, time: u64, decisions: &mut Decisions) -> io::Result<()> {
        if self.smoking != 0 {
            self.frame =
                (self.smoking * FINAL_SMOKE_FRAME / FIRE_SMOKING_DAYS).min(FINAL_SMOKE_FRAME);
        } else if time >= self.next_frame {
            self.next_frame = time.wrapping_add(fire_delay(decisions)?);
            self.frame = (self.frame + 1) % FIRE_FRAMES;
        }
        Ok(())
    }
}

fn fire_delay(decisions: &mut Decisions) -> io::Result<u64> {
    let ms = FIRE_MIN_FRAME_MS + decisions.c_modulo(FIRE_FRAME_VARIATION_MS)?;
    // ANIM_THRESHOLD rounds to the nearest 100 ms and fires half a tick early.
    Ok(
        ((ms + ANIMATION_QUANTUM_MS / 2) / ANIMATION_QUANTUM_MS * ANIMATION_QUANTUM_MS
            - ANIMATION_QUANTUM_MS / 2) as u64,
    )
}

#[derive(Default)]
pub(in crate::city) struct WasteFire {
    pub(in crate::city) start: bool,
    until: u64,
    next_frame: u64,
    pub(in crate::city) frame: Option<i32>,
}

impl WasteFire {
    pub(in crate::city) fn animate(
        &mut self,
        time: u64,
        visible_ms: u64,
        decisions: &mut Decisions,
    ) -> io::Result<()> {
        if self.start {
            self.start = false;
            self.until = time + visible_ms;
        }
        if time >= self.until {
            self.frame = None;
        } else if time >= self.next_frame {
            self.next_frame = time.wrapping_add(fire_delay(decisions)?);
            self.frame = Some(self.frame.map_or(0, |f| (f + 1) % FIRE_FRAMES));
        }
        Ok(())
    }
}

impl City {
    pub(super) fn update_fire(&mut self, id: BuildingId, point: Point) -> io::Result<()> {
        let b = self.buildings.get_mut(id).expect("visited fire");
        let Operation::Civic(Civic::Fire(fire)) = &mut b.operation else {
            unreachable!()
        };
        let tile = &mut self.tiles[(point.y * self.side + point.x) as usize];
        let increment = 1 + FIRE_PROTECTION_ACCELERATION * i32::from(tile.coverage & FIRE != 0);
        if fire.burning == 0 {
            fire.burning = self.decisions.c_modulo(FIRE_INITIAL_BURN_SPAN)?;
        }
        if fire.burning > FIRE_BURNING_DAYS {
            if fire.smoking == 0 {
                fire.smoking = self.decisions.c_modulo(FIRE_INITIAL_SMOKE_SPAN)?;
            }
            fire.smoking += increment;
            if fire.smoking > FIRE_SMOKING_DAYS {
                self.remove_and_restore_site(id, point);
            }
            return Ok(());
        }
        fire.burning += increment;
        fire.spread_in -= 1;
        tile.pollution += 1;
        if fire.spread_in != 0 {
            return Ok(());
        }
        fire.spread_in = FIRE_SPREAD_DAYS;
        let (dx, dy) = match self.decisions.c_modulo(FIRE_SPREAD_ODDS)? {
            0 => (0, -1),
            1 => (0, 1),
            2 => (1, 0),
            3 => (-1, 0),
            _ => return Ok(()),
        };
        let q = Point {
            x: point.x + dx,
            y: point.y + dy,
        };
        if let Some(target_id) = self.buildings.at(q) {
            let (origin, target) = self.buildings.get(target_id).unwrap();
            if self.decisions.c_modulo(100)? < target.kind.metadata().fire_chance
                && self.tiles[(q.y * self.side + q.x) as usize].coverage & FIRE == 0
                && target.kind != BuildingKind::Fire
            {
                self.torch_building(target_id, origin)?;
            }
        }
        Ok(())
    }

    pub(in crate::city) fn torch_building(
        &mut self,
        id: BuildingId,
        origin: Point,
    ) -> io::Result<()> {
        let b = self.buildings.get_mut(id).expect("live fire target");
        if b.kind == BuildingKind::Fire {
            return Ok(());
        }
        let size = i32::from(b.kind.metadata().size);
        if let Operation::Civic(Civic::Residence(h)) = &mut b.operation {
            let victims = h.occupants / FIRE_CASUALTY_DIVISOR;
            self.stats.population.deaths_m += victims;
            self.stats.population.unnat_deaths_m += victims;
            self.people_pool += h.occupants - victims;
            h.occupants = 0;
        }
        self.remove_and_restore_site(id, origin);
        self.place_fires(origin, size, 0)
    }

    pub(in crate::city) fn place_fires(
        &mut self,
        origin: Point,
        size: i32,
        burning: i32,
    ) -> io::Result<()> {
        for y in origin.y..origin.y + size {
            for x in origin.x..origin.x + size {
                let p = Point { x, y };
                let mut fire = self.create_building(BuildingKind::Fire, self.tech_level)?;
                if let Operation::Civic(Civic::Fire(f)) = &mut fire.operation {
                    f.burning = burning;
                }
                self.place_building(p, 1, fire)?;
                self.stats.group_count[BuildingKind::Fire.id() as usize] += 1;
                self.recent_point = p;
                let tile = &mut self.tiles[(y * self.side + x) as usize];
                if tile.ground != Terrain::Water {
                    tile.ground = Terrain::Desert;
                    tile.frame = 0;
                }
            }
        }
        self.update_frontiers(
            origin,
            Point {
                x: origin.x + size,
                y: origin.y + size,
            },
        );
        Ok(())
    }
}
