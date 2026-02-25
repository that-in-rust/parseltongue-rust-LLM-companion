# Keyboard Shortcuts System

## Overview

The keyboard shortcuts system allows users to customize key bindings for common graph interactions like pinning, expanding, hiding nodes, and more.

## Current Strategy: **Option C - Hybrid (Hover + Selection)**

### How It Works

**Priority**: Hover takes precedence when both are present

**Method 1 - Hover (Faster)**:
1. **Hover over a node** (FloatingActionPanel appears after 150ms delay)
2. **Press keyboard shortcut** (e.g., `Ctrl+B`)
3. **Action applies** to the hovered node

**Method 2 - Selection (Explicit)**:
1. **Click to select** a node (node gets highlighted)
2. **Press keyboard shortcut** (e.g., `Ctrl+B`)
3. **Action applies** to the selected node (if not hovering)

### Why This Approach?

- ✅ **Fast for power users**: No click needed, just hover + shortcut
- ✅ **Predictable fallback**: Works with explicit selection too
- ✅ **Visual Feedback**: FloatingActionPanel shows hover target, selection highlight shows selected node
- ✅ **Safe**: Hover delays (150ms) prevent accidental actions
- ✅ **Accessible**: Multiple ways to accomplish the same task
- ✅ **VS Code Friendly**: Shortcuts only active when targeting a node

### Default Shortcuts

| Action | Default Key | Description |
|--------|-------------|-------------|
| Pin/Unpin Node | `Ctrl+B` | Toggle pin status of selected node |
| Expand Node | `ENTER` | Expand/collapse selected node |
| Toggle Focus Mode | `Ctrl+I` | Toggle focus mode for pinned nodes |
| Jump to Code | `Ctrl+J` | Open selected node in editor |
| Hide Node | `Ctrl+H` | Hide selected node from view |
| Fit View | `Ctrl+0` | Fit all nodes in view |
| Focus Search | `Ctrl+F` | Focus the search bar |
| Navigate to Parent | `ARROWUP` | Select parent node (TODO) |
| Navigate to Child | `ARROWDOWN` | Select child node (TODO) |

## How to Customize

1. Open Settings → **Keyboard Shortcuts...**
2. Click **Edit** next to a shortcut
3. Press the new key combination
4. Click **Save**

Shortcuts are saved to `localStorage` and persist across sessions.

## Implementation Details

### Code Location

**Main Handler**: `apps/vscode_extension/src/webview/components/App.tsx` (lines ~1576-1746)

**Settings UI**: `apps/vscode_extension/src/webview/components/KeyboardShortcutsSettings.tsx`

**Hover Tracking**: `apps/vscode_extension/src/webview/components/D3View/rendering/FloatingActionPanelController.tsx` (lines 102, 141)

### How the Hybrid Strategy Works

```typescript
// Track hover state (line 119)
const [hoveredNodeId, setHoveredNodeId] = useState<string | null>(null);

// Priority: hover takes precedence over selection (line 1582-1583)
const selectedNodeId = currentState?.context?.selectedNodeId;
const targetNodeId = hoveredNodeId || selectedNodeId;
const isHoverTarget = hoveredNodeId !== null;
```

### How VS Code Conflicts Are Prevented

```typescript
// Use capture phase to intercept before VS Code
document.addEventListener('keydown', handleKeyDown, { capture: true });

// When a shortcut matches, prevent VS Code from handling it
event.preventDefault();
event.stopPropagation();
```

**Scope**: Shortcuts only work when a node is targeted (hover or selection), so they won't interfere with normal VS Code operations when the graph isn't active.

---

## 🔄 How to Change Strategy

### Revert to **Option A: Selection-Only**

Shortcuts only work on **selected nodes** (must click first).

**Steps to revert:**

1. **Remove hover state** from `App.tsx` line 119:
   ```typescript
   // Delete this line:
   const [hoveredNodeId, setHoveredNodeId] = useState<string | null>(null);
   ```

2. **Update keyboard handler** in `App.tsx` lines 1582-1583:
   ```typescript
   // Change from:
   const targetNodeId = hoveredNodeId || selectedNodeId;
   const isHoverTarget = hoveredNodeId !== null;

   // To:
   const targetNodeId = selectedNodeId;
   ```

3. **Remove hover callback** from `floatingPanelHandlers` in `App.tsx` lines 900-903:
   ```typescript
   // Delete these lines:
   onNodeHoverChange: (nodeId: string | null) => {
     setHoveredNodeId(nodeId);
     logger.debug(`[App.tsx] Hovered node changed: ${nodeId}`);
   },
   ```

4. **Remove hover notifications** from `FloatingActionPanelController.tsx`:
   - Delete line 102: `this.handlers.onNodeHoverChange?.(node.id);`
   - Delete line 141: `this.handlers.onNodeHoverChange?.(null);`

5. **Update this documentation** to reflect selection-only strategy

**Trade-offs:**
- ✅ More predictable (explicit click required)
- ✅ No accidental actions
- ❌ Slower workflow (requires click)

---

### Switch to **Option B: Hover-Only**

Shortcuts only work on **hovered nodes** (no selection fallback).

**Steps to implement:**

