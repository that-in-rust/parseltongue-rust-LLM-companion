# Part 6 — targets in `core`

**State: DECIDED 2026-11-15, not built. Stub: the work packet is written when this part is picked.**

## Decision

The user chose what is most explicit for a coding agent that reads the graph: a named
external target, with the sysroot source left out of the run.

- An edge into `core` names its target as far as itoa's source proves it, plus the crate and
  a flag that the target's source was not read (ARCH F3).
- `use core::mem::MaybeUninit` proves the full path. `u128::from(x)` proves `u128::from`, not
  which `From` impl.
- A method call on a value whose type comes from `core` (`.len()`, `.get_unchecked(..)`)
  proves only the method name. It stays unresolved, with that name and a reason.
- REQ-RUST-038.0 is revised to allow an excluded-sysroot run to pass on these terms.

Options not taken: running with the sysroot source (the one measured run did not finish name
resolution in 2,400 s; it loaded `core`, `alloc` and `std`, and a `core`-only run was never
measured), and storing every such edge unresolved with no target.

## Requirement

Not written yet. It must state, per site shape, how much of the name is proven.

## Waiting for

- The expected graph's 119 external rows carry full paths such as
  `<u128 as core::convert::From<u64>>::from`. Each must be cut back to what source proves, or
  moved to the unresolved list.
- Some itoa-to-itoa edges need a `core` return type: the signed `write` calls
  `Unsigned::fmt(self.unsigned_abs(), ..)`. These stay unresolved.
