//! Tax categories, exemption codes, document types and rounding regimes shared
//! by **both sides** of an accounting ledger.
//!
//! # Why this crate exists
//!
//! Accounts receivable and accounts payable differ in almost everything: who is
//! owed, which way the tax runs, whether the tax is recoverable, which account it
//! lands in. They agree **exactly** on the vocabulary — the UNCL5305 category
//! codes, the CEF `VATEX-*` exemption reasons, which categories must carry a
//! zero rate, and the rules about which combinations are legal.
//!
//! That agreement is not a coincidence to be re-derived twice. Duplicating the
//! table is how a purchase invoice ends up filed under a code the validator
//! rejects on the other side of the business, and it is invisible until a
//! customer or a tax authority rejects a document that was internally
//! consistent.
//!
//! So this crate holds the shared part and nothing else: no amounts, no
//! documents, no storage. It is a table plus the arithmetic policy, and both
//! sides import it.
//!
//! # The four things that live here, and why
//!
//! 1. **[`TaxCategory`]** — UNCL5305, with its separate CEF reason code. Two
//!    independent vocabularies on every taxable line; conflating them is a
//!    rejection cause.
//! 2. **[`DocumentType`]** — UNTDID 1001. Shared because both a sales invoice and
//!    a purchase invoice can be a credit note, and the codes are the same
//!    vocabulary.
//! 3. **[`RoundingPolicy`]** — because **four jurisdictions mandate mutually
//!    incompatible arithmetic** and no default is safe. See that type's docs.
//! 4. **[`IssueReason`]** — because ZATCA BR-KSA-17 requires one of five reasons
//!    on any credit or debit note, and EN 16931 has no field for it at all. The
//!    asymmetry between the two is the reason a field has to exist here.

pub mod tax;

pub use tax::{TaxCategory, is_all_or_none_reverse_charge};

/// How to resolve an exact tie in a rounding decision.
///
/// Owned here rather than borrowed from a ledger crate, because it is *policy*
/// vocabulary and not arithmetic machinery: the ledger has an `RoundingMode` that
/// does the same four things, but a table of tax codes has no business depending
/// on a double-entry engine to express "round ties away from zero".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TieMode {
    /// Round to the nearest representable value; ties go to the even digit.
    /// ISO 80000-1 rule A, ASTM E29, IEEE 754's default.
    HalfEven,
    /// Round to the nearest representable value; ties go away from zero.
    /// ZATCA E-Invoicing vF §10.
    HalfUp,
    /// Truncate toward zero.
    TowardZero,
    /// Round away from zero.
    AwayFromZero,
}

impl TieMode {
    /// On an exact tie, does the value round away from the candidate below?
    ///
    /// `lower_is_even` says whether the candidate below the tie is even.
    pub fn takes_upper_on_tie(self, lower_is_even: bool) -> bool {
        match self {
            // Half-even takes the even neighbour, so it moves up only when the
            // lower candidate is odd.
            Self::HalfEven => !lower_is_even,
            Self::HalfUp | Self::AwayFromZero => true,
            Self::TowardZero => false,
        }
    }
}

/// The UNTDID 1001 document type code.
///
/// Direction is carried **here**, not by a sign, on both the AR and AP sides.
/// See [`DocumentType::increases_receivable`] for the AR reading and the note on
/// AP below.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DocumentType {
    /// 380 — an ordinary invoice.
    Invoice,
    /// 381 — a credit note. Reduces what the counterparty owes.
    CreditNote,
    /// 383 — a debit note. Increases what the counterparty owes.
    DebitNote,
    /// 384 — a corrected invoice. **Jurisdiction-locked**: Peppol only permits
    /// this code when both parties are German organisations, and the Netherlands
    /// prefers it where Peppol defaults to 381.
    Corrected,
    /// 386 — a prepayment invoice. EU Directive 2006/112/EC Article 220(4)
    /// requires an invoice for *any* payment on account before the supply.
    Prepayment,
    /// 389 — a self-billed invoice.
    SelfBilled,
}

