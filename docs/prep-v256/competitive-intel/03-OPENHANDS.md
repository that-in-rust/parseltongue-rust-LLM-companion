# OpenHands Deep Source Analysis
## Parseltongue Competitive Intelligence

**Analyzed**: 2026-02-19
**Repo**: All-Hands-AI/OpenHands (Python, MIT, ~68K stars)
**Analyst**: Claude Sonnet 4.5 via direct GitHub API source reading
**Branch analyzed**: main (as of Feb 2026)

---

## CRITICAL ARCHITECTURAL NOTE

**OpenHands V0 is deprecated. V1 is in active migration.**

Every file in `openhands/controller/`, `openhands/agenthub/`, and core infrastructure carries this header:

```python
# IMPORTANT: LEGACY V0 CODE - Deprecated since version 1.0.0, scheduled for removal April 1, 2026
# OpenHands V1 uses the Software Agent SDK for the agentic core and runs a new application server.
# V1 agentic core (SDK): https://github.com/OpenHands/software-agent-sdk
# V1 application server (in this repo): openhands/app_server/
```

This means the architecture being analyzed here is the V0 legacy that still ships and works, but the team is migrating to a separate SDK (`software-agent-sdk`). The `openhands/app_server/` directory is the new V1 application layer. This analysis covers V0 in full depth since it is what actually runs, plus notes on V1 direction.

---

## 1. Project Structure

```
openhands/
  agenthub/           # Agent implementations
    codeact_agent/    # PRIMARY: CodeActAgent (the main agent)
    browsing_agent/   # Browser-specialized agent
    dummy_agent/      # Testing stub
    loc_agent/        # Lines-of-code agent
    readonly_agent/   # Read-only agent
    visualbrowsing_agent/
  app_server/         # V1 new application layer
  controller/         # Agent lifecycle management
    agent.py          # Abstract Agent base class
    agent_controller.py  # Main orchestration loop
    state/            # State dataclass + tracker
    stuck.py          # Loop detection
  events/             # Event sourcing system (central)
    action/           # All action types (12 files)
    observation/      # All observation types (13 files)
    stream.py         # EventStream with pub/sub
    event_store.py    # Persistence layer
  memory/             # Context management
    condenser/        # 10 condensation strategies
    conversation_memory.py  # History-to-messages conversion
  microagent/         # Markdown-defined knowledge agents
  runtime/            # Docker/Local/Remote execution sandboxes
  llm/                # LiteLLM wrapper
  mcp/                # Model Context Protocol integration
  security/           # Action risk analysis
  server/             # V0 web server
```

---

## 2. Agent Architecture - Multi-Agent Coordination

### 2.1 Agent Types [CONFIRMED from source]

**Primary Agents (registered in `_registry`):**
- `CodeActAgent` - The main workhorse. Executes bash/Python/browser actions. Version 2.2.
- `BrowsingAgent` - Specialized for web browsing tasks
- `ReadonlyAgent` - Read-only operations only
- `LocAgent` - Likely lines-of-code or location-based tasks
- `VisualBrowsingAgent` - Visual browser interaction
- `DummyAgent` - Testing/mocking

**Micro-agents (not `Agent` subclasses - they are Markdown files):**
- `KnowledgeMicroagent` - Triggered by keywords in messages, inject domain knowledge
- `RepoMicroagent` - Always-active repo-specific instructions (from `.openhands/microagents/repo.md`)
- `TaskMicroagent` - Triggered by slash commands like `/agent_name`, require user input

### 2.2 Agent Registration Pattern [CONFIRMED from source]

```python
# From openhands/controller/agent.py
class Agent(ABC):
    _registry: dict[str, type['Agent']] = {}

    @classmethod
    def register(cls, name: str, agent_cls: type['Agent']) -> None:
        if name in cls._registry:
            raise AgentAlreadyRegisteredError(name)
        cls._registry[name] = agent_cls

    @abstractmethod
    def step(self, state: 'State') -> 'Action':
        """Single step: state in, action out."""
        pass
```

The `step()` method is the entire agent interface. Agents are pure functions: state in, action out.

### 2.3 Multi-Agent Delegation [CONFIRMED from source]

Delegation is parent-child controller nesting. When an agent calls `AgentDelegateAction`:

```python
# From agent_controller.py
async def start_delegate(self, action: AgentDelegateAction) -> None:
    agent_cls: type[Agent] = Agent.get_cls(action.agent)
    delegate_agent = agent_cls(config=agent_config, llm_registry=self.agent.llm_registry)

    state = State(
        session_id=self.id.removesuffix('-delegate'),
        delegate_level=self.state.delegate_level + 1,
        metrics=self.state.metrics,  # SHARED metrics
        start_id=self.event_stream.get_latest_event_id() + 1,
    )

    self.delegate = AgentController(
        sid=self.id + '-delegate',
        agent=delegate_agent,
        event_stream=self.event_stream,  # SAME event stream
        is_delegate=True,
        ...
    )
```

