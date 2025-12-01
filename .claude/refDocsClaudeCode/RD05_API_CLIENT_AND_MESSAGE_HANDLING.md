# DEMINIFIED API CLIENT AND MESSAGE HANDLING ARCHITECTURE

## Complete API Interaction Flow Analysis from cli.js

---

## 1. API CLIENT INITIALIZATION

### Anthropic SDK Client Creation
```javascript
// Core client instantiation (lines ~1311-1312)
new Anthropic({
  apiKey,
  dangerouslyAllowBrowser: true
});

// Client configuration
class YG extends BaseClient {
  constructor({
    baseURL = process.env.ANTHROPIC_BASE_URL,
    apiKey = process.env.ANTHROPIC_API_KEY ?? null,
    authToken = process.env.ANTHROPIC_AUTH_TOKEN ?? null,
    ...options
  } = {}) {
    // Defaults
    this.baseURL = options.baseURL || "https://api.anthropic.com"
    this.timeout = options.timeout ?? 600000  // 10 minutes default
    this.maxRetries = options.maxRetries ?? 2
    this.logger = options.logger ?? console
    this.logLevel = options.logLevel || "warn"
    this.apiKey = typeof apiKey === 'string' ? apiKey : null
    this.authToken = authToken

    // Initialize resource endpoints
    this.completions = new Completions(this)
    this.messages = new Messages(this)
    this.models = new Models(this)
    this.beta = new Beta(this)
  }
}
```

### Authentication Headers
```javascript
// API Key authentication (primary method)
async apiKeyAuth(request) {
  if (this.apiKey == null) return;
  return { "X-Api-Key": this.apiKey }
}

// Bearer token authentication (OAuth)
async bearerAuth(request) {
  if (this.authToken == null) return;
  return { Authorization: `Bearer ${this.authToken}` }
}

// Header validation and construction
async authHeaders(request) {
  return mergeHeaders([
    await this.apiKeyAuth(request),
    await this.bearerAuth(request)
  ])
}
```

---

## 2. MESSAGE CREATION API

### Beta Messages Creation
```javascript
// Beta Messages class (supports advanced features)
class BetaMessages extends Resource {
  create(params, options) {
    const { betas, ...messageParams } = params;

    // Model deprecation warnings
    if (messageParams.model in deprecatedModels) {
      console.warn(`Model '${messageParams.model}' is deprecated and will reach end-of-life on ${deprecatedModels[messageParams.model]}`)
    }

    // Timeout calculation for non-streaming
    let timeout = this._client._options.timeout;
    if (!messageParams.stream && timeout == null) {
      const maxOutputTokens = modelMaxOutputTokens[messageParams.model] ?? undefined;
      timeout = this._client.calculateNonstreamingTimeout(
        messageParams.max_tokens,
        maxOutputTokens
      )
    }

    // POST request with beta headers
    return this._client.post('/v1/messages?beta=true', {
      body: messageParams,
      timeout: timeout ?? 600000,
      ...options,
      headers: mergeHeaders([{
        'anthropic-beta': betas?.toString() ?? undefined
      }, options?.headers]),
      stream: params.stream ?? false
    })
  }
}
```

### Message Structure
```javascript
const messageRequest = {
  model: "claude-sonnet-4-20250514",
  max_tokens: 8192,
  messages: [
    {
      role: "user",
      content: "User prompt here"
    }
  ],
  tools: [/* tool definitions */],
  betas: ["claude-code-20250219", "interleaved-thinking-2025-05-14"],
  metadata: {
    user_id: "session-uuid"
  },
  thinking: {
    type: "enabled",
    budget_tokens: 1024
  },
  stream: true
}
```

---

## 3. STREAMING RESPONSE HANDLING

