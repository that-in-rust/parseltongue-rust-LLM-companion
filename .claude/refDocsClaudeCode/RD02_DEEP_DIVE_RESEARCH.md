# Claude Code: Deep-Dive Research Document
## Atomic-Level Deconstruction of cli.js (10.9MB)

**Version:** 2.0.55
**Bundle Size:** 10,862,686 bytes (10.9MB)
**Total Lines:** 4,609 lines (minified)
**Analysis Date:** 2025-11-30

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Bundle Structure & Module Map](#2-bundle-structure--module-map)
3. [Tool System - Deminified](#3-tool-system---deminified)
4. [Agent System - Deminified](#4-agent-system---deminified)
5. [Hook System - Deminified](#5-hook-system---deminified)
6. [API Client - Deminified](#6-api-client---deminified)
7. [Terminal UI Architecture](#7-terminal-ui-architecture)
8. [HTTP Client (Undici)](#8-http-client-undici)
9. [Key Variable Mapping](#9-key-variable-mapping)
10. [Architectural Insights](#10-architectural-insights)

---

## 1. Executive Summary

Claude Code's cli.js is a **single-file JavaScript bundle** containing:

| Component | Approximate Size | Key Functions |
|-----------|------------------|---------------|
| Core Runtime | ~500KB | Module loading, polyfills |
| Tool System | ~800KB | 15+ tool implementations |
| Agent System | ~400KB | Subagent spawning, lifecycle |
| Hook System | ~300KB | 11 event types, execution engine |
| API Client | ~600KB | Anthropic SDK, streaming |
| Terminal UI | ~2MB | Ink/React, 4,501 createElement calls |
| HTTP Client | ~1MB | Undici, fetch implementation |
| Utilities | ~1.5MB | Lodash, Git, file operations |
| Dependencies | ~4MB | Bundled node_modules |

### Minification Pattern

The bundler uses aggressive minification:
- Single-letter variables: `A`, `Q`, `B`, `G`, `Z`, `I`, `Y`, `J`, `W`
- Numeric suffixes: `9` (end), `0` (alternate), `1` (tertiary)
- Module IDs: `U9`, `J1`, `K0`, `D0`, `FA`, `SA`, `BA`

---

## 2. Bundle Structure & Module Map

### Shebang & Entry Point (Lines 1-10)

```javascript
#!/usr/bin/env node
// @anthropic-ai/claude-code v2.0.55
// Want to see the unminified source? We're hiring!
```

### Custom Module System (Line 8)

```javascript
// DEMINIFIED: Lazy module loader
var createLazyModule = (factory, cachedValue) => () => {
  if (factory) {
    cachedValue = factory(factory = null);
  }
  return cachedValue;
};
```

### Module Sections Map

| Lines | Section | Key Exports |
|-------|---------|-------------|
| 1-10 | Header | Shebang, version |
| 8-500 | Core Runtime | `M()` loader, polyfills |
| 500-800 | File System | `MA` facade, `SK()` symlinks |
| 800-1200 | Session State | `nN9()` global state |
| 1000-1800 | Lodash Polyfills | Maps, Sets, equality |
| 1900-2100 | Git Integration | Status, commits, branches |
| 2100-2300 | System Utilities | Prime check, templates |
| 2500-3500 | CLI Commands | Commander.js, subcommands |
| 3500-4000 | Main Entry | Configuration, initialization |
| 3600-4200 | React/Ink UI | Components, rendering |
| 4000-4609 | Program Execution | Entry point, exports |

---

## 3. Tool System - Deminified

### 3.1 Tool Definition Interface

```javascript
// DEMINIFIED: Standard tool structure
interface ToolDefinition {
  name: string;                    // Tool identifier
  inputSchema: ZodSchema;          // Zod validation schema

  // Check if safe to run in parallel
  isConcurrencySafe: (input: any) => boolean;

  // Optional pre-execution validation
  validateInput?: (input: any, context: ToolContext) => Promise<ValidationResult>;

  // Main execution function
  call: (
    input: any,
    context: ToolContext,
    canUseTool: PermissionChecker,
    parentMessage: Message,
    onProgress?: ProgressCallback
  ) => Promise<ToolResult>;

  // Format result for API
  mapToolResultToToolResultBlockParam: (
    result: ToolResult,
    toolUseId: string
  ) => ToolResultBlock;
}
```

### 3.2 Read Tool - Complete Implementation

```javascript
// DEMINIFIED: Read Tool
const ReadTool = {
  name: "Read",

  inputSchema: zod.object({
    file_path: zod.string().describe("Absolute path to file"),
    offset: zod.number().optional().describe("Start line (0-indexed)"),
    limit: zod.number().optional().describe("Number of lines to read")
  }),

  isConcurrencySafe: () => true,  // Reading is always safe

  async call(input, context) {
    const { file_path, offset = 0, limit } = input;

    // Validate absolute path
    if (!path.isAbsolute(file_path)) {
      return { data: "Error: Path must be absolute", isError: true };
    }

    const fileSystem = getFileSystem();

    // Check file exists
    if (!fileSystem.existsSync(file_path)) {
      return { data: `Error: File not found: ${file_path}`, isError: true };
    }

    // Check file type
    const stats = fileSystem.statSync(file_path);
    if (stats.isDirectory()) {
      return { data: "Error: Path is a directory", isError: true };
    }

    // Read content
    const content = fileSystem.readFileSync(file_path, { encoding: "utf-8" });
    const lines = content.split('\n');

    // Apply pagination
    const startLine = offset;
    const endLine = limit ? Math.min(offset + limit, lines.length) : lines.length;
    const selectedLines = lines.slice(startLine, endLine);

    // Format with line numbers (1-indexed display)
    const formatted = selectedLines.map((line, idx) => {
      const lineNum = (startLine + idx + 1).toString().padStart(6, ' ');
      // Truncate long lines
      const displayLine = line.length > 2000 ? line.slice(0, 2000) + '...' : line;
      return `${lineNum}→${displayLine}`;
    }).join('\n');

    return {
      data: formatted,
      metadata: {
        file_path,
        total_lines: lines.length,
        lines_returned: selectedLines.length,
        offset: startLine
      }
    };
  },

  mapToolResultToToolResultBlockParam(result, toolUseId) {
    return {
      type: "tool_result",
      tool_use_id: toolUseId,
      content: result.data,
      is_error: result.isError || false
    };
  }
};
```

### 3.3 Bash Tool - Complete Implementation

```javascript
// DEMINIFIED: Bash Tool
const BashTool = {
  name: "Bash",

  inputSchema: zod.object({
    command: zod.string().describe("Shell command to execute"),
    description: zod.string().optional(),
    timeout: zod.number().optional().default(120000),
    run_in_background: zod.boolean().optional().default(false),
    dangerouslyDisableSandbox: zod.boolean().optional()
  }),

  isConcurrencySafe: () => true,

  async call(input, context, canUseTool, parentMessage, onProgress) {
    const { command, timeout = 120000, run_in_background } = input;
    const MAX_TIMEOUT = 600000;  // 10 minutes

    const effectiveTimeout = Math.min(timeout, MAX_TIMEOUT);
    const shellId = run_in_background ? generateUUID() : null;

    // Setup environment
    const env = {
      ...process.env,
      CLAUDE_CODE_SESSION_ID: getSessionId(),
      CLAUDE_PROJECT_DIR: context.projectDir,
      PWD: context.cwd
    };

    return new Promise((resolve) => {
      const child = spawn('/bin/bash', ['-c', command], {
        cwd: context.cwd,
        env,
        timeout: effectiveTimeout
      });

      let stdout = '';
      let stderr = '';
      const startTime = Date.now();

      child.stdout.on('data', (chunk) => {
        const data = chunk.toString();
        stdout += data;

        // Stream progress
        if (onProgress) {
          onProgress({
            toolUseId: context.toolUseId,
            stdout: data
          });
        }
      });

      child.stderr.on('data', (chunk) => {
        stderr += chunk.toString();
      });

      child.on('exit', (exitCode) => {
        const duration = Date.now() - startTime;

        // Truncate large output
        const MAX_OUTPUT = 30000;
        if (stdout.length > MAX_OUTPUT) {
          stdout = stdout.slice(0, MAX_OUTPUT) +
            `\n\n[Truncated: ${stdout.length} total bytes]`;
        }

        resolve({
          data: {
            stdout,
            stderr,
            exit_code: exitCode ?? -1,
            duration_ms: duration,
            ...(shellId && { shell_id: shellId })
          },
          isError: exitCode !== 0
        });
      });

      child.on('error', (error) => {
        resolve({
          data: { stdout: '', stderr: error.message, exit_code: -1 },
          isError: true
        });
      });
    });
  }
};
```

### 3.4 Edit Tool - Complete Implementation

```javascript
// DEMINIFIED: Edit Tool
const EditTool = {
  name: "Edit",

  inputSchema: zod.object({
    file_path: zod.string(),
    old_string: zod.string(),
    new_string: zod.string(),
    replace_all: zod.boolean().default(false)
  }),

  isConcurrencySafe: () => false,  // File writes must be sequential

  validateInput: async (input, context) => {
    // Must read file before editing
    if (!context.hasReadFile(input.file_path)) {
      return {
        result: false,
        message: "Must read file with Read tool before editing",
        errorCode: "FILE_NOT_READ"
      };
    }

    // Strings must differ
    if (input.old_string === input.new_string) {
      return {
        result: false,
        message: "old_string and new_string must be different"
      };
    }

    return { result: true };
  },

  async call(input, context) {
    const { file_path, old_string, new_string, replace_all } = input;
    const fs = getFileSystem();

    const original = fs.readFileSync(file_path, 'utf-8');
    let modified;
    let count = 0;

    if (replace_all) {
      // Replace all occurrences
      modified = original.split(old_string).join(new_string);
      count = (original.match(new RegExp(escapeRegex(old_string), 'g')) || []).length;
    } else {
      // Single replacement with uniqueness check
      const firstIdx = original.indexOf(old_string);
      if (firstIdx === -1) {
        return { data: "Error: old_string not found", isError: true };
      }

      const secondIdx = original.indexOf(old_string, firstIdx + 1);
      if (secondIdx !== -1) {
        return {
          data: "Error: old_string appears multiple times. Use replace_all or add context",
          isError: true
        };
      }

      modified = original.slice(0, firstIdx) + new_string +
                 original.slice(firstIdx + old_string.length);
      count = 1;
    }

    // Write atomically
    fs.writeFileSync(file_path, modified, { encoding: 'utf-8', flush: true });

    // Track line changes
    const linesAdded = (new_string.match(/\n/g) || []).length;
    const linesRemoved = (old_string.match(/\n/g) || []).length;
    context.trackLineChanges(linesAdded, linesRemoved);

    return {
      data: {
        success: true,
        replacements: count,
        diff: generateUnifiedDiff(original, modified, file_path)
      }
    };
  }
};
```

### 3.5 Tool Execution Pipeline

```javascript
// DEMINIFIED: Tool execution orchestrator
async function* executeToolUse(toolBlock, assistantMessage, canUseTool, context) {
  const { name: toolName, input: toolInput, id: toolUseId } = toolBlock;

  // 1. FIND TOOL
  const tool = context.tools.find(t => t.name === toolName);
  if (!tool) {
    yield { message: createErrorResult(toolUseId, `Unknown tool: ${toolName}`) };
    return;
  }

  // 2. VALIDATE SCHEMA
  const validation = tool.inputSchema.safeParse(toolInput);
  if (!validation.success) {
    yield { message: createErrorResult(toolUseId, formatZodError(validation.error)) };
    return;
  }

  // 3. CUSTOM VALIDATION
  if (tool.validateInput) {
    const customCheck = await tool.validateInput(validation.data, context);
    if (!customCheck.result) {
      yield { message: createErrorResult(toolUseId, customCheck.message) };
      return;
    }
  }

  // 4. PRE-TOOL HOOKS
  const hookResults = await executeHooks('PreToolUse', {
    tool_name: toolName,
    tool_input: validation.data,
    tool_use_id: toolUseId
  }, context);

  if (hookResults.blocked) {
    yield { message: createErrorResult(toolUseId, hookResults.message) };
    return;
  }

  const finalInput = hookResults.modifiedInput || validation.data;

  // 5. CHECK PERMISSIONS
  const permission = await canUseTool(tool, finalInput, context);
  if (permission.behavior !== 'allow') {
    yield { message: createErrorResult(toolUseId, permission.message) };
    return;
  }

  // 6. EXECUTE
  const startTime = Date.now();
  try {
    const result = await tool.call(finalInput, context, canUseTool, assistantMessage);
    trackDuration(Date.now() - startTime);

    // 7. FORMAT RESULT
    const formatted = tool.mapToolResultToToolResultBlockParam(result, toolUseId);
    yield { message: formatted };

    // 8. POST-TOOL HOOKS
    await executeHooks('PostToolUse', {
      tool_name: toolName,
      tool_input: finalInput,
      tool_result: result
    }, context);

  } catch (error) {
    trackDuration(Date.now() - startTime);
    yield { message: createErrorResult(toolUseId, formatError(error)) };
  }
}
```

### 3.6 Concurrent Tool Execution Queue

```javascript
// DEMINIFIED: Manages parallel vs sequential tool execution
class ToolExecutionQueue {
  constructor(tools, canUseTool, context) {
    this.tools = tools;
    this.canUseTool = canUseTool;
    this.context = context;
    this.queue = [];
  }

  addTool(toolBlock, message) {
    const tool = this.tools.find(t => t.name === toolBlock.name);
    if (!tool) return;

    const validation = tool.inputSchema.safeParse(toolBlock.input);
    const isSafe = validation.success && tool.isConcurrencySafe(validation.data);

    this.queue.push({
      id: toolBlock.id,
      block: toolBlock,
      message,
      status: 'queued',
      isConcurrencySafe: isSafe,
      promise: null,
      results: []
    });

    this.process();
  }

  canExecute(isSafe) {
    const executing = this.queue.filter(t => t.status === 'executing');

    if (executing.length === 0) return true;
    if (isSafe && executing.every(t => t.isConcurrencySafe)) return true;

    return false;
  }

  async process() {
    for (const item of this.queue) {
      if (item.status !== 'queued') continue;

      if (this.canExecute(item.isConcurrencySafe)) {
        this.execute(item);
      } else if (!item.isConcurrencySafe) {
        break;  // Unsafe tools block queue
      }
    }
  }

  async execute(item) {
    item.status = 'executing';

    const generator = executeToolUse(
      item.block, item.message, this.canUseTool, this.context
    );

    for await (const result of generator) {
      item.results.push(result);
    }

    item.status = 'completed';
    this.process();  // Continue queue
  }
}
```

---

## 4. Agent System - Deminified

### 4.1 Agent Definition Structure

```javascript
// DEMINIFIED: Agent configuration
interface AgentDefinition {
  agentType: string;           // Unique identifier
  whenToUse: string;           // Triggering description
  tools: string[] | ['*'];     // Available tools
  source: 'built-in' | 'plugin' | 'userSettings' | 'projectSettings';
  baseDir: string;
  model: 'inherit' | 'sonnet' | 'opus' | 'haiku';
  color?: string;
  forkContext?: boolean;       // Receive full conversation history
  getSystemPrompt: () => string;
}
```

### 4.2 Built-in Agents

```javascript
// DEMINIFIED: Built-in agent definitions
const BUILT_IN_AGENTS = {
  'general-purpose': {
    agentType: 'general-purpose',
    whenToUse: `General-purpose agent for researching complex questions,
                searching for code, and executing multi-step tasks.`,
    tools: ['*'],
    source: 'built-in',
    model: 'sonnet',
    getSystemPrompt: () => `
You are an agent for Claude Code, Anthropic's official CLI for Claude.
Given the user's message, you should use the tools available to complete
the task. Do what has been asked; nothing more, nothing less.

Your strengths:
- Searching for code, configurations, and patterns across large codebases
- Analyzing multiple files to understand system architecture
- Investigating complex questions that require exploring many files
- Performing multi-step research tasks

Guidelines:
- For file searches: Use Grep or Glob for broad searches. Use Read for specific files.
- For analysis: Start broad and narrow down.
- Be thorough: Check multiple locations, consider naming conventions.
- NEVER create files unless absolutely necessary.
- NEVER proactively create documentation files.
- In final response, share relevant file names and code snippets.
- File paths MUST be absolute.
- Avoid emojis.`
  },

  'explore': {
    agentType: 'explore',
    whenToUse: 'Fast codebase exploration with pattern matching',
    tools: ['Read', 'Grep', 'Glob', 'LS'],
    source: 'built-in',
    model: 'haiku',
    getSystemPrompt: () => `
You are a fast exploration agent optimized for quickly finding files and patterns.
Use Grep and Glob efficiently. Return concise results with file paths.`
  },

  'plan': {
    agentType: 'plan',
    whenToUse: 'Architectural planning and design',
    tools: ['Read', 'Grep', 'Glob'],
    source: 'built-in',
    model: 'sonnet',
    getSystemPrompt: () => `
You are an architectural planning agent. Analyze codebases and create
implementation plans. Focus on patterns, dependencies, and structure.`
  }
};
```

### 4.3 Agent Spawning (Task Tool)

```javascript
// DEMINIFIED: Task tool implementation
const TaskTool = {
  name: 'Task',

  inputSchema: zod.object({
    description: zod.string().describe('3-5 word task description'),
    prompt: zod.string().describe('Detailed task instructions'),
    subagent_type: zod.string().describe('Agent type to use'),
    model: zod.enum(['sonnet', 'opus', 'haiku']).optional(),
    resume: zod.string().optional().describe('Agent ID to resume')
  }),

  async call(input, context) {
    const { description, prompt, subagent_type, model: modelOverride, resume } = input;

    // 1. FIND AGENT DEFINITION
    const agentDef = findAgent(subagent_type, context.agents);
    if (!agentDef) {
      return { data: `Unknown agent type: ${subagent_type}`, isError: true };
    }

    // 2. ASSIGN COLOR
    const color = assignAgentColor(subagent_type, context.colorMap);

    // 3. RESOLVE MODEL
    const model = resolveModel(modelOverride || agentDef.model, context.parentModel);

    // 4. FIRE SubagentStart HOOK
    await executeHooks('SubagentStart', {
      agent_id: generateAgentId(),
      agent_type: subagent_type
    }, context);

    // 5. BUILD CONTEXT
    const agentContext = {
      systemPrompt: agentDef.getSystemPrompt(),
      tools: resolveTools(agentDef.tools, context.availableTools),
      model,
      color,
      // Fork context if enabled
      messages: agentDef.forkContext ? context.messages : []
    };

    // 6. ADD SUB-AGENT CONTEXT INJECTION
    const injectedPrompt = `
You are in a sub-agent context.
Only complete the specific sub-agent task you have been assigned below.

Task: ${prompt}`;

    // 7. EXECUTE AGENT LOOP
    const result = await runAgentLoop(injectedPrompt, agentContext, context);

    // 8. FIRE SubagentStop HOOK
    await executeHooks('SubagentStop', {
      agent_id: result.agentId,
      agent_type: subagent_type,
      result: result.finalMessage
    }, context);

    // 9. TRACK METRICS
    trackAgentMetrics({
      duration: result.duration,
      tokens: result.tokenUsage,
      model,
      agentType: subagent_type
    });

    return { data: result.finalMessage };
  }
};
```

### 4.4 Color Assignment Algorithm

```javascript
// DEMINIFIED: Agent color assignment
const COLOR_POOL = [
  'claudeBlue', 'claudePurple', 'claudeOrange',
  'claudeGreen', 'claudeRed', 'claudeYellow'
];

function assignAgentColor(agentType, colorMap) {
  // Return existing color if already assigned
  if (colorMap.has(agentType)) {
    return colorMap.get(agentType);
  }

  // Get used colors
  const usedColors = new Set(colorMap.values());

  // Find first available color
  const available = COLOR_POOL.find(c => !usedColors.has(c));
  const color = available || COLOR_POOL[colorMap.size % COLOR_POOL.length];

  colorMap.set(agentType, color);
  return color;
}
```

### 4.5 Agent Loop Execution

```javascript
// DEMINIFIED: Main agent execution loop
async function runAgentLoop(prompt, agentContext, parentContext) {
  const { systemPrompt, tools, model, color } = agentContext;
  const startTime = Date.now();

  // Initialize conversation
  const messages = [
    { role: 'user', content: prompt }
  ];

  let totalTokens = { input: 0, output: 0, cached: 0 };
  let iteration = 0;
  const MAX_ITERATIONS = 100;

  while (iteration < MAX_ITERATIONS) {
    iteration++;

    // Call API
    const response = await createMessage({
      model: resolveModelString(model),
      system: systemPrompt,
      messages,
      tools: formatToolsForAPI(tools),
      max_tokens: 8192
    });

    // Track tokens
    totalTokens.input += response.usage.input_tokens;
    totalTokens.output += response.usage.output_tokens;
    totalTokens.cached += response.usage.cache_read_input_tokens || 0;

    // Check for stop
    if (response.stop_reason === 'end_turn') {
      const textContent = response.content.find(c => c.type === 'text');
      return {
        agentId: generateAgentId(),
        finalMessage: textContent?.text || '',
        duration: Date.now() - startTime,
        tokenUsage: totalTokens
      };
    }

    // Process tool uses
    const toolUses = response.content.filter(c => c.type === 'tool_use');
    if (toolUses.length === 0) {
      break;  // No tools and not end_turn - unexpected
    }

    // Execute tools (possibly in parallel)
    const toolResults = await executeToolsConcurrently(toolUses, tools, parentContext);

    // Add to conversation
    messages.push({ role: 'assistant', content: response.content });
    messages.push({ role: 'user', content: toolResults });
  }

  return {
    agentId: generateAgentId(),
    finalMessage: 'Agent reached iteration limit',
    duration: Date.now() - startTime,
    tokenUsage: totalTokens
  };
}
```

---

## 5. Hook System - Deminified

### 5.1 All 11 Hook Events

```javascript
// DEMINIFIED: Hook event definitions
const HOOK_EVENTS = {
  PreToolUse: {
    summary: 'Before any tool executes',
    description: 'Validate, modify, or block tool calls',
    matcherField: 'tool_name',
    exitCodes: {
      0: 'Success - continue',
      2: 'BLOCK - deny tool execution',
      other: 'Non-blocking error'
    }
  },

  PostToolUse: {
    summary: 'After tool completes',
    description: 'React to results, provide feedback',
    matcherField: 'tool_name',
    exitCodes: {
      0: 'Success - stdout shown',
      2: 'Feed stderr back to Claude',
      other: 'Non-blocking'
    }
  },

  UserPromptSubmit: {
    summary: 'When user submits prompt',
    description: 'Add context, validate, or block prompts',
    matcherField: 'user_prompt',
    exitCodes: { 0: 'Continue', 2: 'Block submission' }
  },

  Stop: {
    summary: 'When main agent considers stopping',
    description: 'Validate task completeness',
    matcherField: 'reason',
    exitCodes: {
      0: 'Allow stop',
      2: 'Show to model but CONTINUE'
    }
  },

  SubagentStart: {
    summary: 'When subagent begins',
    matcherField: 'agent_type',
    exitCodes: { 0: 'stdout shown to subagent' }
  },

  SubagentStop: {
    summary: 'When subagent completing',
    matcherField: 'agent_type',
    exitCodes: { 0: 'Silent', 2: 'Show to subagent, continue' }
  },

  SessionStart: {
    summary: 'Session begins',
    description: 'Load context, set environment',
    matcherField: '*',
    special: 'Can persist env vars via $CLAUDE_ENV_FILE'
  },

  SessionEnd: {
    summary: 'Session ends',
    description: 'Cleanup, logging'
  },

  PreCompact: {
    summary: 'Before context compaction',
    description: 'Preserve critical information',
    exitCodes: { 2: 'Block compaction' }
  },

  Notification: {
    summary: 'When notification sent',
    description: 'React to user notifications'
  },

  StatusLine: {
    summary: 'Continuous status display',
    description: 'Update status bar content'
  }
};
```

### 5.2 Hook Execution Engine

```javascript
// DEMINIFIED: Core hook executor
async function executeHooks(eventName, input, context) {
  const hooks = getHooksForEvent(eventName, context.registeredHooks);
  if (hooks.length === 0) {
    return { blocked: false };
  }

  // Build hook input
  const hookInput = {
    session_id: context.sessionId,
    transcript_path: context.transcriptPath,
    cwd: context.cwd,
    permission_mode: context.permissionMode,
    hook_event_name: eventName,
    ...input
  };

  // Run all matching hooks IN PARALLEL
  const results = await Promise.all(
    hooks
      .filter(hook => matchesHook(hook.matcher, hookInput))
      .map(hook => executeHook(hook, hookInput, context))
  );

  // Aggregate results
  return aggregateHookResults(results, eventName);
}

// DEMINIFIED: Execute single hook
async function executeHook(hook, input, context) {
  const { type, command, prompt, timeout = 60000 } = hook;

  switch (type) {
    case 'command':
      return executeCommandHook(command, input, timeout);

    case 'prompt':
      return executePromptHook(prompt, input, timeout, context);

    case 'function':
      return hook.fn(input, context);

    default:
      throw new Error(`Unknown hook type: ${type}`);
  }
}

// DEMINIFIED: Command hook execution
async function executeCommandHook(command, input, timeout) {
  // Replace ${CLAUDE_PLUGIN_ROOT} with actual path
  const resolvedCommand = command.replace(
    /\$\{CLAUDE_PLUGIN_ROOT\}/g,
    process.env.CLAUDE_PLUGIN_ROOT || ''
  );

  return new Promise((resolve) => {
    const child = spawn('/bin/bash', ['-c', resolvedCommand], {
      timeout,
      env: {
        ...process.env,
        CLAUDE_PROJECT_DIR: input.cwd
      }
    });

    // Pipe input as JSON
    child.stdin.write(JSON.stringify(input));
    child.stdin.end();

    let stdout = '';
    let stderr = '';

    child.stdout.on('data', d => stdout += d);
    child.stderr.on('data', d => stderr += d);

    child.on('exit', (code) => {
      try {
        const output = stdout ? JSON.parse(stdout) : {};
        resolve({
          exitCode: code,
          output,
          stderr,
          blocked: code === 2
        });
      } catch {
        resolve({
          exitCode: code,
          output: { message: stdout },
          stderr,
          blocked: code === 2
        });
      }
    });
  });
}

// DEMINIFIED: Prompt hook execution (uses Claude)
async function executePromptHook(prompt, input, timeout, context) {
  // Replace variables in prompt
  const resolvedPrompt = prompt
    .replace('$TOOL_INPUT', JSON.stringify(input.tool_input))
    .replace('$TOOL_NAME', input.tool_name)
    .replace('$USER_PROMPT', input.user_prompt);

  const response = await createMessage({
    model: 'claude-3-haiku-20240307',  // Fast model for hooks
    messages: [{ role: 'user', content: resolvedPrompt }],
    max_tokens: 1024
  });

  const text = response.content[0]?.text || '';

  // Parse decision
  const isApproved = /approve|allow|yes/i.test(text);
  const isDenied = /deny|block|no/i.test(text);

  return {
    exitCode: isDenied ? 2 : 0,
    output: {
      decision: isDenied ? 'deny' : 'allow',
      reason: text
    },
    blocked: isDenied
  };
}
```

### 5.3 Permission Aggregation

```javascript
// DEMINIFIED: Aggregate multiple hook results
function aggregateHookResults(results, eventName) {
  // Permission hierarchy: deny > ask > allow > passthrough
  const PERMISSION_PRIORITY = {
    deny: 4,
    ask: 3,
    allow: 2,
    passthrough: 1
  };

  let finalPermission = 'passthrough';
  let systemMessages = [];
  let modifiedInput = null;

  for (const result of results) {
    const { output, blocked } = result;

    // Collect system messages
    if (output.systemMessage) {
      systemMessages.push(output.systemMessage);
    }

    // Track input modifications
    if (output.updatedInput) {
      modifiedInput = { ...modifiedInput, ...output.updatedInput };
    }

    // Determine permission
    const permission = output.permissionDecision || (blocked ? 'deny' : 'allow');
    if (PERMISSION_PRIORITY[permission] > PERMISSION_PRIORITY[finalPermission]) {
      finalPermission = permission;
    }
  }

  return {
    blocked: finalPermission === 'deny',
    permission: finalPermission,
    systemMessage: systemMessages.join('\n'),
    modifiedInput
  };
}
```

---

## 6. API Client - Deminified

### 6.1 Client Initialization

```javascript
// DEMINIFIED: Anthropic API client setup
function createAnthropicClient(config) {
  const {
    apiKey,
    bearerToken,
    baseURL = 'https://api.anthropic.com',
    timeout = 600000,
    maxRetries = 2
  } = config;

  return {
    baseURL,
    timeout,
    maxRetries,

    headers: {
      'anthropic-version': '2023-06-01',
      'content-type': 'application/json',
      ...(apiKey && { 'x-api-key': apiKey }),
      ...(bearerToken && { 'authorization': `Bearer ${bearerToken}` })
    },

    async createMessage(params) {
      return this.post('/v1/messages', params);
    },

    async createMessageStream(params) {
      return this.stream('/v1/messages', { ...params, stream: true });
    }
  };
}
```

### 6.2 Streaming Response Handler

```javascript
// DEMINIFIED: SSE stream processor
class MessageStream {
  constructor(response) {
    this.response = response;
    this.message = null;
    this.currentBlock = null;
    this.buffer = '';
  }

  async *[Symbol.asyncIterator]() {
    const reader = this.response.body.getReader();
    const decoder = new TextDecoder();

    while (true) {
      const { value, done } = await reader.read();
      if (done) break;

      this.buffer += decoder.decode(value, { stream: true });

      // Parse SSE events
      const events = this.buffer.split('\n\n');
      this.buffer = events.pop();  // Keep incomplete event

      for (const event of events) {
        const parsed = this.parseSSE(event);
        if (parsed) {
          yield* this.processEvent(parsed);
        }
      }
    }
  }

  parseSSE(event) {
    const lines = event.split('\n');
    let eventType = 'message';
    let data = '';

    for (const line of lines) {
      if (line.startsWith('event: ')) {
        eventType = line.slice(7);
      } else if (line.startsWith('data: ')) {
        data += line.slice(6);
      }
    }

    if (!data) return null;

    return {
      type: eventType,
      data: JSON.parse(data)
    };
  }

  *processEvent(event) {
    switch (event.type) {
      case 'message_start':
        this.message = event.data.message;
        yield { type: 'message_start', message: this.message };
        break;

      case 'content_block_start':
        this.currentBlock = event.data.content_block;
        this.currentBlock.index = event.data.index;
        yield { type: 'content_block_start', block: this.currentBlock };
        break;

      case 'content_block_delta':
        const delta = event.data.delta;
        if (delta.type === 'text_delta') {
          this.currentBlock.text = (this.currentBlock.text || '') + delta.text;
          yield { type: 'text_delta', text: delta.text };
        } else if (delta.type === 'input_json_delta') {
          this.currentBlock.partial_json =
            (this.currentBlock.partial_json || '') + delta.partial_json;
          yield { type: 'input_json_delta', json: delta.partial_json };
        }
        break;

      case 'content_block_stop':
        // Parse complete tool input
        if (this.currentBlock.type === 'tool_use' && this.currentBlock.partial_json) {
          this.currentBlock.input = JSON.parse(this.currentBlock.partial_json);
          delete this.currentBlock.partial_json;
        }
        yield { type: 'content_block_stop', block: this.currentBlock };
        this.currentBlock = null;
        break;

      case 'message_delta':
        Object.assign(this.message, event.data.delta);
        yield { type: 'message_delta', delta: event.data.delta };
        break;

      case 'message_stop':
        yield { type: 'message_stop', message: this.message };
        break;
    }
  }
}
```

### 6.3 Retry Logic

```javascript
// DEMINIFIED: Exponential backoff retry
async function withRetry(fn, options = {}) {
  const { maxRetries = 2, initialDelay = 1000 } = options;

  let lastError;
  for (let attempt = 0; attempt <= maxRetries; attempt++) {
    try {
      return await fn();
    } catch (error) {
      lastError = error;

      // Don't retry on auth errors
      if (error.status === 401 || error.status === 403) {
        throw error;
      }

      // Check for rate limit with retry-after
      if (error.status === 429) {
        const retryAfter = error.headers?.['retry-after'];
        if (retryAfter) {
          await sleep(parseInt(retryAfter) * 1000);
          continue;
        }
      }

      // Exponential backoff
      if (attempt < maxRetries) {
        const delay = initialDelay * Math.pow(2, attempt);
        const jitter = Math.random() * 1000;
        await sleep(delay + jitter);
      }
    }
  }

  throw lastError;
}
```

---

## 7. Terminal UI Architecture

### 7.1 Component Statistics

| Pattern | Count | Purpose |
|---------|-------|---------|
| `createElement` | 4,501 | JSX compilation |
| `useState` | 376 | Local state |
| `useEffect` | 167 | Side effects |
| `useCallback` | 149 | Memoized callbacks |
| `useMemo` | 122 | Memoized values |
| `memo` | 538 | Component memoization |
| `Provider` | 1,590 | Context providers |
| `Context` | 1,483 | Context consumers |

### 7.2 Layout System (Yoga/Flexbox)

```javascript
// DEMINIFIED: Ink layout patterns
const LayoutComponent = ({ children }) => {
  return createElement(Box, {
    flexDirection: 'column',
    padding: 1,
    margin: 1,
    borderStyle: 'round',
    borderColor: 'claudeBlue'
  }, children);
};
```

### 7.3 Semantic Color System

```javascript
// DEMINIFIED: Theme colors
const SEMANTIC_COLORS = {
  error: 'red',
  warning: 'yellow',
  success: 'green',
  permission: 'cyan',
  text: 'white',
  suggestion: 'magenta',
  claude: 'blue',
  inactive: 'gray',
  dim: 'darkGray',
  bashBorder: 'green',
  promptBorder: 'blue'
};
```

### 7.4 Key UI Components

```javascript
// DEMINIFIED: Main components
const UI_COMPONENTS = {
  // Main chat interface
  bXA: 'InteractiveChat',

  // Conversation browser
  FC9: 'ConversationBrowser',

  // State provider
  M7: 'StateProvider',

  // Trust dialog
  JC9: 'TrustDialog',

  // Onboarding
  lH9: 'OnboardingSetup',

  // Welcome screen
  HJ9: 'WelcomeScreen',

  // Help dialog
  UJ9: 'HelpDialog',

  // IDE connect
  tT3: 'IDEConnector',

  // Network check
  cH9: 'NetworkCheck'
};
```

---

## 8. HTTP Client (Undici)

### 8.1 Error Classes (Extracted)

```javascript
// DEMINIFIED: Undici error hierarchy
class UndiciError extends Error {
  constructor(message) {
    super(message);
    this.name = 'UndiciError';
    this.code = 'UND_ERR';
  }
}

class ConnectTimeoutError extends UndiciError {
  constructor(message) {
    super(message || 'Connect Timeout Error');
    this.name = 'ConnectTimeoutError';
    this.code = 'UND_ERR_CONNECT_TIMEOUT';
  }
}

class HeadersTimeoutError extends UndiciError {
  constructor(message) {
    super(message || 'Headers Timeout Error');
    this.name = 'HeadersTimeoutError';
    this.code = 'UND_ERR_HEADERS_TIMEOUT';
  }
}

class BodyTimeoutError extends UndiciError {
  constructor(message) {
    super(message || 'Body Timeout Error');
    this.name = 'BodyTimeoutError';
    this.code = 'UND_ERR_BODY_TIMEOUT';
  }
}

class ResponseStatusCodeError extends UndiciError {
  constructor(message, status, headers, body) {
    super(message || 'Response Status Code Error');
    this.name = 'ResponseStatusCodeError';
    this.code = 'UND_ERR_RESPONSE_STATUS_CODE';
    this.status = status;
    this.statusCode = status;
    this.headers = headers;
    this.body = body;
  }
}
```

---

## 9. Key Variable Mapping

### 9.1 Core Identifiers

| Minified | Deminified | Purpose |
|----------|------------|---------|
| `M` | `createLazyModule` | Module loader |
| `MA` | `fileSystem` | FS facade |
| `R8` | `TaskTool` | Agent spawner |
| `nN9` | `createGlobalState` | Session state factory |
| `IQ` | `globalState` | Session singleton |
| `A0` | `getSessionId` | Session ID getter |
| `gBA` | `getCwd` | Current directory |
| `SK` | `resolveSymlink` | Symlink resolver |
| `XT` | `isInWorkTree` | Git check |

### 9.2 React/Ink Patterns

| Minified | Deminified | Purpose |
|----------|------------|---------|
| `AC` | `InkWrapper` | Ink renderer |
| `j` | `Box` | Layout container |
| `$` | `Text` | Text element |
| `L0` | `Select` | Selection menu |
| `a4` | `TextInput` | Text input |
| `A4` | `Spinner` | Loading indicator |

---

## 10. Architectural Insights

### 10.1 Design Patterns

1. **Lazy Module Loading**: Custom `M()` function defers initialization
2. **Singleton State**: `nN9()` creates single global session state
3. **Command Pattern**: Tools are command objects with standardized interface
4. **Observer Pattern**: Hooks observe tool/agent lifecycle events
5. **Strategy Pattern**: Hook types (command, prompt, function) are strategies
6. **Concurrent Queue**: Tools execute in parallel when safe

### 10.2 Performance Optimizations

1. **Memoization**: 538 `memo()` calls for React components
2. **Lazy Loading**: Deferred module initialization
3. **Parallel Execution**: Safe tools run concurrently
4. **Streaming**: SSE for real-time API responses
5. **Buffered I/O**: 2000-char chunks for stdout/stderr

### 10.3 Security Measures

1. **Path Validation**: Absolute paths required
2. **File Read Tracking**: Must read before edit
3. **Permission Hooks**: PreToolUse can block
4. **Sandbox Mode**: Optional command sandboxing
5. **Trust Dialog**: Workspace approval required

### 10.4 Extensibility Points

1. **Custom Agents**: `.claude/agents/*.md`
2. **Custom Commands**: `.claude/commands/*.md`
3. **Custom Skills**: `.claude/skills/*/SKILL.md`
4. **Custom Hooks**: `hooks/hooks.json`
5. **MCP Servers**: `.mcp.json`
6. **Plugins**: Full component customization

---

## Appendix: Files Generated

| File | Content |
|------|---------|
| `ARCHITECTURE.md` | High-level architectural overview |
| `DEEP_DIVE_RESEARCH.md` | This document |
| `AGENT_SYSTEM_DEMINIFIED.js` | Agent system code |
| `HOOK_SYSTEM_DEMINIFIED.js` | Hook system code |
| `HOOK_SYSTEM_ANALYSIS.md` | Hook documentation |
| `DEMINIFIED_API_CLIENT_AND_MESSAGE_HANDLING.md` | API client analysis |

---

*Generated by atomic-level analysis using general-purpose, Explore, and Plan agents*
*Claude Code v2.0.55 - 2025-11-30*
