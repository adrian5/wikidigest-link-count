/*
 * Small utility struct for counting direct and indirect (redirect) links.
 */
use std::cmp::Ordering;
use std::ops::AddAssign;

#[derive(Clone, Copy, Debug, Default)]
pub struct LinkCount {
    pub direct: u32,
    pub indirect: u32,
}

impl LinkCount {
    pub fn new(direct: u32, indirect: u32) -> Self {
        Self { direct, indirect }
    }

    pub fn total(self) -> u32 {
        self.direct + self.indirect
    }
}

impl AddAssign for LinkCount {
    fn add_assign(&mut self, other: LinkCount) {
        *self = Self {
            direct: self.direct + other.direct,
            indirect: self.indirect + other.indirect,
        };
    }
}

impl Ord for LinkCount {
    fn cmp(&self, other: &LinkCount) -> Ordering {
        (self.total()).cmp(&other.total())
    }
}

impl PartialOrd for LinkCount {
    fn partial_cmp(&self, other: &LinkCount) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for LinkCount {}

impl PartialEq for LinkCount {
    fn eq(&self, other: &LinkCount) -> bool {
        (self.direct + self.indirect) == (other.direct + other.indirect)
    }
}
