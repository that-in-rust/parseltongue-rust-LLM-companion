# CR-codex-architecture: OpenAI Codex Deep Architecture Analysis

**Date**: 2026-02-19
**Tool**: Parseltongue Graph Analysis v1.7.3
**Server**: http://localhost:7780
**Codebase**: OpenAI Codex CLI (codex-rs)
**Total Entities**: 15,901 | **Total Edges**: 136,130
**Languages**: Rust (primary), C (bubblewrap vendor), Python, TypeScript, JavaScript

---

## 1. Core Entity Map

### 1.1 Entity Summary by Subsystem

```
+========================+============+=======================================+
| SUBSYSTEM              | # ENTITIES | KEY FILES / CRATES                    |
+========================+============+=======================================+
| Sandbox (all platforms)| 415        | linux-sandbox/, windows-sandbox-rs/,  |
|                        |            | core/src/seatbelt*, vendor/bubblewrap |
+------------------------+------------+---------------------------------------+
| Tool System            | 555        | core/src/tools/, core/src/function_   |
|                        |            | tool.rs, protocol/src/dynamic_tools   |
+------------------------+------------+---------------------------------------+
| Agent / Multi-Agent    | 183        | core/src/agent/, tui/src/multi_       |
|                        |            | agents.rs, protocol/src/items.rs      |
+------------------------+------------+---------------------------------------+
| Session Management     | 194        | tui/src/session_log.rs, file-search/  |
|                        |            | src/lib.rs, app-server/src/           |
+------------------------+------------+---------------------------------------+
| Exec / Process         | 786        | exec/, exec-server/, core/src/        |
|                        |            | unified_exec/, execpolicy/            |
+------------------------+------------+---------------------------------------+
| MCP Integration        | 550        | mcp-server/, rmcp-client/,            |
|                        |            | core/src/mcp/, core/src/connectors    |
+------------------------+------------+---------------------------------------+
| Protocol Layer         | 1,106      | protocol/src/, app-server-protocol/   |
|                        |            | src/protocol/                         |
+------------------------+------------+---------------------------------------+
| Command / CLI          | 344        | cli/src/main.rs, cli/src/lib.rs       |
+------------------------+------------+---------------------------------------+
| Approval / Policy      | 178        | protocol/src/approvals.rs, exec-      |
|                        |            | server/src/posix/mcp_escalation_      |
|                        |            | policy.rs, utils/cli/                 |
+------------------------+------------+---------------------------------------+
| Patch System           | 214        | apply-patch/src/lib.rs, core/src/     |
|                        |            | tools/handlers/apply_patch.rs         |
+------------------------+------------+---------------------------------------+
| Security (Landlock)    | 16         | linux-sandbox/src/, core/src/lib.rs   |
+------------------------+------------+---------------------------------------+
| Security (Seatbelt)    | 23         | cli/src/debug_sandbox/seatbelt.rs,    |
|                        |            | core/src/seatbelt*                    |
+------------------------+------------+---------------------------------------+
| Permissions            | 22         | core/src/skills/permissions,          |
|                        |            | core/src/context_manager/updates.rs   |
+------------------------+------------+---------------------------------------+
| ExecPolicy             | 248        | execpolicy/, execpolicy-legacy/       |
+------------------------+------------+---------------------------------------+
| Network                | 207        | vendor/bubblewrap/network.c,          |
|                        |            | core/src/config/network_proxy_spec    |
+------------------------+------------+---------------------------------------+
```

### 1.2 Key Entities by Type

#### Sandbox Subsystem (Platform-Specific)
```
TYPE       | ENTITY KEY                                                    | FILE
-----------+---------------------------------------------------------------+-------------------------------------
enum       | SandboxCommand:____codex_rs_cli_src_main                      | cli/src/main.rs
enum       | SandboxType:____codex_rs_cli_src_debug_sandbox                | cli/src/debug_sandbox.rs
struct     | LandlockCommand:____codex_rs_cli_src_lib                      | cli/src/lib.rs
struct     | SeatbeltCommand:____codex_rs_cli_src_lib                      | cli/src/lib.rs
struct     | LandlockCommand:____codex_rs_linux_sandbox_src_linux_run_main | linux-sandbox/src/linux_run_main.rs
struct     | DenialLogger:____codex_rs_cli_src_debug_sandbox_seatbelt      | cli/src/debug_sandbox/seatbelt.rs
struct     | SandboxDenial:____codex_rs_cli_src_debug_sandbox_seatbelt     | cli/src/debug_sandbox/seatbelt.rs
module     | landlock:____codex_rs_linux_sandbox_src_lib                    | linux-sandbox/src/lib.rs
module     | seatbelt:____codex_rs_core_src_lib                            | core/src/lib.rs
```

#### Tool System
```
TYPE       | ENTITY KEY                                                    | FILE
-----------+---------------------------------------------------------------+-------------------------------------
struct     | ToolOrchestrator:____codex_rs_core_src_tools_orchestrator     | core/src/tools/orchestrator.rs
enum       | ToolKind:____codex_rs_core_src_tools_registry                 | core/src/tools/registry.rs
enum       | ToolEmitter:____codex_rs_core_src_tools_events                | core/src/tools/events.rs
enum       | ToolEventStage:____codex_rs_core_src_tools_events             | core/src/tools/events.rs
enum       | ToolEventFailure:____codex_rs_core_src_tools_events           | core/src/tools/events.rs
enum       | CollabTool:____codex_rs_exec_src_exec_events                  | exec/src/exec_events.rs
enum       | CollabToolCallStatus:____codex_rs_exec_src_exec_events        | exec/src/exec_events.rs
enum       | FunctionCallError:____codex_rs_core_src_function_tool         | core/src/function_tool.rs
method     | dispatch:____codex_rs_core_src_tools_registry                 | core/src/tools/registry.rs
method     | run:____codex_rs_core_src_tools_orchestrator                  | core/src/tools/orchestrator.rs
method     | run_attempt:____codex_rs_core_src_tools_orchestrator          | core/src/tools/orchestrator.rs
```

