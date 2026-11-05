use data::LinkReference;

/// Failures when converting or walking a store-backed sequence.
#[derive(Debug, thiserror::Error)]
pub enum SequenceError<T: LinkReference> {
    /// An underlying store operation failed, including a missing link.
    #[error(transparent)]
    Store(#[from] crate::Error<T>),
    /// The unsigned magnitude cannot fit in the external reference range.
    #[error("raw number magnitude {0} exceeds MAX / 2")]
    RawNumberOutOfRange(T),
    /// This store does not recognize the encoded code unit as external.
    #[error("Unicode conversion requires an external reference range for code units")]
    ExternalReferencesDisabled,
    /// A decoded payload cannot be represented as a UTF-16 code unit.
    #[error("payload {0} cannot be represented as a UTF-16 code unit")]
    CodeUnitOutOfRange(T),
    /// The address type is too narrow to hold the input code unit.
    #[error("UTF-16 code unit {0} does not fit the address type")]
    CodeUnitDoesNotFit(u16),
    /// The symbol does not satisfy its configured criterion.
    #[error("link {0} is not a Unicode symbol")]
    NotUnicodeSymbol(T),
    /// The sequence does not satisfy its configured criterion.
    #[error("link {0} is not a Unicode sequence")]
    NotUnicodeSequence(T),
    /// A nonterminal link references an ancestor during traversal.
    #[error("cycle at sequence link {0}")]
    CyclicSequence(T),
    /// The code units contain an unpaired surrogate, invalid in a Rust string.
    #[error(transparent)]
    InvalidUtf16(#[from] std::string::FromUtf16Error),
}