### Stream Class Architecture
```javascript
class MessageStream {
  constructor() {
    this.messages = []
    this.receivedMessages = []
    this.controller = new AbortController()
    this._currentMessage = undefined
    this._resolvePromise = () => {}
    this._rejectPromise = () => {}
    this._eventHandlers = {}
    this._ended = false
    this._errored = false
    this._aborted = false

    // Promises for async operations
    this._connectedPromise = new Promise((resolve, reject) => {
      this._resolveConnected = resolve
      this._rejectConnected = reject
    })

    this._finalPromise = new Promise((resolve, reject) => {
      this._resolveFinal = resolve
      this._rejectFinal = reject
    })
  }

  // Create stream from API
  static createMessage(client, params, options) {
    const stream = new MessageStream()

    // Add user messages to stream state
    for (let message of params.messages) {
      stream._addMessageParam(message)
    }

    // Start streaming
    stream._run(() => stream._createMessage(client, {
      ...params,
      stream: true
    }, {
      ...options,
      headers: {
        ...options?.headers,
        "X-Stainless-Helper-Method": "stream"
      }
    }))

    return stream
  }
}
```

### Server-Sent Events (SSE) Processing
```javascript
class SSEDecoder {
  constructor() {
    this._buffer = new Uint8Array()
    this._carriageReturn = null
  }

  decode(chunk) {
    // Append to buffer
    this._buffer = concatenateArrays([this._buffer, chunk])

    const events = []
    let lineStart = 0

    while (lineStart < this._buffer.length) {
      const newlineIndex = findNewline(this._buffer, lineStart)

      if (newlineIndex === -1) break

      const line = decodeText(this._buffer.subarray(lineStart, newlineIndex))
      lineStart = newlineIndex + 1

      // Parse SSE format: "event: type\ndata: json\n\n"
      if (line.startsWith('event:')) {
        this._event = line.substring(6).trim()
      } else if (line.startsWith('data:')) {
        this._data.push(line.substring(5).trim())
      } else if (line === '') {
        // Empty line signals end of event
        if (this._event && this._data.length > 0) {
          events.push({
            event: this._event,
            data: this._data.join('\n'),
            raw: this._chunks
          })
          this._event = null
          this._data = []
          this._chunks = []
        }
      }
    }

    // Keep remaining buffer
    this._buffer = this._buffer.subarray(lineStart)

    return events
  }
}
```

### Stream Event Types Processing
```javascript
async *iterateStreamEvents(response, controller) {
  const decoder = new SSEDecoder()

  for await (const chunk of response.body) {
    for (const event of decoder.decode(chunk)) {
      // Parse different event types
      if (event.event === 'completion') {
        yield JSON.parse(event.data)
      }

      if (event.event === 'message_start' ||
          event.event === 'message_delta' ||
          event.event === 'message_stop' ||
          event.event === 'content_block_start' ||
          event.event === 'content_block_delta' ||
          event.event === 'content_block_stop') {
        yield JSON.parse(event.data)
      }

      if (event.event === 'ping') {
        continue  // Heartbeat, no action needed
      }

      if (event.event === 'error') {
        throw new APIError(undefined, JSON.parse(event.data), undefined, response.headers)
      }
    }
  }

  // Flush remaining data
  for (const event of decoder.flush()) {
    yield JSON.parse(event)
  }
}
```

---

## 4. INCREMENTAL MESSAGE ASSEMBLY