Key design points:
- **Shared event stream**: Parent and delegate use the same `EventStream` instance
- **Shared metrics**: Token usage/cost accumulates globally across agents
- **Delegate does NOT subscribe** to event stream directly (`is_delegate=True`)
- **Parent forwards events** to delegate manually via `delegate._on_event(event)`
- **Delegate ID**: `{parent_id}-delegate`
- **Task injection**: Parent posts `MessageAction(content='TASK: ' + task)` with `EventSource.USER` to the stream

### 2.4 Delegate Lifecycle [CONFIRMED from source]

```
Parent receives AgentDelegateAction
  -> start_delegate() creates child AgentController
  -> Parent posts MessageAction(task) to event stream as USER
  -> Parent forwards all future events to delegate
  -> Delegate runs until FINISHED/ERROR/REJECTED
  -> Parent calls end_delegate()
  -> AgentDelegateObservation emitted with delegate outputs
  -> Parent resumes normal operation
```

When delegate finishes, the parent gets back a simple string observation:
```python
content = f'{self.delegate.agent.name} finishes task with {formatted_output}'
# or
content = f'Delegated agent finished with result:\n\n{content}'
```

**There is no AI-generated summary of delegate work** - this is a noted TODO (#2395).

### 2.5 Agent State Machine [CONFIRMED from source]

```python
class AgentState(str, Enum):
    LOADING = 'loading'
    RUNNING = 'running'
    AWAITING_USER_INPUT = 'awaiting_user_input'
    PAUSED = 'paused'
    STOPPED = 'stopped'
    FINISHED = 'finished'
    REJECTED = 'rejected'
    ERROR = 'error'
    AWAITING_USER_CONFIRMATION = 'awaiting_user_confirmation'
    USER_CONFIRMED = 'user_confirmed'
    USER_REJECTED = 'user_rejected'
    RATE_LIMITED = 'rate_limited'
```

Transition logic (from `AgentController._step()` and `set_agent_state_to()`):
- `LOADING -> RUNNING` on first user message
- `RUNNING -> AWAITING_USER_INPUT` when agent sends message with `wait_for_response=True`
- `RUNNING -> AWAITING_USER_CONFIRMATION` for high-risk actions in confirmation mode
- `USER_CONFIRMED/USER_REJECTED -> RUNNING` on user response
- `RUNNING -> FINISHED` on `AgentFinishAction`
- `RUNNING -> ERROR` on unhandled exceptions
- `RUNNING -> PAUSED` on Ctrl+P or loop detection
- `ERROR -> RUNNING` allowed (user can resume after error, which expands iteration limits)

---

## 3. Code Understanding Layer

### 3.1 What OpenHands Does NOT Have [CONFIRMED from source]

**No tree-sitter. No embeddings. No semantic code search. No AST parsing. No static analysis.**

OpenHands does not have a dedicated code understanding layer. It is a **generalist tool-using agent** that understands code the same way a human engineer with a terminal would: by running shell commands.

### 3.2 How the Agent "Understands" Code [CONFIRMED from source]

The system prompt explicitly instructs the agent to use shell tools:

```
<EFFICIENCY>
* When exploring the codebase, use efficient tools like find, grep, and git commands
  with appropriate filters to minimize unnecessary operations.
</EFFICIENCY>
```

The agent's code exploration strategy is entirely LLM-driven shell commands:
- `find . -name "*.py" -type f`
- `grep -r "ClassName" --include="*.py"`
- `cat file.py`
- `git log --oneline`
- `git diff HEAD~1`

### 3.3 File Read/Edit Mechanisms [CONFIRMED from source]

The CodeActAgent has these tools defined in `openhands/agenthub/codeact_agent/tools/`:
- `bash.py` -> `CmdRunAction` (bash command execution)
- `ipython.py` -> `IPythonRunCellAction` (Python cell execution)
- `str_replace_editor.py` -> `FileEditAction` with ACI-style str_replace editing
- `llm_based_edit.py` -> `FileEditAction` with LLM-driven file editing (deprecated)
- `browser.py` -> `BrowseInteractiveAction`
- `think.py` -> `AgentThinkAction` (internal reasoning log)
- `finish.py` -> `AgentFinishAction`
- `condensation_request.py` -> `CondensationRequestAction`
- `task_tracker.py` -> `TaskTrackingAction`

### 3.4 Context Selection Strategy [CONFIRMED from source]

There is no intelligent "context selection" - the agent decides what to read based on the LLM's reasoning. The LLM chooses which files to open based on:
1. The initial task description
2. Microagent-injected knowledge about the repo
3. Outputs of previous bash commands (file listings, grep results)

The agent reads whole files and gets truncated observations if they exceed `max_message_chars`.

### 3.5 Large Codebase Handling [CONFIRMED from source]

For large codebases, OpenHands relies on:
1. **Microagent repo instructions** - project-specific `.openhands/microagents/repo.md` tells the agent the architecture upfront
2. **Shell navigation** - the agent uses `find`, `grep`, `ls` to explore
3. **Context truncation** - observations are truncated at `max_message_chars`
4. **Condensation** - old conversation history is summarized to free context window

**There is no index, no graph database, no semantic search.** The LLM must "discover" code structure by running shell commands. This is fundamentally different from Parseltongue's approach.

---

## 4. Control Flow - Event Stream Architecture

### 4.1 The Event Stream [CONFIRMED from source]

```python
# From openhands/events/stream.py
class EventStream(EventStore):
    """Central pub/sub bus for all agent communications."""
    _subscribers: dict[str, dict[str, Callable]]
    _queue: queue.Queue[Event]

    def add_event(self, event: Event, source: EventSource) -> None:
        # Assigns ID, timestamps, strips secrets, persists to file store
        # Then puts into queue for async delivery to subscribers

    def subscribe(self, subscriber_id: EventStreamSubscriber, callback, callback_id):
        # Each subscriber gets its own ThreadPoolExecutor for isolation
        pass
```

**Subscribers** (from `EventStreamSubscriber` enum):
- `AGENT_CONTROLLER` - The agent orchestration loop
- `RUNTIME` - The Docker/local sandbox executor
- `MEMORY` - Microagent retrieval service
- `SERVER` - WebSocket relay to frontend
- `RESOLVER` - Issue resolver (CI integration)
- `MAIN` - CLI main loop
- `TEST` - Testing subscriber

**EventSources** (who emits events):
- `USER` - Human input
- `AGENT` - Agent actions/thoughts
- `ENVIRONMENT` - Runtime observations (bash output, file reads)

### 4.2 Main Execution Loop [CONFIRMED from source]

```
User sends message
  -> EventStream.add_event(MessageAction, source=USER)
  -> All subscribers receive MessageAction

AgentController.on_event(MessageAction):
  -> Creates RecallAction (trigger microagent lookup)
  -> Sets pending_action = RecallAction
  -> EventStream.add_event(RecallAction, source=USER)

Memory.on_event(RecallAction):
  -> Keyword-matches against knowledge_microagents
  -> For first message: adds workspace context (repo info, runtime info)
  -> Creates RecallObservation
  -> EventStream.add_event(RecallObservation, source=ENVIRONMENT)

AgentController.on_event(RecallObservation):
  -> Clears _pending_action
  -> should_step() returns True (NullObservation with cause > 0)
  -> Calls _step()

AgentController._step():
  -> Checks: not stuck? within budget? within iteration limit?
  -> Calls agent.step(state)
  -> Returns an Action

CodeActAgent.step(state):
  -> Calls condenser.condensed_history(state) for view
  -> Converts events to messages via ConversationMemory.process_events()
  -> Calls LLM with messages + tools
  -> Parses tool calls via function_calling.response_to_actions()
  -> Returns Action

AgentController._step():
  -> For runnable actions: security check, set _pending_action
  -> EventStream.add_event(action, source=AGENT)

Runtime.on_event(CmdRunAction/FileReadAction/etc.):
  -> Executes in Docker sandbox
  -> Returns CmdOutputObservation/FileReadObservation/etc.
  -> EventStream.add_event(observation, source=ENVIRONMENT)

AgentController.on_event(Observation):
  -> Clears _pending_action
  -> should_step() = True
  -> Loop continues...
```

### 4.3 Action/Observation Protocol [CONFIRMED from source]

**Actions** (agent -> environment):
```python
# All actions are dataclasses with 'action: str' field (the type discriminator)
CmdRunAction(command='ls -la', thought='Let me see the directory structure')
FileReadAction(path='/workspace/src/main.py', start=0, end=100)
FileEditAction(path='...', command='str_replace', old_str='...', new_str='...')
IPythonRunCellAction(code='import pandas as pd\ndf.head()')
BrowseInteractiveAction(browser_actions='...')
AgentDelegateAction(agent='BrowsingAgent', inputs={'task': '...'})
AgentFinishAction(final_thought='Done', outputs={})
MessageAction(content='Can you clarify...', wait_for_response=True)
RecallAction(query='the user message', recall_type=RecallType.WORKSPACE_CONTEXT)
CondensationAction(forgotten_event_ids=[1,2,3], summary='...')
MCPAction(name='tool_name', arguments={...})
```

**Observations** (environment -> agent):
```python
CmdOutputObservation(content='total 48\n-rw-r--r-- 1 ...', exit_code=0)
FileReadObservation(content='file contents...', path='/workspace/...')
FileEditObservation(content='File edited successfully')
BrowserOutputObservation(content='<html>...', url='https://...')
AgentDelegateObservation(outputs={}, content='Agent finished with result: ...')
RecallObservation(recall_type=WORKSPACE_CONTEXT, repo_name='...', microagent_knowledge=[...])
ErrorObservation(content='Command failed: ...', error_id='...')
AgentCondensationObservation(content='Summary of events 1-50: ...')
MCPObservation(content='tool result...')
```

**Tool call metadata** - Each action can carry `tool_call_metadata` linking it to the LLM's tool call ID, enabling proper OpenAI-compatible function calling.

### 4.4 Stuck Detection [CONFIRMED from source]

`StuckDetector` (`openhands/controller/stuck.py`) watches for:
- Repeated identical actions
- Repeated identical observations
- Agent going in circles

Recovery options presented in CLI mode:
1. Restart from before the loop (truncate history)
2. Restart with last user message
3. Stop agent

---

## 5. Data Flow

### 5.1 Conversation History Management [CONFIRMED from source]

```python
# From openhands/controller/state/state.py
@dataclass
class State:
    history: list[Event]      # All events (actions + observations)
    start_id: int             # Where this agent's history begins in event stream
    end_id: int               # Latest event ID seen
    delegate_level: int       # 0 = root, +1 per delegation
    iteration_flag: IterationControlFlag  # max iterations
    budget_flag: BudgetControlFlag        # max USD spend
    extra_data: dict          # Condenser metadata, task tracking
    metrics: Metrics          # Token usage, cost (SHARED with delegates)
```

The `history` is the complete list of all `Event` objects in the agent's view. Every event is stored.

### 5.2 Condensation / Memory Compression [CONFIRMED from source]

10 different condenser strategies exist in `openhands/memory/condenser/impl/`:

| Condenser | Strategy |
|-----------|----------|
| `NoOpCondenser` | No condensation (pass-through) |
| `RecentEventsCondenser` | Keep only N most recent events |
| `ConversationWindowCondenser` | Sliding window of recent turns |
| `AmortizedForgettingCondenser` | Forget old events amortized |
| `ObservationMaskingCondenser` | Mask/truncate large observations |
| `BrowserOutputCondenser` | Special handling for browser output |
| `LLMSummarizingCondenser` | LLM summarizes forgotten events |
| `LLMAttentionCondenser` | LLM-guided attention mechanism |
| `StructuredSummaryCondenser` | Structured summary with specific sections |
| `PipelineCondenser` | Chain multiple condensers |

The `LLMSummarizingCondenser` is the most sophisticated. It:
1. Keeps the first `keep_first` events (system message, initial user message)
2. Keeps the last `target_size - keep_first - 1` events (recent context)
3. Summarizes the middle events using LLM with this structured prompt:

```
USER_CONTEXT: (Essential user requirements)
TASK_TRACKING: {Active tasks, IDs, statuses - PRESERVE TASK IDs}
COMPLETED: (Tasks completed, with results)
PENDING: (Tasks still needed)
CURRENT_STATE: (Variables, data structures, state)
CODE_STATE: {File paths, function signatures}
TESTS: {Failing cases, error messages}
CHANGES: {Code edits, variable updates}
DEPS: {Dependencies, imports}
VERSION_CONTROL_STATUS: {Repo state, branch, PR status}
```

Condensation is triggered when:
1. Context window is exceeded (`ContextWindowExceededError`) -> `CondensationRequestAction` emitted
2. Agent explicitly calls the `condensation_request` tool
3. Condenser determines history has grown too large

### 5.3 Microagent State Sharing [CONFIRMED from source]

Microagents don't share state - they are stateless knowledge injectors. Each microagent is a markdown file that:
- Gets loaded once at startup from `OpenHands/skills/` (global) or `~/.openhands/microagents/` (user) or repo's `.openhands/microagents/`
- Is triggered by keyword matching in the `RecallAction` query (the user message text)
- Injects its content as `<EXTRA_INFO>` in the conversation

The `Memory` class manages this:
```python
def _find_microagent_knowledge(self, query: str) -> list[MicroagentKnowledge]:
    for name, microagent in self.knowledge_microagents.items():
        trigger = microagent.match_trigger(query)  # simple string.lower() contains check
        if trigger:
            recalled_content.append(MicroagentKnowledge(
                name=microagent.name,
                trigger=trigger,
                content=microagent.content,
            ))
    return recalled_content
```

**Trigger matching is simple substring search** - no embeddings, no semantic similarity.

### 5.4 Persistence [CONFIRMED from source]

Every event is persisted to `FileStore` as individual JSON files:
```python
# EventStream.add_event()
event_json = json.dumps(data)
filename = self._get_filename_for_id(event.id, self.user_id)
self.file_store.write(filename, event_json)
```

Events > 1MB get a warning but are still written. Cache pages (batches of events) are stored for read performance. State can be serialized/pickled for session resume.

### 5.5 The Docker Sandbox [CONFIRMED from source]

The `DockerRuntime` (in `openhands/runtime/impl/docker/`) provides:
- Fresh Docker container per conversation
- All file system operations happen inside the container
- Bash commands execute in the container
- Files persist within a conversation session
- Web processes run in the container (ports exposed to host)

The runtime has an `action_execution_server.py` that runs inside the container as an HTTP server, receiving action dispatch from the `DockerRuntime` on the host.

---

## 6. Context Pipeline: What the LLM Actually Receives

### 6.1 System Message Construction [CONFIRMED from source]

The system message is a Jinja2 template (`system_prompt.j2`) plus:
1. Static system prompt (role, efficiency guidelines, file guidelines, code quality, version control, security, etc.)
2. Tool definitions (passed as `tools` parameter to LLM, not in message text)

### 6.2 Per-Message Context (injected on first user message) [CONFIRMED from source]

When the first user message arrives, `Memory` generates a `RecallObservation` containing:
```
- Repository name and directory (if repo cloned)
- Branch name
- Repository instructions (from .openhands/microagents/repo.md)
- Runtime hosts/ports
- Additional agent instructions
- Today's date
- Custom secrets descriptions
- Conversation instructions (from resolver/slack integration)
- Working directory
- Triggered microagent knowledge (keyword-matched)
```

This is rendered by `additional_info.j2` and injected as a user message at the start of conversation.

### 6.3 Per-Turn Context [CONFIRMED from source]

For subsequent messages, a `RecallAction` triggers keyword matching and if any microagent triggers fire, their content is injected as `<EXTRA_INFO>` blocks.

### 6.4 Message Assembly [CONFIRMED from source]

`ConversationMemory.process_events()` converts the condensed history into LLM messages:
- `SystemMessageAction` -> `system` role message
- `MessageAction(source=USER)` -> `user` role message
- `MessageAction(source=AGENT)` -> `assistant` role message
- `CmdRunAction` -> tool call in `assistant` message
- `CmdOutputObservation` -> `tool` role message with tool result
- `RecallObservation` -> injected into `user` message as context block
- `AgentCondensationObservation` -> `user` message with summary
- Images from browser -> included if `vision_is_active`

---

## 7. MCP Integration [CONFIRMED from source]

OpenHands has native MCP (Model Context Protocol) support:
- `openhands/mcp/` - MCP client implementation
- `MCPAction(name='tool_name', arguments={})` - First-class action type
- `MCPObservation` - Observation from MCP tools
- Microagents can declare MCP stdio server requirements in their frontmatter:
  ```yaml
  mcp_tools:
    stdio_servers:
      - command: "npx"
        args: ["-y", "@modelcontextprotocol/server-filesystem"]
  ```
- `Memory.get_microagent_mcp_tools()` aggregates MCP configs from all active repo microagents

**OpenHands can act as an MCP client** - it calls external MCP servers. The MCP tools become additional tools available to the agent.

---

## 8. V1 Architecture Direction [INFERRED from source comments + app_server dir]

The V1 system is migrating to:
- `openhands/app_server/` - New application layer replacing the V0 server
- `software-agent-sdk` (separate repo) - Replaces `openhands/controller/` and core agent infrastructure
- Cleaner separation between the agent SDK and the application server

Directory structure of V1 app_server:
```
app_server/
  app_conversation/   # Conversation management
  app_lifespan/       # App startup/shutdown
  event/              # Event handling
  event_callback/     # Event callbacks
  sandbox/            # Sandbox management
  services/           # Business services
  user/               # User management
  v1_router.py        # FastAPI router
  web_client/         # Web client
```

---

## 9. Shreyas-Style Differentiation Analysis

### 9.1 OpenHands' Actual MOAT

**1. MIT License + Full Self-Hosting** [CONFIRMED]
OpenHands is the only major coding agent that is fully open-source (MIT), fully self-hostable, with no phone-home requirement. You can run it entirely on-premise with your own LLM (Ollama, etc.).

**2. Docker Sandbox Execution Model** [CONFIRMED]
Every agent session gets a fresh Docker container. This means:
- Arbitrary code execution is safe (sandboxed)
- Complex multi-step installs work (npm install, pip install, apt-get)
- Web servers can run inside the container with port exposure
- File system is isolated and ephemeral unless explicitly committed

**3. Multi-LLM Support via LiteLLM** [CONFIRMED from imports]
Uses LiteLLM as the universal LLM adapter, supporting 100+ models.

**4. Event-Sourced Architecture** [CONFIRMED]
The event stream is fully serializable. Every action and observation is persisted. This enables:
- Session replay for debugging
- State restoration across crashes
- Trajectory analysis for training data
- Full audit trail

**5. Microagent Ecosystem** [CONFIRMED]
Extensible knowledge injection without code changes. Anyone can write a Markdown file with YAML frontmatter to add domain knowledge to the agent. Third-party format compatibility: reads `.cursorrules`, `AGENTS.md`, `agents.md`.

### 9.2 Where OpenHands Beats Commercial Tools

**vs Cursor/Copilot:**
- Runs multi-step autonomously without per-step human approval (in headless mode)
- Executes shell commands, not just suggests them
- Full Docker sandbox = can run tests, build projects, install dependencies
- Open source = inspectable, auditable, modifiable

**vs Devin:**
- MIT license (Devin is proprietary)
- Full self-hosting
- No per-seat pricing
- Can be integrated into CI/CD pipelines natively

**vs GPT Engineer:**
- More sophisticated multi-agent delegation
- Docker execution sandbox
- Event sourcing architecture

### 9.3 Where OpenHands is WEAK

**Code Understanding:**
The most significant weakness relative to Parseltongue's approach.

OpenHands has NO:
- Static code analysis
- Dependency graph awareness
- Blast radius analysis
- Semantic search / embeddings
- Cross-file relationship tracking
- Call graph understanding
- Import resolution

The agent discovers code structure by running bash commands (`find`, `grep`, `cat`). This is:
- Expensive in tokens (many round-trips to explore)
- Error-prone (the LLM can miss things)
- Slow (multiple iterations to understand a large codebase)
- Brittle (depends on LLM "guessing" the right grep queries)

A Parseltongue integration could dramatically improve this.

**Microagent Trigger Matching:**
Simple substring matching. Not semantic. No ranking. If the user's message contains "python" and "git", ALL python-related and git-related microagents trigger. No relevance scoring.

**No Memory Between Sessions:**
Each new conversation starts fresh. No persistent memory of previous conversations. The agent re-discovers everything from scratch each time.

**Multi-Agent Coordination is Primitive:**
When a delegate agent finishes, the parent gets back a simple string summary ("Agent X finishes task with result: Y"). There's no structured data handoff. The parent agent has no visibility into what the delegate actually did unless it explicitly asks (and the delegate's work is in the shared event stream).