1. **Update keyboard handler** in `App.tsx` line 1582:
   ```typescript
   // Change from:
   const targetNodeId = hoveredNodeId || selectedNodeId;

   // To:
   const targetNodeId = hoveredNodeId; // No fallback to selection
   ```

2. **Update this documentation** to reflect hover-only strategy

**Trade-offs:**
- ✅ Fastest workflow (hover-only)
- ❌ No fallback for intentional selection
- ❌ May be confusing if hover doesn't register

---

## Testing

### Manual Test Cases

#### Basic Functionality

1. **Hover + shortcut (MAIN USE CASE)**:
   - Hover over a node (wait for FloatingActionPanel to appear)
   - Press `Ctrl+B` WITHOUT clicking
   - ✅ Node should be pinned/unpinned
   - ✅ Console should log: `Keyboard shortcut triggered: PIN_SELECTED... on hovered node: <nodeId>`

2. **Selection + shortcut (FALLBACK)**:
   - Click a node (to select it)
   - Move mouse away (no hover)
   - Press `Ctrl+B`
   - ✅ Node should be pinned/unpinned
   - ✅ Console should log: `...on selected node: <nodeId>`

3. **Hover priority over selection**:
   - Click node A (to select it)
   - Hover over node B (different node)
   - Press `Ctrl+B`
   - ✅ Node B should be pinned/unpinned (hover takes priority)

4. **No target + shortcut**:
   - Deselect (press Escape)
   - Don't hover over any node
   - Press `Ctrl+B`
   - ✅ Nothing should happen

5. **VS Code conflict**:
   - Hover over a node
   - Press `Ctrl+B` (default Bold in VS Code)
   - ✅ Should pin node, NOT make text bold

#### Global Shortcuts (No Target Needed)

6. **Global shortcuts work anytime**:
   - Without selection or hover, press:
     - `Ctrl+F` → ✅ Search bar should focus
     - `Ctrl+I` → ✅ Focus mode should toggle
     - `Ctrl+0` → ✅ View should fit all nodes

#### Hover Timing

7. **Quick hover doesn't trigger**:
   - Quickly move mouse over a node (less than 150ms)
   - Press `Ctrl+B` immediately
   - ✅ Nothing should happen (hover delay not met)

8. **Panel hover persistence**:
   - Hover over node → panel appears
   - Move mouse onto the FloatingActionPanel
   - Press `Ctrl+B`
   - ✅ Should still work (panel keeps hover active)

#### All Shortcuts on Hover

9. **Test all node shortcuts via hover**:
   - Hover over a node, press:
     - `Ctrl+B` → ✅ Pin/Unpin
     - `Enter` → ✅ Expand/Collapse
     - `Ctrl+J` → ✅ Jump to Code (if node has file)
     - `Ctrl+H` → ✅ Hide Node

#### Custom Shortcuts

10. **Custom shortcut**:
    - Go to Settings → Keyboard Shortcuts
    - Change "Pin/Unpin" to `Ctrl+P`
    - Hover over a node, press `Ctrl+P`
    - ✅ Node should be pinned/unpinned

### Automated Tests (TODO)

```typescript
// Example test structure
describe('Keyboard Shortcuts', () => {
  it('should pin selected node on Ctrl+B', () => {
    // Select node
    // Simulate Ctrl+B keypress
    // Assert node is pinned
  });

  it('should not trigger without selection', () => {
    // Deselect all nodes
    // Simulate Ctrl+B keypress
    // Assert no changes
  });
});
```

---

## Architecture

```
User presses key
    ↓
handleKeyDown (capture phase)
    ↓
Check if node selected? → No → Ignore
    ↓ Yes
Match against shortcuts → No → Ignore
    ↓ Yes
event.preventDefault() (block VS Code)
    ↓
handleShortcutAction(id, nodeId)
    ↓
Execute action (pin, expand, etc.)
```

---

## Future Enhancements

- [ ] Add keyboard navigation (arrow keys to move selection)
- [ ] Add "Navigate to Parent/Child" implementations
- [ ] Add shortcut hints overlay (show available shortcuts)
- [ ] Add conflict detection (warn if shortcut conflicts with VS Code)
- [ ] Add keyboard shortcuts export/import
- [ ] Add global shortcuts (work without selection, e.g., Ctrl+0 for Fit View)

---

## Troubleshooting

### Shortcuts not working?

1. **Check if a node is selected** (highlighted with blue border)
2. **Check browser console** for logs: `[App.tsx] Keyboard shortcut triggered`
3. **Verify shortcut configuration** in Settings → Keyboard Shortcuts
4. **Try default shortcuts** first (reset to defaults in settings)

### VS Code shortcuts still triggering?

- This is expected if **no node is selected**
- Shortcuts only override VS Code when a node is active
- Make sure you clicked a node first

### Shortcut customization not saving?

- Check browser localStorage is enabled
- Check for errors in console
- Try "Reset All to Defaults" and reconfigure

---

## Related Files

- `App.tsx` - Main keyboard handler (~line 1534)
- `KeyboardShortcutsSettings.tsx` - Settings UI component
- `SettingsPanel.tsx` - Settings panel integration
- `LeftToolbar.tsx` - Toolbar integration

---

Last Updated: 2025-10-09
