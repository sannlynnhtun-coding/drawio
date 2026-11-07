use base64::{Engine as _, engine::general_purpose::STANDARD};
use std::{
    fs::File,
    io::{self, Read, Write},
    path::Path,
};

pub const MAX_FILE: usize = 32 * 1024 * 1024;
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn graph_model(node: roxmltree::Node<'_, '_>) -> bool {
    node.has_tag_name("mxGraphModel") && node.children().any(|n| n.has_tag_name("root"))
}

fn validate_xml(xml: &str) -> Result<()> {
    if xml.len() > MAX_FILE {
        return Err("Diagram exceeds the prototype's 32 MiB limit".into());
    }
    let doc = roxmltree::Document::parse(xml.strip_prefix('\u{feff}').unwrap_or(xml))?;
    let root = doc.root_element();
    if graph_model(root) {
        return Ok(());
    }
    if !root.has_tag_name("mxfile") {
        return Err("Expected a draw.io XML document".into());
    }
    let mut pages = root
        .children()
        .filter(|n| n.has_tag_name("diagram"))
        .peekable();
    if pages.peek().is_none() {
        return Err("The draw.io file has no pages".into());
    }
    let mut remaining = MAX_FILE;
    for page in pages {
        if page.children().any(graph_model) {
            continue;
        }
        let encoded = page.text().ok_or("Missing diagram contents")?;
        let compressed = STANDARD.decode(encoded.trim())?;
        let mut inflated = Vec::new();
        flate2::read::DeflateDecoder::new(compressed.as_slice())
            .take((remaining + 1) as u64)
            .read_to_end(&mut inflated)?;
        remaining = remaining
            .checked_sub(inflated.len())
            .ok_or("Expanded diagram exceeds 32 MiB")?;
        let decoded = percent_encoding::percent_decode(&inflated).decode_utf8()?;
        if !graph_model(roxmltree::Document::parse(&decoded)?.root_element()) {
            return Err("Invalid compressed draw.io page".into());
        }
    }
    Ok(())
}

pub fn read_diagram(path: &Path) -> Result<String> {
    let mut xml = String::new();
    File::open(path)?
        .take((MAX_FILE + 1) as u64)
        .read_to_string(&mut xml)?;
    if xml.len() > MAX_FILE {
        return Err("Diagram exceeds the prototype's 32 MiB limit".into());
    }
    let xml = xml
        .strip_prefix('\u{feff}')
        .map(str::to_owned)
        .unwrap_or(xml);
    validate_xml(&xml)?;
    Ok(xml)
}

pub fn write_diagram(path: &Path, xml: &str, expected: Option<&str>) -> Result<()> {
    validate_xml(xml)?;
    if xml.len() > MAX_FILE {
        return Err("Diagram exceeds the prototype's 32 MiB limit".into());
    }
    let mut temp =
        tempfile::NamedTempFile::new_in(path.parent().ok_or("Missing destination folder")?)?;
    temp.write_all(xml.as_bytes())?;
    temp.as_file().sync_all()?;
    if let Some(expected) = expected {
        if read_diagram(path)? != expected {
            return Err("The original changed on disk. Use Save As to keep your edits.".into());
        }
        temp.persist(path)?;
    } else {
        temp.persist_noclobber(path).map_err(|err| {
            if err.error.kind() == io::ErrorKind::AlreadyExists {
                io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "Choose a new filename for Save As",
                )
            } else {
                err.error
            }
        })?;
    }
    Ok(())
}