impl DocumentType {
    /// The UNTDID 1001 code, which is what appears on the wire.
    pub fn code(self) -> &'static str {
        match self {
            Self::Invoice => "380",
            Self::CreditNote => "381",
            Self::DebitNote => "383",
            Self::Corrected => "384",
            Self::Prepayment => "386",
            Self::SelfBilled => "389",
        }
    }

    /// Whether this document type **increases** what the *seller* owes the buyer.
    ///
    /// On the AR side that is an increase in the receivable. On the AP side it is
    /// an increase in the payable — and that is the trap: a purchase credit note
    /// (381) *reduces* what we owe, and the code is the same, so this method's
    /// name has to be read as "increases the counterparty's obligation to pay".
    pub fn increases_receivable(self) -> bool {
        matches!(self, Self::Invoice | Self::DebitNote | Self::Prepayment)
    }

    /// Whether this document type may exist without referencing another.
    ///
    /// EU Directive 2006/112/EC Article 219: *"Any document or message that amends
    /// and refers specifically and unambiguously to the initial invoice shall be
    /// treated as an invoice."* Both conditions are conjunctive, so a document
    /// that amends without referring unambiguously is **not** treated as an
    /// invoice — it cannot correct the original and the original's input tax
    /// stands. Applies identically to a supplier credit note, which is why the
    /// rule lives in the shared crate.
    pub fn requires_reference(self) -> bool {
        matches!(self, Self::CreditNote | Self::DebitNote | Self::Corrected)
    }
}

/// Why a corrective document was issued.
///
/// ZATCA BR-KSA-17 makes this **mandatory** on a credit or debit note, with five
/// permitted reasons drawn from Article 54 of its VAT Implementing Regulation.
/// EN 16931 has no such field at all, which is precisely why the field has to
/// exist: the strictest common jurisdiction requires it and the standard does not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IssueReason {
    /// Cancellation or suspension of the supply.
    CancelledOrSuspended,
    /// A material change in the nature of the supply changing the VAT due.
    NatureOfSupplyChanged,
    /// Amendment of a pre-agreed value.
    PreAgreedValueAmended,
    /// Return of goods or services.
    GoodsOrServicesReturned,
    /// A change to the seller's or buyer's details.
    PartyDetailsChanged,
}

impl IssueReason {
    /// The stable wire code.
    pub fn code(self) -> &'static str {
        match self {
            Self::CancelledOrSuspended => "01",
            Self::NatureOfSupplyChanged => "02",
            Self::PreAgreedValueAmended => "03",
            Self::GoodsOrServicesReturned => "04",
            Self::PartyDetailsChanged => "05",
        }
    }

    /// The five permitted reasons are a closed list, so a sixth cannot be
    /// invented at a call site.
    pub const ALL: [Self; 5] = [
        Self::CancelledOrSuspended,
        Self::NatureOfSupplyChanged,
        Self::PreAgreedValueAmended,
        Self::GoodsOrServicesReturned,
        Self::PartyDetailsChanged,
    ];
}

/// Which jurisdiction's arithmetic a document is computed under.
///
/// Persisted on the document because the answer must not change retroactively:
/// an invoice recomputed under a different policy than it was issued under is a
/// different document, and the difference is cents on real statements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RoundingPolicy {
    /// EN 16931 / Peppol §9: round each line net to two decimals, sum rounded
    /// values, compute VAT per (category, rate) group, never re-round.
    En16931Group,
    /// Australian GST Act s9-90, total-invoice rule: add unrounded GST per
    /// taxable supply, round the total once.
    GstTotalInvoice,
    /// Australian GST Act s9-90, taxable-supply rule: round GST per supply to
    /// the recorded precision, then sum and round.
    GstTaxableSupply,
    /// UK VAT Notice 700 §17.5: below half a penny down, at or above up.
    HmrcSeventeenFive,
}

impl RoundingPolicy {
    /// The tie-breaking direction this policy mandates.
    ///
    /// Required explicitly because EN 16931 says nothing, and the two obvious
    /// platform defaults disagree with each other: PostgreSQL `numeric` rounds
    /// ties away from zero, `double precision` rounds ties to even, and neither
    /// is HMRC's "round up at half a penny".
    pub fn tie_mode(self) -> TieMode {
        match self {
            Self::En16931Group | Self::GstTotalInvoice | Self::GstTaxableSupply => TieMode::HalfUp,
            Self::HmrcSeventeenFive => TieMode::AwayFromZero,
        }
    }
}

