//! Tax categories and the exemption codes that go with them.
//!
//! EN 16931 puts **two independent vocabularies** on every taxable line, and
//! conflating them is a rejection cause:
//!
//! - the **category** (BT-151 / BT-118, UNCL5305): `S`, `Z`, `E`, `AE`, `K`,
//!   `G`, `O`, `L`, `M`;
//! - the **exemption reason code** (BT-121, CEF `VATEX-*`): a separate list,
//!   `VATEX-EU-AE`, `VATEX-EU-IC` and so on.
//!
//! This crate exists because **both sides of a ledger need the same table**.
//! Accounts receivable and accounts payable differ in almost everything else —
//! who is owed, which way the tax runs, whether it is recoverable — and agree
//! exactly on the category codes, the reason codes, the rates that must be zero,
//! and the rules about which combinations are legal. Duplicating that table is
//! how a purchase invoice ends up filed under a code the validator rejects on the
//! other side of the business.

/// The UNCL5305 tax category for a taxable item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaxCategory {
    /// `S` — standard rate. **Requires** a non-zero rate (BR-CO-4).
    Standard,
    /// `Z` — zero-rated. Rate must be zero, but the VAT is in the price.
    ZeroRated,
    /// `E` — exempt. Rate and tax amount must be zero (BR-E-*).
    Exempt,
    /// `AE` — reverse charge. Rate and tax must be zero (BR-AE-5..9), both
    /// parties' VAT IDs must be present, and the reason code must be
    /// `VATEX-EU-AE`. All-or-none: a document may not mix AE and non-AE lines
    /// (BR-AE-1).
    ReverseCharge,
    /// `K` — intra-community supply. Zero VAT, reason `VATEX-EU-IC`, plus a
    /// delivery date **and** country of delivery (BR-IC-11/12).
    IntraCommunity,
    /// `G` — free export.
    FreeExport,
    /// `O` — not subject to VAT (outside the scope).
    NotSubject,
    /// `L` — Canary Islands IGIC.
    CanaryIslandsIgic,
    /// `M` — Ceuta/Melilla IPSI.
    CeutaMelillaIpsi,
}

impl TaxCategory {
    /// The UNCL5305 code.
    pub fn code(self) -> &'static str {
        match self {
            Self::Standard => "S",
            Self::ZeroRated => "Z",
            Self::Exempt => "E",
            Self::ReverseCharge => "AE",
            Self::IntraCommunity => "K",
            Self::FreeExport => "G",
            Self::NotSubject => "O",
            Self::CanaryIslandsIgic => "L",
            Self::CeutaMelillaIpsi => "M",
        }
    }

    /// Whether this category is expected to carry a zero tax amount.
    pub fn expects_zero_tax(self) -> bool {
        !matches!(self, Self::Standard)
    }

    /// The CEF `VATEX-*` exemption reason code this category requires.
    ///
    /// `None` for the categories that need no reason, which is only `S`.
    pub fn exemption_reason(self) -> Option<&'static str> {
        match self {
            Self::ReverseCharge => Some("VATEX-EU-AE"),
            Self::IntraCommunity => Some("VATEX-EU-IC"),
            Self::Exempt => Some("VATEX-EU-D"),
            Self::FreeExport => Some("VATEX-EU-G"),
            Self::CanaryIslandsIgic => Some("VATEX-EU-L"),
            Self::CeutaMelillaIpsi => Some("VATEX-EU-M"),
            Self::NotSubject => Some("VATEX-EU-O"),
            Self::ZeroRated => Some("VATEX-EU-Z"),
            Self::Standard => None,
        }
    }

    /// Whether this category's rate must be zero.
    ///
    /// Zero-rated is the trap: its VAT is *in* the price, so the computed tax is
    /// zero, but it is not an exemption and it still needs a reason code.
    pub fn rate_must_be_zero(self) -> bool {
        !matches!(self, Self::Standard)
    }

    /// Whether this category requires a buyer VAT identifier.
    ///
    /// EN 16931 makes BT-49 conditional rather than mandatory, which is why a
    /// validator accepts an invoice a tax authority would reject.
    pub fn requires_buyer_vat_id(self) -> bool {
        matches!(
            self,
            Self::ReverseCharge
                | Self::IntraCommunity
                | Self::CanaryIslandsIgic
                | Self::CeutaMelillaIpsi
        )
    }
}