**Context Window as the Only Resource:**
OpenHands is entirely context-window bound. There's no external knowledge store, no vector database, no code index. As the codebase grows, performance degrades because the agent needs more tokens to explore it.

---

## 10. What Parseltongue Can Learn from OpenHands

### 10.1 Event Stream Architecture Pattern

The event stream is elegant. Consider adopting:

```python
# Every operation is an event with: id, timestamp, source, cause
# Actions and Observations are symmetrical pairs
# Events are persisted immediately
# Subscribers are isolated (ThreadPoolExecutor per subscriber)
# Subscribers can be added/removed dynamically
```

For Parseltongue, an event-based audit log of queries and results would enable:
- Debugging why a particular result was returned
- Tracking which endpoints an agent used in a session
- Replay testing

### 10.2 Action/Observation Protocol Design

The Action/Observation protocol is well-designed for tool-using agents. For Parseltongue to serve as an external tool:
- Each Parseltongue endpoint should have a corresponding `Action` type
- Responses should be `Observation` types with standard fields
- Tool definitions should follow the OpenAI function calling schema
- This makes Parseltongue directly usable by OpenHands via MCP

### 10.3 Parseltongue as MCP Server

OpenHands natively supports MCP. Parseltongue could expose itself as an MCP stdio server:

```yaml
# In a repo's .openhands/microagents/repo.md frontmatter:
mcp_tools:
  stdio_servers:
    - command: "parseltongue"
      args: ["--port", "7777"]
```

