//! Rocket construction, launch animation and evacuation.
use super::*;
use crate::rules::{DAYS_PER_MONTH, MAX_TECH_LEVEL, rocket::*};
use crate::{Commodity, save::Element};

/// Current construction or launch phase of a rocket pad.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub enum RocketStage {
    #[default]
    Building,
    Awaiting,
    Launching,
    Launch,
    Done,
}
#[derive(Default)]
pub(super) struct Rocket {
    pub(super) stage: RocketStage,
    pub(super) steps: i32,
    working_days: i32,
    pub(super) busy: i32,
    pub(super) frame: u8,
    next_frame: u64,
}
impl Rocket {
    pub(super) fn ready_to_launch(&self) -> bool {
        self.stage == RocketStage::Awaiting
    }

    pub(super) fn load(&mut self, n: &Element) -> io::Result<()> {
        self.steps = n.child_i32("steps", 0)?;
        if let Some(stage) = n.child("stage") {
            self.stage = match stage.text().as_ref() {
                "building" => RocketStage::Building,
                "awaiting" => RocketStage::Awaiting,
                "done" => RocketStage::Done,
                other => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("unknown rocket stage {other}"),
                    ));
                }
            };
        }
        Ok(())
    }
    pub(super) fn store(&self, n: &mut Element) {
        n.set_child("steps", self.steps);
        n.set_child(
            "stage",
            match self.stage {
                RocketStage::Building => "building",
                RocketStage::Done => "done",
                _ => "awaiting",
            },
        );
    }
    pub(super) fn animate(&mut self, time: u64) {
        match self.stage {
            RocketStage::Building | RocketStage::Awaiting => {
                self.frame =
                    FRAME_STEPS.partition_point(|&threshold| self.steps >= threshold) as u8;
            }
            RocketStage::Launching if time >= self.next_frame => {
                self.next_frame = time + FRAME_INTERVAL_MS;
                self.frame += 1;
                if self.frame >= LAUNCH_FRAME {
                    self.stage = RocketStage::Launch;
                }
            }
            RocketStage::Done => self.frame = FINISHED_FRAME,
            _ => {}
        }
    }
}
impl City {
    /// Launches a completed rocket from its pad's origin.
    pub fn launch_rocket(&mut self, p: Point) -> bool {
        let Some(id) = self.buildings.at(p) else {
            return false;
        };
        if self.buildings.get(id).unwrap().0 != p {
            return false;
        }
        let Operation::Rocket(r) = &mut self.buildings.get_mut(id).unwrap().operation else {
            return false;
        };
        if !r.ready_to_launch() {
            return false;
        }
        r.stage = RocketStage::Launching;
        true
    }
    pub(super) fn update_rocket(&mut self, id: BuildingId, p: Point) -> io::Result<()> {
        use Commodity::*;
        let b = self.buildings.get_mut(id).unwrap();
        let Operation::Rocket(r) = &mut b.operation else {
            unreachable!()
        };
        let paid = r.stage != RocketStage::Done
            && finance::expense(
                &mut self.balance,
                &mut self.stats.expenses.rockets,
                &mut self.events,
                RUNNING_COST,
            );
        let launch = paid && r.stage == RocketStage::Launch;
        if paid && r.stage == RocketStage::Building {
            let steps = (STEPS - r.steps)
                .min(b.stock[Labor as usize] / LABOR)
                .min(b.stock[Goods as usize] / GOODS)
                .min(b.stock[Steel as usize] / STEEL)
                .min(
                    (BuildingKind::RocketPad.metadata().rule(Waste).max_load
                        - b.stock[Waste as usize])
                        / WASTE,
                );
            for (c, r) in [
                (Labor, -LABOR),
                (Goods, -GOODS),
                (Steel, -STEEL),
                (Waste, WASTE),
            ] {
                b.production.change(&mut b.stock, c, r * steps);
            }
            r.steps += steps;
            if steps != 0 {
                r.working_days += 1;
            }
            if r.steps >= STEPS {
                r.stage = RocketStage::Awaiting;
                self.events.push_back(CityEvent::RocketReady(p));
            }
        }
        if self.day % DAYS_PER_MONTH == DAYS_PER_MONTH - 1 {
            b.production.close_month();
            r.busy = std::mem::take(&mut r.working_days);
        }
        if launch {
            r.stage = RocketStage::Done;
            self.rockets_launched += 1;
            let chance = f64::from(b.installed_tech.min(self.tech_level).min(MAX_TECH_LEVEL))
                / f64::from(MAX_TECH_LEVEL)
                * (f64::from(self.rockets_launched) * SUCCESS_GAIN_PER_LAUNCH
                    + INITIAL_SUCCESS_FACTOR)
                    .min(1.0);
            if self.decisions.bernoulli("rocket.success", chance)? {
                self.rockets_successful += 1;
                if self.rockets_successful > QUALIFYING_LAUNCHES {
                    self.evacuate_rocket(p)?;
                } else {
                    self.events.push_back(CityEvent::RocketSuccess(p));
                }
            } else {
                self.events.push_back(CityEvent::RocketFailed(p));
                self.rockets_successful = 0;
                let x = p.x
                    + self
                        .decisions
                        .normal_offset("rocket.center-x", CRASH_DEVIATION, p.x)?;
                let y = p.y
                    + self
                        .decisions
                        .normal_offset("rocket.center-y", CRASH_DEVIATION, p.y)?;
                for _ in 0..CRASH_FIRE_ATTEMPTS {
                    let fire = Point {
                        x: x + self
                            .decisions
                            .normal_offset("rocket.fire-x", FIRE_DEVIATION, x)?,
                        y: y + self
                            .decisions
                            .normal_offset("rocket.fire-y", FIRE_DEVIATION, y)?,
                    };
                    if let Some(b) = self.buildings.at(fire) {
                        let origin = self.buildings.get(b).unwrap().0;
                        self.torch_building(b, origin)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn evacuate_rocket(&mut self, p: Point) -> io::Result<()> {
        let pool = self.people_pool.min(PASSENGER_CAPACITY);
        self.people_pool -= pool;
        self.stats.population.evacuated_t += pool;
        let mut remaining = PASSENGER_CAPACITY - pool;
        while remaining > 0 {
            let mut housed = 0;
            for id in self.visit_order(VisitPhase::Evacuation, self.day as u64)? {
                if let Some(b) = self.buildings.get_mut(id)
                    && let Operation::Civic(c) = &mut b.operation
                    && let Some(left) = c.evacuate_one()
                {
                    housed += left;
                    self.stats.population.evacuated_t += 1;
                    remaining -= 1;
                    if remaining == 0 {
                        break;
                    }
                }
            }
            if housed == 0 {
                break;
            }
        }
        let vacant = self.people_pool == 0
            && !self
                .buildings
                .iter()
                .any(|(_, b)| matches!(&b.operation,Operation::Civic(c) if c.occupants()>0));
        if vacant && !self.game_end {
            self.game_end = true;
            self.events.push_back(CityEvent::RocketWon(p));
        } else {
            self.events.push_back(CityEvent::RocketEvacuated(p));
        }
        Ok(())
    }
}
