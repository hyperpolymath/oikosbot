// SPDX-License-Identifier: MPL-2.0
// OikosBot Policy: Finding Taxonomy (intent axis)
//
// OikosBot classifies every finding on three orthogonal axes, layered
// intent -> action -> outcome. The canonical definition (all three axes and
// their allowed values) lives in
// .machine_readable/descriptiles/NEUROSYM.a2ml [finding-taxonomy]:
//
//   1. intent       must | intend | wish
//   2. maintenance  corrective | adaptive | perfective | preventive
//   3. locus        systems | compliance | externalities
//
// Only the INTENT axis is numerically derivable: it is a pure function of the
// finding's confidence score, and is identical to the gitbot-fleet Safety
// Triangle (eliminate / substitute / control) and its 0.95 / 0.85 thresholds.
// `maintenance` and `locus` are categorical tags attached when a rule is
// authored, so they are not computed here.
//
// Example Eclexia policy, not executed by the builtin Rust adapter.
// No cost proof is supplied for the declared resource annotations.

// Legacy numeric intent classification only; not authorization to auto-fix.
def intent_is_must(confidence: Float) -> Bool
    @requires: energy < 0.01J, carbon < 0.001gCO2e
    @optimize: minimize energy
{
    confidence >= 0.95
}

// intent = INTEND <=> "substitute" <=> proven replacement, needs review.
def intent_is_intend(confidence: Float) -> Bool
    @requires: energy < 0.01J
{
    confidence >= 0.85 && confidence < 0.95
}

// intent = WISH   <=> "control" <=> human judgement required.
def intent_is_wish(confidence: Float) -> Bool
    @requires: energy < 0.01J
{
    confidence < 0.85
}

// Confidence alone never authorizes mutation. This sample has no authority
// adapter, so automatic action is disabled for every input.
def is_auto_actionable(confidence: Float) -> Bool
    @requires: energy < 0.01J, latency < 1ms
{
    false
}
