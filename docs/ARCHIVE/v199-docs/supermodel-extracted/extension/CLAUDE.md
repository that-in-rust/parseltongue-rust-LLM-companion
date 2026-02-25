# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Build and Development
- `npm run build` - Full build (prepares API client + builds extension)
- `npm run build-extension` - Build extension only (uses esbuild)
- `npm run prepare-api-client` - Generate API client from OpenAPI spec
- `npm run clean` - Clean build outputs
- `npm run clean:all` - Clean all outputs including API client

### Testing
- `npm run test` - Run Jest unit tests with coverage
- `npm run test:vscode` - Run VS Code integration tests
- `npm run test:all` - Run both unit and integration tests
- `npm run compile:tests` - Compile test TypeScript files

### Code Quality
- `npm run lint` - Run ESLint on source files

### Packaging
- `npm run package:local` - Package for local development
- `npm run package` - Package for Azure production

## Architecture Overview

This is a VS Code extension that visualizes codebases as interactive graphs. It consists of two main parts:

### Extension Host (`src/extension/`)
- **Main entry**: `extension.ts` - Registers commands, webview, and authentication
- **Commands**: User-facing commands like process workspace, login/logout, classify code
- **Services**: Core business logic including upload, graph data, auth, and backend API communication
- **Auth**: OAuth integration via `SupermodelAuthProvider` for GitHub authentication
- **State Management**: Uses XState machines for backend processing workflow

### Webview (`src/webview/`)
- **React application** that renders inside VS Code webview panel
- **D3-based visualization**:
  - `D3ViewHost` - Main component using custom D3 force-directed layout
- **State Management**: XState machines for view state and graph interactions
- **Communication**: Message passing between webview and extension host

### Key Data Flow
1. User runs "Process Workspace" command
2. Extension scans workspace files (based on include/exclude globs)
3. Files uploaded to backend API for analysis
4. Backend returns graph data (nodes/edges representing code structure)
5. Webview renders interactive visualization
6. User can explore, select nodes, and view properties

### Configuration
- Environment switching (local, azure-prod, custom)
- File inclusion/exclusion patterns via VS Code settings
- Mock mode for offline development
- OpenAI key integration for code classification

### Technology Stack
- **Frontend**: React, TypeScript, D3.js, XState
- **Backend Communication**: Axios, REST APIs
- **Build**: esbuild, TypeScript, Tailwind CSS
- **Testing**: Jest, VS Code Test Framework

### Key Files to Understand
- `src/extension/extension.ts` - Extension activation and command registration
- `src/webview/components/App.tsx` - Main webview React component
- `src/extension/services/uploadService.ts` - Core file upload and processing logic
- `src/extension/services/messageHandler.ts` - Webview-extension communication
- `src/shared/messageTypes.ts` - Message interface definitions
- `package.json` - Extension manifest and configuration