
# Stable folder synchronized across all branches for PRDs Architecture & Design docs

``` bash
git add .stable/ && git commit -m "Update stable folder" && git stash push -u -m "backup_with_untracked" && git checkout main && git add .stable/ && git commit --allow-empty -m "Update stable folder" && git push origin main && git checkout - && git merge main --no-ff -m "Sync stable from main" && git stash pop
```

Your .stable folder becomes permanently synchronized across all branches

- any changes you make to stable files are automatically committed to both your current branch AND the main branch
- while preserving all your other work, creating a universal configuration folder that never gets lost when switching between projects


# Notes

# RAW

``` text
 Journey 1: PRD → Impact Analysis

  Current LLM Tools

  PM: "We need to add rate limiting to all API endpoints"

  LLM: *searches for "endpoint" "api" "route"*
  LLM: "I found 12 files that might be relevant..."
  LLM: *misses 8 endpoints registered dynamically*
  LLM: *doesn't know which endpoints are public vs internal*

  With Live Dependency Graph

  PM: "We need to add rate limiting to all API endpoints"

  Query: graph.entities_by_type("http_handler")
         .filter(|e| e.is_public())
         .with_callers()

  Result:
  ┌────────────────────────────────────────────────────────────┐
  │ PUBLIC ENDPOINTS (23 total)                                │
  │                                                            │
  │ /api/users/*     → UserService → Database (high traffic)  │
  │ /api/payments/*  → PaymentService → Stripe (critical)     │
  │ /api/webhooks/*  → WebhookProcessor (external callers)    │
  │                                                            │
  │ Shared middleware: AuthMiddleware (inserted at 3 points)  │
  │ Suggested insertion point: router.rs:47 (covers all)      │
  └────────────────────────────────────────────────────────────┘

  PRD becomes TESTABLE before code is written.

  Differentiation: LLM knows WHERE to insert, not just WHAT to write.

  ---
  Journey 2: Architecture → Interface Discovery

  Current LLM Tools

  Architect: "Add a caching layer to the database queries"

  LLM: *reads DatabaseService*
  LLM: "I'll add caching here..."
  LLM: *doesn't know 5 other services bypass DatabaseService*
  LLM: *doesn't know QueryBuilder returns raw SQL strings*
  LLM: *creates cache that's immediately stale*

  With Live Dependency Graph

  Architect: "Add a caching layer to the database queries"

  Query: graph.callers_of("Database::query")
         .group_by(|c| c.module)
         .with_signatures()

  Result:
  ┌────────────────────────────────────────────────────────────┐
  │ DATABASE ACCESS PATTERNS                                   │
  │                                                            │
  │ VIA DatabaseService (recommended):                         │
  │   UserRepo::find_by_id(id: i64) → User                    │
  │   OrderRepo::list(filter: Filter) → Vec<Order>            │
  │                                                            │
  │ DIRECT ACCESS (needs migration):                           │
  │   ReportGenerator::raw_query(sql: String) → RawRows  ⚠️   │
  │   LegacyImporter::bulk_insert(data: Vec<Row>) → ()   ⚠️   │
  │                                                            │
  │ CACHE INVALIDATION POINTS:                                 │
  │   User::save() → invalidate UserRepo cache                │
  │   Order::update() → invalidate OrderRepo cache            │
  └────────────────────────────────────────────────────────────┘

  Architecture decision: Cache at Repo layer, migrate 2 legacy callers.

  Differentiation: LLM sees the FULL call graph, not just the file it's reading.

  ---
  Journey 3: TDD → Test Boundary Discovery

  Current LLM Tools

  Developer: "Write tests for the PaymentProcessor"

  LLM: *reads PaymentProcessor*
  LLM: "I'll mock... hmm, what should I mock?"
  LLM: *guesses* Stripe, Database, Logger
  LLM: *misses* NotificationService that's called conditionally
  LLM: *mocks wrong interface* - test passes, prod fails

  With Live Dependency Graph

  Developer: "Write tests for the PaymentProcessor"

  Query: graph.entity("PaymentProcessor")
         .dependencies()
         .with_signatures()
         .categorize()

  Result:
  ┌────────────────────────────────────────────────────────────┐
  │ PAYMENTPROCESSOR TEST BOUNDARIES                           │
  │                                                            │
  │ MUST MOCK (external I/O):                                  │
  │   StripeClient::charge(amount: Money, card: Card) → Result │
  │   Database::transaction<T>(f: Fn) → T                      │
  │                                                            │
  │ SHOULD MOCK (side effects):                                │
  │   NotificationService::send(user: User, msg: Message)      │
  │   AuditLog::record(event: AuditEvent)                      │
  │                                                            │
  │ REAL IMPLEMENTATION OK (pure logic):                       │
  │   PriceCalculator::compute(items: Vec<Item>) → Money       │
  │   Validator::check(input: PaymentInput) → Result           │
  │                                                            │
  │ EXISTING TEST FIXTURES:                                    │
  │   tests/fixtures/stripe_mock.rs (compatible)               │
  │   tests/fixtures/db_mock.rs (compatible)                   │
  └────────────────────────────────────────────────────────────┘

  TDD setup is AUTOMATIC, not guesswork.

  Differentiation: LLM knows test boundaries from STRUCTURE, not from reading all code.

  ---
  Journey 4: Implementation → Blast Radius Awareness

  Current LLM Tools

  Developer: "Rename User.email to User.email_address"

  LLM: *finds 15 usages via grep*
  LLM: *renames them*
  LLM: *misses* template files, SQL strings, JSON serialization
  LLM: *breaks* API contract with mobile app

  Developer finds out in production.

  With Live Dependency Graph

  Developer: "Rename User.email to User.email_address"

  Query: graph.entity("User::email")
         .all_references()
         .with_context()

  Result:
  ┌────────────────────────────────────────────────────────────┐
  │ BLAST RADIUS: 47 references across 6 categories           │
  │ RISK LEVEL: HIGH (public API affected)                    │
  │                                                            │
  │ RUST CODE (23 refs): Safe to auto-rename                  │
  │                                                            │
  │ DATABASE (3 refs):                                         │
  │   migrations/003_users.sql:12 → needs migration           │
  │   queries/find_user.sql:5 → manual update                 │
  │                                                            │
  │ API CONTRACTS (2 refs): ⚠️ BREAKING CHANGE                │
  │   api/v1/user.rs → UserResponse { email: String }         │
  │   openapi.yaml:234 → email field                          │
  │                                                            │
  │ TESTS (19 refs): Auto-update after code change            │
  │                                                            │
  │ SUGGESTION: Keep API as "email", rename internal only     │
  └────────────────────────────────────────────────────────────┘

  Developer knows BEFORE making the change.

  Differentiation: LLM has COMPLETE knowledge, not text-search approximation.

  ---
  Journey 5: Refactoring → Safe Extraction

  Current LLM Tools

  Developer: "Extract payment logic into a separate service"

  LLM: *reads OrderController*
  LLM: *extracts PaymentService*
  LLM: *moves 3 methods*
  LLM: *creates circular dependency* with existing code
  LLM: *breaks* because moved method needed private state

  With Live Dependency Graph

  Developer: "Extract payment logic into a separate service"

  Query: graph.entity("OrderController")
         .methods_matching("payment|charge|refund")
         .with_dependencies()
         .check_extractability()

  Result:
  ┌────────────────────────────────────────────────────────────┐
  │ EXTRACTION ANALYSIS                                        │
  │                                                            │
  │ SAFE TO EXTRACT (self-contained):                          │
  │   charge_card(card, amount) → uses only StripeClient      │
  │   calculate_total(items) → pure function                  │
  │                                                            │
  │ NEEDS REFACTORING FIRST:                                   │
  │   process_payment() → uses self.user_cache (private)      │
  │   → Suggestion: Pass user as parameter instead            │
  │                                                            │
  │ WOULD CREATE CIRCULAR DEPENDENCY:                          │
  │   refund() → calls self.notify_customer()                 │
  │   → notify_customer() calls OrderController methods       │
  │   → Suggestion: Extract notification to separate service  │
  │                                                            │
  │ PROPOSED NEW STRUCTURE:                                    │
  │   PaymentService ← OrderController                         │
  │   NotificationService ← PaymentService, OrderController   │
  └────────────────────────────────────────────────────────────┘

  Refactoring is VALIDATED before execution.

  Differentiation: LLM prevents architectural mistakes, not just writes code.

  ---
  Journey 6: New Feature → Pattern Discovery

  Current LLM Tools

  Developer: "Add a webhook handler for Stripe events"

  LLM: *searches for "webhook"*
  LLM: *finds 0 results*
  LLM: "I'll create a new pattern..."
  LLM: *creates pattern inconsistent with existing code*
  LLM: *duplicates error handling that exists elsewhere*

  With Live Dependency Graph

  Developer: "Add a webhook handler for Stripe events"

  Query: graph.entities_by_type("http_handler")
         .filter(|e| e.is_external_callback())
         .with_pattern()

  Result:
  ┌────────────────────────────────────────────────────────────┐
  │ EXISTING WEBHOOK PATTERNS                                  │
  │                                                            │
  │ Pattern: ExternalCallbackHandler (used 3 times)           │
  │                                                            │
  │   github_webhook.rs:                                       │
  │     fn handle(payload: Bytes, sig: Header) → Result       │
  │     → verify_signature() → parse_event() → dispatch()     │
  │                                                            │
  │   slack_webhook.rs: (same pattern)                        │
  │   twilio_webhook.rs: (same pattern)                       │
  │                                                            │
  │ SHARED UTILITIES:                                          │
  │   WebhookVerifier::verify(secret, sig, body) → bool       │
  │   EventDispatcher::dispatch(event) → Result               │
  │                                                            │
  │ GENERATED SCAFFOLD:                                        │
  │   stripe_webhook.rs following ExternalCallbackHandler     │
  └────────────────────────────────────────────────────────────┘

  New code follows EXISTING patterns automatically.

  Differentiation: LLM discovers patterns from STRUCTURE, not from examples in prompt.

  ---
  Summary: The Paradigm Shift

  | Aspect             | Current LLM Tools     | With Live Dependency Graph |
  |--------------------|-----------------------|----------------------------|
  | Code Understanding | Text search           | Structural graph           |
  | Context Limit      | ~100K tokens          | Entire codebase            |
  | Blast Radius       | Unknown until runtime | Known before edit          |
  | Test Boundaries    | Guessed               | Computed from graph        |
  | Pattern Discovery  | Needs examples        | Inferred from structure    |
  | Refactoring Safety | Hope-based            | Validated                  |
  | API Contracts      | Easily broken         | Explicitly tracked         |
  | PRD → Code         | Hallucinated path     | Verified insertion points  |

  ---
  The Key Insight

  Current tools: LLM + Text Search = Intelligent Guessing

  Parseltongue:  LLM + Live Graph = Verified Understanding

  The graph makes the LLM's hallucinations IMPOSSIBLE
  because every claim is verifiable against the structure.

```