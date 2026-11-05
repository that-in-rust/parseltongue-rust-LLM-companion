# Which `Sender`? What a tiny model learned about Rust bindings

**The model fit under 100 MB of RAM. It did not find an extra dependency we could trust.** Those are separate results: the first tells us we can ship a small model; the second tells us what not to ask this version to decide.

## One name, two arrows

Imagine two classrooms each have a student named Sam. A note says "give this to Sam." Knowing the name gives you two candidates. Knowing which classroom the note came from can tell you which one to visit.

There is an actual example in `futures-channel` 0.3.31. Two functions named `channel` both mention `Sender` in their return type. Here is where rust-analyzer's **Go to Definition** points for each:

| Where you click on `Sender` | Where you land |
|---|---|
| `src/mpsc/mod.rs:349` | `src/mpsc/mod.rs:123`, the MPSC sender |
| `src/oneshot.rs:104` | `src/oneshot.rs:26`, the one-shot sender |

Those are two different types. Drawing the wrong arrow gives a reviewer the wrong answer about which interface a change might affect. In this pair, checking the enclosing module was enough. It is not a universal Rust rule: an import or re-export can bring a type in from elsewhere.

## What happened when we tried it?

We put the original 64 binding weights and 384 newly trained weights inside a pretrained MiniLM encoder. Rust read the source; the model scored two candidate declarations or declined to choose. Rust-analyzer checked the answers afterward, not during prediction.

- **It fit.** The model file was **17.45 MB**. The highest memory reading for the whole CPU inference process was **92.09 MB** across 15 runs on one Apple Silicon Mac, scanning one crate per process. That is not a promise for every workspace.
- **It could switch answers.** In a controlled example, changing `alpha::Header` to `beta::Header` made the model pick the other declaration. Removing the qualifier made it decline to choose.
- **The simple rule did the useful work on real code.** Across five crates held out from our training, there were **108** return-type references with two same-named candidates. When exactly one candidate was in the same module as an unqualified reference, the rule picked it: **56 confirmed, 5 unverified, 47 left open**.
- **Guessing would have added wrong arrows.** On those 47 open cases the model leaned toward the right candidate **11** times, the wrong one **4** times, and tied **32** times. None met the acceptance threshold chosen before the test. The result was **zero additional accepted bindings**, not 11.

## What this means for Parseltongue

Keep **"look here next"** separate from **"this interface depends on that type."** A weak preference can help an agent decide which file to open. It should not become an arrow that later tools treat as fact.

The next experiment should give the model the trail from the signature to each candidate: which module owns it, which `use` brings it into scope, and which `pub use` exposes it under another name. Allow "neither": in **11 of the 108** cases rust-analyzer found the answer *outside* the offered pair. The test for a new version is whether it adds confirmed edges on fresh crates without adding wrong ones, still below the same memory limit.

[Experiment and reproduction instructions](README.md) | [Recorded measurements](real-results.json) | [Both candidates and rust-analyzer's answer for every reference](real-predictions.jsonl)
