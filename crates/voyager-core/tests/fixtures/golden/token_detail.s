; Trailing line comment, a multi-line block comment, a nested block comment,
; and a continuation-split @variable@ reference — for User Story 3's
; token-level detail coverage (quickstart.md Scenario 3).
X = 1 ; trailing comment on a real statement

/* a block comment
   spanning multiple
   physical lines */

/* outer /* inner */ outer again */

FILEO NETO = '@ParentDir@,
@ScenarioDir@file.mtx'
