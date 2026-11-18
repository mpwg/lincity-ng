//! City-local random choices and streaming LinCity decision tapes (version 1).
//!
//! Attach a recorder or reader before loading or generating a city. Keep each
//! returned placement ID with its building until demolition. Pass the live IDs
//! to each visit, then update the buildings in the returned slice order.
//! A replay or I/O error invalidates the simulation run.

use crate::Point;
use rand::distr::{Distribution, weighted::WeightedIndex};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{RngExt, SeedableRng};
use rand_distr::{Binomial, Normal};
use std::collections::HashSet;
use std::fmt;
use std::io::{self, BufRead, BufWriter, Write};
use std::str::FromStr;

const HEADER: &str = "LINCITY-SIMULATION-TRACE 1";

/// Identity in one run, assigned in placement order starting at one.
#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct PlacementId(u64);

/// Traversals whose order can affect the simulation.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum VisitPhase {
    Animation,
    Daily,
    Evacuation,
}

impl VisitPhase {
    fn tag(self) -> &'static str {
        match self {
            Self::Animation => "A",
            Self::Daily => "D",
            Self::Evacuation => "E",
        }
    }
}

enum Mode {
    Native,
    Record(BufWriter<Box<dyn Write + Send>>),
    Replay(Box<dyn BufRead + Send>),
}

/// Independent decision streams for one city. Only tape outcomes are portable
/// between Rust and the C++ implementation.
pub struct Decisions {
    mode: Mode,
    rng: StdRng,
    c_rng: StdRng,
    last_id: u64,
    event: u64,
    line: String,
}

fn invalid(event: u64, detail: &str) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("simulation trace event {event}: {detail}"),
    )
}

struct Fields<'a> {
    tokens: std::str::SplitAsciiWhitespace<'a>,
    event: u64,
}

impl Fields<'_> {
    fn take<T: FromStr>(&mut self) -> io::Result<T> {
        self.tokens
            .next()
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| invalid(self.event, "missing or invalid event field"))
    }

    fn check(&self, valid: bool, detail: &str) -> io::Result<()> {
        if valid {
            Ok(())
        } else {
            Err(invalid(self.event, detail))
        }
    }

    fn end(mut self) -> io::Result<()> {
        let ended = self.tokens.next().is_none();
        self.check(ended, "trailing event fields")
    }
}

impl Decisions {
    /// Seeds both native streams. C-style draws and distribution draws advance independently.
    pub fn native(seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let c_rng = StdRng::from_rng(&mut rng);
        Self {
            mode: Mode::Native,
            rng,
            c_rng,
            last_id: 0,
            event: 0,
            line: String::new(),
        }
    }

    /// Records native outcomes. Call `finish` to write the terminator and flush the tape.
    pub fn record(seed: u64, output: impl Write + Send + 'static) -> io::Result<Self> {
        let mut output = BufWriter::new(Box::new(output) as Box<dyn Write + Send>);
        writeln!(output, "{HEADER}")?;
        Ok(Self {
            mode: Mode::Record(output),
            ..Self::native(seed)
        })
    }

    /// Reads one event at a time, retaining at most the largest event line.
    pub fn replay(mut input: impl BufRead + Send + 'static) -> io::Result<Self> {
        let mut line = String::new();
        input.read_line(&mut line)?;
        if line.trim_end_matches(['\n', '\r']) != HEADER {
            return Err(invalid(0, "invalid header"));
        }
        Ok(Self {
            mode: Mode::Replay(Box::new(input)),
            ..Self::native(0)
        })
    }

