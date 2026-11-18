//! LinCity's gzip/plain XML city files and the migration from 2130 to 2140.
use flate2::{Compression, read::MultiGzDecoder, write::GzEncoder};
use quick_xml::Writer;
use std::io::{self, BufReader, Read, Write};

const LEGACY_VERSION: u32 = 2130;
const CURRENT_VERSION: u32 = 2140;
const LEGACY_DEFAULT_SIDE: i32 = 100;
const GZIP_MAGIC: [u8; 2] = [0x1f, 0x8b];

pub(crate) mod accounts;
mod xml;
pub use xml::Element;
mod migration;
#[cfg(test)]
mod tests;

pub(crate) fn invalid(message: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.to_string())
}

pub(crate) fn integer(s: &str) -> io::Result<i64> {
    let digits = s.strip_prefix(['+', '-']).unwrap_or(s);
    if let Some(hex) = digits
        .strip_prefix("0x")
        .or_else(|| digits.strip_prefix("0X"))
    {
        let magnitude = u64::from_str_radix(hex, 16).map_err(invalid)?;
        let signed = if s.starts_with('-') {
            -i128::from(magnitude)
        } else {
            i128::from(magnitude)
        };
        return i64::try_from(signed).map_err(invalid);
    }
    s.parse().map_err(invalid)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SaveGame {
    root: Element,
    version: u32,
}

impl SaveGame {
    pub fn new(side: i32) -> Self {
        let mut root = Element::new("lc-game");
        root.set_attr("ldsv-version", CURRENT_VERSION.to_string());
        root.push(Element::new("globals"));
        root.section("map").set_attr("size", side.to_string());
        Self {
            root,
            version: CURRENT_VERSION,
        }
    }
    pub fn version(&self) -> u32 {
        self.version
    }
    pub fn globals(&self) -> &Element {
        self.root.child("globals").expect("validated globals")
    }
    pub fn map(&self) -> &Element {
        self.root.child("map").expect("validated map")
    }
    pub fn globals_mut(&mut self) -> &mut Element {
        self.root.child_mut("globals").expect("validated globals")
    }
    pub fn map_mut(&mut self) -> &mut Element {
        self.root.child_mut("map").expect("validated map")
    }

    pub fn read(mut reader: impl Read) -> io::Result<Self> {
        let mut prefix = [0; 2];
        reader.read_exact(&mut prefix)?;
        let bytes = prefix.as_slice().chain(reader);
        let root = if prefix == GZIP_MAGIC {
            Element::read(BufReader::new(MultiGzDecoder::new(bytes)))?
        } else {
            Element::read(BufReader::new(bytes))?
        };
        if root.name == "SaveGame" {
            return Err(invalid(
                "Detected a load/save format that is too old. See the readme for how to update the load/save format.",
            ));
        }
        if root.name != "lc-game" {
            return Err(invalid(format!(
                "invalid save file: expected <lc-game>, found <{}>",
                root.name
            )));
        }
        let version = root
            .attr("ldsv-version")
            .filter(|v| !v.is_empty())
            .ok_or_else(|| invalid("failed to parse load/save version: missing attribute ldsv-version in <lc-game>"))?
            .parse()
            .map_err(invalid)?;
        if version < LEGACY_VERSION {
            return Err(invalid("load/save version too old"));
        }
        if version > CURRENT_VERSION {
            return Err(invalid("load/save version too new"));
        }
        let globals = root
            .child("globals")
            .ok_or_else(|| invalid("missing globals"))?;
        let map = root.child("map").ok_or_else(|| invalid("missing map"))?;
        let size = if version == LEGACY_VERSION {
            globals
                .child("world_side_len")
                .map(|e| integer(&e.text()))
                .transpose()?
                .unwrap_or(i64::from(LEGACY_DEFAULT_SIDE))
        } else {
            integer(
                map.attr("size")
                    .ok_or_else(|| invalid("missing map size"))?,
            )?
        };
        if !(i64::from(crate::rules::MIN_MAP_SIDE)..=i64::from(crate::rules::MAX_MAP_SIDE))
            .contains(&size)
        {
            return Err(invalid("invalid map size"));
        }
        for e in map
            .elements()
            .filter(|e| matches!(e.name.as_str(), "MapTile" | "Construction"))
        {
            let attr = |name| {
                integer(
                    e.attr(name)
                        .ok_or_else(|| invalid(format!("missing {name} on {}", e.name)))?,
                )
            };
            attr("group")?;
            for name in ["map-x", "map-y"] {
                if !(0..size).contains(&attr(name)?) {
                    return Err(invalid(format!("{} outside map", e.name)));
                }
            }
        }
        Ok(Self { root, version })
    }

    pub fn write_xml(&self, output: impl Write) -> io::Result<()> {
        self.root.write(&mut Writer::new(output))
    }

    pub fn write_gzip(&self, output: impl Write) -> io::Result<()> {
        let mut encoder = GzEncoder::new(output, Compression::default());
        self.write_xml(&mut encoder)?;
        encoder.finish()?;
        Ok(())
    }

    pub fn upgrade(&mut self) -> io::Result<()> {
        if self.version() == LEGACY_VERSION {
            migration::upgrade(self)?;
        }
        self.root
            .set_attr("ldsv-version", CURRENT_VERSION.to_string());
        self.version = CURRENT_VERSION;
        Ok(())
    }
}