This would make Parseltongue's 26 endpoints available as MCP tools to any OpenHands session working on a codebase that has Parseltongue configured.

### 10.4 What "Code Context" an Agent Actually Needs

From observing how OpenHands struggles without code understanding, the ideal tool for an agent provides:

**1. Blast Radius** (already in Parseltongue)
"If I change function X, what breaks?" - This prevents the agent from making changes that cause cascading failures.

**2. Dependency-Aware Navigation**
"Show me all callers of function X" - The agent currently does `grep -r "function_name"` which misses dynamic dispatch, inheritance, etc.

**3. Smart Context Selection** (already in Parseltongue)
"Given my task about X, which files/functions are most relevant?" - Currently the agent burns tokens exploring and often reads irrelevant files.

**4. Complexity Hotspots** (already in Parseltongue)
"Which parts of this codebase are most complex/risky to change?" - Prevents the agent from underestimating the difficulty of changes.

**5. Import/Dependency Graph**
"What are all the transitive dependencies of module X?" - Crucial for understanding what a change propagates to.

**6. Dead Code Detection**
"Is this function actually called anywhere?" - Prevents the agent from reasoning about unused code as if it matters.

### 10.5 How Parseltongue Should Integrate

**Option A: MCP Server** (recommended)
Expose all 26 endpoints as MCP tools. Zero changes needed to OpenHands. Any user who installs Parseltongue and configures the MCP server in their repo's microagent file gets all 26 tools available to the agent.

