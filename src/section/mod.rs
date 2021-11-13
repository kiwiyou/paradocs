use ego_tree::NodeRef;
use scraper::Node;
use selectors::attr::CaseSensitivity;

use crate::atom::{parse_text_inside, TextPart};

pub mod table;

pub fn parse_section_header(maybe_section_header: NodeRef<Node>) -> Option<Vec<TextPart>> {
    let section_header = maybe_section_header.value().as_element()?;

    if !(section_header.name() == "h2"
        && section_header.has_class("section-header", CaseSensitivity::CaseSensitive))
    {
        return None;
    }

    Some(parse_text_inside(maybe_section_header))
}

pub fn is_section_header(maybe_section_header: NodeRef<Node>) -> bool {
    maybe_section_header
        .value()
        .as_element()
        .map_or(false, |section_header| {
            section_header.name() == "h2"
                && section_header.has_class("section-header", CaseSensitivity::CaseSensitive)
        })
}
