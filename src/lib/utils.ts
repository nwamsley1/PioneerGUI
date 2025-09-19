import type { JsonValue } from './types';

export type JsonPath = Array<string | number>;

export function getValue(root: JsonValue, path: JsonPath): JsonValue | undefined {
  return path.reduce<JsonValue | undefined>((acc, key) => {
    if (acc === undefined) {
      return undefined;
    }
    if (acc === null || typeof acc === 'boolean' || typeof acc === 'number' || typeof acc === 'string') {
      return acc;
    }
    if (Array.isArray(acc) && typeof key === 'number') {
      return acc[key];
    }
    if (!Array.isArray(acc) && typeof key === 'string') {
      return (acc as Record<string, JsonValue>)[key];
    }
    return undefined;
  }, root);
}

export function setValue(root: JsonValue, path: JsonPath, value: JsonValue): JsonValue {
  if (path.length === 0) {
    return value;
  }
  const [head, ...rest] = path;
  if (Array.isArray(root)) {
    if (typeof head !== 'number') return root;
    const clone = [...root];
    clone[head] = setValue(clone[head], rest, value);
    return clone;
  }
  if (root === null || typeof root !== 'object') {
    return root;
  }
  const clone: Record<string, JsonValue> = { ...(root as Record<string, JsonValue>) };
  if (rest.length === 0) {
    clone[head as string] = value;
  } else {
    clone[head as string] = setValue(clone[head as string], rest, value);
  }
  return clone;
}

export function deleteKey(root: JsonValue, path: JsonPath): JsonValue {
  if (path.length === 0) {
    return root;
  }
  const [head, ...rest] = path;
  if (Array.isArray(root)) {
    if (typeof head !== 'number') return root;
    const clone = [...root];
    if (rest.length === 0) {
      clone.splice(head, 1);
    } else {
      clone[head] = deleteKey(clone[head], rest);
    }
    return clone;
  }
  if (root === null || typeof root !== 'object') {
    return root;
  }
  const clone: Record<string, JsonValue> = { ...(root as Record<string, JsonValue>) };
  if (rest.length === 0) {
    delete clone[head as string];
  } else {
    clone[head as string] = deleteKey(clone[head as string], rest);
  }
  return clone;
}

export function collectPaths(value: JsonValue, prefix: string[] = []): string[] {
  if (value === null || typeof value === 'boolean' || typeof value === 'number' || typeof value === 'string') {
    return [prefix.join('.')];
  }
  if (Array.isArray(value)) {
    return value.flatMap((item, index) => collectPaths(item, [...prefix, index.toString()]));
  }
  return Object.entries(value as Record<string, JsonValue>).flatMap(([key, child]) =>
    collectPaths(child, [...prefix, key])
  );
}

export function deepMerge(base: JsonValue, override: JsonValue): JsonValue {
  if (override === null || typeof override !== 'object' || Array.isArray(override)) {
    return override;
  }
  if (base === null || typeof base !== 'object' || Array.isArray(base)) {
    return { ...(override as Record<string, JsonValue>) };
  }
  const result: Record<string, JsonValue> = { ...(base as Record<string, JsonValue>) };
  for (const [key, value] of Object.entries(override as Record<string, JsonValue>)) {
    const baseValue = (base as Record<string, JsonValue>)[key];
    result[key] = deepMerge(baseValue, value);
  }
  return result;
}

export function toDisplayPath(path: string[]): string {
  return path.join(' › ');
}

export function stringifyValue(value: JsonValue): string {
  if (Array.isArray(value)) {
    return value.map((item) => stringifyValue(item)).join(', ');
  }
  if (value === null) return 'null';
  if (typeof value === 'object') return JSON.stringify(value, null, 2);
  return String(value);
}
