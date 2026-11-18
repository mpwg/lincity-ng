//! Graphics-resource identifiers, frame indices and relative offsets used by a renderer.
use super::civic::Civic;
use super::industry::IndustryOperation as Industry;
use super::*;
use crate::Commodity;

/// One visible layer, drawn at a pixel offset from its building's anchor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpriteLayer {
    pub resource: &'static str,
    pub frame: i32,
    pub offset: (i32, i32),
}

/// Ordered layers of a building, including smoke, fire and other foreground sprites.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildingVisual {
    pub origin: Point,
    pub kind: BuildingKind,
    pub transparent: bool,
    pub layers: Vec<SpriteLayer>,
}

impl Building {
    pub(super) fn update_sprite(&mut self) {
        let (resource, frame) = match &self.operation {
            Operation::Station(_) => {
                let low = self.stock[Commodity::LowVoltage as usize];
                let capacity = self.kind.metadata().rule(Commodity::LowVoltage).max_load;
                (
                    if low > capacity / 2 {
                        "SubstationOn"
                    } else if low > capacity / 10 {
                        "SubstationOff"
                    } else {
                        "Substation"
                    },
                    0,
                )
            }
            Operation::Market(m) => (
                match m.fill {
                    ..10 => "MarketEmpty",
                    10..20 => "MarketLow",
                    20..50 => "MarketMed",
                    _ => "MarketFull",
                },
                0,
            ),
            Operation::Industry(Industry::Coal(_)) => (
                crate::production::CoalMine::resource(self.stock[Commodity::Coal as usize]),
                0,
            ),
            Operation::Industry(Industry::Tip(t)) => (self.sprite_resource, t.frame()),
            Operation::Civic(Civic::Fire(f)) if f.smoking != 0 => ("FireWasteLand", f.frame),
            Operation::Civic(Civic::Monument(m)) if m.completed => ("Monument", 0),
            _ => return,
        };
        self.sprite_resource = resource;
        self.sprite_frame = frame;
    }
}

impl City {
    /// Returns a building's current visible layers from any square of its footprint.
    pub fn building_visual(&self, point: Point) -> Option<BuildingVisual> {
        let (origin, b) = self.buildings.get(self.buildings.at(point)?)?;
        let mut overlay = Vec::new();
        let mut layer = |resource, frame, offset| {
            if frame >= 0 {
                overlay.push(SpriteLayer {
                    resource,
                    frame,
                    offset,
                });
            }
        };
        let mut resource = b.sprite_resource;
        let mut frame = b.sprite_frame;
        let mut transparent = b.transparent;
        match &b.operation {
            Operation::Coal(s) => {
                resource = s.resource;
                for (f, offset) in s.smoke.iter().zip(crate::power::CoalPlant::SMOKE_OFFSETS) {
                    layer("BlackSmoke", *f, offset);
                }
            }
            Operation::Wind(s) => {
                resource = s.resource;
                frame = s.frame;
            }
            Operation::Line(s) => frame = s.frame,
            Operation::Transport(t) => {
                frame = t.frame;
                transparent = t.transparent;
                if let Some(f) = t.fire.frame {
                    layer("Fire", f, (0, 0));
                }
            }
            Operation::Market(m) => layer("Fire", m.frame, (0, 0)),
            Operation::Industry(Industry::Light(s)) => {
                resource = s.resource;
                for (f, offset) in s
                    .smoke
                    .iter()
                    .zip(crate::production::LightIndustry::SMOKE_OFFSETS)
                {
                    layer("GraySmoke", *f, offset);
                }
            }
            Operation::Industry(Industry::Ore(s)) => frame = s.frame,
            Operation::Industry(Industry::Heavy(s)) => {
                resource = s.resource;
                frame = s.frame;
            }
            Operation::Farm(f) => frame = i32::from(f.frame),
            Operation::Workshop(w) => frame = i32::from(w.frame),
            Operation::Rocket(r) => frame = i32::from(r.frame),
            Operation::Commune(c) => frame = i32::from(c.frame),
            Operation::Civic(Civic::Service(s)) => frame = s.frame,
            Operation::Civic(Civic::Education(s)) => {
                frame = s.frame;
                if frame != 0 && b.kind == BuildingKind::School {
                    layer("ChildOnSwing", s.swing, (0, 0));
                }
            }
            Operation::Civic(Civic::Monument(s)) => frame = s.frame,
            Operation::Civic(Civic::Fire(s)) => frame = s.frame,
            Operation::Civic(Civic::Shanty(s)) => {
                if let Some(f) = s.frame {
                    layer("Fire", f, (0, 0));
                }
            }
            _ => {}
        }
        let mut layers = vec![SpriteLayer {
            resource,
            frame,
            offset: (0, 0),
        }];
        layers.extend(overlay);
        Some(BuildingVisual {
            kind: b.kind,
            origin,
            transparent,
            layers,
        })
    }
}
