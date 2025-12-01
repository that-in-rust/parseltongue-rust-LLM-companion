# Claude Code Hook System - Deep Analysis

## Executive Summary

The Claude Code CLI contains a sophisticated **Hook System** that allows users to intercept and modify the application's behavior at critical execution points. Hooks can execute shell commands, run prompts through the AI model, delegate to subagents, or execute JavaScript callbacks.

## Architecture Overview

### Hook Execution Flow

```
User Action → Hook Event Trigger → Get Registered Hooks → Execute in Parallel → Aggregate Results → Continue/Block
```

### Key Components

1. **Hook Event Registry** - 11 different event types
2. **Hook Configuration Types** - 5 different hook types (command, prompt, agent, callback, function)
3. **Matcher System** - Pattern matching to filter which hooks run
4. **Parallel Execution Engine** - Runs multiple hooks concurrently
5. **Result Aggregation** - Combines outputs from parallel hooks
6. **Permission System Integration** - Hooks can modify permission decisions

---

## Hook Events (11 Types)

### 1. PreToolUse
**When**: Before any tool is executed
**Input**: `{ tool_name, tool_input, tool_use_id }`
**Matcher**: Tool name (e.g., "Bash", "Write", "Edit")
**Exit Codes**:
- `0` - Allow (stdout/stderr hidden)
- `2` - **BLOCK** tool call, show stderr to model
- Other - Show stderr to user, allow tool

**Use Cases**:
- Validate file paths before editing
- Run linters before writing code
- Check permissions before bash commands
- Inject custom validation logic

### 2. PostToolUse
**When**: After tool execution completes
**Input**: `{ tool_name, tool_input, tool_response, tool_use_id }`
**Matcher**: Tool name
**Exit Codes**:
- `0` - Success (stdout in transcript mode)
- `2` - Show stderr to model immediately
- Other - Show stderr to user

**Use Cases**:
- Post-process tool outputs
- Log tool executions
- Verify file changes
- Update external systems

### 3. UserPromptSubmit
**When**: User submits a prompt
**Input**: `{ prompt }`
**Matcher**: None
**Exit Codes**:
- `0` - Append stdout to prompt
- `2` - **BLOCK** processing, erase prompt, show stderr
- Other - Show stderr, continue

**Use Cases**:
- Inject context before queries
- Validate user input
- Add session-specific instructions
- Block certain types of requests

### 4. Notification
**When**: System notification displayed
**Input**: `{ message, title, notification_type }`
**Matcher**: Notification type ("permission_prompt", "idle_prompt", "auth_success", "elicitation_dialog")
**Exit Codes**:
- `0` - Success
- Other - Show stderr

**Use Cases**:
- External logging
- Alert systems
- Analytics tracking

### 5. SessionStart
**When**: New session begins
**Input**: `{ source }` (source: "startup", "resume", "clear", "compact")
**Matcher**: Session source
**Exit Codes**:
- `0` - Inject stdout as context
- Other - Show stderr (blocking errors ignored)

**Use Cases**:
- Load project context
- Initialize environment
- Set session variables
- Display welcome messages

### 6. SessionEnd
**When**: Session terminating
**Input**: `{ reason }` (reason: "clear", "logout", "prompt_input_exit", "other")
**Matcher**: End reason
**Exit Codes**:
- `0` - Success
- Other - Show stderr

**Use Cases**:
- Cleanup operations
- Save session state
- Update external systems
- Generate reports

### 7. Stop
**When**: Claude concludes response
**Input**: `{ stop_hook_active, agent_id, agent_transcript_path }`
**Matcher**: Agent type
**Exit Codes**:
- `0` - Continue (stdout/stderr hidden)
- `2` - Show stderr to model, **continue conversation**
- Other - Show stderr to user

**Use Cases**:
- Validate output quality
- Check for code completeness
- Enforce response formatting
- Add follow-up questions

### 8. SubagentStart
**When**: Subagent (Task tool) starts
**Input**: `{ agent_id, agent_type }`
**Matcher**: Agent type
**Exit Codes**:
- `0` - Inject stdout to subagent
- Other - Show stderr (blocking ignored)

**Use Cases**:
- Configure subagent context
- Add agent-specific instructions
- Initialize agent state

### 9. SubagentStop
**When**: Subagent concludes
**Input**: `{ agent_id, agent_transcript_path }`
**Matcher**: Agent type
**Exit Codes**:
- `0` - Continue
- `2` - Show stderr to subagent, **continue**
- Other - Show stderr to user

