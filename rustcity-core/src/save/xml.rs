//! The save vocabulary includes module-specific and ordered history elements.
//! Retain those fields and their order when editing the surrounding document.
use std::borrow::Cow;
use std::io::{self, BufRead, Write};

use quick_xml::events::{BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer, XmlVersion};

use super::invalid;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Content {
    Element(Element),
    Text(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Element {
    pub name: String,
    attributes: Vec<(String, String)>,
    content: Vec<Content>,
}

impl Element {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            attributes: vec![],
            content: vec![],
        }
    }

    pub fn attr(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }

    pub fn set_attr(&mut self, name: &str, value: impl Into<String>) {
        if let Some((_, v)) = self.attributes.iter_mut().find(|(k, _)| k == name) {
            *v = value.into();
        } else {
            self.attributes.push((name.into(), value.into()));
        }
    }

    pub fn elements(&self) -> impl Iterator<Item = &Element> {
        self.content.iter().filter_map(|item| match item {
            Content::Element(e) => Some(e),
            _ => None,
        })
    }

    pub fn elements_mut(&mut self) -> impl Iterator<Item = &mut Element> {
        self.content.iter_mut().filter_map(|item| match item {
            Content::Element(e) => Some(e),
            _ => None,
        })
    }

    pub fn child(&self, name: &str) -> Option<&Self> {
        self.elements().find(|e| e.name == name)
    }
    pub fn child_mut(&mut self, name: &str) -> Option<&mut Self> {
        self.elements_mut().find(|e| e.name == name)
    }

    pub fn text(&self) -> Cow<'_, str> {
        match self.content.as_slice() {
            [Content::Text(text)] => Cow::Borrowed(text),
            items => Cow::Owned(
                items
                    .iter()
                    .filter_map(|c| match c {
                        Content::Text(t) => Some(t.as_str()),
                        _ => None,
                    })
                    .collect(),
            ),
        }
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.content = vec![Content::Text(text.into())];
    }

    pub fn push(&mut self, child: Self) {
        self.content.push(Content::Element(child));
    }

    pub fn set_child(&mut self, name: &str, value: impl ToString) {
        self.section(name).set_text(value.to_string());
    }

    pub fn section(&mut self, name: &str) -> &mut Self {
        let pos = self
            .content
            .iter()
            .position(|s| matches!(s, Content::Element(e) if e.name == name));
        let index = pos.unwrap_or_else(|| {
            self.push(Element::new(name));
            self.content.len() - 1
        });
        match &mut self.content[index] {
            Content::Element(e) => e,
            _ => unreachable!(),
        }
    }

    pub fn remove_attr(&mut self, name: &str) {
        self.attributes.retain(|(k, _)| k != name);
    }

    pub fn retain_elements(&mut self, mut keep: impl FnMut(&Element) -> bool) {
        self.content.retain(|c| match c {
            Content::Element(e) => keep(e),
            _ => true,
        });
    }

    pub fn i32(&self) -> io::Result<i32> {
        i32::try_from(super::integer(&self.text())?).map_err(invalid)
    }

    pub fn child_i32(&self, name: &str, default: i32) -> io::Result<i32> {
        self.child(name)
            .map(Element::i32)
            .transpose()
            .map(|v| v.unwrap_or(default))
    }

    pub fn attr_i32(&self, name: &str) -> io::Result<i32> {
        let v = self
            .attr(name)
            .ok_or_else(|| invalid(format!("missing {} on {}", name, self.name)))?;
        i32::try_from(super::integer(v)?).map_err(invalid)
    }

    fn append_text(&mut self, text: &str) {
        if let Some(Content::Text(s)) = self.content.last_mut() {
            s.push_str(text);
        } else {
            self.content.push(Content::Text(text.into()));
        }
    }

    fn start(start: BytesStart<'_>, version: XmlVersion) -> io::Result<Self> {
        let mut e = Self::new(start.name().0);
        for attr in start.attributes() {
            let attr = attr.map_err(invalid)?;
            e.attributes.push((
                attr.key.0.to_owned(),
                attr.normalized_value(version)
                    .map_err(invalid)?
                    .into_owned(),
            ));
        }
        Ok(e)
    }

    pub(super) fn read(input: impl BufRead) -> io::Result<Self> {
        let mut reader = Reader::from_reader(input);
        reader.config_mut().expand_empty_elements = true;
        let mut stack: Vec<Self> = Vec::new();
        let mut root = None;
        let mut buf = Vec::new();
        let mut version = XmlVersion::Implicit1_0;
        loop {
            match reader.read_event_into(&mut buf).map_err(invalid)? {
                Event::Start(tag) => {
                    // LinCity's deepest history entry is eight elements deep.
                    if stack.len() >= 128 {
                        return Err(invalid("XML nesting exceeds save format limit"));
                    }
                    stack.push(Self::start(tag, version)?);
                }
                Event::End(_) => {
                    let child = stack
                        .pop()
                        .ok_or_else(|| invalid("unexpected closing element"))?;
                    if let Some(parent) = stack.last_mut() {
                        parent.push(child);
                    } else if root.replace(child).is_some() {
                        return Err(invalid("multiple root elements"));
                    }
                }
                Event::GeneralRef(r) => {
                    let parent = stack
                        .last_mut()
                        .ok_or_else(|| invalid("entity outside root"))?;
                    if let Some(c) = r.resolve_char_ref().map_err(invalid)? {
                        parent.append_text(c.encode_utf8(&mut [0; 4]));
                    } else {
                        parent.append_text(
                            quick_xml::escape::resolve_xml_entity(&r)
                                .ok_or_else(|| invalid("unknown XML entity"))?,
                        );
                    }
                }
                Event::Text(t) => {
                    if let Some(parent) = stack.last_mut() {
                        parent.append_text(&t.xml_content(version));
                    } else if !t.trim().is_empty() {
                        return Err(invalid("text outside root element"));
                    }
                }
                Event::CData(t) => stack
                    .last_mut()
                    .ok_or_else(|| invalid("CDATA outside root"))?
                    .append_text(&t.xml_content(version)),
                Event::Decl(d) => version = d.xml_version().map_err(invalid)?,
                Event::DocType(_) => return Err(invalid("DTD is unsupported in save files")),
                Event::Eof => break,
                _ => {}
            }
            buf.clear();
        }
        if !stack.is_empty() {
            return Err(invalid("incomplete save XML"));
        }
        root.ok_or_else(|| invalid("missing root element"))
    }

    pub(super) fn write(&self, writer: &mut Writer<impl Write>) -> io::Result<()> {
        let mut start = BytesStart::new(&self.name);
        for (key, value) in &self.attributes {
            start.push_attribute((key.as_str(), value.as_str()));
        }
        writer.write_event(Event::Start(start.borrow()))?;
        for item in &self.content {
            match item {
                Content::Text(text) => writer.write_event(Event::Text(BytesText::new(text)))?,
                Content::Element(element) => element.write(writer)?,
            }
        }
        writer.write_event(Event::End(start.to_end()))
    }
}
