use std::{ops::Range, ptr};

use crate::{
    awesome_links,
    parser::{self, Annotation, AnnotationTree, MdAnnotation},
};
use anyhow::Result;
use itertools::Itertools;
use range_ops::{RangeOps, Ranges};
use smallvec::SmallVec;
use url::Url;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AwesomeLink {
    pub url: Url,
    pub title: String,
    pub breadcrumbs: SmallVec<[String; 2]>,
    pub description: Option<String>,
    pub link_type: AwesomeLinkType,
    pub source_lines: Range<usize>,
}

impl AwesomeLink {
    pub fn as_github_repo(&self) -> Option<(&str, &str)> {
        if self.url.domain() == Some("github.com") && self.link_type == AwesomeLinkType::Repo {
            let path = self.url.path();
            let mut path = path.split('/').skip(1);
            let owner = path.next()?;
            let repo = path.next()?;
            Some((owner, repo))
        } else {
            None
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, PartialOrd)]
pub enum AwesomeLinkType {
    Repo,
    Article,
    Video,
    Podcast,
    Book,
    Course,
    Other,
}

pub fn extract_awesome_links(md: &str) -> Result<Vec<AwesomeLink>> {
    let (text, anns) = parser::parse_md(&md)?;
    let anns_idx = parser::AnnotationTree::new(&anns);

    let links_anns = anns.iter().filter(|a| a.as_link().is_some()).collect_vec();
    let mut awesome_links = Vec::<AwesomeLink>::with_capacity(links_anns.len());

    for ann in links_anns.iter() {
        let Some(link) = ann.as_link() else {
            continue;
        };
        if link.href.starts_with('#') {
            continue;
        }
        let Some(url) = Url::parse(&link.href).ok() else {
            continue;
        };

        //if this link isn't part of a list item...we're not interested
        //TODO: me probably want to extract a bunch of features and train a model instead

        let Some((li_ann, li)) = anns_idx
            .parent(ann)
            .and_then(|p| p.as_list_item().map(|li| (p, li)))
        else {
            continue;
        };

        if !is_link_first(ann, li_ann, &anns_idx) {
            continue;
        }

        // we also look for any nested lists so we can ignore them from the description
        let first_nested_list = anns_idx
            .children(li_ann)
            .filter(|c| c.as_list().is_some())
            .next();

        let links_prefix = find_links_prefix(li_ann, &text, &anns_idx);

        let link_text = link.text(&text);

        let description = li
            .as_range()
            .sub(0..link.end)
            .sub(
                first_nested_list
                    .map(|a| a.as_range().start..text.len())
                    .unwrap_or(0..0),
            )
            .sub(links_prefix.unwrap_or(0..0));
        let description_text = description.outer_range().map(|r| {
            text[r]
                .trim_start_matches(|c: char| !c.is_alphabetic())
                .trim_end()
        });
        let breadcrumbs = collect_breadcrumbs(li_ann, &text, &anns_idx);
        let source_lines = link.start_line..link.end_line;
        let link_type = infer_link_type(&url, &link_text, description_text.unwrap_or_default());

        //TODO: Filter out irrelevant breadcrumbs (that apply to all links)
        //TODO: Filter out links before the table of contents

        awesome_links.push(AwesomeLink {
            url: url,
            title: link_text.to_string(),
            description: description_text.map(String::from),
            breadcrumbs: breadcrumbs.into(),
            link_type: link_type,
            source_lines,
        });
    }

    Ok(awesome_links)
}

fn collect_breadcrumbs(
    li: &MdAnnotation,
    doc: &str,
    idx: &AnnotationTree<MdAnnotation>,
) -> Vec<String> {
    // Several things can be breadcrumbs
    // 1. First of all there's headers:
    //      some of them will be excluded because they don't add any value
    // 2. Parent list item prefixes

    // let's start with the easy ones, headers
    let sections = idx
        .ancestors(li)
        .flat_map(|a| a.as_heading_section())
        .flat_map(|section| {
            //we convert the section into a heading here
            idx.query(section.start..section.start + 1)
                .flat_map(|a| a.as_heading())
        });

    let heading_breadcrumbs = sections
        .map(|heading| {
            (
                heading.start,
                heading.text(doc).replace("📷", "").trim().to_string(),
            )
        })
        .into_iter()
        .collect_vec();

    heading_breadcrumbs
        .into_iter()
        .sorted_by_key(|&(key, _)| key)
        .map(|(_, v)| v)
        .collect_vec()
}

fn infer_link_type(url: &Url, link_text: &str, description: &str) -> AwesomeLinkType {
    let url_type = match url.domain() {
        Some("github.com") => Some(AwesomeLinkType::Repo),
        Some("youtube.com") => Some(AwesomeLinkType::Video),
        Some("youtu.be") => Some(AwesomeLinkType::Video),
        Some("podcasts.apple.com") => Some(AwesomeLinkType::Podcast),
        Some("www.youtube.com") => Some(AwesomeLinkType::Video),
        Some("www.rust-lang.org") => Some(AwesomeLinkType::Article),
        _ => None,
    };
    match url_type {
        Some(t) => t,
        None => {
            if link_text.contains("book") {
                AwesomeLinkType::Book
            } else if link_text.contains("course") {
                AwesomeLinkType::Course
            } else {
                AwesomeLinkType::Other
            }
        }
    }
}

fn is_link_first(
    link: &MdAnnotation,
    container: &MdAnnotation,
    idx: &AnnotationTree<MdAnnotation>,
) -> bool {
    idx.children(container)
        .next()
        .map(|c| ptr::eq(c, link))
        .unwrap_or_default()
}

fn find_links_prefix(
    ann: &MdAnnotation,
    doc: &str,
    ann_idx: &AnnotationTree<MdAnnotation>,
) -> Option<Range<usize>> {
    let lit = ann.as_list_item().expect("not a list item");
    let li_text = ann.text(doc);

    let prefix_text = li_text.split_terminator(['-', '—'].as_ref()).collect_vec();
    let [prefix_text, _next, remainder @ ..] = prefix_text.as_slice() else {
        return None;
    };

    let prefix_range = ann.as_range().start..ann.as_range().start + prefix_text.len();

    // we have found a prefix bit of text, now we just need to check that the majority of characers are links
    // we do this by taking the union of it and any links overlapping with the list item. Then stripping any
    // punctuation and whitespace characters and hopefully only having 1-2 characters left 👍

    let mut unlinked_prefix_range = Ranges::from(vec![prefix_range.clone()]);
    for link in ann_idx
        .children(&ann)
        .filter(|c| c.as_link().is_some())
        .map(|c| c.as_range())
    {
        unlinked_prefix_range = unlinked_prefix_range.sub(link);
    }

    let non_trivial_chars = unlinked_prefix_range
        .iter()
        .map(|r| &doc[r.clone()])
        .join("")
        .chars()
        .filter(|c| c.is_alphanumeric())
        .count();

    if non_trivial_chars > 2 {
        return None;
    }

    Some(prefix_range)
    // here we try to find a dash such that the majority of the text before it consists of links (excluding punctuation)
    // let alt_link = {
    //     // here we search for links that occur before a dash-like character
    //     let li_links = anns_idx
    //         .children(li_ann)
    //         .filter(|c| c.as_link().is_some())
    //         .collect::<Vec<_>>();
    //     let Some(li_dash) = li_text
    //         .find(|c: char| c == '-' || c == '—') else {
    //             None
    //         };
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_awesome_links() {
        let md = r#"
# Awesome Rust

## Table of Contents

* Item 1
    * Item 2
    * Item 3
"#;

        let (text, annotations) = parser::parse_md(md).unwrap();
    }
}
