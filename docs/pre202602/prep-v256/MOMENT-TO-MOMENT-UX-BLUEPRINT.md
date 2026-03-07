# Moment-to-Moment UX Blueprint

**Primary Segment:** OSS Rust Contributors, Large Repos  
**Document Purpose:** Define the felt experience. Not what the system does — what the user feels.

---

## The Core Insight

> The user doesn't want "search results." They want to feel like someone who understands the codebase is sitting next to them, pointing at the right file, saying "start here."

Everything else is implementation.

---

## Part 1: The Felt Journey (7 Moments)

### MOMENT 0: The Empty Box

**User sees:** Chat box, current branch indicator, repo scope badge

**User does:** Types: "How does authentication work?"

**What happens:**
- System classifies intent: `explain-architecture` (84% confidence)
- System identifies scope: 3 relevant crates, 47 candidate entities

**User feels:** "It knows where I am. I don't have to explain everything."

**Critical detail:** The branch indicator matters. It says "I see your context." The repo scope badge says "I won't search outside." This reduces cognitive load before the user types a single word.

---

### MOMENT 1: The Check-In

**User sees:**
```
I interpreted your intent as: Explain authentication architecture (84%)

Is this right? [Yes] [No, I meant something else]
```

**User does:** Accepts or corrects

**What happens:**
- If accepted: Proceed to Option Cards
- If corrected: System offers 2-3 alternatives:
  - "Find where auth is called"
  - "Find auth implementation details"
  - "Find auth test coverage"

**User feels:** "I'm in control. This isn't a black box."

**Why this matters:** Every search tool skips this step. They guess and dump results. By asking, we:
1. Reduce wrong-path exploration
2. Build trust through transparency
3. Create a "conversation" not a "query"

**The Shreyas Principle:** Users forgive mistakes if you acknowledge uncertainty. They don't forgive false confidence.

---

### MOMENT 2: The Option Cards (The Most Critical Moment)

**User sees:** 3-5 cards, NOT a list of search results

**What makes this different from search results:**

| Search Results | Option Cards |
|----------------|--------------|
| Ranked by keyword match | Clustered by semantic meaning |
| Show snippet | Show full context package |
| "Here's what matched" | "Here's what you probably want" |
| User must read to compare | User can scan to compare |
| No cost information | Token cost visible upfront |

**What each card shows:**

```
┌─────────────────────────────────────────────────────────────┐
│ 🎯 auth::service::authenticate_user                         │
│                                                             │
│ 📍 crates/auth/src/service.rs:47-89                         │
│                                                             │
│ 🔗 Related: login(), validate_token(), User, AuthError      │
│                                                             │
│ ⚠️ Risk: Medium (12 callers, 3 downstream modules)          │
│                                                             │
│ 📊 ~900 tokens | 89% match | ✅ Up to date                  │
│                                                             │
│ Why this card?                                              │
│   • Lexical match: "authenticate" in function name          │
│   • Semantic: Central to auth flow                          │
│   • Graph: Most connected auth entity                       │
└─────────────────────────────────────────────────────────────┘
```

**User feels:** "I can make a decision without reading code. I understand my options."

**The hidden work:**
- Entity extraction (`ra_ap_hir`)
- Graph traversal (find neighbors, calculate centrality)
- Token estimation (signature + docs + related entities)
- Freshness check (git hash comparison)

**Why cards, not list:** A list says "here's everything we found." Cards say "here's what matters." The user doesn't want to filter. They want to choose.

---

### MOMENT 3: The Commitment (Card Selection)

**User sees:** Clicked card expands slightly, showing a bit more detail

**User does:** Confirms this is the right target

**What happens:**
- System prepares high-ROI context packet
- System pre-computes related entities for deep dive
- System warms up blast radius calculation (just in case)

**User feels:** "Low risk to try. If I'm wrong, I can go back."

**The psychological contract:** At this moment, the user is making a bet. They're saying "I think this is it." The system must:
1. Make the bet feel small (easy to reverse)
2. Reward the bet quickly (show value immediately)
3. Not punish wrong bets (offer alternatives, not dead ends)

---

### MOMENT 4: The Preview (Trust-Building)

**User sees:**
```
Preview (900 tokens):

┌─ Signature Map ─────────────────────────────────┐
│ authenticate_user(creds: Credentials)           │
│   -> Result<User, AuthError>                    │
│                                                 │
│ Called by: login(), api_middleware()            │
│ Calls: validate_credentials(), create_session() │
└─────────────────────────────────────────────────┘

┌─ Type Flow ─────────────────────────────────────┐
│ Credentials ──> validate_credentials()          │
│              ──> User (on success)              │
│              ──> AuthError (on failure)         │
└─────────────────────────────────────────────────┘

[Deep Dive] [Compare with another] [Different target]
```