#### MCP Integration
```
TYPE       | ENTITY KEY                                                    | FILE
-----------+---------------------------------------------------------------+-------------------------------------
enum       | McpToolCallStatus:____codex_rs_exec_src_exec_events           | exec/src/exec_events.rs
enum       | McpOAuthLoginSupport:____codex_rs_core_src_mcp_auth           | core/src/mcp/auth.rs
enum       | McpSubcommand:____codex_rs_cli_src_mcp_cmd                    | cli/src/mcp_cmd.rs
enum       | ExecPolicyOutcome:..._mcp_escalation_policy                   | exec-server/src/posix/mcp_...
enum       | ClientState:____codex_rs_rmcp_client_src_rmcp_client          | rmcp-client/src/rmcp_client.rs
fn         | handle_exec_approval_request:..._mcp_server_src_exec_approval | mcp-server/src/exec_approval.rs
fn         | handle_patch_approval_request:..._mcp_server_src_patch_approva| mcp-server/src/patch_approval.rs
fn         | run_codex_tool_session_inner:..._codex_tool_runner            | mcp-server/src/codex_tool_runner.rs
fn         | accessible_connectors_from_mcp_tools:..._core_src_connectors  | core/src/connectors.rs
```

#### Approval / Policy
```
TYPE       | ENTITY KEY                                                    | FILE
-----------+---------------------------------------------------------------+-------------------------------------
enum       | ApprovalModeCliArg:..._approval_mode_cli_arg                  | utils/cli/src/approval_mode_cli_arg.rs
enum       | CommandApprovalBehavior:..._app_server_test_client             | app-server-test-client/src/lib.rs
enum       | ElicitationAction:____codex_rs_protocol_src_approvals         | protocol/src/approvals.rs
enum       | NetworkApprovalProtocol:____codex_rs_protocol_src_approvals   | protocol/src/approvals.rs
```

---

## 2. Call Graph Analysis

### 2.1 ToolOrchestrator::run -- The Heart of Tool Execution

**Entity**: `rust:method:run:____codex_rs_core_src_tools_orchestrator:T1806535705`
**Location**: `codex-rs/core/src/tools/orchestrator.rs`
**CBO**: 28 | **RFC**: 28 | **WMC**: 28 | **Health**: F

#### Forward Callees (28 total)

```
ToolOrchestrator::run()
    |
    +---> exec_approval_requirement()        [Line 122: Check if approval needed]
    +---> default_exec_approval_requirement() [Line 123: Fallback approval]
    +---> requirements_toml()                 [Line 157: Load requirements config]
    +---> sandbox_mode_for_first_attempt()    [Line 163: Determine sandbox mode]
    +---> select_initial()                    [Line 165: Select initial strategy]
    +---> sandbox_preference()                [Line 167: Get sandbox preference]
    +---> enabled()                           [Line 175: Check if tool enabled]
    +---> escalate_on_failure()               [Line 220: Handle failures]
    +---> wants_no_sandbox_approval()         [Line 228: Skip sandbox approval?]
    +---> Sandbox()                           [Line 240: Create sandbox context]
    +---> Codex()                             [Line 240: Reference Codex context]
    +---> build_denial_reason_from_output()   [Line 253: Denial messages]
    +---> should_bypass_approval()            [Line 257: Bypass check]
    +---> start_approval_async()              [Line 269: Begin approval flow]
    +---> tool_decision()                     [Line 270: Make tool decision]
    +---> Rejected()                          [Line 274: Handle rejection]
    +---> run_attempt()                       [Line 294: Execute the tool call]
```

#### Reverse Callers (41 total -- via unresolved `run`)

```
                    Who calls ToolOrchestrator::run()?
                    ==================================
                                   |
    +------------------------------+-------------------------------+
    |                              |                               |
cli_main()               run_ratatui_app()            run_jobs() [memories]
[cli/src/main.rs:772]    [tui/src/lib.rs:725]        [core/src/memories/phase1.rs]
    |                              |                               |
    +--- main_execve_wrapper()     +--- run_fuzzy_file_search()    +--- start_memories_startup_task()
         [exec-server/posix.rs]         [app-server/...]                [core/src/memories/start.rs]
    |
    +--- intercept_apply_patch()           +--- run_attempt()
         [core/tools/handlers/apply_       |    [core/tools/orchestrator.rs:76]
          patch.rs:239]                    |
                                           +--- run_exec_like()
                                                [core/tools/handlers/shell.rs:332]
                                           |
                                           +--- open_session_with_sandbox()
                                                [core/unified_exec/process_manager.rs:639]
                                           |
                                           +--- spawn_task()
                                                [core/src/tasks/mod.rs:144]
```

### 2.2 cli_main -- The Entry Point

**Entity**: `rust:fn:cli_main:____codex_rs_cli_src_main:T1631266894`
**Location**: `codex-rs/cli/src/main.rs`
**CBO**: 52 | **RFC**: 52 | **WMC**: 52 | **Health**: F (God function)

#### Forward Callees (52 total -- Top Command Dispatch Tree)

```
cli_main()
    |
    +---> parse() / try_parse_from()                    [CLI arg parsing]
    +---> prepend_config_flags()                        [Config merging]
    +---> load_with_cli_overrides_and_harness_overrides() [Config loading]
    +---> to_overrides() / parse_overrides()            [Override processing]
    |
    +---> COMMAND DISPATCH:
    |     |
    |     +---> run_interactive_tui()          [Interactive mode - TUI]
    |     +---> run_app()                      [App server mode]
    |     +---> run_main()                     [Non-interactive / piped]
    |     +---> run_main_with_transport()       [Transport-specific run]
    |     |
    |     +---> SANDBOX COMMANDS:
    |     |     +---> run_command_under_seatbelt()   [macOS sandbox]
    |     |     +---> run_command_under_landlock()    [Linux sandbox]
    |     |     +---> run_command_under_windows()     [Windows sandbox]
    |     |
    |     +---> UTILITY COMMANDS:
    |     |     +---> run_apply_command()             [Apply patches]
    |     |     +---> run_execpolicycheck()           [Check exec policy]
    |     |     +---> run_debug_app_server_command()  [Debug mode]
    |     |
    |     +---> AUTH COMMANDS:
    |     |     +---> run_login_with_device_code()   [Device login]
    |     |     +---> run_login_with_chatgpt()       [ChatGPT login]
    |     |     +---> run_login_with_api_key()       [API key login]
    |     |     +---> run_login_status()             [Status check]
    |     |     +---> run_logout()                   [Logout]
    |     |
    |     +---> SESSION MANAGEMENT:
    |           +---> finalize_resume_interactive()  [Resume session]
    |           +---> finalize_fork_interactive()    [Fork session]
    |           +---> fork_picker_logic_with_session_id()
    |           +---> resume_picker_logic_with_session_id()
    |
    +---> SCHEMA GENERATION:
    |     +---> generate_json_with_experimental()   [JSON schema]
    |     +---> generate_ts_with_options()           [TypeScript types]
    |
    +---> print_completion()                        [Shell completion]
    +---> enable_feature_in_config()                [Feature flags]
    +---> disable_feature_in_config()
    +---> handle_app_exit()                         [Cleanup]
```

