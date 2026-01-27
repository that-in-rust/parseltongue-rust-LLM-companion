# Start
``` text
So how would Shreyas think about this?

Who is the user segment?
What differentiation are we offering?


```

# Key Ideas

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


You're right - STDIO is the way to go for Parseltongue:

| Transport | MCP Support | Use Case                                    |
|-----------|-------------|---------------------------------------------|
| STDIO     | 10/10 IDEs  | Local CLI, single-user, microsecond latency |
| HTTP      | 9/10 IDEs   | Cloud/enterprise (future option)            |
| SSE       | Deprecated  | Legacy only                                 |

## Vision 01 MCP for all languages

Parseltongue provides
- CPU driven code search replacement for grep with STDIO
    - is live-code-database
        - tests
        - text files like MDs Txts
        - URL links to images and other assets
    - pre-computed dependency graph
    - 2 tables
        - base.db the initial one which you started Parseltongue on
        - live.db
    - constant diff visualization

## Vision 02 Open-Code-Fork for Rust OSS workflow indexed on the power of DEPENDENCY GRAPHs & Functional Programming

- Everything in Vision 01
- Will be based NOT on tree-sitter but on rust-analyzer
- Will be working specifically for needs of users who want to constantly reason their codebase via dependency-graph
- We want to create visuals and control for experienced OSS Rust devs
    - they can allow agents to do the job
    - they can slow it down to check their work
- Dependency Graph as first citizen of thinking itself
    - Agent PRD
        - ISG Diff impact
    - Agent Architecture
        - ISG Diff estimations
    - Agent TDD
        - ISG Diff impact
    - Agent Rubbber Duck Debugging of ISG
    - Agents to visualize changes in ISG in HTML format
        - trigger LLM driven narrative based on that
- Modes
    - no-std mode
    - std mode (default)


### Why Vision 02

Because we cannot force users to use dependency-graphs and associated search tools - for that we need more control

### Remarks

- Parseltongue failed because of
    - lack of a good interface to use it
    - lack of live querying, especially when code is evolving so fast
- We want to get jobs in Rust
- Rust is almost a functional langauge, which most Full Stack languages are NOT
- Highest paid jobs are in Rust
- Optimizing for Rust means optimizing for high scrutiny codebases
- Workflows that we can think of
    - Workflow 01 : Full Stack devs of Springboot Java, React, Go, Nodejs, Python
        - Frotend IDE
            - Antigravity
            - Lovable etc.
        - Backend IDE
            - Claude Code, Open-code
        - Do they look at code after generating it?
            - Frontend yes because you need to have an eye
                - Playwright is used but not perfect
            - Backend no because as long as end to end tests are working & Postman
                - Backend code is templatized, because there are only so many architectures
                - Bigger issue is
                    - migration
                    - deployment
                    - cut-over of some sort or changes to data
    - Workflow 02 : OSS level contribution to Rust tools or libraries or embedded systems
        - Code needs to be audited
        - Code needs to be reasoned at the level dependency graphs which is NOT native to models which are still optimized for Full Stack use cases
- We cannot compete with
    - IDEs trained for specific outcome
    - Models trained for specific outcome
- We can compete on a niche which will not be easily attacked with taste x ease-of-use
    - and it should be something which is NOT a huge opportunity in terms of $
        - platform companies make money from business application layer, not from library layer
        - platform companies are the libray layer
        - platform companies do not invest in tooling for library layer
            - Golang google
            - React FB
        - Platform companies do not invest in tooling for library layer
            - So we can fulfil that need
- Number of LOCs will 10x or 20x - so reading the code will be a new bottle-neck
    - So again a dependency graph will be a thing
- Code-blocks
    - higher abstraction than interfaces
    - different abstraction than modules
    - more like clusters of high-data or high-control flow units

### Nation wants to know

- How to write highest predictability LLM driven code, what will be the characteristics of that code?
    - Functional Programming which can have mathematical correction proved by Lean etc.
    - Code needs to be predictable as in a simulation can be run on it





## Vision 03 New IDE Interface-Signature-Graphs for Functional Programming in Rust

- What if you could write Dependency-Graph in an IDE which treats Interfaces as a first-class citizen?
    - Interface Signature Graph is a collection of
        - Interface Signatures which are Primary Keys
        - These Primary keys will be small txt files
        - Edit these primary keys
            - Change the code or Signature while being super-explicit e.g. serde :: Serialize
            - Changes the forward and backward dependencies
                - in the above example an external library serde will become a calling node

- What if what you wrote then gets deposited in txt files so that it is relevantly compiled by compilers 
    - Rust has 3 layers
        - Creates
            - Folders
                - Files
                    - Code
                    - Tests
                    - Comments or Txt MD
    - You write in a graphical interface where main function is root node
        - will edit the interfaces themselves
        - the interfaces will be then combined and collected into a rs file
            - the rs files will be put into a codebase the way compiler expects it to be
            - the rs files will be compiled by the compiler






