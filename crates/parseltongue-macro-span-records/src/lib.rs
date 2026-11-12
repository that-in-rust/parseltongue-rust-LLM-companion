//! Safe compatibility records for the pinned token-tree leaf library.
//! Adapted from rust-analyzer span, MIT OR Apache-2.0, commit
//! 6e37b2d62127d445d77069031b416c48c9b0b4f2.

pub use ra_ap_syntax::{Edition, TextRange, TextSize};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FileId(u32);

impl FileId {
    pub const fn from_raw(value: u32) -> Self {
        Self(value)
    }
    pub const fn index(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ErasedFileAstId(u32);

impl ErasedFileAstId {
    pub const fn from_raw(value: u32) -> Self {
        Self(value)
    }
    pub const fn into_raw(self) -> u32 {
        self.0
    }
}

pub const ROOT_ERASED_FILE_AST_ID: ErasedFileAstId = ErasedFileAstId(0);
pub const FIXUP_ERASED_FILE_AST_ID_MARKER: ErasedFileAstId = ErasedFileAstId(u32::MAX);
pub const NO_DOWNMAP_ERASED_FILE_AST_ID_MARKER: ErasedFileAstId = ErasedFileAstId(u32::MAX - 1);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EditionedFileId(u32);

impl EditionedFileId {
    pub const MAX_FILE_ID: u32 = 0x007f_ffff;
    pub const fn new(file: FileId, edition: Edition) -> Self {
        assert!(file.index() <= Self::MAX_FILE_ID);
        Self(file.index() | ((edition as u32) << 23))
    }
    pub const fn current_edition(file: FileId) -> Self {
        Self::new(file, Edition::CURRENT)
    }
    pub const fn from_raw(value: u32) -> Self {
        Self(value)
    }
    pub const fn as_u32(self) -> u32 {
        self.0
    }
    pub const fn file_id(self) -> FileId {
        FileId(self.0 & Self::MAX_FILE_ID)
    }
    pub const fn edition(self) -> Edition {
        read_encoded_edition_value((self.0 >> 23) & 0xff)
    }
}

impl From<EditionedFileId> for FileId {
    fn from(value: EditionedFileId) -> Self {
        value.file_id()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SyntaxContext(u32);

impl SyntaxContext {
    pub const fn root(edition: Edition) -> Self {
        Self(edition as u32)
    }
    pub const fn from_u32(value: u32) -> Self {
        Self(value)
    }
    pub const fn into_u32(self) -> u32 {
        self.0
    }
    pub const fn edition(self) -> Edition {
        read_encoded_edition_value(self.0 & 7)
    }
    pub const fn is_root(self) -> bool {
        self.0 < 8
    }
}

impl std::fmt::Display for SyntaxContext {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, formatter)
    }
}

const fn read_encoded_edition_value(value: u32) -> Edition {
    match value {
        0 => Edition::Edition2015,
        1 => Edition::Edition2018,
        2 => Edition::Edition2021,
        _ => Edition::Edition2024,
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SpanAnchor {
    pub file_id: EditionedFileId,
    pub ast_id: ErasedFileAstId,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Span {
    pub range: TextRange,
    pub anchor: SpanAnchor,
    pub ctx: SyntaxContext,
}

impl Span {
    pub fn cover(self, other: Self) -> Self {
        if self.anchor == other.anchor {
            Self {
                range: self.range.cover(other.range),
                ..self
            }
        } else {
            self
        }
    }
    pub fn eq_ignoring_ctx(self, other: Self) -> bool {
        self.anchor == other.anchor && self.range == other.range
    }
    pub fn join(
        self,
        other: Self,
        differing_anchor: impl FnOnce(Self, Self) -> Option<Self>,
    ) -> Option<Self> {
        if self.anchor.ast_id == FIXUP_ERASED_FILE_AST_ID_MARKER {
            return Some(other);
        }
        if other.anchor.ast_id == FIXUP_ERASED_FILE_AST_ID_MARKER {
            return Some(self);
        }
        if self.anchor != other.anchor {
            return differing_anchor(self, other);
        }
        (self.ctx == other.ctx).then(|| Self {
            range: self.range.cover(other.range),
            ..self
        })
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{}:{:?}@{:?}#{}",
            self.anchor.file_id.file_id().index(),
            self.anchor.ast_id,
            self.range,
            self.ctx
        )
    }
}

#[cfg(test)]
mod span_compatibility_contract_tests {
    use super::*;
    #[test]
    fn test_span_identity_roundtrip() {
        for edition in [
            Edition::Edition2015,
            Edition::Edition2018,
            Edition::Edition2021,
            Edition::Edition2024,
        ] {
            for file in [0, 1, EditionedFileId::MAX_FILE_ID] {
                let identity = EditionedFileId::new(FileId::from_raw(file), edition);
                assert_eq!(identity.file_id().index(), file);
                assert_eq!(identity.edition(), edition);
                assert_eq!(EditionedFileId::from_raw(identity.as_u32()), identity);
                assert_eq!(SyntaxContext::root(edition).edition(), edition);
            }
        }
    }
    #[test]
    fn test_span_join_boundaries() {
        let initial = Span {
            range: TextRange::new(0.into(), 1.into()),
            anchor: SpanAnchor {
                file_id: EditionedFileId::new(FileId::from_raw(0), Edition::Edition2024),
                ast_id: ROOT_ERASED_FILE_AST_ID,
            },
            ctx: SyntaxContext::root(Edition::Edition2024),
        };
        let next = Span {
            range: TextRange::new(2.into(), 3.into()),
            ..initial
        };
        assert_eq!(
            initial.join(next, |_, _| None).map(|span| span.range),
            Some(TextRange::new(0.into(), 3.into()))
        );
        let foreign = Span {
            anchor: SpanAnchor {
                file_id: EditionedFileId::new(FileId::from_raw(1), Edition::Edition2024),
                ..initial.anchor
            },
            ..next
        };
        assert_eq!(initial.join(foreign, |_, _| None), None);
        let fixup = Span {
            anchor: SpanAnchor {
                ast_id: FIXUP_ERASED_FILE_AST_ID_MARKER,
                ..initial.anchor
            },
            ..initial
        };
        assert_eq!(fixup.join(foreign, |_, _| None), Some(foreign));
        assert_eq!(foreign.join(fixup, |_, _| None), Some(foreign));
        assert_eq!(
            initial.join(
                Span {
                    ctx: SyntaxContext::root(Edition::Edition2021),
                    ..next
                },
                |_, _| None
            ),
            None
        );
    }
}