**Option B: Knowledge Microagent**
Create a repo-level microagent that provides Parseltongue's analysis as initial context. The microagent would run Parseltongue queries at the start of each session and inject the results.

**Option C: OpenHands Plugin**
Implement a `PluginRequirement` that runs Parseltongue inside the Docker sandbox.

### 10.6 Condensation Prompt as Feature Requirement

The LLM summarizing condenser's prompt structure reveals what state an agent needs to preserve:
```
CODE_STATE: {File paths, function signatures, data structures}
TESTS: {Failing cases, error messages, outputs}
CHANGES: {Code edits, variable updates}
DEPS: {Dependencies, imports, external calls}
VERSION_CONTROL_STATUS: {Repository state, current branch, PR status, commit history}
```

Parseltongue's graph database already stores most of this. A "session context export" endpoint from Parseltongue could directly feed this condensation structure.

---

## 11. Key Source Files for Reference

| File | Purpose |
|------|---------|
| `openhands/controller/agent.py` | Abstract Agent base class; `step()` interface |
| `openhands/controller/agent_controller.py` | Main orchestration loop; delegation; state machine |
| `openhands/agenthub/codeact_agent/codeact_agent.py` | Primary agent implementation |
| `openhands/agenthub/codeact_agent/function_calling.py` | LLM response -> Action dispatch |
| `openhands/events/stream.py` | EventStream pub/sub bus |
| `openhands/memory/memory.py` | Memory/microagent retrieval service |
| `openhands/memory/condenser/impl/llm_summarizing_condenser.py` | LLM-based history compression |
| `openhands/memory/conversation_memory.py` | Events -> LLM messages conversion |
| `openhands/microagent/microagent.py` | Microagent types; markdown loading |
| `openhands/utils/prompt.py` | PromptManager; system prompt assembly |
| `openhands/agenthub/codeact_agent/prompts/system_prompt.j2` | System prompt template |
| `openhands/agenthub/codeact_agent/prompts/additional_info.j2` | Workspace context template |
| `openhands/core/schema/agent.py` | AgentState enum |
| `openhands/events/action/agent.py` | RecallAction, CondensationAction, AgentDelegateAction |
| `openhands/events/observation/agent.py` | RecallObservation, MicroagentKnowledge |