### Message State Management
```javascript
function processStreamEvent(event, currentMessage) {
  switch (event.type) {
    case 'message_start':
      // Initialize new message
      if (currentMessage) {
        throw new Error(`Unexpected event order, got ${event.type} before receiving "message_stop"`)
      }
      return event.message

    case 'message_delta':
      // Update message metadata
      if (event.delta.container) currentMessage.container = event.delta.container
      if (event.delta.stop_reason) currentMessage.stop_reason = event.delta.stop_reason
      if (event.delta.stop_sequence) currentMessage.stop_sequence = event.delta.stop_sequence
      currentMessage.usage.output_tokens = event.usage.output_tokens

      if (event.usage.input_tokens != null) {
        currentMessage.usage.input_tokens = event.usage.input_tokens
      }
      if (event.usage.cache_creation_input_tokens != null) {
        currentMessage.usage.cache_creation_input_tokens = event.usage.cache_creation_input_tokens
      }
      if (event.usage.cache_read_input_tokens != null) {
        currentMessage.usage.cache_read_input_tokens = event.usage.cache_read_input_tokens
      }
      if (event.usage.server_tool_use != null) {
        currentMessage.usage.server_tool_use = event.usage.server_tool_use
      }
      return currentMessage

    case 'content_block_start':
      // Add new content block
      currentMessage.content.push(event.content_block)
      return currentMessage

    case 'content_block_delta':
      const contentBlock = currentMessage.content.at(event.index)

      switch (event.delta.type) {
        case 'text_delta':
          if (contentBlock.type === 'text') {
            currentMessage.content[event.index] = {
              ...contentBlock,
              text: (contentBlock.text || '') + event.delta.text
            }
          }
          break

        case 'citations_delta':
          if (contentBlock.type === 'text') {
            currentMessage.content[event.index] = {
              ...contentBlock,
              citations: [...(contentBlock.citations ?? []), event.delta.citation]
            }
          }
          break

        case 'input_json_delta':
          if (isToolUse(contentBlock)) {
            const jsonBuffer = contentBlock.__json_buf || ''
            const updatedBuffer = jsonBuffer + event.delta.partial_json

            const updatedBlock = { ...contentBlock }
            Object.defineProperty(updatedBlock, '__json_buf', {
              value: updatedBuffer,
              enumerable: false,
              writable: true
            })

            if (updatedBuffer) {
              try {
                updatedBlock.input = parsePartialJSON(updatedBuffer)
              } catch (error) {
                throw new Error(`Unable to parse tool parameter JSON: ${error}. JSON: ${updatedBuffer}`)
              }
            }

            currentMessage.content[event.index] = updatedBlock
          }
          break

        case 'thinking_delta':
          if (contentBlock.type === 'thinking') {
            currentMessage.content[event.index] = {
              ...contentBlock,
              thinking: contentBlock.thinking + event.delta.thinking
            }
          }
          break

        case 'signature_delta':
          if (contentBlock.type === 'thinking') {
            currentMessage.content[event.index] = {
              ...contentBlock,
              signature: event.delta.signature
            }
          }
          break
      }
      return currentMessage

    case 'content_block_stop':
      return currentMessage

    case 'message_stop':
      return currentMessage
  }
}
```

---

## 5. TOOL USE AND RESULT CYCLE

### Tool Execution Loop
```javascript
class BetaToolRunner {
  constructor(client, params, options) {
    this.client = client
    this._params = {
      params: {
        ...params,
        messages: structuredClone(params.messages)
      }
    }
    this._options = {
      ...options,
      headers: mergeHeaders([{
        "x-stainless-helper": "BetaToolRunner"
      }, options?.headers])
    }
    this._iterationCount = 0
  }

  async *[Symbol.asyncIterator]() {
    while (true) {
      // Check max iterations
      if (this._params.params.max_iterations &&
          this._iterationCount >= this._params.params.max_iterations) {
        break
      }

      this._iterationCount++

      const { max_iterations, ...requestParams } = this._params.params

      let stream

      // Create stream or non-stream request
      if (requestParams.stream) {
        stream = this.client.beta.messages.stream(
          { ...requestParams },
          this._options
        )
        this._messagePromise = stream.finalMessage()
        yield stream
      } else {
        this._messagePromise = this.client.beta.messages.create(
          { ...requestParams, stream: false },
          this._options
        )
        yield this._messagePromise
      }

      // Get final message
      const { role, content } = await this._messagePromise
      this._params.params.messages.push({ role, content })

      // Generate tool results
      const toolResultMessage = await generateToolResponse(
        this._params.params,
        this._params.params.messages.at(-1)
      )

      if (toolResultMessage) {
        this._params.params.messages.push(toolResultMessage)
      }

      // Stop if no tool use or completed
      if (!toolResultMessage) {
        break
      }
    }

    if (!this._messagePromise) {
      throw new Error("ToolRunner concluded without a message from the server")
    }

    return await this._messagePromise
  }
}
```

