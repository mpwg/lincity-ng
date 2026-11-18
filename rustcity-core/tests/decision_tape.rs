use rustcity_core::Point;
use rustcity_core::decisions::{Decisions, VisitPhase::*};
use std::io::{self, Cursor, Write};
use std::sync::{Arc, Mutex};

const SMALL: &str = include_str!("../../lincity-core/tests/replay/decisions-v1.txt");

fn replay(tape: &str) -> io::Result<Decisions> {
    Decisions::replay(Cursor::new(tape.as_bytes().to_vec()))
}

fn run_small(tape: &str) -> io::Result<()> {
    let mut d = replay(tape)?;
    let smith = d.placed(Point { x: 12, y: 12 }, 27)?;
    let p = Point { x: 8, y: 8 };
    let mut other = None;
    for day in 1..=5 {
        match day {
            2 => other = Some(d.placed(p, 7)?),
            4 => other = None,
            5 => other = Some(d.placed(p, 46)?),
            _ => (),
        }
        let mut members = vec![smith];
        members.extend(other);
        d.visit(Animation, day * 1000, &mut members)?;
        if let Some(other) = other {
            assert_eq!(members, [other, smith]);
        }
        d.visit(Daily, day + 96, &mut members)?;
        if day == 2 {
            assert_eq!(members, [smith, other.unwrap()]);
        }
    }
    d.finish()
}

#[test]
fn versioned_smithy_tape_reorders_each_phase_and_tracks_replacement() {
    run_small(SMALL).unwrap();
}

#[test]
fn rejects_damaged_tapes_and_mismatched_placements_or_visits() {
    for (from, to) in [
        ("TRACE 1", "TRACE 7"),
        ("P 1 12 12 27", "P 1 12 13 27"),
        ("P 3 8 8 46", "P 2 8 8 46"),
        ("A 1000", "D 1000"),
        ("D 97", "D 96"),
        ("A 2000 2 2 1", "A 2000 2 2 2"),
        ("A 2000 2 2 1", "A 2000 3 2 1 3"),
        ("D 101 2 3 1", "D 101 2 2 1"),
        ("Z\n", ""),
        ("Z\n", "Z\nC 7\n"),
    ] {
        assert!(
            run_small(&SMALL.replace(from, to)).is_err(),
            "{from} -> {to}"
        );
    }
}

#[derive(Clone, Default)]
struct Buffer(Arc<Mutex<Vec<u8>>>);

impl Write for Buffer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn native_and_recorded_streams_are_instance_local_and_replay_exactly() {
    fn draw(d: &mut Decisions) -> io::Result<Vec<i32>> {
        let a = d.placed(Point { x: 3, y: 4 }, 7)?;
        let b = d.placed(Point { x: 8, y: 9 }, 7)?;
        let mut values = vec![];
        for day in 0..32 {
            let mut order = [a, b];
            d.visit(Daily, day, &mut order)?;
            d.visit(Evacuation, day, &mut order)?;
            values.push(i32::from(order[0] == a));
            values.push(d.c_modulo(31)?);
            values.push(d.binomial("river.lake-count", 10, 0.5)?);
            values.push(i32::from(d.bernoulli("rocket.success", 0.7)?));
            values.push(d.discrete("vehicle.heading", &[4., 0., 1.])?);
            values.push(d.uniform("river.start-x", 0, 99)?);
            values.push(d.normal_offset("rocket.fire-x", 10., 37)?);
        }
        Ok(values)
    }
    let writer = Buffer::default();
    let mut a = Decisions::record(714, writer.clone()).unwrap();
    let mut b = Decisions::native(714);
    let recorded = draw(&mut a).unwrap();
    a.finish().unwrap();
    assert_eq!(recorded, draw(&mut b).unwrap());
    let mut c = Decisions::replay(Cursor::new(writer.0.lock().unwrap().clone())).unwrap();
    assert_eq!(draw(&mut c).unwrap(), recorded);
    c.finish().unwrap();
}

#[test]
fn recorded_draws_validate_the_site_domain_and_coordinate_overflow() {
    fn samples(records: &str) -> Decisions {
        replay(&format!("LINCITY-SIMULATION-TRACE 1\n{records}\nZ\n")).unwrap()
    }
    assert_eq!(samples("C 2147483647").c_modulo(100).unwrap(), 47);
    assert!(samples("C -1").c_random().is_err());
    assert!(
        samples("S rocket.success 1")
            .bernoulli("rocket.success", 0.)
            .is_err()
    );
    assert!(
        samples("S river.start-y 1")
            .uniform("river.start-x", 0, 2)
            .is_err()
    );
    assert!(
        samples("S vehicle.heading 1")
            .discrete("vehicle.heading", &[1., 0.])
            .is_err()
    );
    assert!(
        samples("S rocket.fire-x 1")
            .normal_offset("rocket.fire-x", 10., i32::MAX)
            .is_err()
    );
    assert!(
        samples("S river.lake-count 9")
            .binomial("river.lake-count", 10, 1.)
            .is_err()
    );
}

#[test]
fn malformed_draw_and_terminator_report_the_reference_event_number() {
    let mut stream = replay("LINCITY-SIMULATION-TRACE 1\nC 2147483648\n").unwrap();
    assert_eq!(
        stream.c_random().unwrap_err().to_string(),
        "simulation trace event 1: random draw outside generator range"
    );
    let mut stream = replay("LINCITY-SIMULATION-TRACE 1\nC 4\nZ\nZ\n").unwrap();
    assert_eq!(stream.c_random().unwrap(), 4);
    assert_eq!(
        stream.finish().unwrap_err().to_string(),
        "simulation trace event 1: trailing data or read failure"
    );
}

#[test]
fn nonfinite_and_overflowing_weights_return_errors_without_sampling() {
    for weights in [
        &[f64::INFINITY, 1.0][..],
        &[f64::MAX, f64::MAX],
        &[f64::NAN],
    ] {
        let mut d = Decisions::native(714);
        assert!(d.discrete("invalid", weights).is_err());
    }
}
