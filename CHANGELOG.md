# Changelog

All notable changes to this project are documented here. Format: [Keep a
Changelog](https://keepachangelog.com/) — versions follow [semver](https://semver.org).

## [Unreleased]

## [0.1.0] - 2026-10-06

Initial release, extracted from `invoice-kit 0.1.0` so that accounts payable
imports the same table rather than re-deriving it.

### Added

- `TaxCategory` (UNCL5305) with its separate CEF `VATEX-*` exemption reason
  codes, and `is_all_or_none_reverse_charge` for BR-AE-1.
- `DocumentType` (UNTDID 1001) and `IssueReason` (ZATCA BR-KSA-17's five).
- `RoundingPolicy` for the four mutually incompatible regimes, and `TieMode` for
  the tie direction they each mandate.
- `DocumentReference`, whose `is_unambiguous` encodes Article 219's "specifically
  and unambiguously" — parallel numbering series are permitted, so a bare number
  does not identify a document.
