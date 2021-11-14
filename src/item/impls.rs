use ego_tree::NodeRef;
use scraper::Node;
use selectors::attr::CaseSensitivity;

use crate::{
    atom::{parse_text_inside, parse_text_outside, TextPart},
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

pub fn parse_empty_impl(maybe_empty_impl: NodeRef<Node>) -> Option<Impl> {
    let empty_impl = maybe_empty_impl.value().as_element()?;

    if !(empty_impl.name() == "div" && empty_impl.has_class("impl", CaseSensitivity::CaseSensitive))
    {
        return None;
    }

    for child in maybe_empty_impl.children() {
        if let Some(element) = child.value().as_element() {
            if element.name() == "h3"
                && element.has_class("in-band", CaseSensitivity::CaseSensitive)
            {
                return Some(Impl {
                    target: parse_text_inside(child),
                    items: vec![],
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
        } else if let Some(name) = parse_srclink(child) {
            items.push(Item {
                name,
                info: Default::default(),
                description: None,
            });
        } else if let Some(toggle) = parse_toggle_item(child) {
            items.push(toggle);
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
    while let Some(child) = children.next() {
        if let Some(empty) = parse_empty_impl(child) {
            impls.push(empty);
        } else if let Some(heading) = parse_impl_heading(child) {
            impls.push(Impl {
                target: heading.title,
                items: vec![],
            });
        } else if let Some(items) = parse_impl_items(child) {
            if let Some(last_impl) = impls.last_mut() {
                last_impl.items = items;
            }
        } else {
            return None;
        }
    }

    Some(impls)
}

pub fn parse_implementor(maybe_implementor: NodeRef<Node>) -> Option<Impl> {
    let implementor = maybe_implementor.value().as_element()?;

    if !(implementor.name() == "details"
        && implementor.has_class("implementors-toggle", CaseSensitivity::CaseSensitive))
    {
        return None;
    }

    let mut children = maybe_implementor.children();
    let maybe_summary = children.next()?;
    let summary = maybe_summary.value().as_element()?;

    if summary.name() != "summary" {
        return None;
    }

    let target = parse_srclink(maybe_summary.first_child()?)?;

    let items = children
        .next()
        .and_then(parse_impl_items)
        .unwrap_or_default();

    Some(Impl { target, items })
}

pub fn parse_implementor_or_empty(node: NodeRef<Node>) -> Option<Impl> {
    parse_implementor(node).or_else(|| parse_empty_impl(node))
}

fn parse_srclink(maybe_srclink: NodeRef<Node>) -> Option<Vec<TextPart>> {
    let srclink = maybe_srclink.value().as_element()?;

    if !(srclink.name() == "div"
        && srclink.has_class("has-srclink", CaseSensitivity::CaseSensitive))
    {
        return None;
    }

    for child in maybe_srclink.children() {
        if let Some(code_header) = child.value().as_element() {
            if code_header.has_class("code-header", CaseSensitivity::CaseSensitive)
            {
                return Some(parse_text_inside(child));
            }
        }
    }
    None
}

fn parse_toggle_item(maybe_toggle_item: NodeRef<Node>) -> Option<Item> {
    let toggle_item = maybe_toggle_item.value().as_element()?;

    if toggle_item.name() != "details" {
        return None;
    }

    eprintln!("{:#?}", toggle_item);

    let mut children = maybe_toggle_item.children();
    let maybe_summary = children.next()?;
    let summary = maybe_summary.value().as_element()?;

    if summary.name() != "summary" {
        return None;
    }

    let srclink = parse_srclink(maybe_summary.first_child()?)?;

    let doc_block = parse_doc_block(children.next()?)?;

    Some(Item {
        name: srclink,
        info: Default::default(),
        description: Some(doc_block.sections),
    })
}