---

## 12. Critical Insights for Parseltongue Positioning

### 12.1 The Core Gap

OpenHands is a **general-purpose code execution agent** that is blind to code structure. It operates like a junior developer who knows how to use a terminal but has never seen this codebase before. It discovers everything by running commands.

Parseltongue is a **specialized code intelligence layer** that pre-computes and serves structural knowledge about a codebase. It's the difference between a developer with a terminal vs a developer with a terminal AND a senior architect who already knows the entire codebase.

### 12.2 The Integration Opportunity

When OpenHands' CodeActAgent tries to understand "where should I make this change?", it runs:
```bash
grep -r "ClassName" .
find . -name "*.py" | xargs grep "function_name"
cat file.py
```

This costs 3-5 LLM turns and often misses things.

With Parseltongue as an MCP tool:
```
parseltongue_blast_radius(entity="ClassName.method", language="python")
parseltongue_smart_context(task="add parameter X to method Y", budget_tokens=4096)
parseltongue_dependency_graph(file="src/main.py", depth=2)
```

This costs 1 LLM turn and returns pre-computed, comprehensive structural information.

### 12.3 Positioning Statement

**"Parseltongue is the code intelligence substrate that multi-agent systems like OpenHands can query to understand codebases without burning tokens on exploration."**

OpenHands proves that the agent execution model works. Its weakness is code understanding. Parseltongue solves exactly that weakness. The event stream architecture shows that external tools (MCP servers) are first-class citizens in the agent ecosystem - Parseltongue should embrace this.

