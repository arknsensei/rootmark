// Fixed reasons the views send with `promote` (contract §2.4: a reason is
// required). No Obsidian import: tested in test/p2_0.test.ts.

/** the reason of a one-click promotion of an unverified block (P2-0/Z3) */
export const UNVERIFIED_REASON = "the human's own edit, not attested at save (unverified → canonical)";
