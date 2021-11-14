use ego_tree::NodeRef;
use scraper::Node;
use selectors::attr::CaseSensitivity;

use crate::{
    atom::{parse_text_outside, TextPart},
    header::{parse_doc_block, parse_item_info},
};

use super::{Impl, Item};

pub struct ImplHeading<'a> {
    pub title: Vec<TextPart<'a>>,
}

pub fn parse_impl_heading(maybe_impl_header: NodeRef<Node>) -> Option<ImplHeading> {
    let impl_header = maybe_impl_header.value().as_element()?;

    if !(impl_header.name() == "h3"
        && impl_header.has_class("impl", CaseSensitivity::CaseSensitive))
    {
        return None;
    }

    for child in maybe_impl_header.children() {
        if let Some(element) = child.value().as_element() {
            if element.name() == "code"
                && element.has_class("in-band", CaseSensitivity::CaseSensitive)
            {
                return Some(ImplHeading {
                    title: parse_text_outside(child),
                });
            }
        }
    }
    None
}

pub fn parse_impl_items(maybe_impl_items: NodeRef<Node>) -> Option<Vec<Item>> {
    let impl_items = maybe_impl_items.value().as_element()?;

    if !(impl_items.name() == "div"
        && impl_items.has_class("impl-items", CaseSensitivity::CaseSensitive))
    {
        return None;
    }

    let mut items = vec![];

    for child in maybe_impl_items.children() {
        if let Some(heading) = parse_item_heading(child) {
            items.push(Item {
                name: heading.title,
                info: Default::default(),
                description: None,
            })
        } else if let Some(item_info) = parse_item_info(child) {
            if let Some(last_item) = items.last_mut() {
                last_item.info = item_info;
            }
        } else if let Some(doc_block) = parse_doc_block(child) {
            if let Some(last_item) = items.last_mut() {
                last_item.description = Some(doc_block.sections);
            }
        }
    }
    Some(items)
}

struct ItemHeading<'a> {
    title: Vec<TextPart<'a>>,
}

fn parse_item_heading(maybe_item_heading: NodeRef<Node>) -> Option<ItemHeading> {
    let item_heading = maybe_item_heading.value().as_element()?;

    if item_heading.name() != "h4" {
        return None;
    }

    for child in maybe_item_heading.children() {
        if let Some(element) = child.value().as_element() {
            if element.name() == "code" {
                return Some(ItemHeading {
                    title: parse_text_outside(child),
                });
            }
        }
    }
    None
}

pub fn parse_impl_div(maybe_impl_list: NodeRef<Node>) -> Option<Vec<Impl>> {
    let impl_list = maybe_impl_list.value().as_element()?;

    if impl_list.name() != "div" {
        return None;
    }

    let mut impls = vec![];

    let mut children = maybe_impl_list.children();
    while let Some(maybe_heading) = children.next() {
        let heading = parse_impl_heading(maybe_heading)?;
        let items = parse_impl_items(children.next()?)?;
        impls.push(Impl {
            target: heading.title,
            items,
        });
    }

    Some(impls)
}