    fn read_event(&mut self, kind: &str) -> io::Result<Option<Fields<'_>>> {
        self.event += 1;
        let Mode::Replay(input) = &mut self.mode else {
            return Ok(None);
        };
        self.line.clear();
        input.read_line(&mut self.line)?;
        let mut tokens = self.line.split_ascii_whitespace();
        if tokens.next() != Some(kind) {
            return Err(invalid(self.event, "unexpected event kind or end of input"));
        }
        Ok(Some(Fields {
            tokens,
            event: self.event,
        }))
    }

    fn write_event(&mut self, args: fmt::Arguments<'_>) -> io::Result<()> {
        if let Mode::Record(output) = &mut self.mode {
            writeln!(output, "{args}")?;
        }
        Ok(())
    }

    /// Registers a new placement, including a replacement at an earlier coordinate.
    pub fn placed(&mut self, point: Point, group: u16) -> io::Result<PlacementId> {
        let id = self.last_id + 1;
        if let Some(mut fields) = self.read_event("P")? {
            let actual = (
                fields.take()?,
                fields.take()?,
                fields.take()?,
                fields.take()?,
            );
            fields.check(
                actual == (id, point.x, point.y, group),
                "construction placement differs",
            )?;
            fields.end()?;
        } else {
            self.write_event(format_args!("P {id} {} {} {group}", point.x, point.y))?;
        }
        self.last_id = id;
        Ok(PlacementId(id))
    }

    /// Orders all live participants. Daily native visits shuffle the input order.
    /// Animation and evacuation retain the caller's native order.
    pub fn visit(
        &mut self,
        phase: VisitPhase,
        time: u64,
        order: &mut [PlacementId],
    ) -> io::Result<()> {
        if let Some(mut fields) = self.read_event(phase.tag())? {
            let timing = (fields.take()?, fields.take()?);
            fields.check(
                timing == (time, order.len()),
                "construction visit time or count differs",
            )?;
            let mut available: HashSet<_> = order.iter().copied().collect();
            for slot in order {
                let id = PlacementId(fields.take()?);
                fields.check(
                    available.remove(&id),
                    "duplicate or unknown construction visit",
                )?;
                *slot = id;
            }
            return fields.end();
        }
        if phase == VisitPhase::Daily {
            order.shuffle(&mut self.rng);
        }
        if let Mode::Record(output) = &mut self.mode {
            write!(output, "{} {time} {}", phase.tag(), order.len())?;
            for id in order {
                write!(output, " {}", id.0)?;
            }
            writeln!(output)?;
        }
        Ok(())
    }

    /// Draws the nonnegative pre-modulo value in the Linux C generator's range.
    pub fn c_random(&mut self) -> io::Result<i32> {
        if let Some(mut fields) = self.read_event("C")? {
            let value = fields.take::<i32>()?;
            fields.check(value >= 0, "random draw outside generator range")?;
            fields.end()?;
            Ok(value)
        } else {
            let value = self.c_rng.random_range(0..=i32::MAX);
            self.write_event(format_args!("C {value}"))?;
            Ok(value)
        }
    }

    /// Applies the simulation's C remainder operation to a full recorded draw.
    pub fn c_modulo(&mut self, modulus: i32) -> io::Result<i32> {
        if modulus <= 0 {
            return Err(invalid(self.event, "non-positive random modulus"));
        }
        Ok(self.c_random()? % modulus)
    }

    fn sample(
        &mut self,
        site: &str,
        low: i32,
        high: i32,
        draw: impl FnOnce(&mut StdRng) -> i32,
    ) -> io::Result<i32> {
        if site.is_empty() || site.bytes().any(|c| c.is_ascii_whitespace()) || low > high {
            return Err(invalid(self.event, "invalid distribution site or range"));
        }
        if let Some(mut fields) = self.read_event("S")? {
            let matching_site = fields.tokens.next() == Some(site);
            fields.check(matching_site, "unexpected distribution site")?;
            let value = fields.take()?;
            fields.check(
                (low..=high).contains(&value),
                "distribution outcome out of range",
            )?;
            fields.end()?;
            Ok(value)
        } else {
            let value = draw(&mut self.rng);
            if !(low..=high).contains(&value) {
                return Err(invalid(self.event, "distribution outcome out of range"));
            }
            self.write_event(format_args!("S {site} {value}"))?;
            Ok(value)
        }
    }

    pub fn uniform(&mut self, site: &str, low: i32, high: i32) -> io::Result<i32> {
        self.sample(site, low, high, |rng| rng.random_range(low..=high))
    }

    pub fn bernoulli(&mut self, site: &str, p: f64) -> io::Result<bool> {
        if !(0.0..=1.0).contains(&p) {
            return Err(invalid(self.event, "invalid probability"));
        }
        Ok(
            self.sample(site, i32::from(p == 1.0), i32::from(p != 0.0), |rng| {
                i32::from(rng.random_bool(p))
            })? != 0,
        )
    }

    pub fn binomial(&mut self, site: &str, trials: i32, p: f64) -> io::Result<i32> {
        if !(0.0..=1.0).contains(&p) || trials < 0 {
            return Err(invalid(self.event, "invalid binomial parameters"));
        }
        let min = if p == 1.0 { trials } else { 0 };
        let max = if p == 0.0 { 0 } else { trials };
        let distribution = Binomial::new(trials as u64, p)
            .map_err(|_| invalid(self.event, "invalid binomial parameters"))?;
        self.sample(site, min, max, |rng| distribution.sample(rng) as i32)
    }

    pub fn discrete(&mut self, site: &str, weights: &[f64]) -> io::Result<i32> {
        let max =
            i32::try_from(weights.len()).map_err(|_| invalid(self.event, "too many weights"))? - 1;
        let distribution =
            WeightedIndex::new(weights).map_err(|_| invalid(self.event, "invalid weights"))?;
        if !weights.iter().sum::<f64>().is_finite() {
            return Err(invalid(self.event, "invalid weights"));
        }
        let choice = self.sample(site, 0, max, |rng| distribution.sample(rng) as i32)?;
        if weights[choice as usize] == 0.0 {
            return Err(invalid(self.event, "unavailable distribution outcome"));
        }
        Ok(choice)
    }

    /// Samples a normal offset, truncating toward zero before adding it to the map coordinate.
    pub fn normal_offset(&mut self, site: &str, deviation: f64, origin: i32) -> io::Result<i32> {
        if !deviation.is_finite() || deviation <= 0.0 {
            return Err(invalid(self.event, "invalid normal deviation"));
        }
        let distribution = Normal::new(0.0, deviation)
            .map_err(|_| invalid(self.event, "invalid normal deviation"))?;
        self.sample(
            site,
            i32::MIN.saturating_sub(origin),
            i32::MAX.saturating_sub(origin),
            |rng| distribution.sample(rng) as i32,
        )
    }

    /// Finishes a run and verifies that the tape contains no further events.
    pub fn finish(mut self) -> io::Result<()> {
        if let Some(fields) = self.read_event("Z")? {
            fields.end()?;
        }
        match self.mode {
            Mode::Replay(mut reader) => loop {
                let remaining = reader.fill_buf()?;
                if remaining.is_empty() {
                    return Ok(());
                }
                if !remaining.iter().all(u8::is_ascii_whitespace) {
                    return Err(invalid(self.event, "trailing data"));
                }
                let count = remaining.len();
                reader.consume(count);
            },
            Mode::Record(mut writer) => {
                writeln!(writer, "Z")?;
                writer.flush()
            }
            Mode::Native => Ok(()),
        }
    }
}