**User does:** Reviews, then clicks "Deep Dive" or "Compare"

**What happens:**
- System reveals this is a preview, not the full picture
- System shows what it knows AND what it doesn't know
- System offers paths forward, not just "accept/reject"

**User feels:** "This is trustworthy. It shows its work."

**The Trust Equation:**
```
Trust = (Transparency × Accuracy) / Surprise
```

Every time the system shows uncertainty, trust increases. Every time the system is wrong without warning, trust decreases. The preview is the trust-building moment.

---

### MOMENT 5: The Deep Dive (The Value Reveal)

**User sees:**

```
Deep Dive: auth::service::authenticate_user

┌─ Execution Path ────────────────────────────────────────┐
│ 1. Receive Credentials                                  │
│ 2. Validate format (validate_credentials)               │
│ 3. Check database (find_user_by_email) ──┐              │
│    └─> [Not found] ──> AuthError::UserNotFound          │
│ 4. Verify password (verify_hash) ──┐                    │
│    └─> [Mismatch] ──> AuthError::InvalidPassword        │
│ 5. Create session (create_session)                      │
│ 6. Return User with session token                       │
└─────────────────────────────────────────────────────────┘

┌─ Related Tests ─────────────────────────────────────────┐
│ • test_authenticate_user_success (auth_test.rs:23)      │
│ • test_authenticate_user_wrong_password (auth_test.rs:45)│
│ • test_authenticate_user_not_found (auth_test.rs:67)    │
└─────────────────────────────────────────────────────────┘

┌─ Known Unknowns ─────────────────────────────────────────┐
│ ⚠️ Rate limiting: Not visible in this function          │
│ ⚠️ Token refresh: Handled in separate module            │
│ ⚠️ OAuth flow: Different entry point (oauth.rs)         │
└─────────────────────────────────────────────────────────┘

[Generate patch plan] [Show blast radius] [Find similar patterns]
```

**User does:** Absorbs, then picks next action

**What happens:**
- System reveals execution flow (MIR-based control flow)
- System shows related tests (pattern matching on `#[test]`)
- System admits what it doesn't know ("Known Unknowns")

**User feels:** "This is what I came for. I understand now."

**The Known Unknowns section is critical.** Most tools pretend to know everything. By explicitly showing gaps, we:
1. Prevent the user from making false assumptions
2. Build credibility through honesty
3. Create a follow-up opportunity ("Want to explore OAuth flow?")

---

### MOMENT 6: The Action Bar (Momentum Preservation)

**User sees:**
```
What would you like to do next?

[Generate patch plan]  [Run blast radius]  [Get review checklist]

[Ask follow-up question]  [Start over with different intent]
```

**User does:** Picks next action WITHOUT losing context

**What happens:**
- System preserves all context from moments 0-5
- System switches modes: explain → change/risk
- System keeps session memory for thread

**User feels:** "I'm not starting over. My journey continues."

**The Momentum Principle:** Traditional tools force a reset at every step. Search → results → new search. We preserve momentum. The user builds on what they learned, they don't repeat it.

---

### MOMENT 7: The Final Answer (The Handoff)

**User sees:**
```
Answer: auth::service::authenticate_user

📍 Exact location: crates/auth/src/service.rs:47-89

📊 Confidence: 89%

❓ Unknowns:
   • Rate limiting behavior (not in scope)
   • Token expiration (separate module)

💡 Suggested next steps:
   • Read oauth.rs for alternative auth paths
   • Check middleware.rs for request flow

[Copy to clipboard] [Open in editor] [Share this context]
```

**User does:** Proceeds to edit/review/share

**What happens:**
- System provides exact files/lines
- System admits confidence level
- System offers next steps (not dead end)

**User feels:** "I can act immediately. I know what I know, and I know what I don't know."

---

## Part 2: Segment-Specific Experiences

### Segment A: New OSS Contributor (First contribution)

**Context:** They've never seen this codebase. They don't know the module structure. They're afraid of breaking something.

**Their emotional journey:**