**Use Cases**:
- Validate subagent outputs
- Post-process results
- Quality checks

### 10. PreCompact
**When**: Before conversation compaction
**Input**: `{ trigger, custom_instructions }` (trigger: "manual", "auto")
**Matcher**: Trigger type
**Exit Codes**:
- `0` - Append stdout as custom compact instructions
- `2` - **BLOCK** compaction
- Other - Show stderr, continue compaction

**Use Cases**:
- Add compaction guidelines
- Preserve important context
- Block compaction when needed

### 11. PermissionRequest
**When**: Permission dialog shown
**Input**: `{ tool_name, tool_input, permission_suggestions }`
**Matcher**: Tool name
**Exit Codes**:
- `0` - Use hook decision (JSON output)
- Other - Show stderr

**Special**: Can return `hookSpecificOutput.decision` to auto-approve/deny

**Use Cases**:
- Auto-approve trusted operations
- Custom permission logic
- Modify tool inputs before execution
- Change permission rules dynamically

---

## Hook Configuration Types

### 1. Command Hook (Shell Execution)

```json
{
  "type": "command",
  "command": "jq -r '.tool_input.file_path' | xargs -r gofmt -w",
  "timeout": 60,
  "statusMessage": "Running Go formatter..."
}
```

**Features**:
- Receives hook input via stdin as JSON
- Can use any shell command
- Environment variables: `HOOK_EVENT`, `HOOK_NAME`, `HOOK_INPUT_FILE`
- Default timeout: 60 seconds

### 2. Prompt Hook (AI Evaluation)

```json
{
  "type": "prompt",
  "prompt": "Review this tool call: $ARGUMENTS. Return {\"ok\": true/false, \"reason\": \"...\"}",
  "timeout": 60,
  "model": "claude-sonnet-4"
}
```

**Features**:
- `$ARGUMENTS` replaced with hook input JSON
- Runs through Claude model
- Must return JSON: `{ ok: boolean, reason?: string }`
- Uses fast model by default (can override with `model` field)

### 3. Agent Hook (Subagent Delegation)

```json
{
  "type": "agent",
  "prompt": "(args) => `Analyze: ${JSON.stringify(args)}`",
  "timeout": 120
}
```

**Features**:
- Delegates to a specialized subagent
- Function receives parsed hook input
- Runs in separate conversation context

### 4. Callback Hook (JavaScript Function - Session Only)

```javascript
{
  type: "callback",
  callback: async (hookInput, toolUseID, signal) => {
    // hookInput: full context object
    // signal: AbortSignal for cancellation
    return {
      continue: true,
      suppressOutput: false,
      // ... other hook output fields
    };
  },
  timeout: 30,
  errorMessage: "Validation failed"
}
```

**Features**:
- Programmatic hook registration
- Not persisted (session-only)
- Full JavaScript execution
- Can access application state

### 5. Function Hook (Stop Hooks Only - Session Only)

```javascript
{
  type: "function",
  callback: async (messages, signal) => {
    // messages: conversation history
    // Return true to continue, false to block
    return true;
  },
  timeout: 5,
  errorMessage: "Condition not met"
}
```

**Features**:
- Only for Stop/SubagentStop events
- Receives full conversation messages
- Boolean return (true = continue, false = block)

---

## Hook Output Structure

Hooks can return JSON to control execution:

```json
{
  // Flow Control
  "continue": true,              // Whether to continue operation
  "suppressOutput": false,       // Hide stdout from transcript
  "stopReason": "...",          // Reason shown when continue=false

  // Permission Control (PermissionRequest hooks)
  "decision": "approve",        // "approve" or "block"
  "reason": "...",             // Explanation
  "systemMessage": "...",      // Warning shown to user

  // Event-Specific Outputs
  "hookSpecificOutput": {
    "hookEventName": "PreToolUse",

    // PreToolUse
    "permissionDecision": "allow", // "allow", "deny", "ask"
    "permissionDecisionReason": "...",
    "updatedInput": {},           // Modified tool input

    // UserPromptSubmit/SessionStart/SubagentStart/PostToolUse
    "additionalContext": "...",   // Injected context

    // PostToolUse (MCP tools only)
    "updatedMCPToolOutput": {},   // Modified output

    // PermissionRequest
    "decision": {
      "behavior": "allow",        // "allow" or "deny"
      "updatedInput": {},        // For allow
      "updatedPermissions": [],  // Rule changes
      "message": "...",          // For deny
      "interrupt": false         // Whether to interrupt
    }
  },

  // Async Execution
  "async": true,                 // Run in background
  "asyncTimeout": 15000          // Timeout in ms
}
```

