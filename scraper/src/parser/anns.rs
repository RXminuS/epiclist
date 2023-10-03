use enum_as_inner::EnumAsInner;
use enum_delegate;
use enum_variant_macros::*;
use reusable::{reusable, reuse};
use smallvec::SmallVec;
use std::{borrow::Cow, ops::Range};
use trait_gen::trait_gen;

#[reusable(annotation)]
#[derive(Debug)]
pub struct BaseAnnotation {
    pub start: usize,
    pub end: usize,
    pub depth: usize,
}

#[reusable(src_position)]
#[derive(Debug)]
pub struct SourcePosition {
    pub start_line: usize,
    pub end_line: usize,
}

#[derive(Debug, EnumAsInner)]
#[enum_delegate::implement(Annotation)]
pub enum MdAnnotation {
    Paragraph(MdParagraph),
    Heading(MdHeading),
    HeadingSection(MdHeadingSection),
    Link(MdLink),
    List(MdList),
    ListItem(MdListItem),
    Emphasis(MdEmphasis),
    Strong(MdStrong),
    Strikethrough(MdStrikethrough),
}

#[derive(Debug, EnumAsInner, FromVariants)]
pub enum MdAnnotationBuilder {
    Paragraph(MdParagraphBuilder),
    Heading(MdHeadingBuilder),
    HeadingSection(MdHeadingSectionBuilder),
    Link(MdLinkBuilder),
    List(MdListBuilder),
    ListItem(MdListItemBuilder),
    Emphasis(MdEmphasisBuilder),
    Strong(MdStrongBuilder),
    Strikethrough(MdStrikethroughBuilder),
}

#[enum_delegate::register]
pub trait Annotation {
    fn as_range(&self) -> Range<usize>;
    fn depth(&self) -> usize;
    fn text<'a>(&self, doc: &'a str) -> &'a str {
        &doc[self.as_range()]
    }
}

#[reuse(annotation, src_position)]
#[derive(Debug, Builder)]
#[builder(pattern = "owned", derive(Debug))]
pub struct MdHeading {
    pub level: u8,
}

#[reuse(annotation, src_position)]
#[derive(Debug, Builder)]
#[builder(pattern = "owned", derive(Debug))]
pub struct MdHeadingSection {}

#[reuse(annotation, src_position)]
#[derive(Debug, Builder)]
#[builder(pattern = "owned", derive(Debug))]
pub struct MdParagraph {}

#[reuse(annotation, src_position)]
#[derive(Debug, Builder)]
#[builder(pattern = "owned", derive(Debug))]
pub struct MdLink {
    pub href: String,
    pub title: Option<String>,
}

#[reuse(annotation, src_position)]
#[derive(Debug, Builder)]
#[builder(pattern = "owned", derive(Debug))]
pub struct MdList {}

#[reuse(annotation, src_position)]
#[derive(Debug, Builder)]
#[builder(pattern = "owned", derive(Debug))]
pub struct MdListItem {}

#[reuse(annotation, src_position)]
#[derive(Debug, Builder)]
#[builder(pattern = "owned", derive(Debug))]
pub struct MdEmphasis {}
#[reuse(annotation, src_position)]
#[derive(Debug, Builder)]
#[builder(pattern = "owned", derive(Debug))]
pub struct MdStrong {}
#[reuse(annotation, src_position)]
#[derive(Debug, Builder)]
#[builder(pattern = "owned", derive(Debug))]
pub struct MdStrikethrough {}

#[trait_gen(T -> MdList, MdListItem, MdParagraph, MdHeading, MdHeadingSection, MdLink, MdEmphasis, MdStrong, MdStrikethrough)]
impl Annotation for T {
    fn as_range(&self) -> Range<usize> {
        self.start..self.end
    }

    fn depth(&self) -> usize {
        self.depth
    }
}
