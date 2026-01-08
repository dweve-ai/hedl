/**
 * React Integration Example for hedl-wasm
 *
 * This example demonstrates how to integrate hedl-wasm into a React application
 * with proper error handling, loading states, and TypeScript support.
 *
 * Features:
 * - WASM initialization on mount
 * - Real-time HEDL validation
 * - Live JSON preview
 * - Error display
 * - Token statistics
 * - Copy to clipboard
 */

import React, { useState, useEffect, useCallback, useRef } from 'react';
import init, {
  parse,
  validate,
  format,
  getStats,
  version
} from 'hedl-wasm';

/**
 * Main HEDL Editor Component
 */
function HedlEditor() {
  const [initialized, setInitialized] = useState(false);
  const [initError, setInitError] = useState(null);
  const [hedl, setHedl] = useState(DEFAULT_HEDL);
  const [json, setJson] = useState('');
  const [error, setError] = useState(null);
  const [warnings, setWarnings] = useState([]);
  const [stats, setStats] = useState(null);
  const [copied, setCopied] = useState(false);

  // Debounce timer
  const debounceTimer = useRef(null);

  // Initialize WASM on mount
  useEffect(() => {
    async function initializeWasm() {
      try {
        await init();
        setInitialized(true);
        console.log(`HEDL WASM v${version()} initialized`);
      } catch (e) {
        setInitError(e.message);
        console.error('Failed to initialize WASM:', e);
      }
    }

    initializeWasm();
  }, []);

  // Convert HEDL to JSON with validation
  const convertToJson = useCallback((hedlInput) => {
    if (!initialized) return;

    try {
      // Validate first
      const validation = validate(hedlInput, true);

      if (validation.valid) {
        // Parse and convert
        const doc = parse(hedlInput);
        const jsonStr = doc.toJsonString(true);
        setJson(jsonStr);
        setError(null);

        // Get statistics
        const tokenStats = getStats(hedlInput);
        setStats({
          schemas: doc.schemaCount,
          entities: Object.values(doc.countEntities()).reduce((a, b) => a + b, 0),
          hedlTokens: tokenStats.hedlTokens,
          jsonTokens: tokenStats.jsonTokens,
          savingsPercent: tokenStats.savingsPercent
        });

        // Set warnings
        setWarnings(validation.warnings.map(w => ({
          line: w.line,
          message: w.message,
          rule: w.rule
        })));
      } else {
        // Validation failed
        setError({
          type: 'validation',
          errors: validation.errors.map(e => ({
            line: e.line,
            message: e.message,
            errorType: e.type
          }))
        });
        setJson('');
        setStats(null);
      }
    } catch (e) {
      // Parse error
      const match = e.message.match(/line (\d+)/i);
      setError({
        type: 'parse',
        message: e.message,
        line: match ? parseInt(match[1]) : null
      });
      setJson('');
      setStats(null);
    }
  }, [initialized]);

  // Debounced conversion on input change
  useEffect(() => {
    if (!initialized) return;

    clearTimeout(debounceTimer.current);
    debounceTimer.current = setTimeout(() => {
      convertToJson(hedl);
    }, 500);

    return () => clearTimeout(debounceTimer.current);
  }, [hedl, initialized, convertToJson]);

  // Format HEDL
  const handleFormat = useCallback(() => {
    if (!initialized) return;

    try {
      const formatted = format(hedl, true);
      setHedl(formatted);
    } catch (e) {
      setError({
        type: 'format',
        message: e.message
      });
    }
  }, [hedl, initialized]);

  // Copy JSON to clipboard
  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(json);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (e) {
      console.error('Failed to copy:', e);
    }
  }, [json]);

  // Loading state
  if (initError) {
    return (
      <div className="hedl-editor error-container">
        <h2>Initialization Error</h2>
        <p>{initError}</p>
      </div>
    );
  }

  if (!initialized) {
    return (
      <div className="hedl-editor loading-container">
        <div className="spinner"></div>
        <p>Loading HEDL WASM...</p>
      </div>
    );
  }

  // Main render
  return (
    <div className="hedl-editor">
      <Header version={version()} />

      <Toolbar
        onFormat={handleFormat}
        onCopy={handleCopy}
        copied={copied}
        hasJson={!!json}
      />

      <div className="editor-container">
        <EditorPane
          title="HEDL Input"
          value={hedl}
          onChange={setHedl}
          error={error}
          warnings={warnings}
        />

        <OutputPane
          title="JSON Output"
          value={json}
          error={error}
        />
      </div>

      {stats && <StatsPanel stats={stats} />}
    </div>
  );
}

