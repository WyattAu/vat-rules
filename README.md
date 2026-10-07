# vat-rules

Tax categories, exemption codes, document types and rounding regimes, shared by
**both sides** of an accounting ledger. No dependencies.

Accounts receivable and accounts payable differ in almost everything: who is
owed, which way the tax runs, whether the tax is recoverable, which account it
lands in. They agree **exactly** on the vocabulary — UNCL5305 category codes,
CEF `VATEX-*` exemption reasons, which categories must carry a zero rate, and
which combinations are legal.

Duplicating that table is how a purchase invoice ends up filed under a code the
validator rejects on the other side of the business, and it is invisible until a
customer or a tax authority rejects a document that was internally consistent.
So it lives here, and both sides import it.

## What is in it

- `TaxCategory` — UNCL5305, with its separate CEF reason code. Two independent
  vocabularies on every taxable line; conflating them is a rejection cause.
- `DocumentType` — UNTDID 1001. Shared because a purchase invoice and a sales
  invoice can both be a credit note, and the codes are the same vocabulary.
- `IssueReason` — ZATCA BR-KSA-17 requires one of five reasons on any credit or
  debit note; EN 16931 has no field for it at all.
- `RoundingPolicy` — four jurisdictions mandate mutually incompatible
  arithmetic, and no default is safe.
- `TieMode` — owned here rather than borrowed from a ledger crate, because it is
  policy vocabulary and not arithmetic machinery. A table of tax codes has no
  business depending on a double-entry engine to say "round ties away from zero".

## Why the rounding policy cannot have a default

- **EN 16931 / Peppol §9**: round each line net to two decimals, sum the rounded
  values, compute VAT per (category, rate) group, do not re-round.
- **Australia, GST Act s9-90**: for two or more taxable supplies a taxpayer may
  instead add *unrounded* GST per supply and round the total once. Twenty lines
  at $3.49 give $6.35 that way and $6.40 the EN 16931 way — a five-cent
  difference, lawful in Australia and invalid under EN 16931. The ATO states that
  seller and buyer need not use the same method.
- **UK, VAT Notice 700 §17.5**: below half a penny down, at or above up — with
  HMRC's own admission that the invoice-total rules "have no statutory basis".
- **EN 16931 is silent on tie-breaking**, and the two obvious defaults disagree:
  PostgreSQL `numeric` rounds ties away from zero, `double precision` rounds ties
  to even.

## Licence

MIT OR Apache-2.0.
