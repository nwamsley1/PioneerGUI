export type JsonValue = string | number | boolean | null | JsonValue[] | { [key: string]: JsonValue };

export interface ConfigSet {
  default_config: JsonValue;
  simplified_config: JsonValue;
  persisted_config?: JsonValue;
  persisted_path?: string | null;
}

export type ConfigSource = 'binary' | 'partial' | 'fallback';

export interface LoadConfigsResponse {
  build: ConfigSet;
  search: ConfigSet;
  source: ConfigSource;
  binary_error?: string | null;
}

export type RunMode = 'buildSpecLib' | 'searchDia';

export interface RunStartedPayload {
  mode: RunMode;
  log_path: string;
  config_path: string;
  persisted_path?: string | null;
}

export interface ProgressPayload {
  mode: RunMode;
  stage_key: string;
  stage_label: string;
  progress: number;
}

export interface LogPayload {
  mode: RunMode;
  stream: 'stdout' | 'stderr';
  line: string;
}

export interface RunCompletePayload {
  mode: RunMode;
  success: boolean;
  exit_code: number | null;
  message?: string | null;
}

export interface ConfigState {
  defaults: JsonValue;
  current: JsonValue;
  important: Set<string>;
  source: ConfigSource;
  lastLoadedPath?: string;
  persistedPath?: string;
}