### 2.3 shell::run_exec_like -- Command Execution Pipeline

**Entity**: `rust:method:run_exec_like:____codex_rs_core_src_tools_handlers_shell:T1655524205`
**Location**: `codex-rs/core/src/tools/handlers/shell.rs`
**CBO**: 24 | **RFC**: 24 | **WMC**: 24 | **Health**: F

```
run_exec_like()
    |
    +---> dependency_env()                  [Get env vars from dependencies]
    +---> extend() / insert() / get()       [Manipulate env map]
    +---> requires_escalated_permissions()   [Security check]
    +---> intercept_apply_patch()            [Intercept patch operations]
    +---> shell()                            [Get shell path]
    +---> begin()                            [Start exec tracking]
    +---> create_exec_approval_requirement_for_command()  [Create approval]
    +---> run() -> ToolOrchestrator::run()   [Delegate to orchestrator]
    +---> finish()                           [Complete tracking]
    +---> format_exec_output_for_model_*()   [Format output for LLM]
```

### 2.4 run_codex_tool_session_inner -- MCP Server Core Loop

**Entity**: `rust:fn:run_codex_tool_session_inner:____codex_rs_mcp_server_src_codex_tool_runner:T1722079562`
**Location**: `codex-rs/mcp-server/src/codex_tool_runner.rs`
**CBO**: 12 | **RFC**: 12 | **WMC**: 12 | **Health**: F

```
run_codex_tool_session_inner()
    |
    +---> effective_approval_id()           [Determine approval context]
    +---> next_event()                      [Event loop: get next event]
    +---> send_event_as_notification()      [Forward events to MCP client]
    |
    +---> EVENT HANDLING:
    |     +---> handle_exec_approval_request()   [Exec approval gateway]
    |     +---> handle_patch_approval_request()  [Patch approval gateway]
    |
    +---> create_call_tool_result_with_thread_id()  [Format response]
    +---> send_response()                            [Return to client]
    +---> lock() / remove()                          [Session cleanup]
```

### 2.5 open_session_with_sandbox -- Sandbox Session Factory

**Entity**: `rust:method:open_session_with_sandbox:____codex_rs_core_src_unified_exec_process_manager:T1648733461`
**Location**: `codex-rs/core/src/unified_exec/process_manager.rs`
**CBO**: 12 | **RFC**: 12 | **WMC**: 12 | **Health**: F

```
open_session_with_sandbox()
    |
    +---> create_env()                                [Create environment]
    +---> apply_unified_exec_env()                    [Apply exec env vars]
    +---> create_exec_approval_requirement_for_command() [Approval check]
    +---> new()                                        [ExecApproval instance]
    +---> clone()                                      [Clone sandbox config]
    +---> run() -> ToolOrchestrator::run()             [Orchestrate with sandbox]
    +---> create_process()                             [Spawn sandboxed process]
```

---

## 3. Blast Radius Analysis

### 3.1 ToolOrchestrator::run -- CRITICAL (3,489 affected entities)

```
  BLAST RADIUS: ToolOrchestrator::run()
  =====================================

  Hop 0 (source): ToolOrchestrator::run()
                          |
  Hop 1 (41 entities): ---+--- cli_main() [CLI entry]
                          +--- run_ratatui_app() [TUI]
                          +--- run_jobs() [Background memories]
                          +--- intercept_apply_patch() [Patch handler]
                          +--- main_execve_wrapper() [Exec server]
                          +--- open_session_with_sandbox() [Sandbox session]
                          +--- run_exec_like() [Shell handler]
                          +--- run_attempt() [Orchestrator retry]
                          +--- spawn_task() [Task spawner]
                          +--- start_memories_startup_task() [Startup]
                          +--- ... (31 more including tests)
                          |
  Hop 2 (3,448 entities): Cascades to virtually the entire codebase!
                          - main() entry points (all binaries)
                          - All test functions
                          - All TUI widgets
                          - All app-server handlers
                          - All memory/rollout operations

  +-----------------------------------------------------------------+
  | IMPACT SCORE: 3,489 / 15,901 = 21.9% of entire codebase       |
  | VERDICT: ToolOrchestrator::run is the SINGLE most critical     |
  |          function in the entire Codex architecture.              |
  +-----------------------------------------------------------------+
```

### 3.2 shell::run_exec_like -- HIGH (33 affected entities)

```
  BLAST RADIUS: shell::run_exec_like()
  =====================================

  Hop 0: run_exec_like()
           |
  Hop 1:  shell::handle() [1 entity]
           |
  Hop 2:  +--- tools::registry::dispatch() [Tool dispatcher]
           +--- tui::chatwidget::defer_or_handle()
           +--- multi_agents::handle() [Sub-agent handler]
           +--- run() [memories phase2]
           +--- 28 test functions for multi-agent behavior:
                - spawn_agent_*, send_input_*, wait_*,
                  close_agent_*, resume_agent_*
```

### 3.3 open_session_with_sandbox -- MODERATE (9 affected entities)

```
  BLAST RADIUS: open_session_with_sandbox()
  ==========================================

  Hop 0: open_session_with_sandbox()
           |
  Hop 1:  exec_command() [Process manager entry]
           |
  Hop 2:  +--- handle() [unified_exec handler]
           +--- exec_command() [test helper]
           +--- multi_unified_exec_sessions [test]
           +--- unified_exec_persists_across_requests [test]
           +--- unified_exec_timeouts [test]
           +--- completed_commands_do_not_persist [test]
           +--- requests_with_large_timeout_are_capped [test]
           +--- reusing_completed_process_returns_unknown [test]
```

### 3.4 run_codex_tool_session_inner -- MODERATE (4 affected entities)

```
  BLAST RADIUS: run_codex_tool_session_inner()
  =============================================

  Hop 0: run_codex_tool_session_inner()
           |
  Hop 1:  +--- run_codex_tool_session()
           +--- run_codex_tool_session_reply()
           |
  Hop 2:  +--- handle_tool_call_codex() [MCP message processor]
           +--- handle_tool_call_codex_session_reply()
```

### 3.5 cli_main -- LOW direct (5 affected), HIGH transitive

```
  BLAST RADIUS: cli_main()
  =========================

  Hop 0: cli_main()
           |
  Hop 1:  main() [Single CLI entry point]
           |
  Hop 2:  +--- arg0_dispatch() [Symlink dispatch]
           +--- main() [apply-patch binary]
           +--- main() [windows-sandbox command_runner]
           +--- main() [windows-sandbox setup_main]
```

---

## 4. Coupling/Cohesion Metrics

### 4.1 Metrics Summary Table

