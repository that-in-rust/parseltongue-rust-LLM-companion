# CR-delegate-agent-coordination.md
# Delegate Agent Coordination System — Deep Dive
# Date: 2026-02-21
# Source: CR08/delegate/delegate/

---

## Overview

Delegate implements a sophisticated multi-agent coordination system with role-based agents, persistent conversations, and message-driven architecture. The system enables parallel AI agents (managers and engineers) to collaborate through asynchronous message passing while maintaining conversation context across multiple turns.

---

## Agent Architecture

### Core Components

```
┌─────────────────────────────────────────────────────────────────────┐
│                      AGENT COORDINATION SYSTEM                         │
│                                                                      │
│  User                                                                │
│    │                                                                 │
│    ▼                                                                 │
│  ┌─────────┐                                                        │
│  │ Web UI   │  ← SSE: Real-time agent activity updates              │
│  │ (React)  │                                                        │
│  └────┬────┘                                                        │
│       │                                                             │
│       ▼                                                             │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                     DAEMON                                     │  │
│  │                                                               │  │
│  │  ┌─────────────────────────────────────────────────────────┐   │  │
│  │  │ Message Router                                         │   │  │
│  │  │   ┌──────────────┐                                       │   │  │
│  │  │   │ Telephone    │                                       │   │  │
│  │  │   │ Exchange     │                                       │   │  │
│  │  │   │  (manager,   │                                       │   │  │
│  │  │   │   alice,     │                                       │   │  │
│  │  │   │   bob...)     │                                       │   │  │
│  │  │   └──────────────┘                                       │   │  │
│  │  └─────────────────────────────────────────────────────────┘   │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### Telephone (Persistent Conversation)

Each (team, agent) pair gets a persistent `Telephone` subprocess:

```python
class Telephone:
    """Bounded-context persistent conversation with Claude Code SDK"""

    # Single persistent subprocess across all turns
    # Auto-rotates when context window fills
    # Enforces permissions via can_use_tool callback
