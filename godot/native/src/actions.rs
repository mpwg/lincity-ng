use std::borrow::Borrow;

use godot::prelude::*;
use rustcity_core::{
    Action, ActionError, ActionQuote, BuildDenial, BuildingKind, City, FLOOD_COST, MAX_TECH_LEVEL,
    Point,
};

use super::{localize_format, localized, point};

#[derive(Clone, Copy)]
pub(super) enum Warning {
    OutOfCredit,
    OutOfMoney,
}

impl Warning {
    fn from_error(error: &ActionError) -> Option<Self> {
        match error {
            ActionError::OutOfMoney => Some(Self::OutOfCredit),
            ActionError::BuildDenied(BuildDenial::OutOfMoney) => Some(Self::OutOfMoney),
            _ => None,
        }
    }

    pub(super) fn kind(self) -> &'static str {
        match self {
            Self::OutOfCredit => "out_of_credit",
            Self::OutOfMoney => "out_of_money",
        }
    }

    pub(super) fn message(self) -> &'static str {
        match self {
            Self::OutOfCredit => "Out of Credit",
            Self::OutOfMoney => "Out of Money",
        }
    }
}

struct Tool {
    action: Action,
    name: &'static str,
    size: u16,
    base_cost: i32,
    required_tech: i32,
}

impl Tool {
    fn from_id(id: i32) -> Option<Self> {
        let (action, name, base_cost) = match id {
            -1 => (Action::Query, "Inspect", 0),
            -2 => (Action::Bulldoze, "Bulldoze", 0),
            -3 => (Action::Evacuate, "Evacuate", 0),
            -4 => (Action::Flood, "Water", FLOOD_COST),
            1..=31 | 36..=41 | 50 => {
                let kind = BuildingKind::from_id(id as u16)?;
                let metadata = kind.metadata();
                return Some(Self {
                    action: Action::Build(kind),
                    name: metadata.name,
                    size: metadata.size,
                    base_cost: metadata.base_cost,
                    required_tech: metadata.required_tech,
                });
            }
            _ => return None,
        };
        Some(Self {
            action,
            name,
            size: 1,
            base_cost,
            required_tech: 0,
        })
    }
}

pub(super) fn tools() -> VarArray {
    [-1, -2, -3, -4]
        .into_iter()
        .chain(1..=50)
        .filter_map(|id| {
            let tool = Tool::from_id(id)?;
            Some(
                vdict! {
                    "id" => id,
                    "name" => &localized(tool.name),
                    "size" => tool.size,
                    "base_cost" => tool.base_cost,
                    "required_tech" => tool.required_tech,
                }
                .to_variant(),
            )
        })
        .collect()
}

fn action_message(error: &ActionError, group: Option<BuildingKind>, current_tech: i32) -> GString {
    use ActionError as E;
    let key = match error {
        E::OutsideMap(_) => "This space is outside the map.",
        E::SpaceOccupied(_) => "This space is occupied.",
        E::NothingToEvacuate(_) => "There is nothing to evacuate here.",
        E::CannotEvacuate(_) | E::ProtectedFire(_) => {
            let key = match error {
                E::CannotEvacuate(_) => "Cannot evacuate {}.",
                _ => "Cannot bulldoze this {}.",
            };
            let name = localized(group.map_or("", |kind| kind.metadata().name));
            return localize_format(key, &[&name.to_string()]);
        }
        E::OutOfMoney => Warning::OutOfCredit.message(),
        E::BuildDenied(BuildDenial::OutOfMoney) => Warning::OutOfMoney.message(),
        E::BuildDenied(BuildDenial::NotEnoughTech) => {
            let required = group.map_or(0, |kind| kind.metadata().required_tech);
            let percent = |value: i32| f64::from(value) * 100.0 / f64::from(MAX_TECH_LEVEL);
            return localize_format(
                "Technology required: {:.1f}%. Current level: {:.1f}%.",
                &[
                    &format!("{:.1}", percent(required)),
                    &format!("{:.1}", percent(current_tech)),
                ],
            );
        }
        E::BuildDenied(BuildDenial::NotEnoughStudents) => {
            "There are not enough students. Build more schools first."
        }
        E::DesertHere(_) => "This space is desert and has no water.",
        E::RequiresRiver => "A port requires a river along the east side.",
        E::NoOre => "There are no ore reserves left here.",
        E::NonemptyTip(_) => "You cannot bulldoze this land fill because it is full of waste.",
        E::IncompleteMonument(_) => {
            "You cannot bulldoze this monument because it is still under construction."
        }
        E::DecisionTrace(_) => "This action is not possible here.",
    };
    localized(key)
}