---

## Matcher System

Matchers filter which hooks run for which tools/events.

### Matcher Syntax

```javascript
// Exact match
"Write"

// Multiple tools (OR)
"Write|Edit|Read"

// Wildcard
"Bash*"      // Matches "Bash", "BashOutput", etc.
"*Tool"      // Matches any tool ending in "Tool"

// Regex
"^(Read|Write)$"

// Empty matcher = matches all
""
```

### Matcher Fields by Event

- **PreToolUse/PostToolUse**: `tool_name`
- **Notification**: `notification_type`
- **SessionStart**: `source`
- **SessionEnd**: `reason`
- **PreCompact**: `trigger`
- **PermissionRequest**: `tool_name`
- **SubagentStart**: `agent_type`
- **Stop**: `agent_type`

---

## Exit Code Behavior

| Exit Code | Behavior | Common Events |
|-----------|----------|---------------|
| `0` | Success, continue operation | All events |
| `2` | **BLOCKING** error (stops operation) | PreToolUse, UserPromptSubmit, PreCompact |
| `2` | Show to model, **continue** | Stop, SubagentStop, PostToolUse |
| Other | Non-blocking error, show stderr to user | All events |

---

## Parallel Execution & Aggregation

### Execution Model

1. **All hooks execute in parallel** for the same event
2. Results are aggregated as they complete
3. **Most restrictive permission wins**:
   - `deny` > `ask` > `allow` > `passthrough`

### Aggregation Rules

```javascript
// Permission decisions aggregate
Hook 1: allow
Hook 2: deny
Hook 3: ask
→ Result: DENY (most restrictive)

// Messages concatenate
Hook 1: "Check 1 passed"
Hook 2: "Check 2 passed"
→ Both messages shown

// Blocking errors stop immediately
Hook 1: exit code 2 (blocking)
Hook 2: still running
→ Operation blocked, other hooks may still complete
```

---

## Hook Storage Locations

### 1. Policy Settings (Managed)
**Path**: Managed settings directory
**Priority**: Highest (cannot be overridden)
**Use**: Organization-wide policies

### 2. User Settings
**Path**: `~/.claude/settings.json`
**Use**: Personal global hooks

### 3. Project Settings
**Path**: `.claude/settings.json`
**Use**: Project-specific hooks (checked into git)

### 4. Local Settings
**Path**: `.claude/settings.local.json`
**Use**: Local project overrides (not checked in)

### 5. Plugin Hooks
**Path**: Plugin manifests
**Use**: Plugin-provided hooks

### 6. Session Hooks (In-Memory)
**Storage**: Application state (not persisted)
**Use**: Temporary programmatic hooks
**API**:
```javascript
registerSessionHook(
  setAppState,
  sessionId,
  hookEvent,
  matcher,
  hookConfig,
  onHookSuccess
);

clearSessionHooks(setAppState, sessionId);
```

---

## Settings Format

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "echo 'Bash command executed' >> /tmp/bash-log.txt",
            "timeout": 30
          }
        ]
      },
      {
        "matcher": "Write|Edit",
        "hooks": [
          {
            "type": "command",
            "command": "jq -r '.tool_input.file_path | select(endswith(\".go\"))' | xargs -r gofmt -w"
          }
        ]
      }
    ],
    "SessionStart": [
      {
        "matcher": "startup",
        "hooks": [
          {
            "type": "command",
            "command": "cat project-context.md"
          }
        ]
      }
    ]
  }
}
```

---

## Async Hooks

Hooks can run in the background using async mode:

### Command Output
```json
{
  "async": true,
  "asyncTimeout": 15000
}
```

### Features
- Hook returns immediately
- Continues running in background
- Results appear as they complete
- Timeout applies (default 15s)

### Registry
```javascript
// Async hooks are tracked in a Map
Map<processId, {
  processId: string,
  hookName: string,
  hookEvent: string,
  toolName?: string,
  command: string,
  startTime: number,
  timeout: number,
  stdout: string,
  stderr: string,
  responseAttachmentSent: boolean,
  shellCommand: ShellCommand
}>

