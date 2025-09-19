<script lang="ts">
  import type { JsonValue } from '../types';
  import type { JsonPath } from '../utils';
  import { stringifyValue } from '../utils';

  export let label: string;
  export let value: JsonValue;
  export let path: JsonPath = [];
  export let depth = 0;
  export let important: Set<string>;
  export let onUpdate: (path: JsonPath, value: JsonValue) => void;

  const pathKey = path.map(String).join('.');
  const isImportant = important.has(pathKey);

  const indent = depth * 16;

  const isPrimitive = (val: JsonValue) =>
    val === null || ['string', 'number', 'boolean'].includes(typeof val);

  function handleBooleanChange(event: Event) {
    const target = event.target as HTMLInputElement;
    onUpdate(path, target.checked);
  }

  function handleNumberChange(event: Event) {
    const target = event.target as HTMLInputElement;
    const next = target.value === '' ? null : Number(target.value);
    onUpdate(path, next as JsonValue);
  }

  function handleStringChange(event: Event) {
    const target = event.target as HTMLInputElement | HTMLTextAreaElement;
    onUpdate(path, target.value);
  }

  $: isArrayValue = Array.isArray(value);
  $: arrayHasOnlyPrimitives = isArrayValue && (value as JsonValue[]).every((item) => isPrimitive(item));
  $: arrayItems = isArrayValue ? (value as JsonValue[]) : [];
  $: arrayText = isArrayValue
    ? arrayItems
        .map((item) => (typeof item === 'object' ? JSON.stringify(item) : stringifyValue(item)))
        .join('\n')
    : '';
  $: arrayRows = arrayText.length === 0 ? 3 : Math.min(8, Math.max(3, arrayText.split('\n').length));

  function commitArrayChanges(event: Event) {
    const target = event.target as HTMLTextAreaElement;
    const lines = target.value
      .split('\n')
      .map((line) => line.trim())
      .filter((line) => line.length > 0);

    const parsed: JsonValue[] = lines.map((line) => {
      if (line === 'true' || line === 'false') return line === 'true';
      const numeric = Number(line);
      if (!Number.isNaN(numeric)) {
        return numeric;
      }
      try {
        return JSON.parse(line);
      } catch (err) {
        return line;
      }
    });
    onUpdate(path, parsed as JsonValue);
  }
</script>

<div class="json-editor" class:important={isImportant} style={`margin-left: ${indent}px`}>
  <div class="label-row">
    <div class="label">{label}</div>
    {#if isImportant}
      <span class="badge">Key parameter</span>
    {/if}
  </div>

  {#if value === null}
    <input type="text" value="null" readonly />
  {:else if typeof value === 'boolean'}
    <label class="toggle">
      <input type="checkbox" checked={value} on:change={handleBooleanChange} />
      <span>{value ? 'Enabled' : 'Disabled'}</span>
    </label>
  {:else if typeof value === 'number'}
    <input type="number" step="any" value={value} on:change={handleNumberChange} />
  {:else if typeof value === 'string'}
    <input type="text" value={value} on:input={handleStringChange} />
  {:else if isArrayValue}
    {#if arrayHasOnlyPrimitives}
      <textarea rows={arrayRows} bind:value={arrayText} on:change={commitArrayChanges} />
      <small>Enter one value per line. Numbers and booleans are detected automatically.</small>
    {:else}
      <div class="child-entries">
        {#each arrayItems as item, index}
          <svelte:self
            label={`[${index}]`}
            {important}
            value={item}
            path={[...path, index]}
            depth={depth + 1}
            onUpdate={onUpdate}
          />
        {/each}
      </div>
    {/if}
  {:else}
    <div class="child-entries">
      {#each Object.entries(value) as [key, child]}
        <svelte:self
          label={key}
          value={child}
          path={[...path, key]}
          depth={depth + 1}
          {important}
          onUpdate={onUpdate}
        />
      {/each}
    </div>
  {/if}
</div>

<style>
  .json-editor {
    background: rgba(255, 255, 255, 0.85);
    border: 1px solid rgba(15, 23, 42, 0.08);
    border-radius: 12px;
    padding: 16px;
    margin-bottom: 12px;
    box-shadow: 0 4px 8px rgba(15, 23, 42, 0.05);
    transition: border-color 0.2s ease;
  }

  .json-editor.important {
    border-color: #2563eb;
    box-shadow: 0 8px 16px rgba(37, 99, 235, 0.15);
  }

  .label-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 8px;
    gap: 12px;
  }

  .label {
    font-weight: 600;
    font-size: 0.95rem;
    color: #0f172a;
  }

  .badge {
    background: #2563eb;
    color: white;
    border-radius: 9999px;
    padding: 2px 10px;
    font-size: 0.75rem;
    letter-spacing: 0.02em;
  }

  input[type='text'],
  input[type='number'],
  textarea {
    width: 100%;
    padding: 10px 12px;
    border: 1px solid rgba(15, 23, 42, 0.16);
    border-radius: 8px;
    font-size: 0.95rem;
    background: white;
    color: #0f172a;
    transition: border 0.2s ease;
  }

  textarea {
    font-family: 'JetBrains Mono', 'Fira Code', 'SFMono-Regular', ui-monospace, monospace;
    resize: vertical;
  }

  input:focus,
  textarea:focus {
    border-color: #2563eb;
    outline: none;
    box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.15);
  }

  .toggle {
    display: flex;
    align-items: center;
    gap: 8px;
    color: #0f172a;
    font-weight: 500;
  }

  .toggle input {
    transform: scale(1.1);
  }

  .child-entries {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  small {
    display: block;
    margin-top: 8px;
    color: #475569;
  }
</style>
