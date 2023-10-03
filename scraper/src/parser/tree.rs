use std::{ops::Range, ptr};

use intervaltree::IntervalTree;

use super::Annotation;

use itertools::*;

pub struct AnnotationTree<'a, T> {
    tree: IntervalTree<usize, &'a T>,
}

impl<'a, T> AnnotationTree<'a, T> {
    pub fn new(data: &'a Vec<T>) -> Self
    where
        T: Annotation,
    {
        let tree = IntervalTree::from_iter(data.into_iter().map(|x| (x.as_range(), x)));

        Self { tree }
    }

    pub fn query(&self, range: Range<usize>) -> impl Iterator<Item = &'a T> + '_ {
        self.tree.query(range).map(|el| el.value)
    }

    pub fn ancestors(&self, cur: &'a T) -> impl Iterator<Item = &'a T> + '_
    where
        T: Annotation,
    {
        let res = self
            .tree
            .query(cur.as_range())
            .filter(|el| !ptr::eq(el.value, cur))
            .filter(|el| {
                el.range.start <= cur.as_range().start
                    && el.range.end >= cur.as_range().end
                    && cur.depth() > el.value.depth()
            })
            .sorted_by_key(|el| {
                (
                    -1 * el.value.depth() as i32,
                    -1 * el.range.start as i32,
                    -1 * el.range.end as i32,
                )
            })
            .map(|el| el.value);

        res
    }

    pub fn descendants(&self, cur: &'a T) -> impl Iterator<Item = &'a T> + '_
    where
        T: Annotation,
    {
        let res = self
            .tree
            .query(cur.as_range())
            .filter(|el| !ptr::eq(el.value, cur))
            .filter(|el| {
                el.range.start >= cur.as_range().start
                    && el.range.end <= cur.as_range().end
                    && cur.depth() < el.value.depth()
            })
            .sorted_by_key(|el| (el.value.depth(), el.range.start, el.range.end))
            .map(|el| el.value);

        res
    }

    pub fn parent(&self, cur: &'a T) -> Option<&'a T>
    where
        T: Annotation,
    {
        self.ancestors(cur)
            .filter(|v| v.depth() == cur.depth() - 1)
            .next()
    }

    pub fn children(&self, cur: &'a T) -> impl Iterator<Item = &'a T> + '_
    where
        T: Annotation,
    {
        self.descendants(cur)
            .filter(|v| v.depth() == cur.depth() + 1)
    }

    pub fn siblings(&self, cur: &'a T) -> impl Iterator<Item = &'a T> + '_
    where
        T: Annotation,
    {
        self.parent(cur)
            .into_iter()
            .flat_map(|p| self.children(p).filter(|v| !ptr::eq(*v, cur)))
    }

    pub fn before(&self, cur: &'a T) -> impl Iterator<Item = &'a T> + '_
    where
        T: Annotation,
    {
        self.siblings(cur)
            .filter(|v| v.as_range().end <= cur.as_range().start)
    }

    pub fn after(&self, cur: &'a T) -> impl Iterator<Item = &'a T> + '_
    where
        T: Annotation,
    {
        self.siblings(cur)
            .filter(|v| v.as_range().start >= cur.as_range().end)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fmt::{Display, Formatter},
        ops::Range,
    };

    use super::*;

    #[derive(Debug)]
    struct TestAnnotation {
        id: &'static str,
        start: usize,
        end: usize,
        depth: usize,
    }

    impl Display for TestAnnotation {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            let r = self.as_range();
            write!(f, "{}:{}-{}", self.id, r.start, r.end)
        }
    }

    impl Annotation for TestAnnotation {
        fn as_range(&self) -> Range<usize> {
            self.start..self.end
        }

        fn depth(&self) -> usize {
            self.depth
        }
    }

    #[test]
    fn test() {
        let data = vec![
            TestAnnotation {
                id: "body",
                start: 0,
                end: 10,
                depth: 0,
            },
            TestAnnotation {
                id: "paragraph1",
                start: 0,
                end: 10,
                depth: 1,
            },
            TestAnnotation {
                id: "link",
                start: 2,
                end: 8,
                depth: 3,
            },
            TestAnnotation {
                id: "icon",
                start: 2,
                end: 3,
                depth: 4,
            },
            TestAnnotation {
                id: "icon",
                start: 3,
                end: 4,
                depth: 4,
            },
            TestAnnotation {
                id: "icon",
                start: 4,
                end: 5,
                depth: 4,
            },
        ];
        let tree = AnnotationTree::new(&data);
        let ancestors = tree
            .ancestors(&data[3])
            .map(|v| v.to_string())
            .collect_vec();
        assert_eq!(ancestors, vec!["link:2-8", "paragraph1:0-10", "body:0-10"]);

        let descendants = tree
            .descendants(&data[1])
            .map(|v| v.to_string())
            .collect_vec();
        assert_eq!(
            descendants,
            vec!["link:2-8", "icon:2-3", "icon:3-4", "icon:4-5"]
        );

        let children = tree.children(&data[0]).map(|v| v.to_string()).collect_vec();
        assert_eq!(children, vec!["paragraph1:0-10"]);

        assert_eq!(tree.parent(&data[3]).unwrap().to_string(), "link:2-8");

        let siblings = tree.siblings(&data[4]).map(|v| v.to_string()).collect_vec();
        assert_eq!(siblings, vec!["icon:2-3", "icon:4-5"]);

        let before = tree.before(&data[4]).map(|v| v.to_string()).collect_vec();
        assert_eq!(before, vec!["icon:2-3"]);

        let after = tree.after(&data[4]).map(|v| v.to_string()).collect_vec();
        assert_eq!(after, vec!["icon:4-5"]);

        assert!(tree.parent(&data[0]).is_none());
        assert!(tree.children(&data[3]).next().is_none());
    }
}
