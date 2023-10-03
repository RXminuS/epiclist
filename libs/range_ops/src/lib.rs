#[macro_use]
extern crate shrinkwraprs;

use itertools::Itertools;
use smallvec::{smallvec, SmallVec};
use std::ops::Range;

//TODO: Make generic in IDX type...I couldn't be bothered for now
//as it makes all the interface defintions much harder to read
#[derive(Shrinkwrap, PartialEq, Clone, Debug)]
pub struct Ranges(SmallVec<[Range<usize>; 4]>);

// #[derive(Shrinkwrap, RefCast)]
// #[shrinkwrap(mutable)]
// #[repr(transparent)]
// struct RangeExt(Range<usize>);
pub trait RangeOps {
    fn sub(&self, other: Range<usize>) -> Ranges;
}

impl Ranges {
    pub fn outer_range(&self) -> Option<Range<usize>> {
        if self.0.is_empty() {
            None
        } else {
            Some(self.first().unwrap().start..self.last().unwrap().end)
        }
    }

    pub fn union_of(ranges: &[Range<usize>]) -> Self {
        let sorted_ranges = ranges
            .into_iter()
            .sorted_by_key(|r| (r.start, r.end))
            .filter(|r| !r.is_empty());
        let mut union: SmallVec<[Range<usize>; 4]> = smallvec![];
        for range in sorted_ranges {
            if let Some(last) = union.last_mut() {
                if last.end >= range.start {
                    last.end = last.end.max(range.end);
                    continue;
                }
            }
            union.push(range.clone());
        }
        Ranges(union)
    }
}

// impl From<Range<usize>> for RangeExt {
//     fn from(value: Range<usize>) -> Self {
//         RangeExt(value)
//     }
// }

// impl<'a> From<&'a Range<usize>> for &'a RangeExt {
//     fn from(value: &'a Range<usize>) -> Self {
//         //use refacast here
//         RangeExt::ref_cast(value)
//     }
// }

impl RangeOps for Range<usize> {
    fn sub(&self, other: Range<usize>) -> Ranges {
        let mut ranges: SmallVec<[Range<usize>; 2]> = smallvec![];

        if self.start < other.start {
            ranges.push(self.start..other.start.min(self.end));
        }
        if self.end > other.end {
            ranges.push(other.end.max(self.start)..self.end);
        }
        Ranges::union_of(&ranges)
    }
}

impl From<Range<usize>> for Ranges {
    fn from(r: Range<usize>) -> Self {
        Ranges(smallvec![r])
    }
}

impl RangeOps for Ranges {
    fn sub(&self, other: Range<usize>) -> Ranges {
        let mut ranges: SmallVec<[Range<usize>; 8]> = smallvec![];
        for range in self.0.iter() {
            ranges.extend(range.sub(other.clone()).0);
        }
        Ranges::union_of(&ranges)
    }
}

impl From<Vec<Range<usize>>> for Ranges {
    fn from(r: Vec<Range<usize>>) -> Self {
        Ranges(r.into())
    }
}
// impl Sub

// impl From<Range<usize>> for Ranges {
//     fn from(r: Range<usize>) -> Self {
//         Ranges(smallvec![r])
//     }
// }

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic_subtractions() {
        assert_eq!((0..10).sub(4..5), Ranges::from(vec![0..4, 5..10]));
        assert_eq!((0..10).sub(0..5), Ranges::from(vec![5..10]));
        assert_eq!((0..10).sub(5..10), Ranges::from(vec![0..5]));
        assert_eq!((0..10).sub(0..10), Ranges::from(vec![]));
        assert_eq!((0..10).sub(0..0), Ranges::from(vec![0..10]));
        assert_eq!((0..10).sub(5..5), Ranges::from(vec![0..10]));

        assert_eq!(
            (0..10).sub(4..5).sub(6..7),
            Ranges::from(vec![0..4, 5..6, 7..10])
        );
    }
}
