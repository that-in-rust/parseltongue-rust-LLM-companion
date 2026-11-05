# Rust interface graphs, explained with one small program

A call graph tells you who calls a method. A public-interface graph tells you who promises to accept or return a type, even if they never call one of its methods. One line of Rust is enough to see why Parseltongue should keep those two graphs distinct.

## A promise, two shapes, one question

These three pieces, in order, are one runnable Rust program.

**1. Make a promise.** A trait is a promise: anything that implements `Shape` must be able to give its name. Here the name is a borrowed string that lasts for the whole program.

```rust
pub trait Shape {
    fn name(&self) -> &'static str;
}
```

**2. Keep it two different ways.** Neither shape needs any stored data for this example.

```rust
pub struct Circle;
pub struct Square;

impl Shape for Circle { fn name(&self) -> &'static str { "circle" } }
impl Shape for Square { fn name(&self) -> &'static str { "square" } }
```

**3. Ask the question.** Read `s: &dyn Shape` as "lend me any shape that keeps the promise."

```rust
pub fn describe(s: &dyn Shape) -> &'static str {
    s.name()
}

fn main() {
    println!("{}", describe(&Circle));
    println!("{}", describe(&Square));
}
```

Output:

```text
circle
square
```

Same line, `s.name()`, two answers. Inside `describe`, the contract is known: it refers to `Shape::name`. The concrete body depends on the value passed in. Someone could add `Triangle` tomorrow without changing `describe`.

## Draw only what the code says

```text
Public signature     describe -- parameter 0 uses --> Shape

Implementations      Circle   -- implements --------> Shape
                     Square   -- implements --------> Shape

Function body        s.name() -- resolves to -------> Shape::name
```

Read the first edge forward: "What type does this API accept?" Read the *same* edge backward: "Which public APIs accept a shape?" No second assertion is necessary. The last edge's **contract destination** is known; neither `Circle::name` nor `Square::name` is an unconditional destination of that call site. Conversely, a direct `Circle.name()` in this example has a known concrete target. In a generic function with `S: Shape`, the definition at the generic call site is still the trait method; the concrete type is chosen at instantiation, unlike dynamic dispatch through `dyn Shape`.

**The one-line test:** replace only `s.name()` with `"unknown"`. Both outputs now say `unknown`. The public signature did not change.

```text
describe -- parameter 0 uses --> Shape         KEEP
s.name() -- resolves to -------> Shape::name   REMOVE: no such call now
```

A reviewer asking "did we change the public interface?" and a reviewer asking "did we change behavior?" deserve different answers: **no** and **yes**. A graph that stores both as untyped "depends on" edges loses that distinction. An ordinary comment above the function is a third case: source spans move, but neither semantic edge changes.

## Which existing tools answer which question?