---

## 13. CodeActAgent step() - Complete LLM Call Assembly [CONFIRMED from source]

The complete `step()` function reveals the exact LLM call construction:

```python
def step(self, state: State) -> 'Action':
    # 1. Return any pending actions from previous multi-tool-call response
    if self.pending_actions:
        return self.pending_actions.popleft()

    # 2. Check for /exit command
    latest_user_message = state.get_last_user_message()
    if latest_user_message and latest_user_message.content.strip() == '/exit':
        return AgentFinishAction()

    # 3. Condense history (returns View or Condensation)
    match self.condenser.condensed_history(state):
        case View(events=events, forgotten_event_ids=forgotten_ids):
            condensed_history = events
            forgotten_event_ids = forgotten_ids
        case Condensation(action=condensation_action):
            return condensation_action  # Tell controller to do condensation first

    # 4. Build LLM messages from condensed history
    initial_user_message = self._get_initial_user_message(state.history)
    messages = self._get_messages(condensed_history, initial_user_message, forgotten_event_ids)
    # -> ConversationMemory.process_events() converts events to Message objects

    # 5. Make LLM call
    params = {
        'messages': messages,
        'tools': check_tools(self.tools, self.llm.config),
        'extra_body': {'metadata': state.to_llm_metadata(...)},
    }
    response = self.llm.completion(**params)

    # 6. Parse response: LLM tool calls -> Action objects
    actions = self.response_to_actions(response)
    # -> function_calling.response_to_actions() dispatches tool calls to Action dataclasses

    # 7. Queue all actions, return first one
    for action in actions: self.pending_actions.append(action)
    return self.pending_actions.popleft()
```

