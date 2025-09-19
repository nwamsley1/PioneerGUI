<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/tauri';
  import { open, save } from '@tauri-apps/api/dialog';
  import { listen } from '@tauri-apps/api/event';
  import JsonEditor from './lib/components/JsonEditor.svelte';
  import type {
    ConfigState,
    LoadConfigsResponse,
    RunMode,
    RunStartedPayload,
    ProgressPayload,
    LogPayload,
    RunCompletePayload,
    JsonValue
  } from './lib/types';
  import { collectPaths, deepMerge, setValue } from './lib/utils';
  import type { JsonPath } from './lib/utils';

  const clone = <T>(value: T): T => JSON.parse(JSON.stringify(value));

  const modes: RunMode[] = ['buildSpecLib', 'searchDia'];
  const modeLabels: Record<RunMode, string> = {
    buildSpecLib: 'BuildSpecLib',
    searchDia: 'SearchDIA'
  };

  type ProgressState = {
    running: boolean;
    mode: RunMode | null;
    stage: string;
    progress: number;
    logPath: string | null;
    configPath: string | null;
    terminalWarning: string | null;
    message: string | null;
  };

  let loading = true;
  let loadError: string | null = null;
  let configSource: LoadConfigsResponse['source'] = 'fallback';
  let binaryError: string | null = null;
  let activeTab: RunMode = 'buildSpecLib';
  let runError: string | null = null;
  let terminalWarning: string | null = null;

  let configStates: Record<RunMode, ConfigState | null> = {
    buildSpecLib: null,
    searchDia: null
  };

  let currentState: ConfigState | null = null;
  $: currentState = configStates[activeTab];

  let progressState: ProgressState = {
    running: false,
    mode: null,
    stage: '',
    progress: 0,
    logPath: null,
    configPath: null,
    terminalWarning: null,
    message: null
  };

  function updateProgress(partial: Partial<ProgressState>) {
    progressState = { ...progressState, ...partial };
  }

  let logBuffer: Array<{ mode: RunMode; stream: 'stdout' | 'stderr'; line: string }> = [];
  const maxLogEntries = 120;

  const listeners: Array<() => void> = [];

  function computeImportant(value: JsonValue): Set<string> {
    const rawPaths = collectPaths(value);
    const result = new Set<string>();
    for (const path of rawPaths) {
      if (!path) continue;
      const segments = path.split('.');
      for (let i = 1; i <= segments.length; i += 1) {
        result.add(segments.slice(0, i).join('.'));
      }
    }
    return result;
  }

  function initialiseState(response: LoadConfigsResponse) {
    configSource = response.source;
    binaryError = response.binary_error ?? null;

    const buildImportant = computeImportant(response.build.simplified_config);
    const searchImportant = computeImportant(response.search.simplified_config);
    const buildDefaults = clone(response.build.default_config);
    const buildCurrent = clone(response.build.persisted_config ?? response.build.default_config);
    const buildLastLoaded = response.build.persisted_config
      ? response.build.persisted_path ?? undefined
      : undefined;

    const searchDefaults = clone(response.search.default_config);
    const searchCurrent = clone(response.search.persisted_config ?? response.search.default_config);
    const searchLastLoaded = response.search.persisted_config
      ? response.search.persisted_path ?? undefined
      : undefined;

    configStates = {
      buildSpecLib: {
        defaults: buildDefaults,
        current: buildCurrent,
        important: buildImportant,
        source: response.source,
        lastLoadedPath: buildLastLoaded,
        persistedPath: response.build.persisted_path ?? undefined
      },
      searchDia: {
        defaults: searchDefaults,
        current: searchCurrent,
        important: searchImportant,
        source: response.source,
        lastLoadedPath: searchLastLoaded,
        persistedPath: response.search.persisted_path ?? undefined
      }
    };
  }

  onMount(async () => {
    try {
      const response = await invoke<LoadConfigsResponse>('load_configs');
      initialiseState(response);
      registerListeners();
    } catch (error) {
      loadError = error instanceof Error ? error.message : String(error);
    } finally {
      loading = false;
    }
  });

  function registerListeners() {
    listen<ProgressPayload>('pioneer-progress', (event) => {
      const payload = event.payload;
      if (progressState.mode && payload.mode !== progressState.mode) return;
      updateProgress({
        running: true,
        mode: payload.mode,
        stage: payload.stage_label,
        progress: payload.progress
      });
    }).then((unlisten) => listeners.push(unlisten));

    listen<LogPayload>('pioneer-log', (event) => {
      const payload = event.payload;
      if (progressState.mode && payload.mode !== progressState.mode) return;
      logBuffer = [...logBuffer, payload].slice(-maxLogEntries);
    }).then((unlisten) => listeners.push(unlisten));

    listen<string>('pioneer-terminal-warning', (event) => {
      terminalWarning = event.payload;
      updateProgress({ terminalWarning: event.payload });
    }).then((unlisten) => listeners.push(unlisten));

    listen<RunCompletePayload>('pioneer-run-complete', (event) => {
      const payload = event.payload;
      if (progressState.mode && payload.mode !== progressState.mode) return;
      const finalProgress = payload.success ? 100 : progressState.progress;
      updateProgress({
        running: false,
        stage: payload.success ? 'Completed' : 'Failed',
        progress: finalProgress,
        message: payload.message ?? (payload.success ? 'Pioneer completed successfully.' : 'Pioneer finished with an error.')
      });
      runError = payload.success ? null : payload.message ?? 'Run failed.';
    }).then((unlisten) => listeners.push(unlisten));
  }

  onDestroy(() => {
    listeners.forEach((unlisten) => unlisten());
  });

  function handleUpdate(mode: RunMode, path: JsonPath, value: JsonValue) {
    const state = configStates[mode];
    if (!state) return;
    const updated = { ...state, current: setValue(state.current, path, value) };
    configStates = { ...configStates, [mode]: updated };
    runError = null;
  }

  function forwardUpdate(mode: RunMode) {
    return (path: JsonPath, value: JsonValue) => handleUpdate(mode, path, value);
  }

  async function loadConfigFromFile(mode: RunMode) {
    try {
      const selected = await open({
        filters: [{ name: 'JSON', extensions: ['json'] }]
      });
      if (!selected || Array.isArray(selected)) return;
      const loaded = await invoke<JsonValue>('read_config', { path: selected });
      const state = configStates[mode];
      if (!state) return;
      const updated = { ...state, current: deepMerge(state.defaults, loaded), lastLoadedPath: selected };
      configStates = { ...configStates, [mode]: updated };
      runError = null;
    } catch (error) {
      runError = error instanceof Error ? error.message : String(error);
    }
  }

  async function saveConfigToFile(mode: RunMode) {
    const state = configStates[mode];
    if (!state) return;
    try {
      const filePath = await save({
        defaultPath: state.lastLoadedPath,
        filters: [{ name: 'JSON', extensions: ['json'] }]
      });
      if (!filePath) return;
      await invoke('save_config', { path: filePath, config: state.current });
      configStates = { ...configStates, [mode]: { ...state, lastLoadedPath: filePath } };
    } catch (error) {
      runError = error instanceof Error ? error.message : String(error);
    }
  }

  function resetToDefaults(mode: RunMode) {
    const state = configStates[mode];
    if (!state) return;
    const updated = { ...state, current: clone(state.defaults), lastLoadedPath: undefined };
    configStates = { ...configStates, [mode]: updated };
    runError = null;
  }

  async function runMode(mode: RunMode) {
    const state = configStates[mode];
    if (!state) return;
    updateProgress({
      running: true,
      mode,
      stage: 'Preparing to launch Pioneer…',
      progress: 0,
      logPath: null,
      configPath: null,
      message: null,
      terminalWarning: null
    });
    runError = null;
    terminalWarning = null;
    logBuffer = [];

    try {
      const payload = await invoke<RunStartedPayload>('run_pioneer', {
        request: {
          mode,
          config: state.current
        }
      });
      updateProgress({ logPath: payload.log_path, configPath: payload.config_path });
      if (payload.persisted_path) {
        const updatedState = configStates[mode];
        if (updatedState) {
          configStates = {
            ...configStates,
            [mode]: { ...updatedState, lastLoadedPath: payload.persisted_path, persistedPath: payload.persisted_path }
          };
        }
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      runError = message;
      updateProgress({ running: false, mode: null, message, stage: 'Failed' });
    }
  }

  function sourceDescription(source: LoadConfigsResponse['source']) {
    switch (source) {
      case 'binary':
        return 'Defaults generated from the Pioneer binary at runtime.';
      case 'partial':
        return 'Partial defaults loaded from Pioneer binary with repository fallbacks.';
      default:
        return 'Defaults loaded from the Pioneer.jl repository fallbacks.';
    }
  }
</script>

<main>
  <header>
    <div>
      <h1>Pioneer GUI</h1>
      <p class="subtitle">Configure Pioneer BuildSpecLib and SearchDIA workflows with synced defaults.</p>
      {#if loading}
        <p class="status">Loading configuration templates…</p>
      {:else if loadError}
        <p class="status error">{loadError}</p>
      {:else}
        <p class="status">{sourceDescription(configSource)}</p>
        {#if binaryError}
          <p class="status warning">{binaryError}</p>
        {/if}
      {/if}
    </div>
  </header>

  {#if !loading && !loadError}
    <section class="workspace">
      <nav class="tabs">
        {#each modes as mode}
          <button
            class:active={mode === activeTab}
            on:click={() => (activeTab = mode)}
            type="button"
          >
            {modeLabels[mode]}
          </button>
        {/each}
      </nav>

      {#if currentState}
        {#key activeTab}
          <section class="config-panel">
            <div class="panel-header">
              <div>
                <h2>{modeLabels[activeTab]} parameters</h2>
                <p class="hint">
                  Edit parameters directly or load an existing JSON file. Highlighted sections correspond to Pioneer’s simplified defaults.
                </p>
              </div>
              <div class="panel-actions">
                <button type="button" on:click={() => loadConfigFromFile(activeTab)}>Load JSON…</button>
                <button type="button" on:click={() => saveConfigToFile(activeTab)}>Save JSON…</button>
                <button type="button" on:click={() => resetToDefaults(activeTab)}>Reset</button>
                <button class="primary" type="button" on:click={() => runMode(activeTab)} disabled={progressState.running}
                  >Run {modeLabels[activeTab]}</button
                >
              </div>
            </div>

            <div class="editor">
              <JsonEditor
                label={modeLabels[activeTab]}
                value={currentState.current}
                path={[]}
                depth={0}
                important={currentState.important}
                onUpdate={forwardUpdate(activeTab)}
              />
            </div>
          </section>
        {/key}
      {/if}

      <section class="status-panel">
        <h3>Run status</h3>
        {#if progressState.running}
          <div class="progress">
            <div class="stage">{progressState.stage}</div>
            <div class="bar">
              <div class="fill" style={`width: ${Math.min(100, progressState.progress)}%`}></div>
            </div>
            <div class="details">
              <span>{Math.round(progressState.progress)}%</span>
              {#if progressState.logPath}
                <span>Log: {progressState.logPath}</span>
              {/if}
            </div>
          </div>
        {:else if progressState.message}
          <p class="status" class:success={runError === null} class:error={runError !== null}>{progressState.message}</p>
        {:else}
          <p>No run in progress.</p>
        {/if}

        {#if terminalWarning}
          <p class="status warning">{terminalWarning}</p>
        {/if}

        {#if runError}
          <p class="status error">{runError}</p>
        {/if}
        {#if currentState?.persistedPath}
          <p class="status info">Stored defaults: {currentState.persistedPath}</p>
        {/if}
        {#if logBuffer.length > 0}
          <div class="log-preview">
            <h4>Recent Pioneer output</h4>
            <pre>{logBuffer.map((entry) => `[${entry.stream}] ${entry.line}`).join('\n')}</pre>
          </div>
        {/if}
      </section>
    </section>
  {/if}
</main>

<style>
  main {
    padding: 32px;
    max-width: 1280px;
    margin: 0 auto;
  }

  header {
    background: rgba(255, 255, 255, 0.9);
    border-radius: 16px;
    padding: 24px 32px;
    margin-bottom: 24px;
    box-shadow: 0 12px 32px rgba(15, 23, 42, 0.08);
  }

  h1 {
    margin: 0;
    font-size: 2rem;
    color: #0f172a;
  }

  .subtitle {
    margin: 4px 0 12px;
    color: #334155;
    font-size: 1rem;
  }

  .status {
    margin: 0;
    font-size: 0.95rem;
    color: #1e293b;
  }

  .status.warning {
    color: #b45309;
  }

  .status.error {
    color: #dc2626;
  }

  .status.success {
    color: #047857;
  }
  .status.info {
    color: #0369a1;
  }
  .workspace {
    display: grid;
    grid-template-columns: 2fr 1fr;
    gap: 24px;
    align-items: start;
  }

  .tabs {
    grid-column: 1 / -1;
    display: inline-flex;
    background: rgba(15, 23, 42, 0.08);
    padding: 4px;
    border-radius: 12px;
    margin-bottom: 12px;
  }

  .tabs button {
    border: none;
    background: transparent;
    padding: 10px 18px;
    border-radius: 10px;
    font-size: 0.95rem;
    color: #1f2937;
    cursor: pointer;
    transition: background 0.2s ease, color 0.2s ease;
  }

  .tabs button.active {
    background: white;
    color: #1d4ed8;
    box-shadow: 0 4px 12px rgba(37, 99, 235, 0.2);
  }

  .tabs button:hover:not(.active) {
    background: rgba(255, 255, 255, 0.6);
  }

  .config-panel {
    background: rgba(255, 255, 255, 0.9);
    border-radius: 16px;
    padding: 24px;
    box-shadow: 0 12px 32px rgba(15, 23, 42, 0.08);
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 16px;
    margin-bottom: 18px;
  }

  .panel-actions {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  button {
    border: none;
    background: rgba(37, 99, 235, 0.1);
    color: #1d4ed8;
    padding: 10px 16px;
    border-radius: 10px;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.2s ease, transform 0.1s ease;
  }

  button:hover {
    background: rgba(37, 99, 235, 0.2);
  }

  button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  button.primary {
    background: #1d4ed8;
    color: white;
  }

  button.primary:hover {
    background: #1e40af;
  }

  .hint {
    margin: 0;
    color: #475569;
    font-size: 0.9rem;
  }

  .editor {
    max-height: 70vh;
    overflow-y: auto;
    padding-right: 12px;
  }

  .status-panel {
    background: rgba(255, 255, 255, 0.9);
    border-radius: 16px;
    padding: 24px;
    box-shadow: 0 12px 32px rgba(15, 23, 42, 0.08);
    position: sticky;
    top: 24px;
  }

  .progress {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .bar {
    height: 10px;
    border-radius: 999px;
    background: rgba(15, 23, 42, 0.1);
    overflow: hidden;
  }

  .fill {
    height: 100%;
    background: linear-gradient(90deg, #60a5fa, #2563eb);
    border-radius: 999px;
  }

  .stage {
    font-weight: 600;
    color: #0f172a;
  }

  .details {
    display: flex;
    justify-content: space-between;
    font-size: 0.85rem;
    color: #475569;
  }

  .log-preview {
    margin-top: 16px;
  }

  .log-preview pre {
    background: rgba(15, 23, 42, 0.9);
    color: #f8fafc;
    padding: 12px;
    border-radius: 10px;
    max-height: 200px;
    overflow-y: auto;
    font-size: 0.85rem;
  }

  @media (max-width: 1024px) {
    .workspace {
      grid-template-columns: 1fr;
    }

    .status-panel {
      position: static;
    }
  }
</style>
