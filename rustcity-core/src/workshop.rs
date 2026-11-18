//! Handicraft production and machine animation.
use crate::{BuildingKind, Commodity};

#[derive(Clone, Default)]
pub(crate) struct Workshop {
    pause: u8,
    working_days: i32,
    pub(crate) busy: i32,
    batches: i32,
    animation: bool,
    next_frame: u64,
    frame: u16,
}

impl Workshop {
    /// Returns the pollution released at the workshop's location.
    pub(crate) fn update(&mut self, kind: BuildingKind, day: i32, stock: &mut [i32; 10]) -> i32 {
        use Commodity::*;
        if kind == BuildingKind::Pottery {
            if day % 100 == 0 {
                self.busy = std::mem::take(&mut self.working_days);
            }
            if self.pause > 0 {
                self.pause -= 1;
                return 0;
            }
        }
        let (output, inputs) = match kind {
            BuildingKind::Blacksmith => (50, [(Coal, 1), (Steel, 1), (Labor, 35)]),
            BuildingKind::Pottery => (35, [(Coal, 2), (Ore, 11), (Labor, 35)]),
            BuildingKind::Mill => {
                let fuel = if i64::from(stock[Coal as usize]) * 60
                    > i64::from(stock[LowVoltage as usize])
                {
                    (Coal, 1)
                } else {
                    (LowVoltage, 60)
                };
                (75, [fuel, (Food, 37), (Labor, 35)])
            }
            _ => unreachable!("not a workshop"),
        };
        let capacity = kind.metadata().rule(Goods).max_load;
        let mut pollution = 0;
        if stock[Goods as usize] <= capacity - output
            && inputs.iter().all(|(c, n)| stock[*c as usize] >= *n)
        {
            for (c, n) in inputs {
                stock[c as usize] -= n;
            }
            stock[Goods as usize] += output;
            self.batches += 1;
            match kind {
                BuildingKind::Blacksmith if self.batches == 100 => {
                    self.animation = true;
                    pollution = 1;
                    self.batches = 0;
                }
                BuildingKind::Pottery => {
                    self.animation = true;
                    pollution = i32::from(self.working_days % 10 == 0);
                    self.batches = 0;
                }
                BuildingKind::Mill => {
                    self.animation = true;
                    if self.batches == 7 {
                        pollution = 1;
                        self.batches = 0;
                    }
                }
                _ => {}
            }
            self.working_days += 1;
        } else if kind != BuildingKind::Mill {
            self.animation = false;
            if kind == BuildingKind::Pottery {
                self.pause = 25;
            }
        }
        if kind != BuildingKind::Pottery && day % 100 == 99 {
            self.busy = std::mem::take(&mut self.working_days);
        }
        pollution
    }

    pub(crate) fn animate(&mut self, kind: BuildingKind, time: u64) -> u16 {
        if kind == BuildingKind::Blacksmith && !self.animation {
            self.frame = 0;
            self.next_frame = 0;
        } else if self.animation && time >= self.next_frame {
            let (delay, frames) = match kind {
                BuildingKind::Blacksmith => (150, 7),
                BuildingKind::Mill => (250, 7),
                BuildingKind::Pottery => (250, 11),
                _ => unreachable!("not a workshop"),
            };
            self.next_frame = time.saturating_add(delay);
            self.frame += 1;
            if kind == BuildingKind::Mill {
                self.animation = false;
            }
            if self.frame >= frames {
                self.frame = 1;
                self.animation = false;
            }
        } else if kind != BuildingKind::Blacksmith && self.busy == 0 {
            self.frame = 0;
        }
        self.frame
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Commodity::*;
    #[test]
    fn blacksmith_resumes_immediately_and_closes_its_books_after_production() {
        let mut shop = Workshop::default();
        let mut stock = [0; 10];
        shop.update(BuildingKind::Blacksmith, 98, &mut stock);
        stock[Labor as usize] = 35;
        stock[Coal as usize] = 1;
        stock[Steel as usize] = 1;
        shop.update(BuildingKind::Blacksmith, 99, &mut stock);
        assert_eq!(stock[Goods as usize], 50);
        assert_eq!(shop.busy, 1);
        assert_eq!(stock[Labor as usize], 0);
    }
    #[test]
    fn pottery_reopens_after_25_idle_ticks_and_pollutes_on_first_batch() {
        let mut shop = Workshop::default();
        let mut stock = [0; 10];
        shop.update(BuildingKind::Pottery, 0, &mut stock);
        stock[Labor as usize] = 70;
        stock[Coal as usize] = 4;
        stock[Ore as usize] = 22;
        for day in 1..=25 {
            assert_eq!(shop.update(BuildingKind::Pottery, day, &mut stock), 0);
        }
        assert_eq!(stock[Goods as usize], 0);
        assert_eq!(shop.update(BuildingKind::Pottery, 26, &mut stock), 1);
        assert_eq!(shop.animate(BuildingKind::Pottery, 100), 1);
        assert_eq!(stock[Goods as usize], 35);
    }
    #[test]
    fn mill_prefers_electricity_on_equal_fill_then_uses_coal() {
        let mut shop = Workshop::default();
        let mut stock = [0; 10];
        stock[Labor as usize] = 70;
        stock[Coal as usize] = 1;
        stock[LowVoltage as usize] = 60;
        stock[Food as usize] = 74;
        shop.update(BuildingKind::Mill, 10, &mut stock);
        assert_eq!((stock[Coal as usize], stock[LowVoltage as usize]), (1, 0));
        shop.update(BuildingKind::Mill, 11, &mut stock);
        assert_eq!(
            (
                stock[Goods as usize],
                stock[Food as usize],
                stock[Coal as usize]
            ),
            (150, 0, 0)
        );
    }
}
