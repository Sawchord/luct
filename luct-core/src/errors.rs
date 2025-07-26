/// Indicates, whether an error should be treated as an inconclusive result, or an unsafe result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    /// Error does not necessarily hint at malicious behaviour
    ///
    /// This includes things such as version incompatibilities, unsupported signature algorithms
    /// or connection errors.
    Inconclusive,

    /// Error indicates, that malicious behaviour may have occured
    ///
    /// Note that "malicious behaviour" includes spec non-complicance, such as deserialization errors as well as
    /// failing signature verification.
    ///
    ///
    Unsafe,
}

pub trait CheckSeverity {
    fn severity(&self) -> Severity;
}
