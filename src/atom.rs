use ego_tree::NodeRef;
use scraper::Node;
use selectors::attr::CaseSensitivity;

#[derive(Debug)]
pub enum Paragraph<'a> {
    Text(Vec<TextPart<'a>>),
    List(Vec<Vec<TextPart<'a>>>),
    Code(Vec<TextPart<'a>>),
}

pub fn parse_p(maybe_p: NodeRef<Node>) -> Option<Vec<TextPart>> {
    let p = maybe_p.value().as_element()?;

    if p.name() != "p" {
        return None;
    }

    Some(parse_text_inside(maybe_p))
}

pub fn parse_list(maybe_list: NodeRef<Node>) -> Option<Vec<Vec<TextPart>>> {
    let list = maybe_list.value().as_element()?;

    if !(list.name() == "ul" || list.name() == "ol") {
        return None;
    }

    let mut list = vec![];

    for child in maybe_list.children() {
        if let Some(li) = child.value().as_element() {
            if li.name() == "li" {
                list.push(parse_text_inside(child));
            }
        }
    }

    Some(list)
}

pub fn parse_code(maybe_code: NodeRef<Node>) -> Option<Vec<TextPart>> {
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

pub struct Pre<'a> {
    pub code: Vec<TextPart<'a>>,
}

pub fn parse_pre(maybe_pre: NodeRef<Node>) -> Option<Pre> {
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

pub fn parse_text_outside(node: NodeRef<Node>) -> Vec<TextPart> {
    let mut buffer = vec![];
    parse_text_outside_to(node, &mut buffer);
    buffer
}

fn parse_text_outside_to<'a>(node: NodeRef<'a, Node>, buffer: &mut Vec<TextPart<'a>>) {
    match node.value() {
        Node::Text(text) => {
            buffer.push(TextPart::Text(text));
        }
        Node::Element(element) => match element.name() {
            "a" => {
                let href = element.attr("href");
                buffer.push(TextPart::BeginStyle(TextStyle::Link(href)));
                parse_text_inside_to(node, buffer);
                buffer.push(TextPart::EndStyle);
            }
            "strong" => {
                buffer.push(TextPart::BeginStyle(TextStyle::Bold));
                parse_text_inside_to(node, buffer);
                buffer.push(TextPart::EndStyle);
            }
            "em" => {
                buffer.push(TextPart::BeginStyle(TextStyle::Italic));
                parse_text_inside_to(node, buffer);
                buffer.push(TextPart::EndStyle);
            }
            "u" => {
                buffer.push(TextPart::BeginStyle(TextStyle::Underline));
                parse_text_inside_to(node, buffer);
                buffer.push(TextPart::EndStyle);
            }
            "del" => {
                buffer.push(TextPart::BeginStyle(TextStyle::Strikethrough));
                parse_text_inside_to(node, buffer);
                buffer.push(TextPart::EndStyle);
            }
            "code" => {
                buffer.push(TextPart::BeginStyle(TextStyle::Monospaced));
                parse_text_inside_to(node, buffer);
                buffer.push(TextPart::EndStyle);
            }
            "br" => {
                buffer.push(TextPart::Text("\n"));
            }
            "span" => {
                if element.has_class("fmt-newline", CaseSensitivity::CaseSensitive) {
                    buffer.push(TextPart::Text("\n"));
                }
                parse_text_inside_to(node, buffer);
            }
            "p" => {
                parse_text_inside_to(node, buffer);
            }
            _ => {}
        },
        _ => {}
    }
}

pub fn parse_text_inside(node: NodeRef<Node>) -> Vec<TextPart> {
    let mut buffer = vec![];
    parse_text_inside_to(node, &mut buffer);
    buffer
}

pub fn parse_text_inside_to<'a>(node: NodeRef<'a, Node>, buffer: &mut Vec<TextPart<'a>>) {
    for child in node.children() {
        parse_text_outside_to(child, buffer);
    }
}

#[derive(Debug)]
pub struct Details<'a> {
    pub summary: Vec<TextPart<'a>>,
    pub detail: Option<Vec<TextPart<'a>>>,
}

pub fn parse_unstable(maybe_unstable: NodeRef<Node>) -> Option<Details> {
    let unstable = maybe_unstable.value().as_element()?;

    if !(unstable.has_class("unstable", CaseSensitivity::CaseSensitive)) {
        return None;
    }

    let result = maybe_unstable
        .first_child()
        .and_then(parse_details)
        .unwrap_or_else(|| Details {
            summary: parse_text_inside(maybe_unstable),
            detail: None,
        });
    Some(result)
}

pub fn parse_portability(maybe_portability: NodeRef<Node>) -> Option<Details> {
    let portability = maybe_portability.value().as_element()?;

    if !(portability.has_class("portability", CaseSensitivity::CaseSensitive)) {
        return None;
    }

    let result = maybe_portability
        .first_child()
        .and_then(parse_details)
        .unwrap_or_else(|| Details {
            summary: parse_text_inside(maybe_portability),
            detail: None,
        });
    Some(result)
}

pub fn parse_deprecated(maybe_deprecated: NodeRef<Node>) -> Option<Details> {
    let deprecated = maybe_deprecated.value().as_element()?;

    if !(deprecated.has_class("deprecated", CaseSensitivity::CaseSensitive)) {
        return None;
    }

    let result = maybe_deprecated
        .first_child()
        .and_then(parse_details)
        .unwrap_or_else(|| Details {
            summary: parse_text_inside(maybe_deprecated),
            detail: None,
        });
    Some(result)
}

fn parse_details(maybe_details: NodeRef<Node>) -> Option<Details> {
    let details = maybe_details.value().as_element()?;

    if details.name() != "details" {
        return None;
    }

    let mut children = maybe_details.children();
    let maybe_summary = children.next()?;
    let summary = maybe_summary.value().as_element()?;

    if summary.name() != "summary" {
        return None;
    }
    let summary = parse_text_inside(maybe_summary);

    let mut detail = vec![];
    children.for_each(|child| parse_text_outside_to(child, &mut detail));

    Some(Details {
        summary,
        detail: Some(detail),
    })
}