| Moment | What they feel | What we provide |
|--------|----------------|-----------------|
| 0 | Overwhelmed | "I see you're on branch X. I'll only search here." |
| 1 | Uncertain | "I think you want X. Is that right?" |
| 2 | Lost | Cards with breadcrumbs: "Here's where this lives" |
| 3 | Anxious | "Low risk to explore. Easy to go back." |
| 4 | Skeptical | "Here's what I know. Here's what I don't." |
| 5 | Enlightened | Deep dive with tests + glossary |
| 6 | Empowered | "Generate patch plan" (they know what to change) |
| 7 | Confident | Exact files + unknowns listed explicitly |

**Success metric:** Opens first PR within 30 minutes, without reading the entire repo.

---

### Segment B: Repeat Contributor (Making a change)

**Context:** They know the codebase. They have a specific task. They want to make the right change efficiently.

**Their emotional journey:**

| Moment | What they feel | What we provide |
|--------|----------------|-----------------|
| 0 | Focused | Quick intent classification |
| 1 | Impatient | Fast confirmation (they know what they want) |
| 2 | Decisive | Cards tied to their issue/keyword |
| 3 | Confident | One click, they know the target |
| 4 | Verification | Preview confirms their mental model |
| 5 | Validation | Deep dive shows blast radius |
| 6 | Ready | "Run blast radius" before they commit |
| 7 | Safe | Patch plan with change impact |

**Success metric:** Correct change with fewer review cycles.

---

### Segment C: Maintainer/Reviewer (Reviewing a PR)

**Context:** They're reviewing someone else's change. They need to catch non-obvious regressions. They don't have time to read everything.

**Their emotional journey:**

| Moment | What they feel | What we provide |
|--------|----------------|-----------------|
| 0 | Suspicious | "What did this PR actually change?" |
| 1 | Curious | Changed-entity map, not just diff |
| 2 | Analytical | Cards show hidden impact edges |
| 3 | Investigative | Control/data-flow warnings |
| 4 | Thorough | Deep dive on affected paths |
| 5 | Certain | "This looks safe" or "This breaks X" |
| 6 | Efficient | Review checklist auto-generated |
| 7 | Decisive | Approve or request changes with evidence |

**Success metric:** Catches non-obvious regressions in <5 minutes.

---

## Part 3: Handling Ambiguity (The Honesty Protocol)

### When Confidence is Low (<70%)

**User sees:**
```
I found 3 candidates that might match. They're close in relevance.

Which sounds right?

○ Authentication login flow (auth::login)
○ Token validation flow (auth::validate_token)  
○ OAuth callback handler (oauth::callback)

[Show me side-by-side preview]
```

**Why this works:**
1. We admit uncertainty upfront
2. We offer structured choices, not a confused list
3. We provide an escape hatch (side-by-side preview)

### When Confidence is Very Low (<50%)

**User sees:**
```
I'm not confident about this one. Here's why:

• "auth" appears in 23 different entities
• Your query doesn't match any function names exactly
• Graph proximity suggests 5 different clusters

What would help:
○ Tell me more about what you're looking for
○ Show me all 23 entities (I'll organize them)
○ Let me search in a specific crate

What I think you might want: [best guess with low confidence badge]
```

**Why this works:**
1. We explain WHY we're uncertain
2. We offer paths forward, not just "I don't know"
3. We still provide a best guess, but with explicit uncertainty

### The Never-Do List

| Never Do | Why |
|----------|-----|
| Show results without confidence | False precision erodes trust |
| Hide uncertainty | User discovers it later, feels betrayed |
| Force a single choice | User may have meant something else |
| Dump all results | User feels overwhelmed, not helped |
| Pretend to know everything | User stops trusting the tool |

---

## Part 4: The Option Card Contract

Every Option Card MUST show:

| Element | Why It Matters |
|---------|----------------|
| Title | User knows what they're looking at |
| Breadcrumb | User knows where it lives |
| Why this card exists | User understands the reasoning |
| Top related entities | User sees the neighborhood |
| Risk badge | User understands change impact |
| Confidence score | User knows how much to trust |
| Token estimate | User can budget their attention |
| Freshness badge | User knows if info is current |

### The Anti-Pattern (What NOT to Do)

```
❌ BAD CARD:
┌─────────────────────────────────────┐
│ authenticate_user                   │
│                                     │
│ pub fn authenticate_user(creds: ... │
│ ...                                 │
│                                     │
│ [View full code]                    │
└─────────────────────────────────────┘
```

**Why this fails:**
- No context about WHERE this is
- No related entities
- No confidence/risk information
- No explanation of WHY it matched
- User must click to learn anything

### The Correct Pattern

