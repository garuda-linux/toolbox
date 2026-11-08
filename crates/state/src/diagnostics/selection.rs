use super::Collector;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Selection {
    One(Collector),
    All,
}

impl Selection {
    const ALL_ID: &'static str = "all";

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::One(collector) => collector.as_str(),
            Self::All => Self::ALL_ID,
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        if id == Self::ALL_ID {
            return Some(Self::All);
        }

        Collector::ALL
            .into_iter()
            .find(|collector| collector.as_str() == id)
            .map(Self::One)
    }

    pub fn collectors(self) -> Vec<Collector> {
        match self {
            Self::One(collector) => vec![collector],
            Self::All => Collector::ALL.to_vec(),
        }
    }
}