/**
 * Header Component
 */
function Header({ version }) {
  return (
    <div className="header">
      <h1>HEDL Editor</h1>
      <span className="version">v{version}</span>
    </div>
  );
}

/**
 * Toolbar Component
 */
function Toolbar({ onFormat, onCopy, copied, hasJson }) {
  return (
    <div className="toolbar">
      <button onClick={onFormat} className="btn-primary">
        Format HEDL
      </button>
      <button
        onClick={onCopy}
        className="btn-secondary"
        disabled={!hasJson}
      >
        {copied ? 'Copied!' : 'Copy JSON'}
      </button>
    </div>
  );
}

/**
 * Editor Pane Component
 */
function EditorPane({ title, value, onChange, error, warnings }) {
  return (
    <div className="pane">
      <div className="pane-header">
        <h2>{title}</h2>
        {error && <span className="badge error">Error</span>}
        {!error && warnings.length > 0 && (
          <span className="badge warning">{warnings.length} warnings</span>
        )}
      </div>

      <textarea
        className="editor-textarea"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        spellCheck={false}
      />

      {error && <ErrorDisplay error={error} />}
      {!error && warnings.length > 0 && <WarningsDisplay warnings={warnings} />}
    </div>
  );
}

/**
 * Output Pane Component
 */
function OutputPane({ title, value, error }) {
  return (
    <div className="pane">
      <div className="pane-header">
        <h2>{title}</h2>
        {value && <span className="badge success">Valid</span>}
      </div>

      <pre className="output-pre">
        {value || (error ? 'Fix errors to see JSON output' : 'Waiting for input...')}
      </pre>
    </div>
  );
}

/**
 * Error Display Component
 */
function ErrorDisplay({ error }) {
  if (error.type === 'validation') {
    return (
      <div className="error-panel">
        <strong>Validation Errors:</strong>
        {error.errors.map((e, i) => (
          <div key={i} className="error-item">
            Line {e.line}: {e.message} ({e.errorType})
          </div>
        ))}
      </div>
    );
  }

  return (
    <div className="error-panel">
      <strong>Parse Error:</strong>
      <div className="error-item">
        {error.line && `Line ${error.line}: `}
        {error.message}
      </div>
    </div>
  );
}

/**
 * Warnings Display Component
 */
function WarningsDisplay({ warnings }) {
  return (
    <div className="warning-panel">
      <strong>Warnings:</strong>
      {warnings.map((w, i) => (
        <div key={i} className="warning-item">
          Line {w.line}: {w.message} [{w.rule}]
        </div>
      ))}
    </div>
  );
}

/**
 * Statistics Panel Component
 */
function StatsPanel({ stats }) {
  return (
    <div className="stats-panel">
      <div className="stat">
        <div className="stat-value">{stats.schemas}</div>
        <div className="stat-label">Schemas</div>
      </div>
      <div className="stat">
        <div className="stat-value">{stats.entities}</div>
        <div className="stat-label">Entities</div>
      </div>
      <div className="stat">
        <div className="stat-value">{stats.hedlTokens}</div>
        <div className="stat-label">HEDL Tokens</div>
      </div>
      <div className="stat">
        <div className="stat-value">{stats.jsonTokens}</div>
        <div className="stat-label">JSON Tokens</div>
      </div>
      <div className="stat">
        <div className="stat-value">{stats.savingsPercent}%</div>
        <div className="stat-label">Savings</div>
      </div>
    </div>
  );
}