```
+====================================================+======+======+======+======+=======+
| ENTITY                                             | CBO  | LCOM | RFC  | WMC  | GRADE |
+====================================================+======+======+======+======+=======+
| FILE: codex-rs/core/src/codex.rs                   | 1124 | 1.0  | 1124 | 1124 | F     |
| FILE: codex-rs/tui/src/chatwidget.rs               | 1075 | 1.0  | 1075 | 1075 | F     |
| FILE: codex-rs/app-server/codex_message_proc.rs    |  929 | 1.0  |  929 |  929 | F     |
| fn: cli_main                                       |   52 | 1.0  |   52 |   52 | F     |
| method: ToolOrchestrator::run                      |   28 | 1.0  |   28 |   28 | F     |
| method: run_exec_like                              |   24 | 1.0  |   24 |   24 | F     |
| fn: intercept_apply_patch                          |   19 | 1.0  |   19 |   19 | F     |
| method: ToolOrchestrator::run_attempt              |   14 | 1.0  |   14 |   14 | F     |
| fn: handle_exec_approval_request                   |   14 | 1.0  |   14 |   14 | F     |
| fn: run_codex_tool_session_inner                   |   12 | 1.0  |   12 |   12 | F     |
| method: open_session_with_sandbox                  |   12 | 1.0  |   12 |   12 | F     |
| fn: run_command_under_seatbelt                     |    1 | 0.0  |    1 |    1 | A     |
| fn: run_command_under_landlock                     |    1 | 0.0  |    1 |    1 | A     |
+====================================================+======+======+======+======+=======+

CBO = Coupling Between Objects    LCOM = Lack of Cohesion of Methods
RFC = Response For a Class        WMC  = Weighted Methods per Class
```

### 4.2 Analysis

```
WELL-DESIGNED (Grade A):
+-----------------------------------------------------------------------+
| run_command_under_seatbelt()  CBO=1  LCOM=0.0                        |
| run_command_under_landlock()  CBO=1  LCOM=0.0                        |
|                                                                       |
| PATTERN: Clean facade functions with single responsibility.           |
| Both delegate to a shared run_command_under_sandbox() helper.         |
| This is textbook Single Responsibility Principle at work.             |
+-----------------------------------------------------------------------+

TIGHTLY COUPLED (Grade F - CRITICAL):
+-----------------------------------------------------------------------+
| codex.rs                      CBO=1124  (GOD FILE)                   |
| chatwidget.rs                 CBO=1075  (GOD FILE)                   |
| codex_message_processor.rs    CBO=929   (GOD FILE)                   |
|                                                                       |
| PATTERN: These three files are the dominant "gravity wells" of the    |
| codebase. codex.rs alone has 1,124 outbound coupling edges, meaning  |
| it touches over 1,100 other entities. This is a maintenance risk.    |
+-----------------------------------------------------------------------+

HIGH COUPLING (Grade F - FUNCTIONS):
+-----------------------------------------------------------------------+
| cli_main()                   CBO=52  (Entry point dispatcher)        |
| ToolOrchestrator::run()      CBO=28  (Tool execution coordinator)    |
| run_exec_like()              CBO=24  (Shell command handler)         |
|                                                                       |
| PATTERN: These are "orchestrator" functions that coordinate many     |
| subsystems. High CBO is somewhat expected for orchestrators, but     |
| cli_main(CBO=52) is doing too much -- it handles auth, sandbox,     |
| config, sessions, schema gen, and feature flags all in one function. |
+-----------------------------------------------------------------------+
```

---

## 5. Sandboxing Architecture

### 5.1 Platform Sandbox Strategy

OpenAI Codex implements a **tri-platform sandboxing architecture** that isolates
command execution at the OS kernel level:

```
+=========================================================================+
|                    CODEX SANDBOXING ARCHITECTURE                        |
+=========================================================================+
|                                                                         |
|  cli_main()                                                            |
|      |                                                                  |
|      +---> SandboxCommand (enum in cli/src/main.rs)                    |
|            |                                                            |
|   +--------+----------+-----------------+                               |
|   |                   |                 |                               |
|   v                   v                 v                               |
| [macOS]            [Linux]          [Windows]                          |
| SeatbeltCommand    LandlockCommand  run_command_under_windows()        |
|   |                  |                |                                 |
|   v                  v                v                                 |
| run_command_under  run_command_     Windows ACL-based sandbox          |
| _seatbelt()        under_landlock()  (windows-sandbox-rs crate)       |
|   |                  |                |                                 |
|   v                  v                v                                 |
| run_command_under  run_command_     +---> apply_read_acls()            |
| _sandbox()         under_sandbox()  +---> add_deny_write_ace()        |
| (shared helper)    (shared helper)  +---> allow_null_device()         |
|   |                  |              +---> apply_no_network_to_env()    |
|   v                  v              +---> setup_orchestrator           |
| spawn_command_     spawn_command_        run_setup_refresh()           |
| under_seatbelt()   under_linux_                                        |
| (codex_core)       sandbox()                                           |
|                    (codex_core)                                         |
|                      |                                                  |
|                      v                                                  |
|                    bubblewrap (bwrap)                                   |
|                    [vendored C code]                                    |
|                    build_bwrap_argv()                                   |
+=========================================================================+
```

### 5.2 Linux Sandbox: Landlock + Bubblewrap

```
+=========================================================================+
|                LINUX SANDBOX ARCHITECTURE                               |
+=========================================================================+
|                                                                         |
|  codex-rs/linux-sandbox/                                               |
|  +--- src/lib.rs                                                       |
|       +--- mod landlock                                                |
|       +--- mod vendored_bwrap                                          |
|                                                                         |
|  codex-rs/linux-sandbox/src/linux_run_main.rs                          |
|  +--- LandlockCommand (struct)                                         |
|  +--- build_bwrap_argv()                                               |
|       |                                                                 |
|       +---> Constructs bubblewrap arguments for namespace isolation    |
|       +---> Sets up filesystem mount binds (read-only / read-write)   |
|       +---> Configures /proc mount                                    |
|       +---> Detects proc mount permission denied failures             |
|                                                                         |
|  codex-rs/vendor/bubblewrap/ (vendored C)                              |
|  +--- bubblewrap.c    [Main namespace setup: PID, mount, user, net]   |
|  +--- bind-mount.c    [Filesystem bind mount logic]                   |
|  +--- network.c       [Network namespace: loopback_setup, rtnl_*]     |
|  +--- utils.c         [Utility functions, socket passing]             |
|                                                                         |
|  SECURITY LAYERS:                                                      |
|  +---> [L1] Linux Namespaces (via bubblewrap/bwrap)                   |
|  |     - PID namespace isolation                                       |
|  |     - Mount namespace with bind mounts                              |
|  |     - Network namespace (optional loopback)                         |
|  |     - User namespace for rootless operation                         |
|  |     - Seccomp BPF filter (sock_filter, sock_fprog)                 |
|  |                                                                      |
|  +---> [L2] Landlock LSM (Linux Security Module)                      |
|  |     - Filesystem access control (AccessFs rules)                    |
|  |     - install_filesystem_landlock_rules_on_current_thread()         |
|  |     - Ruleset -> RulesetAttr -> RulesetCreatedAttr pipeline         |
|  |     - ABI version negotiation (CompatLevel)                         |
|  |                                                                      |
|  +---> [L3] exec-server escalation policy                             |
|        - ExecPolicyOutcome (enum in exec-server/src/posix/)           |
|        - mcp_escalation_policy.rs                                      |
|        - escalate_protocol.rs (EscalateAction enum)                   |
+=========================================================================+
```

