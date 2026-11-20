# PATTERN_ADOPTION

Pattern relationships for vat-rules, declared against the WyattAu
pattern canon per references/adoption-format.md. Rows claim what the
code shows; patterns with no relationship here are omitted.

| Pattern | Status | Evidence |
|---|---|---|
| P2 | adopted | src/lib.rs:8 (one vocabulary crate that both ledger sides import, so category, document type, rounding, and tie code cannot drift between receivable and payable) |
| X4 | partial | src/lib.rs:185 (RoundingPolicy encodes four jurisdictions' mandated arithmetic; spec codes exposed verbatim via src/lib.rs:101, DocumentType at src/lib.rs:81; hand-encoded tables, not generated) |
