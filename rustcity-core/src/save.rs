//! LinCity's gzip/plain XML city files and the migration from 2130 to 2140.
use flate2::{Compression, read::MultiGzDecoder, write::GzEncoder};
use quick_xml::Writer;
use std::io::{self, BufReader, Read, Write};

mod xml;
pub use xml::Element;
mod migration;
#[cfg(test)]
mod tests;

fn invalid(message: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.to_string())
}

fn integer(s: &str) -> io::Result<i64> {
    let (sign, digits) = s.strip_prefix('-').map_or((1, s), |d| (-1, d));
    let value = if let Some(hex) = digits
        .strip_prefix("0x")
        .or_else(|| digits.strip_prefix("0X"))
    {
        i64::from_str_radix(hex, 16)
    } else {
        digits.parse()
    }
    .map_err(invalid)?;
    Ok(sign * value)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SaveGame {
    root: Element,
    version: u32,
}

impl SaveGame {
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
        let root = if prefix == [0x1f, 0x8b] {
            Element::read(BufReader::new(MultiGzDecoder::new(bytes)))?
        } else {
            Element::read(BufReader::new(bytes))?
        };
        if root.name != "lc-game" {
            return Err(invalid("expected lc-game root"));
        }
        let version = root
            .attr("ldsv-version")
            .ok_or_else(|| invalid("missing ldsv-version"))?
            .parse()
            .map_err(invalid)?;
        if !(2130..=2140).contains(&version) {
            return Err(invalid("unsupported save version"));
        }
        let globals = root
            .child("globals")
            .ok_or_else(|| invalid("missing globals"))?;
        let map = root.child("map").ok_or_else(|| invalid("missing map"))?;
        let size = if version == 2130 {
            globals
                .child("world_side_len")
                .map(|e| integer(&e.text()))
                .transpose()?
                .unwrap_or(100)
        } else {
            integer(
                map.attr("size")
                    .ok_or_else(|| invalid("missing map size"))?,
            )?
        };
        if !(3..=46_340).contains(&size) {
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
        if self.version == 2130 {
            migration::upgrade(self)?;
        }
        self.root.set_attr("ldsv-version", "2140");
        self.version = 2140;
        Ok(())
    }
}
