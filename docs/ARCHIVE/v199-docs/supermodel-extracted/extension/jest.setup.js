// jest.setup.js
/* global jest, beforeEach */ // Add beforeEach back for linter, guarded below

// --- Setup testing-library matchers ---
require('@testing-library/jest-dom');

// --- Mock acquireVsCodeApi for webview tests ---
global.acquireVsCodeApi = () => ({
  getState: () => ({}),
  setState: () => {},
  postMessage: () => {},
});

// --- Mock relevant parts of the 'vscode' module for extension unit tests ---

// Store mock implementations separate from the jest.mock call for clarity and potential reuse/reset
const mockVSCode = {
  authentication: {
    getSession: jest.fn(),
    onDidChangeSessions: jest.fn(() => ({ dispose: jest.fn() })), // Return a disposable
    // registerAuthenticationProvider is usually called in activate, less common to mock directly in unit tests unless testing activate itself
    registerAuthenticationProvider: jest.fn(() => ({ dispose: jest.fn() })),
  },
  commands: {
    registerCommand: jest.fn(() => ({ dispose: jest.fn() })),
    executeCommand: jest.fn(),
  },
  env: {
    openExternal: jest.fn(),
    // Add other env properties if needed
  },
  ExtensionContext: {
    // Mock context properties needed by tests (adjust as needed)
    subscriptions: [],
    globalState: {
      get: jest.fn(),
      update: jest.fn(),
      keys: jest.fn(() => []), // Return empty array for keys()
    },
    secrets: {
      get: jest.fn(),
      store: jest.fn(),
      delete: jest.fn(),
      onDidChange: jest.fn(() => ({ dispose: jest.fn() })),
    },
    // Add other context properties if needed (logUri, extensionPath, etc.)
    logUri: { fsPath: "/mock/logs" }, // Example
    extensionPath: { fsPath: "/mock/extension" }, // Example
  },
  ProgressLocation: {
    Notification: 15,
  },
  StatusBarAlignment: {
    Left: 1,
    Right: 2,
  },
  Uri: {
    parse: jest.fn((value) => ({
      toString: () => value,
      fsPath: value.startsWith("file://") ? value.substring(7) : value,
    })),
    file: jest.fn((path) => ({
      toString: () => `file://${path}`,
      fsPath: path,
    })),
  },
  window: {
    createStatusBarItem: jest.fn(() => ({
      show: jest.fn(),
      hide: jest.fn(),
      dispose: jest.fn(),
      // Mock properties often set:
      text: "",
      tooltip: "",
      command: "",
    })),
    createWebviewPanel: jest.fn(),
    registerUriHandler: jest.fn(() => ({ dispose: jest.fn() })),
    registerWebviewViewProvider: jest.fn(() => ({ dispose: jest.fn() })),
    showInformationMessage: jest.fn(),
    showWarningMessage: jest.fn(),
    showErrorMessage: jest.fn(),
    withProgress: jest.fn(),
    showInputBox: jest.fn(),
  },
  workspace: {
    getConfiguration: jest.fn(() => ({
      get: jest.fn(),
      update: jest.fn(),
      // Add other configuration methods if needed
    })),
    findFiles: jest.fn(),
    fs: {
      readFile: jest.fn(),
      writeFile: jest.fn(),
      // Add other fs methods if needed
    },
    getWorkspaceFolder: jest.fn(),
    workspaceFolders: [], // Default to no workspace folders
    // Add other workspace properties/methods if needed
  },
  // Add other top-level vscode namespaces if required by tests
};

// Apply the mock
jest.mock("vscode", () => mockVSCode, { virtual: true });

// Optional: Add helper functions to reset mocks before each test if needed
if (typeof beforeEach === "function") {
  // Reset mocks before each test (only when Jest globals are available)
  beforeEach(() => {
    // Reset all mocks defined in mockVSCode
    Object.values(mockVSCode).forEach((namespace) => {
      if (typeof namespace === "object" && namespace !== null) {
        Object.values(namespace).forEach((mockFn) => {
          if (jest.isMockFunction(mockFn)) {
            mockFn.mockClear();
          }
        });
      }
    });
    // Reset specific mocks with default implementations if necessary
    // mockVSCode.workspace.getConfiguration().get.mockReturnValue(undefined);
    // mockVSCode.workspace.workspaceFolders = [];

    // Reset specific mocks including the new one
    if (mockVSCode.window && mockVSCode.window.showInputBox) {
      mockVSCode.window.showInputBox.mockClear();
    }
    if (mockVSCode.secrets) {
      mockVSCode.secrets.get && mockVSCode.secrets.get.mockClear && mockVSCode.secrets.get.mockClear();
      mockVSCode.secrets.store && mockVSCode.secrets.store.mockClear && mockVSCode.secrets.store.mockClear();
      mockVSCode.secrets.delete && mockVSCode.secrets.delete.mockClear && mockVSCode.secrets.delete.mockClear();
    }
    if (mockVSCode.authentication && mockVSCode.authentication.getSession) {
      mockVSCode.authentication.getSession.mockClear();
    }
  });

  // you can add afterEach/afterAll guards the same way if ever needed
}
