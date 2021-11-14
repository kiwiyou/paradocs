use ego_tree::NodeRef;
use scraper::Node;
use selectors::attr::CaseSensitivity;

use crate::atom::{parse_text_inside, TextPart};

pub fn parse_struct_field_or_variant(maybe: NodeRef<Node>) -> Option<Vec<TextPart>> {
    let element = maybe.value().as_element()?;

    if !(element.name() == "span"
        && (element.has_class("structfield", CaseSensitivity::CaseSensitive)
            || element.has_class("variant", CaseSensitivity::CaseSensitive)))
    {
        return None;
    }

    Some(parse_text_inside(maybe))
}
