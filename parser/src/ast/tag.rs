use core::fmt;

use miden_diagnostics::{SourceSpan, Span, Spanned};

/// A constraint tag used to identify constraints across compilation pipelines.
///
/// When any tags are present, `CURRENT_MAX_ID` must be defined in the root module and the tags
/// must cover the full range `0..=CURRENT_MAX_ID` with no duplicates.
pub type ConstraintTag = Span<u64>;

/// A tag specification for constraints which may expand into multiple constraints.
///
/// Tag specs support:
/// - Single tag: `@tag(5)`
/// - Tag range: `@tag_range(10..20)` (half-open, end-exclusive)
/// - Tag list: `@tag([10, 11, 12])`
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ConstraintTagSpec {
    Single(Span<u64>),
    Range {
        span: SourceSpan,
        start: u64,
        end: u64,
        inclusive: bool,
    },
    List {
        span: SourceSpan,
        tags: Vec<u64>,
    },
}

impl ConstraintTagSpec {
    pub fn single(span: SourceSpan, tag: u64) -> Self {
        Self::Single(Span::new(span, tag))
    }

    pub fn range(span: SourceSpan, start: u64, end: u64, inclusive: bool) -> Self {
        Self::Range { span, start, end, inclusive }
    }

    pub fn list(span: SourceSpan, tags: Vec<u64>) -> Self {
        Self::List { span, tags }
    }

    /// Returns the number of tags described by this spec.
    pub fn len(&self) -> usize {
        match self {
            Self::Single(_) => 1,
            Self::Range { start, end, inclusive, .. } => {
                if *inclusive {
                    if end < start { 0 } else { (*end - *start + 1) as usize }
                } else if end <= start {
                    0
                } else {
                    (*end - *start) as usize
                }
            },
            Self::List { tags, .. } => tags.len(),
        }
    }

    /// Returns true if this spec contains no tags.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns true if this spec describes exactly one tag.
    pub fn is_single(&self) -> bool {
        self.len() == 1
    }

    /// Returns the single tag value if this spec contains exactly one tag.
    pub fn as_single(&self) -> Option<u64> {
        match self {
            Self::Single(tag) => Some(tag.item),
            Self::Range { start, .. } => {
                let len = self.len();
                if len == 1 { Some(*start) } else { None }
            },
            Self::List { tags, .. } => {
                if tags.len() == 1 {
                    tags.first().copied()
                } else {
                    None
                }
            },
        }
    }

    /// Returns the span covering this tag spec.
    pub fn span(&self) -> SourceSpan {
        match self {
            Self::Single(tag) => tag.span(),
            Self::Range { span, .. } | Self::List { span, .. } => *span,
        }
    }

    /// Expands the tag spec into a list of tags, each carrying the tag spec span.
    pub fn expand_spans(&self) -> Vec<Span<u64>> {
        let span = self.span();
        match self {
            Self::Single(tag) => vec![*tag],
            Self::Range { start, end, inclusive, .. } => {
                let end_inclusive = if *inclusive { *end } else { end.saturating_sub(1) };
                (*start..=end_inclusive).map(|tag| Span::new(span, tag)).collect()
            },
            Self::List { tags, .. } => {
                tags.iter().copied().map(|tag| Span::new(span, tag)).collect()
            },
        }
    }
}

impl Spanned for ConstraintTagSpec {
    fn span(&self) -> SourceSpan {
        self.span()
    }
}

impl fmt::Display for ConstraintTagSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Single(tag) => write!(f, "@tag({})", tag.item),
            Self::Range { start, end, inclusive, .. } => {
                let op = if *inclusive { "..=" } else { ".." };
                write!(f, "@tag_range({start}{op}{end})")
            },
            Self::List { tags, .. } => {
                f.write_str("@tag([")?;
                for (idx, tag) in tags.iter().enumerate() {
                    if idx > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{tag}")?;
                }
                f.write_str("])")
            },
        }
    }
}