/// Whether the document mixes reverse-charged and ordinary lines.
///
/// EN 16931's BR-AE-1 is all-or-none, and a document that mixes them is refused
/// by a validator — but only if the validator checks, and the mixing is easy to
/// produce because the categories are per line. This applies to both directions:
/// a reverse-charged sale and a reverse-charged purchase are each all-or-none.
#[must_use]
pub fn is_all_or_none_reverse_charge(categories: &[TaxCategory]) -> bool {
    let reverse = categories
        .iter()
        .filter(|c| **c == TaxCategory::ReverseCharge)
        .count();
    reverse == 0 || reverse == categories.len()
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn every_category_has_a_stable_code() {
        // The codes are wire values, so they are pinned rather than derived.
        for (category, code) in [
            (TaxCategory::Standard, "S"),
            (TaxCategory::ZeroRated, "Z"),
            (TaxCategory::Exempt, "E"),
            (TaxCategory::ReverseCharge, "AE"),
            (TaxCategory::IntraCommunity, "K"),
            (TaxCategory::FreeExport, "G"),
            (TaxCategory::NotSubject, "O"),
            (TaxCategory::CanaryIslandsIgic, "L"),
            (TaxCategory::CeutaMelillaIpsi, "M"),
        ] {
            assert_eq!(category.code(), code);
        }
    }

    #[test]
    fn only_the_standard_category_carries_a_rate() {
        // Every other category must have a zero rate, and only `S` may be
        // non-zero. Zero-rated is the trap: its VAT is *in* the price, so the
        // computed tax is zero without it being an exemption.
        assert!(!TaxCategory::Standard.rate_must_be_zero());
        for category in [
            TaxCategory::ZeroRated,
            TaxCategory::Exempt,
            TaxCategory::ReverseCharge,
            TaxCategory::IntraCommunity,
            TaxCategory::FreeExport,
            TaxCategory::NotSubject,
            TaxCategory::CanaryIslandsIgic,
            TaxCategory::CeutaMelillaIpsi,
        ] {
            assert!(
                category.rate_must_be_zero(),
                "{} must carry a zero rate",
                category.code()
            );
            assert!(
                category.expects_zero_tax(),
                "{} carries no tax",
                category.code()
            );
        }
        assert!(!TaxCategory::Standard.expects_zero_tax());
    }

    #[test]
    fn the_two_vocabularies_are_distinct_lists() {
        // Only `S` needs no exemption reason, so a validator that checks one and
        // not the other will accept a line the other rejects.
        assert_eq!(TaxCategory::Standard.exemption_reason(), None);
        for category in [
            TaxCategory::ReverseCharge,
            TaxCategory::IntraCommunity,
            TaxCategory::Exempt,
            TaxCategory::ZeroRated,
        ] {
            let reason = category.exemption_reason().expect("a reason code");
            assert!(
                reason.starts_with("VATEX-"),
                "{}: {reason} is not a CEF VATEX code",
                category.code()
            );
        }
        // And the codes never collide with the category list.
        assert_ne!(
            Some(TaxCategory::ReverseCharge.code()),
            TaxCategory::ReverseCharge.exemption_reason(),
            "a category code is never also a reason code"
        );
    }

    #[test]
    fn buyer_vat_id_is_required_for_the_cross_border_categories() {
        // BT-49 is *conditional* in EN 16931, which is why a validator accepts an
        // invoice a tax authority would reject.
        for category in [
            TaxCategory::ReverseCharge,
            TaxCategory::IntraCommunity,
            TaxCategory::CanaryIslandsIgic,
            TaxCategory::CeutaMelillaIpsi,
        ] {
            assert!(category.requires_buyer_vat_id(), "{}", category.code());
        }
        for category in [
            TaxCategory::Standard,
            TaxCategory::Exempt,
            TaxCategory::FreeExport,
        ] {
            assert!(!category.requires_buyer_vat_id(), "{}", category.code());
        }
    }

    #[test]
    fn reverse_charge_is_all_or_none() {
        assert!(is_all_or_none_reverse_charge(&[]));
        assert!(is_all_or_none_reverse_charge(&[TaxCategory::Standard]));
        assert!(is_all_or_none_reverse_charge(&[
            TaxCategory::ReverseCharge,
            TaxCategory::ReverseCharge
        ]));
        assert!(!is_all_or_none_reverse_charge(&[
            TaxCategory::ReverseCharge,
            TaxCategory::Standard
        ]));
    }
}
