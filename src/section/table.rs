use ego_tree::NodeRef;
use scraper::Node;
use selectors::attr::CaseSensitivity;

use crate::atom::{
    parse_deprecated, parse_portability, parse_text_inside, parse_text_outside, parse_unstable,
    TextPart,
};

pub fn parse_item_table(maybe_item_table: NodeRef<Node>) -> Option<Vec<ItemRow>> {
    eprintln!("{:#?}", maybe_item_table.value());
    let item_table = maybe_item_table.value().as_element()?;

    if !(item_table.name() == "div"
        && item_table.has_class("item-table", CaseSensitivity::CaseSensitive))
    {
        return None;
    }

    let mut rows = vec![];

    let mut children = maybe_item_table.children();

    while let Some(child) = children.next() {
        if let Some(row) = parse_item_row(child) {
            rows.push(row);
        } else {
            let left = parse_item_left(child)?;
            let right = parse_item_right(children.next()?)?;
            rows.push(ItemRow {
                item: left.text,
                stability: left.stability,
                portability: left.portability,
                deprecation: left.deprecation,
                summary: right,
            });
        }
    }

    Some(rows)
}

#[derive(Debug)]
pub struct ItemRow<'a> {
    pub item: Vec<TextPart<'a>>,
    pub stability: Option<Vec<TextPart<'a>>>,
    pub portability: Option<Vec<TextPart<'a>>>,
    pub deprecation: Option<Vec<TextPart<'a>>>,
    pub summary: Vec<TextPart<'a>>,
}

fn parse_item_row(maybe_item_row: NodeRef<Node>) -> Option<ItemRow> {
    let item_row = maybe_item_row.value().as_element()?;

    if !(item_row.name() == "div" && item_row.has_class("item-row", CaseSensitivity::CaseSensitive))
    {
        return None;
    }

    let mut children = maybe_item_row.children();
    let left = parse_item_left(children.next()?)?;
    let right = parse_item_right(children.next()?)?;

    Some(ItemRow {
        item: left.text,
        stability: left.stability,
        portability: left.portability,
        deprecation: left.deprecation,
        summary: right,
    })
}

struct ItemLeft<'a> {
    text: Vec<TextPart<'a>>,
    stability: Option<Vec<TextPart<'a>>>,
    portability: Option<Vec<TextPart<'a>>>,
    deprecation: Option<Vec<TextPart<'a>>>,
}

fn parse_item_left(maybe_item_left: NodeRef<Node>) -> Option<ItemLeft> {
    let item_left = maybe_item_left.value().as_element()?;

    if !(item_left.name() == "div"
        && item_left.has_class("item-left", CaseSensitivity::CaseSensitive))
    {
        return None;
    }

    let mut children = maybe_item_left.children();
    let text = parse_text_outside(children.next()?);

    let mut stability = None;
    let mut portability = None;
    let mut deprecation = None;
    for child in children {
        stability = stability.or_else(|| parse_unstable(child));
        portability = portability.or_else(|| parse_portability(child));
        deprecation = deprecation.or_else(|| parse_deprecated(child));
    }

    Some(ItemLeft {
        text,
        stability,
        portability,
        deprecation,
    })
}

fn parse_item_right(maybe_item_right: NodeRef<Node>) -> Option<Vec<TextPart>> {
    let item_right = maybe_item_right.value().as_element()?;

    if !(item_right.name() == "div"
        && item_right.has_class("item-right", CaseSensitivity::CaseSensitive))
    {
        return None;
    }

    Some(parse_text_inside(maybe_item_right))
}

pub fn parse_block_table(maybe_table: NodeRef<Node>) -> Option<Vec<ItemRow>> {
    let table = maybe_table.value().as_element()?;

    if !(table.name() == "table" && table.attr("style") == Some("display: block;")) {
        return None;
    }

    let maybe_tbody = maybe_table.first_child()?;
    let tbody = maybe_tbody.value().as_element()?;

    if tbody.name() != "tbody" {
        return None;
    }

    let mut rows = vec![];
    for child in maybe_tbody.children() {
        let tr = child.value().as_element()?;

        if tr.name() != "tr" {
            return None;
        }

        let mut children = child.children();
        let maybe_left = children.next()?;
        let left = maybe_left.value().as_element()?;

        if left.name() != "td" {
            return None;
        }

        let mut left_children = maybe_left.children();

        let text = parse_text_outside(left_children.next()?);

        let mut stability = None;
        let mut portability = None;
        let mut deprecation = None;
        for child in left_children {
            stability = stability.or_else(|| parse_unstable(child));
            portability = portability.or_else(|| parse_portability(child));
            deprecation = deprecation.or_else(|| parse_deprecated(child));
        }

        fn parse_right(maybe_right: NodeRef<Node>) -> Option<Vec<TextPart>> {
            let right = maybe_right.value().as_element()?;

            if right.name() != "td" {
                return None;
            }

            Some(parse_text_inside(maybe_right))
        }

        let right = children.next().and_then(parse_right).unwrap_or_default();

        rows.push(ItemRow {
            item: text,
            stability,
            portability,
            deprecation,
            summary: right,
        })
    }

    Some(rows)
}