### 5.3 macOS Sandbox: Seatbelt (sandbox-exec)

```
+=========================================================================+
|                macOS SANDBOX ARCHITECTURE                               |
+=========================================================================+
|                                                                         |
|  codex-rs/core/src/lib.rs                                              |
|  +--- mod seatbelt                                                     |
|  +--- mod seatbelt_permissions                                         |
|                                                                         |
|  codex-rs/cli/src/debug_sandbox/seatbelt.rs                           |
|  +--- DenialLogger (struct)                                            |
|  |    +--- new()          [Initialize logger]                          |
|  |    +--- on_child_spawn() [Hook after child process starts]         |
|  |    +--- finish()        [Collect denial log]                        |
|  |                                                                      |
|  +--- SandboxDenial (struct)                                           |
|  +--- parse_message()     [Parse seatbelt denial messages]            |
|  +--- start_log_stream()  [Monitor sandbox violations]                |
|                                                                         |
|  SECURITY FLOW:                                                        |
|  +---> spawn_command_under_seatbelt()  [codex_core export]            |
|  |     - Builds seatbelt profile with extensions                       |
|  |     - build_macos_seatbelt_profile_extensions()                     |
|  |     - build_seatbelt_extensions()                                   |
|  |     - create_seatbelt_command_args()                                |
|  |     - create_seatbelt_command_args_with_extensions()                |
|  |                                                                      |
|  +---> compile_permission_profile()                                    |
|  |     - Translates SandboxPermissions to seatbelt SBPL              |
|  |     - from_permissions_with_network()                               |
|  |                                                                      |
|  +---> DenialLogger captures violations via log stream                |
|        - assert_seatbelt_denied() for testing                         |
+=========================================================================+
```

### 5.4 Windows Sandbox: ACL-Based

```
+=========================================================================+
|                WINDOWS SANDBOX ARCHITECTURE                             |
+=========================================================================+
|                                                                         |
|  codex-rs/windows-sandbox-rs/                                          |
|  +--- src/lib.rs                                                       |
|  |    +--- applies_network_block_for_read_only()                      |
|  |    +--- applies_network_block_when_access_is_disabled()            |
|  |    +--- apply_world_writable_scan_and_denies()                     |
|  |                                                                      |
|  +--- src/acl.rs              [Windows ACL manipulation]              |
|  |    +--- add_allow_ace()    [Add allow ACE to DACL]                |
|  |    +--- add_deny_write_ace() [Deny write permissions]              |
|  |    +--- allow_null_device()  [Allow NUL device access]            |
|  |                                                                      |
|  +--- src/env.rs                                                       |
|  |    +--- apply_no_network_to_env() [Env-based network blocking]    |
|  |                                                                      |
|  +--- src/read_acl_mutex.rs                                           |
|  |    +--- acquire_read_acl_mutex() [Thread-safe ACL operations]     |
|  |                                                                      |
|  +--- src/workspace_acl.rs                                             |
|  |    +--- protect_workspace_agents_dir() [Agent dir protection]     |
|  |                                                                      |
|  +--- src/setup_orchestrator.rs  [Setup workflow]                     |
|  |    +--- SetupMarker         [Detect if setup complete]            |
|  |    +--- SandboxUsersFile    [Multi-user sandbox config]           |
|  |    +--- SandboxUserRecord   [Per-user sandbox record]             |
|  |    +--- ElevationPayload    [UAC elevation data]                  |
|  |    +--- run_elevated_setup() [Run with admin privileges]          |
|  |    +--- run_setup_refresh()  [Refresh ACLs after config change]   |
|  |    +--- build_payload_roots() [Compute filesystem roots]          |
|  |    +--- gather_read_roots() / gather_write_roots()                |
|  |    +--- filter_sensitive_write_roots()                             |
|  |                                                                      |
|  +--- sandbox_smoketests.py    [Python integration tests]            |
|       +--- CaseResult (class)                                          |
|       +--- run_sbx()                                                   |
|       +--- assert_exists() / assert_not_exists()                      |
|                                                                         |
|  SECURITY LAYERS:                                                      |
|  +---> [L1] Windows ACLs (NTFS DACL manipulation)                    |
|  |     - Deny-Write ACEs on filesystem paths                          |
|  |     - Allow ACEs for permitted directories                         |
|  |     - Read ACL mutex for thread safety                             |
|  |                                                                      |
|  +---> [L2] Environment-Based Network Blocking                       |
|  |     - apply_no_network_to_env() sets firewall-like env vars       |
|  |                                                                      |
|  +---> [L3] Elevation-Aware Setup                                     |
|        - is_elevated() check                                           |
|        - run_elevated_setup() for UAC prompt                           |
|        - Setup marker to avoid re-running                              |
+=========================================================================+
```

### 5.5 Unified Sandbox Flow (Cross-Platform)

```
+=========================================================================+
|              UNIFIED SANDBOX FLOW (ALL PLATFORMS)                       |
+=========================================================================+
|                                                                         |
|  User types command in TUI/CLI                                         |
|       |                                                                 |
|       v                                                                 |
|  ToolOrchestrator::run()                                               |
|       |                                                                 |
|       +---> sandbox_mode_for_first_attempt()                           |
|       |     +---> select_initial()                                     |
|       |     +---> sandbox_preference()                                 |
|       |                                                                 |
|       +---> run_attempt()                                              |
|       |     |                                                           |
|       |     +---> begin_network_approval()   [Network access check]    |
|       |     +---> network_approval_spec()    [Get network policy]      |
|       |     +---> mode()                     [Determine sandbox mode]  |
|       |     |                                                           |
|       |     +---> run() -----------+                                   |
|       |                            |                                    |
|       |     +--- [if fails] -------+                                   |
|       |     |                                                           |
|       |     +---> finish_deferred_network_approval()                   |
|       |     +---> finish_immediate_network_approval()                  |
|       |                                                                 |
|       +---> [if rejected]:                                             |
|       |     +---> should_bypass_approval()                             |
|       |     +---> start_approval_async()    [User approval prompt]    |
|       |     +---> tool_decision()           [Record decision]         |
|       |     +---> Rejected()                [Block execution]          |
|       |                                                                 |
|       +---> [if escalation needed]:                                    |
|             +---> escalate_on_failure()                                 |
|             +---> wants_no_sandbox_approval()                          |
|             +---> build_denial_reason_from_output()                    |
+=========================================================================+
```