### Tool Result Generation
```javascript
async function generateToolResponse(params, lastMessage) {
  if (!lastMessage ||
      lastMessage.role !== 'assistant' ||
      !lastMessage.content ||
      typeof lastMessage.content === 'string') {
    return null
  }

  const toolUses = lastMessage.content.filter(block => block.type === 'tool_use')

  if (toolUses.length === 0) {
    return null
  }

  return {
    role: 'user',
    content: await Promise.all(toolUses.map(async (toolUse) => {
      const tool = params.tools.find(t => t.name === toolUse.name)

      if (!tool || !('run' in tool)) {
        return {
          type: 'tool_result',
          tool_use_id: toolUse.id,
          content: `Error: Tool '${toolUse.name}' not found`,
          is_error: true
        }
      }

      try {
        let input = toolUse.input

        // Parse input if needed
        if ('parse' in tool && tool.parse) {
          input = tool.parse(input)
        }

        // Execute tool
        const result = await tool.run(input)

        return {
          type: 'tool_result',
          tool_use_id: toolUse.id,
          content: result
        }
      } catch (error) {
        return {
          type: 'tool_result',
          tool_use_id: toolUse.id,
          content: `Error: ${error instanceof Error ? error.message : String(error)}`,
          is_error: true
        }
      }
    }))
  }
}
```

---

## 6. ERROR HANDLING AND RETRIES

### Retry Logic
```javascript
async retryRequest(options, retriesLeft, parentRequestId, headers) {
  // Parse retry-after header
  let retryAfterMs
  const retryAfterMsHeader = headers?.get('retry-after-ms')

  if (retryAfterMsHeader) {
    const value = parseFloat(retryAfterMsHeader)
    if (!Number.isNaN(value)) {
      retryAfterMs = value
    }
  }

  const retryAfterHeader = headers?.get('retry-after')
  if (retryAfterHeader && !retryAfterMs) {
    const value = parseFloat(retryAfterHeader)
    if (!Number.isNaN(value)) {
      retryAfterMs = value * 1000
    } else {
      retryAfterMs = Date.parse(retryAfterHeader) - Date.now()
    }
  }

  // Calculate default retry timeout if not provided
  if (!(retryAfterMs && 0 <= retryAfterMs && retryAfterMs < 60000)) {
    const maxRetries = options.maxRetries ?? this.maxRetries
    retryAfterMs = this.calculateDefaultRetryTimeoutMillis(retriesLeft, maxRetries)
  }

  // Wait before retry
  await sleep(retryAfterMs)

  // Retry the request
  return this.makeRequest(options, retriesLeft - 1, parentRequestId)
}

calculateDefaultRetryTimeoutMillis(retriesLeft, maxRetries) {
  const retriesSoFar = maxRetries - retriesLeft
  const exponentialBackoff = Math.min(0.5 * Math.pow(2, retriesSoFar), 8)
  const jitterFactor = 1 - Math.random() * 0.25
  return exponentialBackoff * jitterFactor * 1000
}

async shouldRetry(response) {
  const xShouldRetry = response.headers.get('x-should-retry')

  if (xShouldRetry === 'true') return true
  if (xShouldRetry === 'false') return false

  // Retry on specific status codes
  if (response.status === 408) return true  // Timeout
  if (response.status === 409) return true  // Conflict
  if (response.status === 429) return true  // Rate limit
  if (response.status >= 500) return true   // Server errors

  return false
}
```

### Error Classification
```javascript
function handleAPIError(error, model, context) {
  // Timeout errors
  if (error instanceof ConnectionTimeoutError ||
      error instanceof APIConnectionError && error.message.toLowerCase().includes('timeout')) {
    return formatError({
      content: 'Request timed out',
      error: 'api_timeout'
    })
  }

  // Rate limiting errors
  if (error instanceof APIError && error.status === 429) {
    const unifiedRateLimitHeaders = extractRateLimitHeaders(error.headers)

    if (unifiedRateLimitHeaders) {
      const warningMessage = formatRateLimitWarning(unifiedRateLimitHeaders)
      if (warningMessage) {
        return formatError({
          content: warningMessage,
          error: 'rate_limit'
        })
      }
    }

    return formatError({
      content: 'API Error: Rate limit reached',
      error: 'rate_limit'
    })
  }

  // Prompt too long errors
  if (error instanceof Error && error.message.includes('prompt is too long')) {
    return formatError({
      content: 'Prompt is too long',
      error: 'invalid_request'
    })
  }

  // Authentication errors
  if (error instanceof APIError && (error.status === 401 || error.status === 403)) {
    return formatError({
      error: 'authentication_failed',
      content: `API Error: ${error.message} · Please run /login`
    })
  }

  // Generic error
  if (error instanceof Error) {
    return formatError({
      content: `API Error: ${error.message}`,
      error: 'unknown'
    })
  }

  return formatError({
    content: 'API Error',
    error: 'unknown'
  })
}
```

