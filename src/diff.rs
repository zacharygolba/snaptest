use std::fmt::{self, Display, Formatter};
use std::iter::FromIterator;

#[derive(Debug)]
pub enum Diff<'a> {
    Lines(Vec<Line<'a>>),
    Same,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Line<'a> {
    Ne(&'a str, &'a str),
    Eq(&'a str),
}

impl<'a> Diff<'a> {
    pub fn new(left: &'a str, right: &'a str) -> Diff<'a> {
        if left == right {
            return Diff::Same;
        }

        left.lines()
            .zip(right.lines())
            .map(|(l, r)| Line::new(l, r))
            .collect()
    }
}

impl<'a> FromIterator<Line<'a>> for Diff<'a> {
    fn from_iter<I>(iter: I) -> Diff<'a>
    where
        I: IntoIterator<Item = Line<'a>>,
    {
        Diff::Lines(Vec::from_iter(iter))
    }
}

impl<'a> Line<'a> {
    pub fn new(left: &'a str, right: &'a str) -> Line<'a> {
        if left == right {
            Line::Eq(left)
        } else {
            Line::Ne(left, right)
        }
    }
}

impl<'a> Display for Line<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Line::Eq(value) => write!(f, "  {}", dimmed!("{}", value)),
            Line::Ne(left, right) => {
                writeln!(f, "{} {}", green!("-"), green!("{}", left))?;
                write!(f, "{} {}", red!("+"), red!("{}", right))
            }
        }
    }
}