```
✅ GOOD CARD:
┌─────────────────────────────────────────────────────────────┐
│ 🎯 auth::service::authenticate_user                         │
│                                                             │
│ 📍 crates/auth/src/service.rs:47-89                         │
│                                                             │
│ 🔗 Related: login(), validate_token(), User, AuthError      │
│                                                             │
│ ⚠️ Risk: Medium (12 callers, 3 downstream modules)          │
│                                                             │
│ 📊 ~900 tokens | 89% match | ✅ Up to date                  │
│                                                             │
│ Why this card?                                              │
│   • Lexical match: "authenticate" in function name          │
│   • Semantic: Central to auth flow                          │
│   • Graph: Most connected auth entity                       │
└─────────────────────────────────────────────────────────────┘
```

---

## Part 5: The Feel Targets (PMF Level)

These are the emotional outcomes we're optimizing for:

| Target | Measurement |
|--------|-------------|
| **Speed to Trust** | User reaches trustworthy target in <60s |
| **Actionability** | User gets actionable context in <120s |
| **Transparency** | User always sees WHY a result was chosen |
| **Reversibility** | User can pivot intent without restart |
| **Honesty** | User sees uncertainty explicitly |

### The Trust Test

After using the tool for 5 minutes, the user should answer "Yes" to:

1. "Did it understand what I was looking for?"
2. "Did it show me why it chose these results?"
3. "Did it admit when it wasn't sure?"
4. "Did I feel in control throughout?"
5. "Would I use this again?"

If any answer is "No," we have a UX bug.

---

## Part 6: The Implementation Contract

### What the UX Requires from Engineering

| UX Requirement | Engineering Implication |
|----------------|------------------------|
| Cards in <2s | Entity extraction must be fast (cache, index) |
| Confidence visible | Every result needs a score |
| Related entities | Graph traversal on every candidate |
| Risk badge | Call graph analysis (fan-in/fan-out) |
| Token estimate | Pre-calculate signature + docs size |
| Freshness badge | Git hash comparison on every query |
| Known unknowns | Explicit "gaps" in the answer |

### What Engineering Requires from UX

| Engineering Need | UX Implication |
|------------------|----------------|
| Clear data schema | Define exact fields for each card |
| Ambiguity rules | Define confidence thresholds |
| Fallback behavior | Define what to show when data is missing |
| Token budget | Define max context size per moment |

---

## Part 7: The Shreyas Principles (Applied)

1. **Users don't want search. They want a guide.**
   - We're not building a search engine. We're building a knowledgeable colleague.

2. **Show your work.**
   - Every result includes "Why this card exists." No black boxes.

3. **Admit uncertainty.**
   - Low confidence is shown explicitly. Users forgive uncertainty. They don't forgive false confidence.

4. **Preserve momentum.**
   - The journey is continuous. No reset at each step. Context accumulates.

5. **Make the bet feel small.**
   - Every choice is easy to reverse. "Back" is always available. No dead ends.

6. **The user is the decision maker.**
   - We recommend. We don't decide. The user always has the final say.

---

## Appendix A: Moment-by-Moment Data Sources

| Moment | Data Needed | Source |
|--------|-------------|--------|
| 0 | Git context, repo structure | `git`, `Cargo.toml` |
| 1 | Intent classification | NLP classifier |
| 2 | Entity candidates, breadcrumbs | `ra_ap_hir`, `ra_ap_hir_def` |
| 3 | Related entities, risk | `ra_ap_hir_ty`, graph traversal |
| 4 | Signatures, type flow | `ra_ap_hir_ty`, type inference |
| 5 | Execution path, tests | `rustc_middle::mir`, test detection |
| 6 | Blast radius, change impact | Call graph, trait impls |
| 7 | Exact files/lines, unknowns | `rustc_span`, gap analysis |

---

## Appendix B: The Anti-Patterns We Avoid

| Anti-Pattern | Why It Fails | Our Solution |
|--------------|--------------|--------------|
| Search results list | User must read to compare | Option cards with comparison built-in |
| Black box ranking | User doesn't trust results | "Why this card exists" section |
| False precision | User discovers errors later | Explicit confidence scores |
| No context | User doesn't know if result is relevant | Breadcrumb + related entities |
| Dead ends | User must restart search | "Different target" always available |
| Docs dump | User feels overwhelmed | Token-budgeted context packages |

---

## Appendix C: The North Star

> In 60 seconds, a new contributor to a 500k-line Rust codebase should feel like they have a knowledgeable guide sitting next to them, pointing at the right file, saying "start here, and here's what you need to know."

If we achieve this, everything else is implementation detail.

---

*Document Version: 1.0*  
*Created: 2026-03-01*  
*Status: Active Blueprint*