// Check for completions
async function checkForNewResponses() {
  // Returns completed async hook results
}
```

---

## Security & Sandboxing

### Permissions
- Hooks execute with **full user permissions**
- Can access filesystem, network, etc.
- Security warning shown in UI

### Policy Controls
```json
{
  "disableAllHooks": true,           // Disable all hooks
  "allowManagedHooksOnly": true      // Only allow managed hooks
}
```

### Workspace Trust
- Hooks disabled until workspace trust accepted
- Prevents malicious hooks in untrusted projects

---

## Advanced Features

### 1. StatusLine Hook
Special hook that runs continuously for status display:

```json
{
  "statusLine": {
    "type": "command",
    "command": "git status --short | head -5"
  }
}
```

Returns array of strings displayed in UI.

### 2. Hook Input Context

Every hook receives:
```json
{
  "hook_event_name": "PreToolUse",
  "working_directory": "/path/to/project",
  "home_directory": "/Users/username",
  // ... event-specific fields
}
```

### 3. MCP Tool Output Modification

PostToolUse hooks can modify MCP tool outputs:
```json
{
  "hookSpecificOutput": {
    "hookEventName": "PostToolUse",
    "updatedMCPToolOutput": {
      // Modified output structure
    }
  }
}
```

---

## Code References (cli.js)

| Feature | Line Range | Function Names |
|---------|-----------|----------------|
| Main execution | ~2935 | `executeHooks`, `qQA` |
| Hook discovery | ~320-380 | `getHooksForEvent`, `lW0` |
| Shell execution | ~369-412 | `executeShellHook`, `pW0` |
| Prompt hooks | ~956-1620 | `executePromptHook`, `yI9` |
| Output parsing | ~410-412 | `parseHookOutput`, `uI9` |
| Output processing | ~321 | `processHookOutput`, `mI9` |
| Async hooks | ~2800-2940 | `checkForNewResponses`, `TZ2` |
| Session hooks | ~341-397 | `registerSessionHook`, `nB1` |
| Settings loading | ~1706-1707 | `getHooksFromSettings` |
| Event metadata | ~3499-3533 | `NjA` |

---

## Use Case Examples

### 1. Auto-format Code Before Writing
```json
{
  "PreToolUse": [{
    "matcher": "Write|Edit",
    "hooks": [{
      "type": "command",
      "command": "jq -r '.tool_input.file_path | select(endswith(\".py\"))' | xargs -r black"
    }]
  }]
}
```

### 2. Log All Bash Commands
```json
{
  "PreToolUse": [{
    "matcher": "Bash",
    "hooks": [{
      "type": "command",
      "command": "jq -r '.tool_input.command' >> ~/.claude/bash-log.txt"
    }]
  }]
}
```

### 3. Load Project Context on Startup
```json
{
  "SessionStart": [{
    "matcher": "startup",
    "hooks": [{
      "type": "command",
      "command": "cat PROJECT_CONTEXT.md"
    }]
  }]
}
```

### 4. AI-Powered Tool Validation
```json
{
  "PreToolUse": [{
    "matcher": "Write",
    "hooks": [{
      "type": "prompt",
      "prompt": "Is this file write safe? $ARGUMENTS\nReturn {\"ok\": true/false, \"reason\": \"...\"}",
      "model": "claude-sonnet-4"
    }]
  }]
}
```

### 5. Session Hook (Programmatic)
```javascript
registerSessionHook(
  setAppState,
  sessionId,
  "PreToolUse",
  "Bash",
  {
    type: "callback",
    callback: async (input, toolUseID, signal) => {
      // Custom validation logic
      if (input.tool_input.command.includes("rm -rf")) {
        return {
          continue: false,
          stopReason: "Dangerous command blocked"
        };
      }
      return { continue: true };
    }
  },
  (hook, result) => {
    console.log("Hook completed:", result);
  }
);
```

---

## Summary

The Claude Code hook system is a **powerful, extensible architecture** that enables:

1. **Complete execution control** at 11 critical lifecycle points
2. **Multiple hook types** (shell, AI, subagent, JavaScript)
3. **Sophisticated pattern matching** for selective execution
4. **Parallel execution** with intelligent result aggregation
5. **Permission system integration** for security
6. **Both persistent and ephemeral** hook registration
7. **Async execution** for background operations
8. **Full programmatic access** via session hooks

This system transforms Claude Code from a simple AI assistant into a **programmable development environment** where every action can be intercepted, validated, modified, or augmented.