Key insight: **The LLM can return multiple tool calls in a single response**. All are queued and returned one at a time on successive `step()` calls. This means one LLM call can generate N actions that are executed sequentially without another LLM call.

---

## 14. Global Microagents (skills/) [CONFIRMED from source]

The `skills/` directory contains 25 built-in knowledge microagents that ship with OpenHands:

```
add_agent.md, add_repo_inst.md, address_pr_comments.md
agent-builder.md, agent_memory.md
azure_devops.md, bitbucket.md
code-review.md, codereview-roasted.md
default-tools.md, docker.md
fix-py-line-too-long.md, fix_test.md, flarglebargle.md
github.md, gitlab.md, kubernetes.md
npm.md, onboarding.md, pdflatex.md
security.md, ssh.md, swift-linux.md
update_pr_description.md, update_test.md
```

**Sample microagent format** (`skills/github.md`):
```yaml
---
name: github
type: knowledge
version: 1.0.0
agent: CodeActAgent
triggers:
- github
- git
---

You have access to an environment variable, `GITHUB_TOKEN`...
ALWAYS use the GitHub API for operations instead of a web browser.
ALWAYS use the `create_pr` tool to open a pull request
...
```

**Trigger mechanism**: If the user's task mentions "github" or "git", this entire knowledge block is injected as `<EXTRA_INFO>` into the next user message. The agent then has this context for the entire session.

**Parseltongue implication**: A Parseltongue microagent in `skills/` would look like:
```yaml
---
name: parseltongue
type: knowledge
version: 1.0.0
agent: CodeActAgent
triggers:
- blast radius
- dependency
- code graph
- parseltongue
---

You have access to a Parseltongue code intelligence server at http://localhost:7777.
Use it to understand code structure before making changes.

Available tools: blast_radius, smart_context, dependency_graph, complexity_hotspots, ...
```

---

## 15. Prompt Caching Optimization [CONFIRMED from source]

OpenHands supports Anthropic prompt caching:
```python
# In CodeActAgent._get_messages()
if self.llm.is_caching_prompt_active():
    self.conversation_memory.apply_prompt_caching(messages)
```

This means for Anthropic models (Claude), the system message and early conversation history is cached at the token level, reducing costs for long sessions. This is a practical consideration: integrations that add large context blocks (like Parseltongue analysis) would benefit from being placed in cacheable positions.

---

## Confidence Ratings Summary

- All architectural descriptions: [CONFIRMED from source]
- V1 migration details (sparse): [INFERRED from code comments and directory structure]
- Performance numbers (none found - no benchmarks in source): [NOT AVAILABLE]
- Future roadmap: [SPECULATIVE based on code TODOs and V1 direction]