---

## 7. RATE LIMITING

### Unified Rate Limit Headers
```javascript
function parseRateLimitHeaders(headers) {
  const status = headers.get('anthropic-ratelimit-unified-status') || 'allowed'
  const resetHeader = headers.get('anthropic-ratelimit-unified-reset')
  const resetsAt = resetHeader ? Number(resetHeader) : undefined
  const fallbackAvailable = headers.get('anthropic-ratelimit-unified-fallback') === 'available'
  const rateLimitType = headers.get('anthropic-ratelimit-unified-representative-claim')
  const overageStatus = headers.get('anthropic-ratelimit-unified-overage-status')
  const overageResetHeader = headers.get('anthropic-ratelimit-unified-overage-reset')
  const overageResetsAt = overageResetHeader ? Number(overageResetHeader) : undefined

  return {
    status,
    resetsAt,
    unifiedRateLimitFallbackAvailable: fallbackAvailable,
    ...(rateLimitType && { rateLimitType }),
    ...(overageStatus && { overageStatus }),
    ...(overageResetsAt && { overageResetsAt }),
    isUsingOverage: status === 'rejected' && (overageStatus === 'allowed' || overageStatus === 'allowed_warning')
  }
}

function formatRateLimitResetTime(timestamp, short = false) {
  const now = Date.now()
  const diffMs = (timestamp * 1000) - now

  if (diffMs <= 0) return 'now'

  const seconds = Math.floor(diffMs / 1000)
  const minutes = Math.floor(seconds / 60)
  const hours = Math.floor(minutes / 60)
  const days = Math.floor(hours / 24)

  if (short) {
    if (days > 0) return `${days}d`
    if (hours > 0) return `${hours}h`
    if (minutes > 0) return `${minutes}m`
    return `${seconds}s`
  }

  if (days > 0) return `in ${days} day${days > 1 ? 's' : ''}`
  if (hours > 0) return `in ${hours} hour${hours > 1 ? 's' : ''}`
  if (minutes > 0) return `in ${minutes} minute${minutes > 1 ? 's' : ''}`
  return `in ${seconds} second${seconds > 1 ? 's' : ''}`
}
```

---

## 8. TOKEN COUNTING

### Input Token Estimation
```javascript
async function countTokens(messages, tools) {
  try {
    const model = getCurrentModel()
    const client = await getAPIClient({ maxRetries: 1, model })
    const betas = getBetaFlags(model)
    const hasThinking = messagesHaveThinking(messages)

    const response = await client.beta.messages.countTokens({
      model: normalizeModelName(model),
      messages: messages.length > 0 ? messages : [{ role: 'user', content: 'foo' }],
      tools,
      ...(betas.length > 0 ? { betas } : {}),
      ...(hasThinking ? {
        thinking: { type: 'enabled', budget_tokens: 1024 },
        max_tokens: 2048
      } : {})
    })

    if (typeof response.input_tokens !== 'number') {
      return null
    }

    return response.input_tokens
  } catch (error) {
    console.error(error)
    return null
  }
}

// Fallback: character-based estimation
function estimateTokens(text) {
  return Math.round(text.length / 4)
}
```

---

## 9. BETA FLAGS AND FEATURES