/**
 * Default HEDL Content
 */
const DEFAULT_HEDL = `%VERSION: 1.0
%STRUCT: User[id,name,email]
%STRUCT: Post[id,title,views]
%NEST: User > Post
---
users: @User
  |alice,Alice Smith,alice@example.com
    |post1,Introduction to HEDL,1250
    |post2,Advanced Features,890
  |bob,Bob Jones,bob@example.com
    |post3,Getting Started,450`;

/**
 * CSS Styles (can be in separate .css file)
 */
const styles = `
.hedl-editor {
  display: flex;
  flex-direction: column;
  height: 100vh;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 15px 20px;
  background: #2d2d2d;
  color: white;
  border-bottom: 1px solid #444;
}

.header h1 {
  margin: 0;
  font-size: 20px;
}

.version {
  font-size: 12px;
  color: #888;
}

.toolbar {
  display: flex;
  gap: 10px;
  padding: 10px 20px;
  background: #f5f5f5;
  border-bottom: 1px solid #ddd;
}

.btn-primary, .btn-secondary {
  padding: 8px 16px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
}

.btn-primary {
  background: #007bff;
  color: white;
}

.btn-primary:hover {
  background: #0056b3;
}

.btn-secondary {
  background: #6c757d;
  color: white;
}

.btn-secondary:hover {
  background: #5a6268;
}

.btn-secondary:disabled {
  background: #ccc;
  cursor: not-allowed;
}

.editor-container {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1px;
  flex: 1;
  background: #ddd;
  overflow: hidden;
}

.pane {
  background: white;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.pane-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 15px;
  background: #f8f8f8;
  border-bottom: 1px solid #ddd;
}

.pane-header h2 {
  margin: 0;
  font-size: 14px;
  color: #555;
}

.badge {
  padding: 2px 8px;
  border-radius: 10px;
  font-size: 11px;
  font-weight: bold;
}

.badge.error {
  background: #dc3545;
  color: white;
}

.badge.warning {
  background: #ffc107;
  color: #333;
}

.badge.success {
  background: #28a745;
  color: white;
}

.editor-textarea {
  flex: 1;
  padding: 15px;
  border: none;
  font-family: 'Courier New', monospace;
  font-size: 14px;
  resize: none;
  outline: none;
}

.output-pre {
  flex: 1;
  padding: 15px;
  margin: 0;
  overflow: auto;
  font-family: 'Courier New', monospace;
  font-size: 14px;
  background: #f8f8f8;
}

.error-panel, .warning-panel {
  padding: 10px 15px;
  border-top: 1px solid #ddd;
  background: #fff3cd;
  max-height: 150px;
  overflow-y: auto;
}

.error-panel {
  background: #f8d7da;
  border-top-color: #f5c6cb;
}

.error-item, .warning-item {
  padding: 5px 0;
  font-family: 'Courier New', monospace;
  font-size: 13px;
}

.stats-panel {
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: 15px;
  padding: 15px 20px;
  background: #f8f8f8;
  border-top: 1px solid #ddd;
}

.stat {
  text-align: center;
}

.stat-value {
  font-size: 24px;
  font-weight: bold;
  color: #007bff;
}

.stat-label {
  font-size: 12px;
  color: #666;
  margin-top: 4px;
}

.loading-container, .error-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100vh;
  gap: 20px;
}

.spinner {
  width: 50px;
  height: 50px;
  border: 4px solid #f3f3f3;
  border-top: 4px solid #007bff;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
`;

export default HedlEditor;

/**
 * Usage in your app:
 *
 * import HedlEditor from './HedlEditor';
 *
 * function App() {
 *   return <HedlEditor />;
 * }
 */
