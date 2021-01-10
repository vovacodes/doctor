#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DocComment<'a> {
    #[cfg_attr(feature = "serde", serde(borrow))]
    pub description: Option<Description<'a>>,
    pub block_tags: Vec<BlockTag<'a>>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Description<'a> {
    #[cfg_attr(feature = "serde", serde(borrow))]
    pub body_items: Vec<BodyItem<'a>>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BlockTag<'a> {
    pub name: &'a str,
    pub body_items: Vec<BodyItem<'a>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum BodyItem<'a> {
    TextSegment(&'a str),
    InlineTag(InlineTag<'a>),
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct InlineTag<'a> {
    pub name: &'a str,
    pub body_lines: Vec<&'a str>,
}

#[cfg(test)]
mod tests {
    use core::fmt::Debug;
    #[cfg(feature = "serde")]
    use serde::{Deserialize, Serialize};
    use std::hash::Hash;

    use super::*;

    fn assert_default<T: Default>() {}
    fn assert_clone<T: Clone>() {}
    fn assert_debug<T: Debug>() {}
    fn assert_hash<T: Hash>() {}
    fn assert_sync_send<T: Sync + Send>() {}

    #[cfg(feature = "serde")]
    fn assert_serde<'de, T: Serialize + Deserialize<'de>>() {}

    #[test]
    fn test_doc_comment_implement_common_traits() {
        assert_default::<DocComment>();
        assert_clone::<DocComment>();
        assert_debug::<DocComment>();
        assert_hash::<DocComment>();
        assert_sync_send::<DocComment>();

        #[cfg(feature = "serde")]
        assert_serde::<DocComment>()
    }
}