### Beta Header Construction
```javascript
function getBetaFlags(model) {
  const betas = []
  const isHaiku = model.includes('haiku')
  const platform = getPlatform()
  const experimentalEnabled = isExperimentalFeaturesEnabled()

  // Claude Code specific beta
  if (!isHaiku) {
    betas.push('claude-code-20250219')
  }

  // OAuth/session authentication beta
  if (isUsingOAuth()) {
    betas.push('authenticated-sessions-2025-03-11')
  }

  // Context management betas
  if (model.includes('[1m]')) {
    // 1M context window
    betas.push('context-1m-2025-08-07')
  } else if (model.includes('claude-sonnet-4-5')) {
    if (featureFlag('sonnet_45_1m_header', 'enabled', false)) {
      betas.push('context-1m-2025-08-07')
    }
  }

  // Interleaved thinking
  if (!isDisabled('DISABLE_INTERLEAVED_THINKING') && supportsInterleaving(model)) {
    betas.push('interleaved-thinking-2025-05-14')
  }

  // Context management
  const preserveThinking = experimentalEnabled && featureFlag('preserve_thinking', 'enabled', false)
  if (isEnabled('USE_API_CONTEXT_MANAGEMENT') || preserveThinking) {
    betas.push('context-management-2025-06-27')
  }

  // Structured outputs
  if (isStructuredModel(model) && featureFlag('tengu_tool_pear')) {
    betas.push('structured-outputs-2025-09-17')
  }

  // Tool use examples
  if (experimentalEnabled && featureFlag('tool_use_examples', 'enabled', false)) {
    betas.push('tool-examples-2025-10-29')
  }

  // Web search (Vertex and Foundry)
  if ((platform === 'vertex' || platform === 'foundry') && supportsWebSearch(model)) {
    betas.push('web-search-2025-03-05')
  }

  // Custom betas from environment
  if (process.env.ANTHROPIC_BETAS && !isHaiku) {
    betas.push(...process.env.ANTHROPIC_BETAS.split(',').map(b => b.trim()).filter(Boolean))
  }

  return betas
}
```

---

## 10. COMPLETE REQUEST FLOW EXAMPLE

### Full API Request Lifecycle
```javascript
async function sendMessage(userMessage, conversationHistory, tools) {
  // 1. Prepare message history
  const messages = [
    ...conversationHistory,
    { role: 'user', content: userMessage }
  ]

  // 2. Get model and configuration
  const model = getCurrentModel()
  const betas = getBetaFlags(model)
  const maxTokens = getMaxTokensForModel(model)

  // 3. Create API client
  const client = await getAPIClient({
    maxRetries: 2,
    model
  })

  // 4. Prepare request parameters
  const requestParams = {
    model: normalizeModelName(model),
    max_tokens: maxTokens,
    messages,
    tools,
    betas,
    metadata: getRequestMetadata(),
    stream: true
  }

  // Add thinking if supported
  if (supportsThinking(model)) {
    requestParams.thinking = {
      type: 'enabled',
      budget_tokens: getThinkingBudget()
    }
  }

  // 5. Create streaming response
  const stream = client.beta.messages.stream(requestParams)

  // 6. Process stream events
  const assembledMessage = {
    role: 'assistant',
    content: [],
    usage: {}
  }

  for await (const event of stream) {
    switch (event.type) {
      case 'message_start':
        assembledMessage.id = event.message.id
        assembledMessage.model = event.message.model
        assembledMessage.usage = event.message.usage
        break

      case 'content_block_start':
        assembledMessage.content.push(event.content_block)
        break

      case 'content_block_delta':
        const block = assembledMessage.content[event.index]

        if (event.delta.type === 'text_delta') {
          block.text = (block.text || '') + event.delta.text
          // Emit text incrementally to UI
          emitTextDelta(event.delta.text)
        } else if (event.delta.type === 'thinking_delta') {
          block.thinking = block.thinking + event.delta.thinking
          emitThinkingDelta(event.delta.thinking)
        } else if (event.delta.type === 'input_json_delta') {
          // Accumulate tool input JSON
          block.__json_buf = (block.__json_buf || '') + event.delta.partial_json
          try {
            block.input = parsePartialJSON(block.__json_buf)
          } catch (e) {
            // Continue accumulating
          }
        }
        break

      case 'message_delta':
        assembledMessage.stop_reason = event.delta.stop_reason
        assembledMessage.usage.output_tokens = event.usage.output_tokens
        break

      case 'message_stop':
        break
    }
  }

  // 7. Handle tool uses
  const toolUses = assembledMessage.content.filter(b => b.type === 'tool_use')

  if (toolUses.length > 0) {
    const toolResults = await executeTools(toolUses, tools)

    // Recursively continue conversation with tool results
    return sendMessage(
      '',  // No new user message
      [...conversationHistory,
       { role: 'user', content: userMessage },
       assembledMessage,
       { role: 'user', content: toolResults }
      ],
      tools
    )
  }

  // 8. Update cost tracking
  trackCost(assembledMessage.usage, model)

  // 9. Return final message
  return assembledMessage
}

// Helper: Execute tools
async function executeTools(toolUses, toolDefinitions) {
  return await Promise.all(toolUses.map(async (toolUse) => {
    const tool = toolDefinitions.find(t => t.name === toolUse.name)

    if (!tool) {
      return {
        type: 'tool_result',
        tool_use_id: toolUse.id,
        content: `Error: Tool '${toolUse.name}' not found`,
        is_error: true
      }
    }

    try {
      const result = await tool.run(toolUse.input)
      return {
        type: 'tool_result',
        tool_use_id: toolUse.id,
        content: result
      }
    } catch (error) {
      return {
        type: 'tool_result',
        tool_use_id: toolUse.id,
        content: `Error: ${error.message}`,
        is_error: true
      }
    }
  }))
}

// Helper: Track usage costs
function trackCost(usage, model) {
  const inputCost = calculateInputCost(usage.input_tokens, model)
  const outputCost = calculateOutputCost(usage.output_tokens, model)
  const cacheReadCost = calculateCacheReadCost(usage.cache_read_input_tokens, model)
  const cacheWriteCost = calculateCacheWriteCost(usage.cache_creation_input_tokens, model)

  const totalCost = inputCost + outputCost + cacheReadCost + cacheWriteCost

  updateSessionCost(totalCost, usage, model)
}
```

