use ego_tree::NodeRef;
use scraper::{ElementRef, Html, Node, Selector};
use selectors::attr::CaseSensitivity;

#[derive(Debug)]
pub struct Document<'a> {
    pub title: Vec<TextPart<'a>>,
    pub since: Option<&'a str>,
    pub declaration: Option<Vec<TextPart<'a>>>,
    pub stability: Option<Vec<TextPart<'a>>>,
    pub portability: Option<Vec<TextPart<'a>>>,
    pub description: Vec<Section<'a>>,
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
    let (stability, portability) =
        item_info.map_or((None, None), |info| (info.stability, info.portability));

    let maybe_top_doc = if stability.is_none() && portability.is_none() {
        maybe_item_info
    } else {
        children.next()
    }?;
    let top_doc = parse_top_doc(maybe_top_doc)?;

    Some(Document {
        title: fqn.title,
        since: fqn.since,
        declaration,
        stability,
        portability,
        description: top_doc.sections,
    })
}

struct Fqn<'a> {
    title: Vec<TextPart<'a>>,
    since: Option<&'a str>,
}

fn parse_fqn(maybe_fqn: NodeRef<Node>) -> Option<Fqn> {
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

struct ItemDecl<'a> {
    code: Vec<TextPart<'a>>,
}

fn parse_item_decl(maybe_item_decl: NodeRef<Node>) -> Option<ItemDecl> {
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

struct ItemInfo<'a> {
    stability: Option<Vec<TextPart<'a>>>,
    portability: Option<Vec<TextPart<'a>>>,
}

fn parse_item_info(maybe_item_info: NodeRef<Node>) -> Option<ItemInfo> {
    let item_info = maybe_item_info.value().as_element()?;

    if !(item_info.name() == "div"
        && item_info.has_class("item-info", CaseSensitivity::CaseSensitive))
    {
        return None;
    }

    let mut stability = None;
    let mut portability = None;
    for child in maybe_item_info.children() {
        if let Some(maybe_stab) = child.value().as_element() {
            if maybe_stab.name() == "div"
                && maybe_stab.has_class("stab", CaseSensitivity::CaseSensitive)
            {
                if maybe_stab.has_class("unstable", CaseSensitivity::CaseSensitive) {
                    stability = Some(parse_text_inside(child));
                } else if maybe_stab.has_class("portability", CaseSensitivity::CaseSensitive) {
                    portability = Some(parse_text_inside(child));
                }
            }
        }
    }

    Some(ItemInfo {
        stability,
        portability,
    })
}

struct TopDoc<'a> {
    sections: Vec<Section<'a>>,
}

fn parse_top_doc(maybe_top_doc: NodeRef<Node>) -> Option<TopDoc> {
    let top_doc = maybe_top_doc.value().as_element()?;

    if !(top_doc.name() == "details"
        && top_doc.has_class("top-doc", CaseSensitivity::CaseSensitive))
    {
        return None;
    }

    for child in maybe_top_doc.children() {
        if let Some(doc_block) = parse_doc_block(child) {
            return Some(TopDoc {
                sections: doc_block.sections,
            });
        }
    }
    None
}

struct DocBlock<'a> {
    sections: Vec<Section<'a>>,
}

#[derive(Debug)]
pub struct Section<'a> {
    pub depth: u8,
    pub heading: Option<Vec<TextPart<'a>>>,
    pub contents: Vec<Paragraph<'a>>,
}

