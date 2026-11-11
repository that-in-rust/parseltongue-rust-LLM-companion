# Reference Map

Use this file first to route the request before loading heavier Rust references.

## 1) Choose the Work Mode

| mode | use when | read next |
| --- | --- | --- |
| `Spec Mode` | Feature request is vague, acceptance criteria are missing, or success is not measurable yet | `rust-executable-specs-playbook.md` |
| `Delivery Mode` | Work needs L1/L2/L3 boundaries, traits, TDD order, and implementation sequencing | `rust-executable-specs-playbook.md`, then `rust-conventions-and-gates.md` |
| `Reliability Mode` | Main risk is async, concurrency, cancellation, API design, diagnostics, unsafe, FFI, or release-hardening | `rust-reliability-reference.md`, then `rust-conventions-and-gates.md` |
| `Review Mode` | The task is code review or design review and the main question is long-term correctness | `rust-reliability-reference.md`, then `rust-conventions-and-gates.md` |

## 2) Choose by Task Type

### Library or Public API

Read in this order:
1. `rust-executable-specs-playbook.md`
2. `rust-reliability-reference.md`
   - `1. API And Type Design`
   - `2. Error Design And Diagnostics`
   - `8. Trait, Async, And Macro Boundary Discipline`
   - `10. Diagnostics, Coverage, And Release Discipline`
3. `rust-conventions-and-gates.md`

### CLI or Binary

Read in this order:
1. `rust-executable-specs-playbook.md`
2. `rust-reliability-reference.md`
   - `2. Error Design And Diagnostics`
   - `4. Testing And Contract Verification`
   - `5. Tooling, Release Gates, And Operations`
   - `10. Diagnostics, Coverage, And Release Discipline`
3. `rust-conventions-and-gates.md`

### Async Service or Backend

Read in this order:
1. `rust-executable-specs-playbook.md`
2. `rust-reliability-reference.md`
   - `2. Error Design And Diagnostics`
   - `3. Async And Concurrency Reliability`
   - `4. Testing And Contract Verification`
   - `5. Tooling, Release Gates, And Operations`
3. `rust-conventions-and-gates.md`

### Parser or Protocol Boundary

Read in this order:
1. `rust-executable-specs-playbook.md`
2. `rust-reliability-reference.md`
   - `1. API And Type Design`
   - `4. Testing And Contract Verification`
   - `6. Performance Without Sacrificing Correctness`
   - `10. Diagnostics, Coverage, And Release Discipline`
3. `rust-conventions-and-gates.md`

### FFI or Unsafe Boundary

Read in this order:
1. `rust-executable-specs-playbook.md`
2. `rust-reliability-reference.md`
   - `2. Error Design And Diagnostics`
   - `9. Unsafe And FFI Containment`
   - `10. Diagnostics, Coverage, And Release Discipline`
3. `rust-conventions-and-gates.md`

### Review or Hardening Pass

Read in this order:
1. `rust-reliability-reference.md`
   - `High-Value Anti-Patterns`
   - `Default Reliability Stack`
   - `11. How An LLM Should Use This File`
2. `rust-conventions-and-gates.md`
3. `rust-executable-specs-playbook.md` only if requirements or traceability are missing

## 3) Default Read Order for Large Rust Tasks

1. Read this file.
2. Read `rust-executable-specs-playbook.md`.
3. Read only the relevant sections of `rust-reliability-reference.md`.
4. Finish with `rust-conventions-and-gates.md`.

## 4) Heading Search

- `rg '^##|^###' references/rust-executable-specs-playbook.md`
- `rg '^##|^###' references/rust-reliability-reference.md`
- `rg '^##|^###' references/rust-conventions-and-gates.md`
