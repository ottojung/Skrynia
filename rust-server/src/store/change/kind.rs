#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Kind {
    Create,
    Replace,
    Delete,
}
impl Kind {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "create" => Some(Self::Create),
            "replace" => Some(Self::Replace),
            "delete" => Some(Self::Delete),
            _ => None,
        }
    }
    pub fn event(self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Replace => "replace",
            Self::Delete => "delete",
        }
    }
    pub fn operation(self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Replace => "modify",
            Self::Delete => "delete",
        }
    }
}
#[cfg(test)]
mod tests {
    use super::Kind;
    #[test]
    fn mutation_events_and_grant_operations_are_distinct() {
        assert_eq!(Kind::parse("replace"), Some(Kind::Replace));
        assert_eq!(Kind::Replace.event(), "replace");
        assert_eq!(Kind::Replace.operation(), "modify");
        assert_eq!(Kind::parse("modify"), None);
    }
}
