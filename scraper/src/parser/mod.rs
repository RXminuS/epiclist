mod anns;
mod err;
mod tree;

pub use anns::*;
pub use tree::*;

use err::Result;
use lazy_static::lazy_static;
use pulldown_cmark::{Options as CmarkOptions, Parser};

use self::err::ParserError;

lazy_static! {
    static ref CMARK_OPTIONS: CmarkOptions = CmarkOptions::from_bits_truncate(
        CmarkOptions::ENABLE_STRIKETHROUGH.bits() | CmarkOptions::ENABLE_TABLES.bits()
    );
}

pub fn parse_md(md: &str) -> Result<(String, Vec<MdAnnotation>)> {
    let parser = Parser::new_ext(md, *CMARK_OPTIONS);

    let mut text: String = String::new();
    let mut annotations: Vec<MdAnnotation> = Vec::new();
    let mut open_annotations: Vec<MdAnnotationBuilder> = Vec::new();
    let mut indentation_depth = 0;
    let mut ignore_content = 0;

    for (event, range) in parser.into_offset_iter() {
        match event {
            pulldown_cmark::Event::Start(tag) => match tag {
                pulldown_cmark::Tag::Image(_, _, _) => {
                    if ignore_content == 0 {
                        text.push_str("📷");
                    }
                    ignore_content += 1;
                }
                pulldown_cmark::Tag::Paragraph => {
                    upsert_newline(&mut text);
                    open_annotations.push(
                        MdParagraphBuilder::default()
                            .start(text.len())
                            .start_line(line_offset(md, range.start))
                            .depth(open_annotations.len())
                            .into(),
                    );
                }
                pulldown_cmark::Tag::List(_) => {
                    upsert_newline(&mut text);
                    open_annotations.push(
                        MdListBuilder::default()
                            .start(text.len())
                            .start_line(line_offset(md, range.start))
                            .depth(open_annotations.len())
                            .into(),
                    );
                    indentation_depth += 1;
                }
                pulldown_cmark::Tag::Item => {
                    upsert_newline(&mut text);
                    open_annotations.push(
                        MdListItemBuilder::default()
                            .start(text.len())
                            .start_line(line_offset(md, range.start))
                            .depth(open_annotations.len())
                            .into(),
                    );
                    if ignore_content == 0 {
                        text.push_str(&"\t".repeat(indentation_depth));
                        text.push_str("• ");
                    }
                }
                pulldown_cmark::Tag::Heading(level, _, _) => {
                    upsert_newline(&mut text);
                    open_annotations.push(
                        MdHeadingBuilder::default()
                            .level(level as u8)
                            .start(text.len())
                            .start_line(line_offset(md, range.start))
                            .depth(open_annotations.len())
                            .into(),
                    );
                }
                pulldown_cmark::Tag::Link(link_type, href, title) => {
                    open_annotations.push(
                        MdLinkBuilder::default()
                            .href(href.to_string())
                            .title({
                                if title.is_empty() {
                                    None
                                } else {
                                    Some(title.to_string())
                                }
                            })
                            .start(text.len())
                            .start_line(line_offset(md, range.start))
                            .depth(open_annotations.len())
                            .into(),
                    );
                }
                _ => {}
            },
            pulldown_cmark::Event::End(tag) => match tag {
                pulldown_cmark::Tag::Image(_, _, _) => {
                    ignore_content -= 1;
                }
                pulldown_cmark::Tag::Heading(_, _, _) => {
                    let ann = open_annotations
                        .pop()
                        .ok_or(ParserError::MismatchedTags)?
                        .into_heading()
                        .map_err(|_| ParserError::MismatchedTags)?
                        .end(text.len())
                        .end_line(line_offset(md, range.end) + 1)
                        .build()?;
                    annotations.push(ann.into());
                }
                pulldown_cmark::Tag::Paragraph => {
                    let ann = open_annotations
                        .pop()
                        .ok_or(ParserError::MismatchedTags)?
                        .into_paragraph()
                        .map_err(|_| ParserError::MismatchedTags)?
                        .end(text.len())
                        .end_line(line_offset(md, range.end) + 1)
                        .build()?;
                    annotations.push(ann.into());
                }
                pulldown_cmark::Tag::List(_) => {
                    let ann = open_annotations
                        .pop()
                        .ok_or(ParserError::MismatchedTags)?
                        .into_list()
                        .map_err(|_| ParserError::MismatchedTags)?
                        .end(text.len())
                        .end_line(line_offset(md, range.end) + 1)
                        .build()?;
                    annotations.push(ann.into());
                    indentation_depth -= 1;
                }
                pulldown_cmark::Tag::Item => {
                    let ann = open_annotations
                        .pop()
                        .ok_or(ParserError::MismatchedTags)?
                        .into_list_item()
                        .map_err(|_| ParserError::MismatchedTags)?
                        .end(text.len())
                        .end_line(line_offset(md, range.end) + 1)
                        .build()?;
                    annotations.push(ann.into());
                }
                pulldown_cmark::Tag::Link(_, _, _) => {
                    let ann = open_annotations
                        .pop()
                        .ok_or(ParserError::MismatchedTags)?
                        .into_link()
                        .map_err(|_| ParserError::MismatchedTags)?
                        .end(text.len())
                        .end_line(line_offset(md, range.end) + 1)
                        .build()?;
                    annotations.push(ann.into());
                }
                _ => {}
            },
            pulldown_cmark::Event::Text(s) => {
                if ignore_content == 0 {
                    text.push_str(&s);
                }
            }
            _ => {}
        }
    }

    //we now create a couple of "meta" annotations that we can only do
    //once all actual annotations are in place
    let mut heading_sections: Vec<MdHeadingSection> = Vec::new();
    let mut open_sections: Vec<(MdHeadingSectionBuilder, &MdHeading)> = Vec::new();
    for heading in annotations.iter().flat_map(|ann| ann.as_heading()) {
        while let Some((open_section, open_heading)) = open_sections.pop() {
            if heading.level <= open_heading.level {
                heading_sections.push(
                    open_section
                        .end(heading.start)
                        .end_line(heading.start_line)
                        .build()?,
                );
            } else {
                open_sections.push((open_section, open_heading));
                break;
            }
        }

        open_sections.push((
            MdHeadingSectionBuilder::default()
                .depth(heading.depth)
                .start(heading.start)
                .start_line(heading.start_line)
                .into(),
            heading,
        ))
    }
    let last_line = text.lines().count();
    let last_char = text.len();
    for (section, _) in open_sections {
        heading_sections.push(section.end(last_char).end_line(last_line).build()?);
    }

    annotations.extend(heading_sections.into_iter().map(|ann| ann.into()));

    annotations.sort_by_key(|ann| (ann.as_range().start, ann.depth(), ann.as_range().end));
    Ok((text, annotations))
}

fn upsert_newline(s: &mut String) {
    if !s.is_empty() && !s.ends_with('\n') {
        s.push('\n');
    }
}

//TODO: Use This
fn line_offset(hay: &str, needle: usize) -> usize {
    hay[..needle].lines().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_md() {
        let md = r#"
# Awesome Rust [![build badge](https://github.com/rust-unofficial/awesome-rust/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/rust-unofficial/awesome-rust/actions/workflows/rust.yml) [![Track Awesome List](https://www.trackawesomelist.com/badge.svg)](https://www.trackawesomelist.com/rust-unofficial/awesome-rust/)

## Items

- Item 1
    - Item 1.1
    - Item 1.2
- Item 2

## Other Items

- Item 1
    - Item 1.1
    - Item 1.2
- Item 2
    - Item 2.1
    - Item 2.2
"#;

        let (text, annotations) = parse_md(md).unwrap();

        println!("{} {:?}", text, annotations);
    }
}