| Reader's question | Tool and representation |
|---|---|
| Who defines, imports, calls, or implements this? | [codebase-memory-mcp](https://github.com/DeusData/codebase-memory-mcp) and [CodeGraphContext](https://github.com/CodeGraphContext/CodeGraphContext) index typed symbol relationships. The former also exposes similarity edges: "similar to" does not mean "calls." |
| What larger path is this call part of? | [GitNexus](https://github.com/abhigyanpatwari/GitNexus) groups symbols and indexes execution flows. A statically inferred path is not an observed execution. |
| Why do these two files depend on each other? | [Clarity](https://github.com/LegacyCodeHQ/clarity-cli) can explain file-level edges with symbol evidence. The file is the endpoint of that graph, not the public parameter type. |
| Where is this symbol used, or what code looks relevant? | [Serena](https://github.com/oraios/serena) provides IDE-style definitions and references. [CodeMogger](https://github.com/glommer/codemogger) searches code chunks. Both can lead to the right file without claiming a public-contract edge. |
| Which API accepts or returns this type? | [RAPx](https://safer-rust.github.io/RAPx-Book/5.3-api.html) distinguishes argument, return, generic and type-transformation edges. The [Trustfall rustdoc schema](https://github.com/obi1kenobi/trustfall-rustdoc-adapter/blob/b24088502d1cc1fcdbdb5037407e3a90c4e33b18/src/rustdoc_schema.graphql) exposes importable paths, impls and return values. These are direct correctness baselines for Parseltongue. |

## What rust-analyzer can teach the model

[Devin's fixture and measurements](1793873685-can-rust-analyzer-draw-everything-accurately-limits-measured-cfg-dyn-proc-macros.md) used `Shape::area` to test concrete, generic and dynamic calls. On a separate `cargo-semver-checks` workspace run, Devin reported **9 unknown types in 37,822 expressions, at 2.3 GB** with build scripts and procedural macros enabled. That measures expression-type coverage in that run. It does not measure graph precision, recall, or a universal memory requirement.

```text
Source on disk                  Graph for Linux
#[cfg(unix)]    fn os_only()    Active; verify this definition
#[cfg(windows)] fn os_only()    Visible in text; not tested here
```

An inactive definition is not a nonexistent definition. Nor is it one whose target has been verified. Procedural macros and generated files add a different limit: the evidence may not exist until a build step runs. Rust-analyzer does its own semantic analysis, but can use Cargo for those inputs. See its [architecture](https://rust-analyzer.github.io/book/contributing/architecture.html) and [build-script configuration](https://rust-analyzer.github.io/book/configuration.html#rust-analyzercargobuildscriptsenable).

**Training plan:** use rust-analyzer offline as a teacher, and cross-check exported contracts with rustdoc or a tiny downstream program. At inference, give the student Rust source and a compact cross-file index, not the teacher's graph. On held-out workspaces, score the exact endpoint, role, direction and configuration. Report confidently wrong edges, missed edges and abstentions separately. Measure **peak CPU process RSS below 100,000,000 bytes**, including model, runtime, tokenizer and working buffers. That is a target, not a demonstrated result. The [training and evaluation note](1793858420-arxiv-training-and-evaluating-a-tiny-rust-interface-graph-generator.md) includes a compiler-checked example from a published crate.

## Could we put the computation inside the model?

**Yes for a bounded operation.** [Qwenputer](https://github.com/divagr18/qwenputer) adds a RISC-V CPU to Qwen3.5-4B using 11 layers whose weights are set by a compiler, not learned from examples. Its mutable memory lives in the model's DeltaNet state. The distinction matters: a machine can execute an exact lookup without having learned either Rust or the contents of your repository. The [published checkpoint](https://huggingface.co/divagr1925/Qwen3.5-4B-Qwenputer) is a precedent for the mechanism, not a sub-100 MB CPU implementation of rust-analyzer.

Consider a hypothetical crate that re-exports `Circle` as `Round` and has a public function returning `Round`. Give an executor a table of scope-qualified IDs and *one active configuration*. Now there is a useful piece of Rust analysis small enough to specify precisely:

```text
Given: make returns Round; Round is a public name for Circle

LOOKUP(scope, Round)  --> public alias #7
FOLLOW_ALIAS(#7)      --> definition #3 (Circle)
EMIT(return, #3)      --> make -- return type --> Circle
                           keep "Round" as the public spelling
```

This resolves a **re-export**, not arbitrary Rust type aliases or trait projections. If the table lacks the alias and two modules contain a `Circle`, exact arithmetic cannot tell which one the programmer meant. Record missing evidence, retrieve it, or abstain. Building that table correctly is part of the end-to-end problem. Do not smuggle a compiler-produced table into a benchmark that claims source-only extraction.

**What lives where?** Put reusable rules in the compiled weights; put this checkout's declarations and configuration in mutable memory. A `git pull` should update facts, not require retraining. Interpreting all of rust-analyzer on an in-model CPU would still require its program, workspace inputs, memory and execution steps. Even four billion weights at four bits each are about **2 GB** before any working state; the published base is not a candidate for a 100 MB process. The research question is whether a *much smaller, purpose-built* executor can perform a useful subset.

**First experiment:** implement these three instructions natively in Rust as the reference. On the same bounded table and instruction trace, compare a tiny compiled-weight executor with the native one, including shadowed names, missing imports, alias cycles and a full table. Require identical output or identical explicit failure on every case. Then let a small learned model choose the instructions from source and score the **whole graph** separately: a perfectly executed wrong instruction still produces a wrong edge. Measure wall time and peak RSS for both on the same CPU. Ship the native executor unless executing inside the weights buys a measured advantage. This is a proposed experiment; no such executor was built or benchmarked for this note.