fn parse_doc_block(maybe_doc_block: NodeRef<Node>) -> Option<DocBlock> {
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

#[derive(Debug)]
pub enum Paragraph<'a> {
    Text(Vec<TextPart<'a>>),
    List(Vec<Vec<TextPart<'a>>>),
    Code(Vec<TextPart<'a>>),
}

fn parse_p(maybe_p: NodeRef<Node>) -> Option<Vec<TextPart>> {
    let p = maybe_p.value().as_element()?;

    if p.name() != "p" {
        return None;
    }

    Some(parse_text_inside(maybe_p))
}

fn parse_list(maybe_list: NodeRef<Node>) -> Option<Vec<Vec<TextPart>>> {
    let list = maybe_list.value().as_element()?;

    if !(list.name() == "ul" || list.name() == "ol") {
        return None;
    }

    let mut list = vec![];

    for child in maybe_list.children() {
        let li = child.value().as_element()?;
        if li.name() != "li" {
            return None;
        }
        list.push(parse_text_inside(child));
    }

    Some(list)
}

fn parse_code(maybe_code: NodeRef<Node>) -> Option<Vec<TextPart>> {
    let code = maybe_code.value().as_element()?;

    if !(code.name() == "div" && code.has_class("example-wrap", CaseSensitivity::CaseSensitive)) {
        return None;
    }

    for child in maybe_code.children() {
        if let Some(code) = parse_pre(child) {
            return Some(code.code);
        }
    }
    None
}

struct Pre<'a> {
    code: Vec<TextPart<'a>>,
}

fn parse_pre(maybe_pre: NodeRef<Node>) -> Option<Pre> {
    let pre = maybe_pre.value().as_element()?;
    if pre.name() != "pre" {
        return None;
    }

    Some(Pre {
        code: parse_text_inside(maybe_pre),
    })
}

#[derive(Debug)]
pub enum TextPart<'a> {
    Text(&'a str),
    BeginStyle(TextStyle<'a>),
    EndStyle,
}

#[derive(Debug)]
pub enum TextStyle<'a> {
    Link(Option<&'a str>),
    Bold,
    Italic,
    Underline,
    Strikethrough,
    Monospaced,
}

fn parse_text_inside(node: NodeRef<Node>) -> Vec<TextPart> {
    fn parse_text_to<'a>(node: NodeRef<'a, Node>, buffer: &mut Vec<TextPart<'a>>) {
        for child in node.children() {
            match child.value() {
                Node::Text(text) => {
                    buffer.push(TextPart::Text(text));
                }
                Node::Element(element) => match element.name() {
                    "a" => {
                        let href = element
                            .attrs()
                            .find_map(|(key, value)| (key == "href").then(|| value));
                        buffer.push(TextPart::BeginStyle(TextStyle::Link(href)));
                        parse_text_to(child, buffer);
                        buffer.push(TextPart::EndStyle);
                    }
                    "strong" => {
                        buffer.push(TextPart::BeginStyle(TextStyle::Bold));
                        parse_text_to(child, buffer);
                        buffer.push(TextPart::EndStyle);
                    }
                    "em" => {
                        buffer.push(TextPart::BeginStyle(TextStyle::Italic));
                        parse_text_to(child, buffer);
                        buffer.push(TextPart::EndStyle);
                    }
                    "u" => {
                        buffer.push(TextPart::BeginStyle(TextStyle::Underline));
                        parse_text_to(child, buffer);
                        buffer.push(TextPart::EndStyle);
                    }
                    "del" => {
                        buffer.push(TextPart::BeginStyle(TextStyle::Strikethrough));
                        parse_text_to(child, buffer);
                        buffer.push(TextPart::EndStyle);
                    }
                    "code" => {
                        buffer.push(TextPart::BeginStyle(TextStyle::Monospaced));
                        parse_text_to(child, buffer);
                        buffer.push(TextPart::EndStyle);
                    }
                    "br" => {
                        buffer.push(TextPart::Text("\n"));
                    }
                    "span" => {
                        if element.has_class("fmt-newline", CaseSensitivity::CaseSensitive) {
                            buffer.push(TextPart::Text("\n"));
                        }
                        parse_text_to(child, buffer);
                    }
                    "p" => {
                        parse_text_to(child, buffer);
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
    let mut buffer = vec![];
    parse_text_to(node, &mut buffer);
    buffer
}
