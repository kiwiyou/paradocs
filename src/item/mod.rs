use ego_tree::NodeRef;
use scraper::Node;
use selectors::attr::CaseSensitivity;

use crate::{
    atom::{parse_text_inside, TextPart},
    header::{ItemInfo, Section},
};

pub mod fields;
pub mod impls;
pub mod table;

pub fn parse_item_header(maybe_section_header: NodeRef<Node>) -> Option<Vec<TextPart>> {
    let section_header = maybe_section_header.value().as_element()?;

    if !(section_header.name() == "h2"
        && (section_header.has_class("section-header", CaseSensitivity::CaseSensitive)
            || section_header.has_class("small-section-header", CaseSensitivity::CaseSensitive)))
    {
        return None;
    }

    Some(parse_text_inside(maybe_section_header))
}

pub fn is_item_header(maybe_section_header: NodeRef<Node>) -> bool {
    maybe_section_header
        .value()
        .as_element()
        .map_or(false, |section_header| {
            section_header.name() == "h2"
                && (section_header.has_class("section-header", CaseSensitivity::CaseSensitive)
                    || section_header
                        .has_class("small-section-header", CaseSensitivity::CaseSensitive))
        })
}

#[derive(Debug)]
pub struct ItemRow<'a> {
    pub name: Vec<TextPart<'a>>,
    pub info: ItemInfo<'a>,
    pub summary: Vec<TextPart<'a>>,
}

#[derive(Debug)]
pub struct Item<'a> {
    pub name: Vec<TextPart<'a>>,
    pub info: ItemInfo<'a>,
    pub description: Option<Vec<Section<'a>>>,
}

#[derive(Debug)]
pub struct Impl<'a> {
    pub target: Vec<TextPart<'a>>,
    pub items: Vec<Item<'a>>,
}