---

## ARCHITECTURE SUMMARY

### Key Components

1. **API Client Layer**
   - Anthropic SDK wrapper
   - Authentication (API key + OAuth)
   - Retry logic with exponential backoff
   - Rate limit handling

2. **Message Layer**
   - Beta messages endpoint
   - Model-specific configurations
   - Beta feature flags
   - Timeout calculations

3. **Streaming Layer**
   - Server-Sent Events (SSE) decoder
   - Incremental message assembly
   - Event-driven architecture
   - Async iterator interface

4. **Tool Execution Layer**
   - Tool use detection
   - Asynchronous tool execution
   - Result formatting
   - Error handling

5. **State Management**
   - Conversation history
   - Token usage tracking
   - Cost calculation
   - Error logging

### Data Flow

```
User Input
    ↓
[Message Construction]
    ↓
[API Client] → POST /v1/messages?beta=true
    ↓
[SSE Stream] ← Server events
    ↓
[Event Processing]
    ├─ message_start
    ├─ content_block_start
    ├─ content_block_delta
    │   ├─ text_delta → UI
    │   ├─ thinking_delta → UI
    │   └─ input_json_delta → Tool buffer
    ├─ message_delta
    └─ message_stop
    ↓
[Tool Detection]
    ↓
[Tool Execution] → Results
    ↓
[Continuation] → New API request with tool results
    ↓
[Final Response] → User
```

### Performance Optimizations

1. **Streaming**: Immediate UI feedback, no waiting for complete response
2. **Partial JSON Parsing**: Progressive tool input assembly
3. **Prompt Caching**: Reuse of system prompts and context
4. **Connection Pooling**: Persistent HTTP connections
5. **Retry Logic**: Automatic recovery from transient failures

### Security Features

1. **API Key Protection**: Environment variables, file descriptors
2. **OAuth Token Management**: Automatic refresh, secure storage
3. **Rate Limit Compliance**: Exponential backoff, header parsing
4. **Error Sanitization**: No sensitive data in error messages
5. **Request Signing**: Authentication headers validation

---

**Analysis Complete**: This architecture represents a production-grade streaming API client with comprehensive error handling, tool execution, and state management capabilities.
