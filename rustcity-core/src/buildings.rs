//! Building ownership and footprint lookup within one city.
use crate::Point;
use slotmap::{SlotMap, new_key_type};

new_key_type! { pub(crate) struct BuildingId; }

struct Placed<T> {
    origin: Point,
    size: i32,
    value: T,
}

pub(crate) struct Buildings<T> {
    objects: SlotMap<BuildingId, Placed<T>>,
    cells: Vec<Option<BuildingId>>,
    side: i32,
}

impl<T> Buildings<T> {
    pub(crate) fn new(side: i32) -> Self {
        Self {
            objects: SlotMap::with_key(),
            cells: vec![None; (side * side) as usize],
            side,
        }
    }
    pub(crate) fn place(&mut self, origin: Point, size: i32, value: T) -> Option<BuildingId> {
        if size < 1
            || origin.x < 1
            || origin.y < 1
            || origin.x >= self.side - size
            || origin.y >= self.side - size
        {
            return None;
        }
        if self
            .rows(origin, size)
            .any(|row| self.cells[row].iter().any(Option::is_some))
        {
            return None;
        }
        let id = self.objects.insert(Placed {
            origin,
            size,
            value,
        });
        for row in self.rows(origin, size) {
            self.cells[row].fill(Some(id));
        }
        Some(id)
    }

    fn rows(
        &self,
        origin: Point,
        size: i32,
    ) -> impl Iterator<Item = std::ops::Range<usize>> + use<T> {
        let side = self.side;
        (origin.y..origin.y + size).map(move |y| {
            let first = (y * side + origin.x) as usize;
            first..first + size as usize
        })
    }

    pub(crate) fn at(&self, p: Point) -> Option<BuildingId> {
        if !(0..self.side).contains(&p.x) || !(0..self.side).contains(&p.y) {
            return None;
        }
        self.cells[(p.y * self.side + p.x) as usize]
    }

    pub(crate) fn get(&self, id: BuildingId) -> Option<(Point, &T)> {
        self.objects
            .get(id)
            .map(|placed| (placed.origin, &placed.value))
    }

    pub(crate) fn get_mut(&mut self, id: BuildingId) -> Option<&mut T> {
        self.objects.get_mut(id).map(|placed| &mut placed.value)
    }

    pub(crate) fn remove(&mut self, id: BuildingId) -> Option<T> {
        let building = self.objects.remove(id)?;
        for row in self.rows(building.origin, building.size) {
            self.cells[row].fill(None);
        }
        Some(building.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rebuild_retires_the_previous_id_and_clears_every_covered_cell() {
        let mut buildings = Buildings::new(8);
        let origin = Point { x: 2, y: 3 };
        let corner = Point { x: 3, y: 4 };
        let old = buildings.place(origin, 2, 17).expect("empty footprint");
        assert_eq!(buildings.at(corner), Some(old));
        assert!(buildings.place(corner, 2, 42).is_none());
        assert!(buildings.at(Point { x: 4, y: 5 }).is_none());
        assert_eq!(buildings.remove(old), Some(17));
        assert_eq!(buildings.at(corner), None);
        let new = buildings.place(origin, 1, 39).unwrap();
        assert_ne!(old, new);
        assert_eq!(buildings.remove(old), None);
        assert_eq!(buildings.at(origin), Some(new));
    }
    #[test]
    fn every_cell_of_the_footprint_must_be_in_the_interior() {
        let mut buildings = Buildings::new(5);
        for (x, y, size) in [
            (0, 1, 1),
            (3, 3, 2),
            (1, 1, 0),
            (i32::MAX, 1, 1),
            (1, 1, i32::MAX),
        ] {
            assert_eq!(buildings.place(Point { x, y }, size, ()), None);
        }
        assert!(buildings.place(Point { x: 1, y: 1 }, 3, ()).is_some());
    }
}