---

## 6. Tool System Architecture

### 6.1 Tool Registry and Dispatch

```
+=========================================================================+
|                    TOOL SYSTEM ARCHITECTURE                             |
+=========================================================================+
|                                                                         |
|  protocol/src/dynamic_tools.rs                                         |
|  +--- DynamicToolCallOutputContentItem (enum)                          |
|                                                                         |
|  core/src/tools/                                                       |
|  +--- registry.rs                                                      |
|  |    +--- ToolKind (enum)         [Categorize tool types]            |
|  |    +--- dispatch()              [Route tool calls to handlers]     |
|  |    +--- dispatch_after_tool_use_hook()  [Post-execution hooks]     |
|  |                                                                      |
|  +--- orchestrator.rs                                                  |
|  |    +--- ToolOrchestrator (struct)                                   |
|  |    |    +--- run()              [Main entry: approval + sandbox]    |
|  |    |    +--- run_attempt()      [Single execution attempt]         |
|  |    |    +--- new()              [Constructor]                       |
|  |    +--- OrchestratorRunResult (struct)                              |
|  |    +--- build_denial_reason_from_output()                           |
|  |                                                                      |
|  +--- events.rs                                                        |
|  |    +--- ToolEmitter (enum)      [Event source tracking]            |
|  |    +--- ToolEventStage (enum)   [begin/running/complete]           |
|  |    +--- ToolEventFailure (enum) [Failure classification]           |
|  |    +--- emit_exec_command_begin() / emit_exec_end()                |
|  |    +--- emit_exec_stage() / emit_patch_end()                       |
|  |                                                                      |
|  +--- mod.rs                                                           |
|  |    +--- build_content_with_timeout()                                |
|  |    +--- format_exec_output_for_model_freeform()                    |
|  |    +--- format_exec_output_for_model_structured()                  |
|  |    +--- format_exec_output_str()                                   |
|  |                                                                      |
|  +--- handlers/                                                        |
|       +--- shell.rs                                                    |
|       |    +--- handle()           [Entry for shell commands]         |
|       |    +--- run_exec_like()    [Execute shell-like commands]      |
|       |                                                                 |
|       +--- apply_patch.rs                                              |
|       |    +--- handle()           [Entry for patch operations]       |
|       |    +--- intercept_apply_patch()  [Patch interception logic]   |
|       |                                                                 |
|       +--- unified_exec.rs                                             |
|       |    +--- handle()           [Unified exec entry]               |
|       |                                                                 |
|       +--- multi_agents.rs                                             |
|       |    +--- handle()           [Multi-agent tool handler]         |
|       |    +--- spawn_agent_*      [Agent lifecycle]                  |
|       |    +--- send_input_*       [Agent input]                      |
|       |    +--- wait_*             [Agent wait/timeout]               |
|       |                                                                 |
|       +--- test_sync.rs           [Test synchronization barrier]      |
|                                                                         |
+=========================================================================+
```

### 6.2 Tool Execution Flow

```
  User Request ("run ls -la")
       |
       v
  tools::registry::dispatch()         <-- Route to correct handler
       |
       +---> [ToolKind match]
       |     +--- Shell     -> shell::handle()
       |     +--- ApplyPatch -> apply_patch::handle()
       |     +--- UnifiedExec -> unified_exec::handle()
       |     +--- MultiAgent -> multi_agents::handle()
       |     +--- Dynamic   -> [MCP dynamic tool]
       |
       v
  shell::handle()
       |
       v
  shell::run_exec_like()
       |
       +---> requires_escalated_permissions()   [Security gate]
       +---> intercept_apply_patch()            [Patch intercept]
       +---> dependency_env()                   [Set up env]
       +---> create_exec_approval_requirement_for_command()
       |
       v
  ToolOrchestrator::run()               <-- Central orchestration
       |
       +---> sandbox_mode_for_first_attempt()
       +---> exec_approval_requirement()
       +---> should_bypass_approval()
       |     |
       |     +---> [if needs approval] start_approval_async()
       |     |     +---> tool_decision() -> [User approves/rejects]
       |     |
       |     +---> [if approved or auto-approved]
       |
       v
  ToolOrchestrator::run_attempt()
       |
       +---> begin_network_approval()    [Network access check]
       +---> [execute command in sandbox]
       +---> finish_*_network_approval()
       |
       v
  [Result returned to model as formatted output]
```

---

## 7. MCP Integration Architecture

### 7.1 MCP Server Stack

```
+=========================================================================+
|                    MCP INTEGRATION ARCHITECTURE                         |
+=========================================================================+
|                                                                         |
|  CODEX AS MCP SERVER (codex-rs/mcp-server/)                           |
|  =========================================                              |
|                                                                         |
|  mcp-server/src/main.rs                                                |
|       |                                                                 |
|       +---> arg0_dispatch_or_else()  [Symlink-based dispatch]         |
|       +---> run_main()               [Start MCP server]               |
|                                                                         |
|  mcp-server/src/message_processor.rs                                   |
|       |                                                                 |
|       +---> handle_tool_call_codex()                                   |
|       |     +---> run_codex_tool_session()                             |
|       |           +---> run_codex_tool_session_inner()  [Event loop]  |
|       |                                                                 |
|       +---> handle_tool_call_codex_session_reply()                     |
|             +---> run_codex_tool_session_reply()                       |
|                   +---> run_codex_tool_session_inner()                 |
|                                                                         |
|  mcp-server/src/codex_tool_runner.rs                                   |
|       |                                                                 |
|       +---> run_codex_tool_session_inner()                             |
|             |                                                           |
|             +---> next_event()                [Poll for events]        |
|             +---> send_event_as_notification() [Forward to client]    |
|             +---> effective_approval_id()                              |
|             |                                                           |
|             +---> APPROVAL ROUTING:                                    |
|             |     +---> handle_exec_approval_request()                 |
|             |     |     +---> send_request()   [Ask client for OK]    |
|             |     |     +---> on_exec_approval_response()             |
|             |     |                                                     |
|             |     +---> handle_patch_approval_request()                |
|             |           +---> send_request()                           |
|             |           +---> on_patch_approval_response()             |
|             |                                                           |
|             +---> create_call_tool_result_with_thread_id()            |
|             +---> send_response()                                      |
|                                                                         |
|  mcp-server/src/exec_approval.rs                                       |
|       +---> handle_exec_approval_request()                             |
|       |     CBO=14 | Grade=F                                          |
|       |     Sends approval prompt to MCP client with:                 |
|       |     - Command text                                             |
|       |     - Working directory                                        |
|       |     - Sandbox status                                           |
|       +---> on_exec_approval_response()                                |
|             Processes client's approve/deny response                   |
|                                                                         |
|  mcp-server/src/patch_approval.rs                                      |
|       +---> handle_patch_approval_request()                            |
|       +---> on_patch_approval_response()                               |
|                                                                         |
+=========================================================================+
```

