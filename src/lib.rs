mod atom;
mod header;
mod item;

use atom::{parse_pre, TextPart};
use header::{parse_fqn, parse_item_decl, parse_item_info, parse_top_doc, Section};
use item::{
    parse_item_header,
    table::{parse_block_table, parse_item_table},
    Impl, Item, ItemRow,
};
use scraper::{Html, Selector};

use crate::{
    header::parse_doc_block,
    item::{
        fields::parse_struct_field_or_variant,
        impls::{parse_impl_div, parse_impl_heading, parse_impl_items},
        is_item_header,
    },
};

#[derive(Debug)]
pub struct Document<'a> {
    pub title: Vec<TextPart<'a>>,
    pub since: Option<&'a str>,
    pub declaration: Option<Vec<TextPart<'a>>>,
    pub stability: Option<Vec<TextPart<'a>>>,
    pub portability: Option<Vec<TextPart<'a>>>,
    pub deprecation: Option<Vec<TextPart<'a>>>,
    pub description: Vec<Section<'a>>,
    pub items: Vec<ItemListing<'a>>,
}

#[derive(Debug)]
pub struct ItemListing<'a> {
    pub heading: Vec<TextPart<'a>>,
    pub kind: ListingType<'a>,
}

#[derive(Debug)]
pub enum ListingType<'a> {
    Table(Vec<ItemRow<'a>>),
    Fields(Vec<Item<'a>>),
    Impls(Vec<Impl<'a>>),
}

pub fn parse_document(html: &Html) -> Option<Document> {
    let select_main = Selector::parse("#main").unwrap();
    let main = html.select(&select_main).next()?;

    let mut children = main.children();

    let maybe_fqn = children.next()?;
    let fqn = parse_fqn(maybe_fqn)?;

    let maybe_decl = children.next();
    let item_decl = maybe_decl.and_then(parse_item_decl).map(|decl| decl.code);
    let pre = maybe_decl.and_then(parse_pre).map(|decl| decl.code);
    let declaration = item_decl.or(pre);

    let maybe_item_info = if declaration.is_none() {
        maybe_decl
    } else {
        children.next()
    };
    let item_info = maybe_item_info.and_then(parse_item_info);
    let (stability, portability, deprecation) = item_info.map_or((None, None, None), |info| {
        (info.stability, info.portability, info.deprecation)
    });

    let maybe_top_doc = if stability.is_none() && portability.is_none() && deprecation.is_none() {
        maybe_item_info
    } else {
        children.next()
    }?;
    let doc_block = parse_top_doc(maybe_top_doc)
        .map(|top_doc| top_doc.doc_block)
        .or_else(|| parse_doc_block(maybe_top_doc))?;

    let mut children = children.peekable();

    let mut listings = vec![];
    while let Some(maybe_heading) = children.next() {
        if let Some(heading) = parse_item_header(maybe_heading) {
            while let Some(maybe_content) = children.peek() {
                if is_item_header(*maybe_content) {
                    break;
                } else if let Some(table) =
                    parse_item_table(*maybe_content).or_else(|| parse_block_table(*maybe_content))
                {
                    children.next();
                    listings.push(ItemListing {
                        heading,
                        kind: ListingType::Table(table),
                    });
                    break;
                } else if let Some(field) = parse_struct_field_or_variant(*maybe_content) {
                    let mut items = vec![];
                    items.push(Item {
                        name: field,
                        info: Default::default(),
                        description: None,
                    });
                    children.next();
                    while let Some(sibling) = children.peek() {
                        if is_item_header(*sibling) {
                            break;
                        } else if let Some(field) = parse_struct_field_or_variant(*sibling) {
                            items.push(Item {
                                name: field,
                                info: Default::default(),
                                description: None,
                            });
                        } else if let Some(item_info) = parse_item_info(*sibling) {
                            if let Some(last_item) = items.last_mut() {
                                last_item.info = item_info;
                            }
                        } else if let Some(description) = parse_doc_block(*sibling) {
                            if let Some(last_item) = items.last_mut() {
                                last_item.description = Some(description.sections);
                            }
                        }
                        children.next();
                    }
                    listings.push(ItemListing {
                        heading,
                        kind: ListingType::Fields(items),
                    });
                    break;
                } else if let Some(impl_heading) = parse_impl_heading(*maybe_content) {
                    let mut impls = vec![];
                    impls.push(Impl {
                        target: impl_heading.title,
                        items: vec![],
                    });
                    while let Some(sibling) = children.peek() {
                        if is_item_header(*sibling) {
                            break;
                        } else if let Some(impl_items) = parse_impl_items(*sibling) {
                            if let Some(last_impl) = impls.last_mut() {
                                last_impl.items = impl_items;
                            }
                        } else if let Some(impl_heading) = parse_impl_heading(*sibling) {
                            impls.push(Impl {
                                target: impl_heading.title,
                                items: vec![],
                            });
                        }
                        children.next();
                    }
                    listings.push(ItemListing {
                        heading,
                        kind: ListingType::Impls(impls),
                    });
                    break;
                } else if let Some(impl_div) = parse_impl_div(*maybe_content) {
                    listings.push(ItemListing {
                        heading,
                        kind: ListingType::Impls(impl_div),
                    });
                    break;
                } else {
                    children.next();
                }
            }
        }
    }

    Some(Document {
        title: fqn.title,
        since: fqn.since,
        declaration,
        stability,
        portability,
        deprecation,
        description: doc_block.sections,
        items: listings,
    })
}
