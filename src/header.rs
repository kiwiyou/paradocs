use ego_tree::NodeRef;
use scraper::{ElementRef, Node};
use selectors::attr::CaseSensitivity;

use crate::atom::{
    parse_code, parse_deprecated, parse_list, parse_p, parse_portability, parse_pre,
    parse_text_inside, parse_unstable, Details, Paragraph, TextPart,
};

pub struct Fqn<'a> {
    pub title: Vec<TextPart<'a>>,
    pub since: Option<&'a str>,
}

pub fn parse_fqn(maybe_fqn: NodeRef<Node>) -> Option<Fqn> {
    let fqn = maybe_fqn.value().as_element()?;

    if !(fqn.name() == "h1" && fqn.has_class("fqn", CaseSensitivity::CaseSensitive)) {
        return None;
    }

    let mut in_band = None;
    let mut out_of_band = None;

    for child in maybe_fqn.children() {
        in_band = in_band.or(parse_in_band(child));
        out_of_band = out_of_band.or(parse_out_of_band(child));
    }

    Some(Fqn {
        title: in_band?.text,
        since: out_of_band?.since,
    })
}

struct InBand<'a> {
    text: Vec<TextPart<'a>>,
}

fn parse_in_band(maybe_in_band: NodeRef<Node>) -> Option<InBand> {
    let in_band = maybe_in_band.value().as_element()?;

    if !(in_band.name() == "span" && in_band.has_class("in-band", CaseSensitivity::CaseSensitive)) {
        return None;
    }

    let text = parse_text_inside(maybe_in_band);

    Some(InBand { text })
}

struct OutOfBand<'a> {
    since: Option<&'a str>,
}

fn parse_out_of_band(maybe_out_of_band: NodeRef<Node>) -> Option<OutOfBand> {
    let out_of_band = maybe_out_of_band.value().as_element()?;

    if !(out_of_band.name() == "span"
        && out_of_band.has_class("out-of-band", CaseSensitivity::CaseSensitive))
    {
        return None;
    }

    for child in maybe_out_of_band.children() {
        if let Some(element) = child.value().as_element() {
            if element.name() == "span"
                && element.has_class("since", CaseSensitivity::CaseSensitive)
            {
                let since = ElementRef::wrap(child).unwrap().text().next()?;
                return Some(OutOfBand { since: Some(since) });
            }
        }
    }

    Some(OutOfBand { since: None })
}

pub struct ItemDecl<'a> {
    pub code: Vec<TextPart<'a>>,
}

pub fn parse_item_decl(maybe_item_decl: NodeRef<Node>) -> Option<ItemDecl> {
    let item_decl = maybe_item_decl.value().as_element()?;

    if !(item_decl.name() == "div"
        && (item_decl.has_class("item-decl", CaseSensitivity::CaseSensitive)
            || item_decl.has_class("type-decl", CaseSensitivity::CaseSensitive)))
    {
        return None;
    }

    for child in maybe_item_decl.children() {
        if let Some(pre) = parse_pre(child) {
            return Some(ItemDecl { code: pre.code });
        }
    }
    None
}

#[derive(Debug, Default)]
pub struct ItemInfo<'a> {
    pub stability: Option<Details<'a>>,
    pub portability: Option<Details<'a>>,
    pub deprecation: Option<Details<'a>>,
}

pub fn parse_item_info(maybe_item_info: NodeRef<Node>) -> Option<ItemInfo> {
    let item_info = maybe_item_info.value().as_element()?;

    if !(item_info.name() == "div"
        && item_info.has_class("item-info", CaseSensitivity::CaseSensitive))
    {
        return None;
    }

    let mut stability = None;
    let mut portability = None;
    let mut deprecation = None;
    for child in maybe_item_info.children() {
        stability = stability.or_else(|| parse_unstable(child));
        portability = portability.or_else(|| parse_portability(child));
        deprecation = deprecation.or_else(|| parse_deprecated(child));
    }

    Some(ItemInfo {
        stability,
        portability,
        deprecation,
    })
}

pub struct TopDoc<'a> {
    pub doc_block: DocBlock<'a>,
}

pub fn parse_top_doc(maybe_top_doc: NodeRef<Node>) -> Option<TopDoc> {
    let top_doc = maybe_top_doc.value().as_element()?;

    if !(top_doc.name() == "details"
        && top_doc.has_class("top-doc", CaseSensitivity::CaseSensitive))
    {
        return None;
    }

    for child in maybe_top_doc.children() {
        if let Some(doc_block) = parse_doc_block(child) {
            return Some(TopDoc { doc_block });
        }
    }
    None
}

pub struct DocBlock<'a> {
    pub sections: Vec<Section<'a>>,
}

#[derive(Debug)]
pub struct Section<'a> {
    pub depth: u8,
    pub heading: Option<Vec<TextPart<'a>>>,
    pub contents: Vec<Paragraph<'a>>,
}

pub fn parse_doc_block(maybe_doc_block: NodeRef<Node>) -> Option<DocBlock> {
    let doc_block = maybe_doc_block.value().as_element()?;

    if !(doc_block.name() == "div"
        && doc_block.has_class("docblock", CaseSensitivity::CaseSensitive))
    {
        return None;
    }

    let mut sections = vec![];

    for child in maybe_doc_block.children() {
        if let Some(element) = child.value().as_element() {
            if element.name().starts_with("h")
                && (b'2'..=b'6').contains(&element.name().as_bytes()[1])
            {
                let depth = element.name().as_bytes()[1] - b'0';
                let heading = parse_text_inside(child);
                sections.push(Section {
                    depth,
                    heading: Some(heading),
                    contents: vec![],
                });
            } else {
                let content = parse_p(child)
                    .map(Paragraph::Text)
                    .or_else(|| parse_list(child).map(Paragraph::List))
                    .or_else(|| parse_code(child).map(Paragraph::Code));
                if let Some(content) = content {
                    if let Some(section) = sections.last_mut() {
                        section.contents.push(content);
                    } else {
                        sections.push(Section {
                            depth: 2,
                            heading: None,
                            contents: vec![content],
                        });
                    }
                }
            }
        }
    }

    Some(DocBlock { sections })
}