/// A reference to another document, by number and issue date.
///
/// "Specifically and unambiguously" is the Article 219 phrase, so the number
/// **and** the date are both required: a bare number is ambiguous across
/// numbering series, which UK VAT law explicitly permits.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DocumentReference {
    /// The referenced document's number, including its series.
    pub number: String,
    /// The referenced document's issue date, `YYYY-MM-DD`.
    pub issue_date: String,
}

impl DocumentReference {
    /// A reference, which is only unambiguous if both parts are present.
    pub fn new(number: impl Into<String>, issue_date: impl Into<String>) -> Self {
        Self {
            number: number.into(),
            issue_date: issue_date.into(),
        }
    }

    /// Whether this reference is specific and unambiguous under Article 219.
    ///
    /// Parallel numbering series are explicitly permitted, so a bare number does
    /// not identify a document.
    pub fn is_unambiguous(&self) -> bool {
        !self.number.trim().is_empty() && !self.issue_date.trim().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_type_codes_are_the_untdid_1001_values() {
        for (ty, code) in [
            (DocumentType::Invoice, "380"),
            (DocumentType::CreditNote, "381"),
            (DocumentType::DebitNote, "383"),
            (DocumentType::Corrected, "384"),
            (DocumentType::Prepayment, "386"),
            (DocumentType::SelfBilled, "389"),
        ] {
            assert_eq!(ty.code(), code);
        }
    }

    #[test]
    fn corrective_documents_need_a_reference_and_ordinary_ones_do_not() {
        for ty in [
            DocumentType::CreditNote,
            DocumentType::DebitNote,
            DocumentType::Corrected,
        ] {
            assert!(ty.requires_reference(), "{}", ty.code());
        }
        for ty in [
            DocumentType::Invoice,
            DocumentType::Prepayment,
            DocumentType::SelfBilled,
        ] {
            assert!(!ty.requires_reference(), "{}", ty.code());
        }
    }

    #[test]
    fn a_reference_without_a_date_is_ambiguous() {
        assert!(DocumentReference::new("INV-2026-0001", "2026-04-01").is_unambiguous());
        assert!(
            !DocumentReference::new("", "2026-04-01").is_unambiguous(),
            "parallel numbering series are permitted, so a bare number does not \\
             identify a document"
        );
        assert!(!DocumentReference::new("INV-2026-0001", "  ").is_unambiguous());
    }

    #[test]
    fn the_five_issue_reasons_are_a_closed_list_with_stable_codes() {
        assert_eq!(IssueReason::ALL.len(), 5, "Article 54 permits exactly five");
        for reason in IssueReason::ALL {
            assert!(
                matches!(reason.code(), "01" | "02" | "03" | "04" | "05"),
                "{}",
                reason.code()
            );
        }
    }

    #[test]
    fn each_rounding_regime_names_its_own_tie_mode() {
        // EN 16931 is silent, so the mode is chosen; HMRC's rule is not the same
        // as half-up on a negative tie.
        assert_eq!(RoundingPolicy::En16931Group.tie_mode(), TieMode::HalfUp);
        assert_eq!(
            RoundingPolicy::HmrcSeventeenFive.tie_mode(),
            TieMode::AwayFromZero
        );

        // Half-even is the IEEE 754 default, and taking it implicitly is exactly
        // the accident this whole type exists to prevent.
        // Half-up always goes up on a tie, which is exactly where it differs
        // from half-even — and it is the difference ZATCA's half-up mandates.
        assert!(TieMode::HalfUp.takes_upper_on_tie(true));
        assert!(TieMode::HalfUp.takes_upper_on_tie(false));
        // Half-even takes whichever neighbour is even.
        assert!(!TieMode::HalfEven.takes_upper_on_tie(true));
        assert!(TieMode::HalfEven.takes_upper_on_tie(false));
        // Toward zero never goes up.
        assert!(!TieMode::TowardZero.takes_upper_on_tie(true));
        assert!(!TieMode::TowardZero.takes_upper_on_tie(false));
    }
}