### 7.2 RMCP Client (Codex as MCP Client)

```
+=========================================================================+
|              CODEX AS MCP CLIENT (rmcp-client)                         |
+=========================================================================+
|                                                                         |
|  codex-rs/rmcp-client/                                                 |
|  +--- src/rmcp_client.rs                                               |
|  |    +--- ClientState (enum)                                          |
|  |    |    [Connecting -> Authenticating -> Ready -> Error]            |
|  |    +--- PendingTransport (enum)                                     |
|  |    +--- create_oauth_transport_and_runtime()                       |
|  |                                                                      |
|  +--- src/perform_oauth_login.rs                                       |
|  |    +--- CallbackOutcome (enum)                                      |
|  |    [Handles OAuth2 device flow for MCP server auth]                |
|  |                                                                      |
|  +--- src/bin/                                                         |
|       +--- test_stdio_server.rs        [Test stdio transport]         |
|       +--- test_streamable_http_server.rs [Test HTTP transport]       |
|       +--- rmcp_test_server.rs         [Test server]                  |
|                                                                         |
|  codex-rs/core/src/mcp/                                                |
|  +--- auth.rs                                                          |
|  |    +--- McpOAuthLoginSupport (enum)                                |
|  |    +--- compute_auth_status()                                       |
|  |    +--- compute_auth_statuses()                                     |
|  |    +--- oauth_login_support()                                       |
|  |                                                                      |
|  codex-rs/core/src/connectors.rs                                       |
|  +--- accessible_connectors_from_mcp_tools()                          |
|  +--- list_accessible_connectors_from_mcp_tools()                     |
|  +--- list_accessible_connectors_from_mcp_tools_with_options()        |
|  +--- list_cached_accessible_connectors_from_mcp_tools()             |
|                                                                         |
|  TRANSPORT MODES:                                                      |
|  +---> stdio  (standard input/output pipe)                            |
|  +---> streamable HTTP (SSE-based)                                    |
|  +---> OAuth2 device flow authentication                              |
|                                                                         |
+=========================================================================+
```

### 7.3 MCP Escalation Policy

```
+=========================================================================+
|              MCP ESCALATION POLICY                                      |
+=========================================================================+
|                                                                         |
|  codex-rs/exec-server/src/posix/                                       |
|  +--- mcp_escalation_policy.rs                                         |
|  |    +--- ExecPolicyOutcome (enum)                                    |
|  |         [Allow | Deny | Escalate]                                  |
|  |                                                                      |
|  +--- escalate_protocol.rs                                             |
|       +--- EscalateAction (enum)                                       |
|            [Defines what actions require escalation]                   |
|                                                                         |
|  FLOW:                                                                 |
|  MCP client requests tool call                                         |
|       |                                                                 |
|       v                                                                 |
|  ExecPolicyOutcome = evaluate_policy(command)                          |
|       |                                                                 |
|       +---> Allow     -> Execute immediately                           |
|       +---> Deny      -> Return error to client                       |
|       +---> Escalate  -> Send approval request via MCP protocol       |
|                          (handle_exec_approval_request)                 |
|                          Wait for client response                      |
|                          (on_exec_approval_response)                   |
|                                                                         |
+=========================================================================+
```

### 7.4 MCP + Approval Integration Diagram

```
+===========================================================================+
|        MCP CLIENT              |     CODEX MCP SERVER                     |
|  (e.g., Claude Desktop)       |     (codex-rs/mcp-server)                |
+===============================+==========================================+
|                               |                                           |
|  call_tool("codex", {...})    |                                           |
|  --------------------------->  |  handle_tool_call_codex()                |
|                               |       |                                   |
|                               |       v                                   |
|                               |  run_codex_tool_session()                |
|                               |       |                                   |
|                               |       v                                   |
|                               |  run_codex_tool_session_inner()          |
|                               |       |                                   |
|                               |       +---> next_event() [loop]          |
|                               |       |                                   |
|                               |       +---> [ExecApproval event]         |
|                               |       |     handle_exec_approval_request()|
|                               |       |         |                         |
|  <-- sampling/createMessage --+-------+---------+                         |
|  "Approve command: ls -la?"   |       |                                   |
|                               |       |                                   |
|  "approved" ------------------> ------+---> on_exec_approval_response()  |
|                               |       |                                   |
|                               |       +---> [PatchApproval event]        |
|                               |       |     handle_patch_approval_request|
|                               |       |         |                         |
|  <-- sampling/createMessage --+-------+---------+                         |
|  "Approve patch to foo.rs?"   |       |                                   |
|                               |       |                                   |
|  "approved" ------------------> ------+---> on_patch_approval_response() |
|                               |       |                                   |
|                               |       +---> send_event_as_notification() |
|  <-- notification/progress ---+-------+     [Progress updates]           |
|                               |       |                                   |
|                               |       v                                   |
|  <-- tool result -------------+  create_call_tool_result_with_thread_id()|
|                               |                                           |
+===============================+==========================================+
```

---

## 8. Complexity Hotspots (File Level)

### 8.1 Top Files by Coupling Degree