```

**Key Features**:
- **Persistent Process**: Single Claude Code subprocess maintained across turns
- **Context Rotation**: Auto-summarizes when `max_context_tokens` exceeded
- **Permission Guards**: Path isolation and bash command deny-lists
- **Memory**: Persistent context across generations

---

## Agent Roles & Permissions

### Manager Agents
- **Role**: `manager`
- **Model**: Default Opus (higher reasoning)
- **Write Access**: Full team directory access (`team_dir/*`)
- **Capabilities**:
  - Edit any file in team directory
  - Create/assign/cancel tasks
  - Access all repositories
- **Use Case**: High-level planning, code review, task management

### Engineer Agents
- **Role**: `engineer` (workers are legacy alias)
- **Model**: Default Sonnet (fast execution)
- **Write Access**: Restricted paths:
  - Agent's own directory: `teams/{team}/agents/{engineer}/`
  - Team shared directory: `teams/{team}/shared/`
  - Task worktree paths: Temporary repo branches
- **Capabilities**:
  - Edit files in allowed paths
  - Work on assigned tasks
  - Limited to registered repositories
- **Use Case**: Implementation, bug fixes, feature development

---

## Message Passing System

### Mailbox Architecture

The mailbox system uses SQLite for persistent message storage:

```python
@dataclass
class Message:
    sender: str          # Who sent it
    recipient: str       # Who should receive it
    time: str           # When sent
    body: str           # Message content
    task_id: int | None  # Associated task (optional)
    delivered_at: str | None  # When made available
    seen_at: str | None      # When picked up (turn start)
    processed_at: str | None # When turn completed
```

### Message Lifecycle

```
┌──────────────────────────────────────────────────────────────────────────────┐
│  MESSAGE LIFECYCLE                                                               │
└──────────────────────────────────────────────────────────────────────────────┘

1. DELIVERED: mailbox_send() → delivered_at = NOW() → available in inbox
2. SEEN: read_inbox() → mark_seen_batch() → agent starts turn
3. PROCESSED: mark_processed_batch() → message lifecycle ends
4. CONTEXT: recent_conversation() provides bidirectional history
```

### Message Batching Strategy

- **Human Priority**: When human sends messages, only human messages are batched
- **Task Grouping**: Messages with same `task_id` processed together (max 5 per batch)
- **Sender Order**: Ensures no messages are skipped
- **Cross-Team Isolation**: Messages filtered by team UUID

---

## Turn Execution Flow

```
┌──────────────────────────────────────────────────────────────────────────────┐
│  TURN EXECUTION FLOW                                                               │
└──────────────────────────────────────────────────────────────────────────────┘

Daemon checks agents_with_unread()
    │
    ▼
Agent selected for turn execution
    │
    ├─▶ Select batch (≤5 messages with same task_id)
    │
    ├─▶ Build prompt (history + new messages)
    │
    ├─▶ Execute via Telephone
    │   │
    │   ├─▶ Send to Claude Code SDK
    │   ├─▶ Stream activity events via SSE
    │   └─▶ Receive response
    │
    ├─▶ Broadcast activity to SSE
    │
    ├─▶ Mark as processed
    │
    └─▶ Optional: Reflection turn (1-in-20 chance)
```

### Runtime Architecture

**Core Function**: `run_turn(hc_home, team, agent, exchange)`

**Process**:
1. **Setup**: Read agent state, determine role/model
2. **Message Selection**: Batch ≤5 messages with same `task_id`
3. **Workspace Resolution**: Set up worktree paths
4. **Context Building**: Prompt with history + new messages
5. **Execution**: Send to Telephone, stream activity
6. **Cleanup**: Mark processed, update session
7. **Reflection**: Optional follow-up turn (5% chance)

---

## State Management

### Agent State

Each agent maintains `state.yaml`:

```yaml
role: engineer|manager         # Primary role
model: sonnet|opus             # Model choice
seniority: senior|junior      # Legacy field
token_budget: 50000           # Optional limit
```

### Session Management

- **Database Sessions**: 1:1 with `run_turn()` calls
- **Telephone Lifetime**: Independent of DB sessions
- **Activity Tracking**: Real-time tool usage via SSE

### Worktree Management

- **Task-Based**: Each task gets isolated git worktree
- **Path Isolation**: Engineers only see allowed paths
- **Cleanup**: Automatic on task completion

---

## Communication Patterns

### 1. Human → Agent Direction

```python
mailbox_send(hc_home, team, "human", "engineer", "Fix this bug", task_id=123)
```
- Creates human-directed turn
- Only human messages batched together
- Frontend shows inline thinking

### 2. Agent → Agent Coordination

```python
mailbox_send(hc_home, team, "engineer1", "manager", "Need review for T0045")
```
- Cross-agent communication
- Task context preserved
- Async processing

### 3. Agent → Human Escalation

```python
mailbox_send(hc_home, team, "manager", "human", "T0039 blocked by API timeout")
```
- Priority routing to human
- Clear context for action

---

## Real-time Activity System

### Activity Ring Buffer

- **Fixed Size**: 1024 entries per (team, agent)
- **SSE Broadcast**: Real-time updates to frontend
- **Tool Tracking**: Records all tool invocations with diffs

### Activity Events

```python
broadcast_activity(
    agent="engineer1",
    team="team1",
    tool="Edit",
    detail="src/main.py",
    task_id=123,
    diff=["+def solve():", "-def main():"]
)
```

### SSE Event Types

- `agent_activity`: Tool usage with optional diff
- `agent_thinking`: Streaming thought process
- `turn_started|ended`: Session lifecycle
- `rate_limit`: API throttling notifications
- `msg_status`: Message read/processed updates

---

## Coordination Guarantees

### Message Ordering

- **FIFO**: Messages processed in arrival order
- **Task Atomicity**: All messages for task processed together
- **No Skips**: Per-sender eligibility ensures fairness

### Resource Isolation

- **Path Isolation**: Engineers restricted to write paths
- **Git Safety**: Forbidden git operations enforced at runtime
- **Sandboxing**: Optional OS-level bash sandboxing

### Fault Tolerance

- **Context Rotation**: Auto-recovers from context window limits
- **Process Restart**: Telephone recreated on config changes
- **Retry Logic**: Failed turns don't block other agents

---

## Performance Characteristics

### Concurrency

- **Parallel Execution**: Multiple agents run concurrently
- **Telephone Sharing**: Single subprocess per (team, agent)
- **Memory Efficiency**: Context rotation prevents bloat

### Scaling

- **Stateless Workers**: Runtime state minimal
- **Database-Centric**: All state persisted in SQLite
- **SSE Broadcast**: 1000+ concurrent clients supported

---

## Security Model

### Permission Enforcement

```python
def can_use_tool(tool_name, tool_input, context):
    # Write-path isolation for engineers
    if tool_name in WRITE_TOOLS and write_paths_restricted:
        file_path = tool_input.get("file_path")
        if not path_in_allowed(file_path, allowed_paths):
            return deny("Write denied: outside allowed paths")

    # Bash command deny-list
    if tool_name == "Bash" and denied_patterns:
        if any(pattern in tool_input["command"] for pattern in deny_list):
            return deny("Command denied: restricted pattern")

    return allow()
```

### Isolation Guarantees

- **Team Separation**: UUID-based message filtering
- **Agent Isolation**: File system path restrictions
- **Process Isolation**: Separate Telephone subprocesses

---

*Generated: 2026-02-21*
*Source: delegate/agent.py, activity.py, mailbox.py, telephone.py, runtime.py*