fn site_message(city: &City, action: Action, target: Point, error: &ActionError) -> GString {
    let group = match action {
        Action::Build(kind) => Some(city.building_kind_at_site(kind, target)),
        _ => city.building_at(target).map(|(kind, _)| kind),
    };
    action_message(error, group, city.report().tech_level)
}

pub(super) fn preview(city: Option<&City>, id: i32, position: Vector2i) -> VarDictionary {
    let tool = Tool::from_id(id);
    let mut result = vdict! {
        "allowed" => false,
        "cost" => 0,
        "size" => tool.as_ref().map_or(1, |tool| tool.size),
    };
    let (Some(city), Some(tool)) = (city, tool) else {
        return result;
    };
    let bounds = 0..city.side();
    if !bounds.contains(&position.x) || !bounds.contains(&position.y) {
        return result;
    }

    let target = point(position);
    result.set("cost", city.quote(tool.action, target).unwrap_or_default());
    match city.preview(tool.action, target) {
        Ok(_) => result.set("allowed", true),
        Err(error) => result.set("reason", &site_message(city, tool.action, target, &error)),
    }
    result
}

pub(super) fn apply(city: &mut City, id: i32, position: Vector2i) -> (GString, Option<Warning>) {
    let Some(tool) = Tool::from_id(id) else {
        return (localized("Unknown tool."), None);
    };
    let target = point(position);
    match city
        .preview(tool.action, target)
        .and_then(|_| city.apply(tool.action, target))
    {
        Ok(()) => (GString::new(), None),
        Err(error) => (
            site_message(city, tool.action, target, &error),
            Warning::from_error(&error),
        ),
    }
}

pub(super) fn preview_batch(city: Option<&City>, id: i32, positions: Array<Vector2i>) -> VarArray {
    batch(city, id, positions, City::preview_batch)
}

pub(super) fn apply_batch(
    city: Option<&mut City>,
    id: i32,
    positions: Array<Vector2i>,
) -> VarArray {
    batch(city, id, positions, City::apply_batch)
}

fn batch<C: Borrow<City>>(
    city: Option<C>,
    id: i32,
    positions: Array<Vector2i>,
    run: impl FnOnce(C, Action, &[Point]) -> Vec<ActionQuote>,
) -> VarArray {
    let reason = if city.is_some() {
        "Unknown tool."
    } else {
        "No city loaded."
    };
    let (Some(city), Some(tool)) = (city, Tool::from_id(id)) else {
        return positions
            .iter_shared()
            .map(|position| {
                vdict! {
                    "allowed" => false,
                    "cost" => 0,
                    "size" => 1,
                    "group" => 0,
                    "anchor" => position,
                    "reason" => &localized(reason),
                }
                .to_variant()
            })
            .collect();
    };

    let current_tech = city.borrow().report().tech_level;
    let targets: Vec<_> = positions.iter_shared().map(point).collect();
    run(city, tool.action, &targets)
        .into_iter()
        .map(|quote| {
            let mut entry = vdict! {
                "allowed" => quote.status.is_ok(),
                "cost" => quote.cost,
                "size" => quote.size,
                "group" => quote.group,
                "anchor" => Vector2i::new(quote.anchor.x, quote.anchor.y),
            };
            if let Err(error) = quote.status {
                let group = BuildingKind::from_id(quote.group);
                entry.set("reason", &action_message(&error, group, current_tech));
                if let Some(warning) = Warning::from_error(&error) {
                    entry.set("warning", warning.kind());
                }
            }
            entry.to_variant()
        })
        .collect()
}

pub(super) fn survey_coal(city: &mut City) -> (GString, Option<Warning>) {
    match city.survey_coal() {
        Ok(()) => (GString::new(), None),
        Err(error) => (action_message(&error, None, 0), Warning::from_error(&error)),
    }
}