```
+==================================================================+
| RANK | OUTBOUND | FILE                                           |
+======+==========+================================================+
|   1  |   1,124  | codex-rs/core/src/codex.rs                     |
|   2  |   1,075  | codex-rs/tui/src/chatwidget.rs                 |
|   3  |     929  | codex-rs/app-server/src/codex_message_proc.rs  |
|   4  |     585  | codex-rs/tui/src/app.rs                        |
|   5  |     544  | codex-rs/tui/src/bottom_pane/chat_composer.rs  |
|   6  |     398  | codex-rs/core/src/config/mod.rs                |
|   7  |     391  | codex-rs/tui/src/chatwidget/tests.rs           |
|   8  |     374  | codex-rs/app-server/bespoke_event_handling.rs  |
|   9  |     373  | codex-rs/tui/src/history_cell.rs               |
|  10  |     282  | codex-rs/app-server-test-client/src/lib.rs     |
|  11  |     272  | codex-rs/app-server-protocol/protocol/v2.rs    |
|  12  |     270  | codex-rs/tui/src/resume_picker.rs              |
|  13  |     264  | codex-rs/core/src/tools/js_repl/mod.rs         |
|  14  |     261  | codex-rs/tui/src/bottom_pane/mod.rs            |
+==================================================================+

ANALYSIS:
- codex.rs (CBO=1124) is the GOD FILE. It likely contains the main
  Codex struct with methods touching every subsystem.
- chatwidget.rs (CBO=1075) is the TUI's central widget, managing
  the chat interface, agent spawning, tool display, and session
  state all in one file.
- codex_message_processor.rs (CBO=929) handles all app-server
  messages, another monolithic coordinator.
```

### 8.2 Architecture Risk Map

```
+==================================================================+
|                    ARCHITECTURE RISK MAP                          |
+==================================================================+
|                                                                    |
|  HIGH RISK (God files, CBO > 500):                               |
|  +-----------------------------------------------------------+   |
|  |                                                             |  |
|  |  codex.rs ---------> chatwidget.rs                         |  |
|  |  (CBO=1124)          (CBO=1075)                            |  |
|  |       \                    /                                |  |
|  |        \                  /                                 |  |
|  |         v                v                                  |  |
|  |  codex_message_processor.rs                                |  |
|  |  (CBO=929)                                                  |  |
|  |         |                                                   |  |
|  |         v                                                   |  |
|  |  app.rs (CBO=585)     config/mod.rs (CBO=398)             |  |
|  |                                                             |  |
|  +-----------------------------------------------------------+   |
|                                                                    |
|  MEDIUM RISK (CBO 20-50):                                        |
|  +-----------------------------------------------------------+   |
|  |  cli_main()            CBO=52                              |  |
|  |  ToolOrchestrator::run CBO=28                              |  |
|  |  run_exec_like()       CBO=24                              |  |
|  |  intercept_apply_patch CBO=19                              |  |
|  +-----------------------------------------------------------+   |
|                                                                    |
|  LOW RISK (CBO 1-5, well-designed):                              |
|  +-----------------------------------------------------------+   |
|  |  run_command_under_seatbelt()  CBO=1  Grade=A             |  |
|  |  run_command_under_landlock()  CBO=1  Grade=A             |  |
|  +-----------------------------------------------------------+   |
|                                                                    |
+==================================================================+
```

---

## 9. Key Architectural Insights

### 9.1 Design Patterns Observed

```
PATTERN 1: PLATFORM FACADE
=============================
The sandbox system uses a clean facade pattern:
- run_command_under_seatbelt() -> run_command_under_sandbox() [shared]
- run_command_under_landlock() -> run_command_under_sandbox() [shared]
- Platform-specific logic is isolated behind a common interface
- These are the ONLY Grade-A entities found (CBO=1)

PATTERN 2: ORCHESTRATOR + HANDLER
====================================
Tool execution follows Orchestrator pattern:
- ToolOrchestrator coordinates approval, sandbox, and execution
- Handlers (shell, apply_patch, unified_exec, multi_agents) provide
  specific tool logic
- registry::dispatch() routes to the correct handler

PATTERN 3: EVENT-DRIVEN MCP
==============================
MCP integration uses an event loop:
- run_codex_tool_session_inner() polls next_event()
- Events are classified and routed to approval handlers
- Responses are sent back via MCP protocol
- Approval flows use sampling/createMessage for human-in-the-loop

PATTERN 4: MULTI-AGENT HIERARCHY
===================================
Agent system supports nested agents:
- spawn_agent() creates sub-agents with explorer role
- resume_agent() restores closed agents
- send_input() pushes data to running agents
- wait() with timeout for agent completion
- Depth limit prevents infinite recursion
```

### 9.2 Critical Dependencies

```
DEPENDENCY CHAIN (most critical path):

  User Input
     |
     v
  cli_main() -----> CBO=52
     |
     v
  ToolOrchestrator::run() -----> CBO=28, Blast Radius=3,489 (21.9%)
     |
     v
  run_attempt() -----> CBO=14
     |
     +---> begin_network_approval()
     +---> [sandbox execution]
     +---> finish_network_approval()
     |
     v
  open_session_with_sandbox() -----> CBO=12
     |
     +---> create_exec_approval_requirement_for_command()
     +---> create_process() [platform-specific sandbox]
     |
     v
  [Platform Sandbox Layer]
     +---> spawn_command_under_seatbelt()  [macOS]
     +---> spawn_command_under_linux_sandbox() [Linux]
     +---> Windows ACL sandbox [Windows]
```

### 9.3 Recommendations for Competitors

```
STRENGTHS TO LEARN FROM:
1. Tri-platform sandbox with kernel-level isolation
2. Clean platform facade (CBO=1 for sandbox entry points)
3. Event-driven MCP integration with approval routing
4. Multi-agent support with depth limiting
5. ExecPolicy system for command classification

WEAKNESSES TO EXPLOIT:
1. God files (codex.rs CBO=1124) - fragile to changes
2. cli_main() does too much (CBO=52) - should be split
3. ToolOrchestrator::run() has 21.9% blast radius - single point of failure
4. All LCOM=1.0 for critical functions - low internal cohesion
5. Many unresolved references in graph - dependency tracking could be better
```

---

## 10. Entity Count Summary

```
+=======================================+
| SEARCH TERM    | ENTITIES FOUND       |
+================+=======================+
| sandbox        | 415                  |
| tool           | 555                  |
| agent          | 183                  |
| session        | 194                  |
| exec           | 786                  |
| mcp            | 550                  |
| protocol       | 1,106               |
| command        | 344                  |
| approval       | 178                  |
| patch          | 214                  |
| landlock       | 16                   |
| seatbelt       | 23                   |
| permission     | 22                   |
| policy         | 385                  |
| network        | 207                  |
| execpolicy     | 248                  |
| rmcp           | 223                  |
| orchestrator   | 33                   |
| context_manager| 16                   |
+================+=======================+
| TOTAL CODEBASE | 15,901 entities      |
|                | 136,130 edges        |
+=======================================+
```

---

*Generated by Parseltongue v1.7.3 graph analysis on 2026-02-19*
*Server: http://localhost:7780 | DB: rocksdb:parseltongue20260219195022/analysis.db*
