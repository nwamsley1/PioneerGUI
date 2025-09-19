# Pioneer GUI

Pioneer GUI is a cross-platform desktop application built with [Tauri](https://tauri.app/) and [Svelte](https://svelte.dev/) for configuring and launching the [`Pioneer.jl`](https://github.com/nwamsley1/Pioneer.jl) pipelines. It surfaces the two major Pioneer workflows—`BuildSpecLib` (library prediction) and `SearchDIA` (DIA analysis)—through dedicated tabs that render the full parameter schema directly from Pioneer’s helper binaries. When the binaries are unavailable the GUI transparently falls back to the JSON templates published in the Pioneer repository so you can still explore the layout.

> **Key idea:** the application always attempts to call the Pioneer helper commands (`params-predict` and `params-search`) at runtime to build the latest parameter templates. Any changes that land in Pioneer will automatically appear in the GUI without manual updates.

---

## Features

- **Runtime parameter discovery** – Generates default parameter structures by calling the installed Pioneer CLI; automatically falls back to the repository templates if the binary is missing or returns an error.
- **Configurable editors** – Presents the entire JSON structure with grouped sections and inline editing for numbers, booleans, strings, and primitive arrays. Important/simplified parameters are highlighted using Pioneer’s curated “simplified” configs.
- **Per-workflow tabs** – Separate views for `BuildSpecLib` and `SearchDIA`, each with its own load/save/reset controls and execution button.
- **External terminal integration** – Launches Pioneer in a dedicated system terminal (PowerShell/Terminal/xterm depending on the OS) while streaming recent log lines and stage updates back into the GUI.
- **Progress monitoring** – Parses Pioneer stdout/stderr for high-level stage hints (parameter tuning, first search, quant search, etc.) and displays a concise progress bar and status history.
- **JSON interoperability** – Load an existing configuration file into either workflow, make adjustments, and save it back out. All file operations use the native OS dialog.

---

## Prerequisites

- **Pioneer CLI installed and on `PATH`**
  - Install Pioneer following the upstream instructions and verify that running `Pioneer --help` from a shell works.
  - The GUI discovers the binary by name (`Pioneer`/`Pioneer.exe`). If it isn’t on `PATH`, add the directory to your environment variables before launching the GUI.
- **Rust toolchain** – Latest stable toolchain for compiling the Tauri backend.
- **Node.js 18+** – Used to build the Svelte frontend (any modern Node LTS release works).
- **Package manager** – `npm`, `pnpm`, or `yarn`. Examples below use `npm`.

Optional but recommended:

- **Git** – To clone this repository.
- **A supported terminal emulator** – The app tries several common terminals when launching Pioneer (`powershell`, `x-terminal-emulator`, `gnome-terminal`, `konsole`, `xfce4-terminal`, `mate-terminal`, `xterm`).

---

## Repository Layout

```
.
├── index.html              # Vite entry point
├── package.json            # Frontend dependencies and scripts
├── src/                    # Svelte application
│   ├── App.svelte          # Main UI with tabs, run controls, and status panel
│   ├── main.ts             # Svelte bootstrap
│   └── lib/                # Form components, types, and JSON helpers
├── src-tauri/              # Tauri backend
│   ├── src/main.rs         # Commands for defaults, config IO, and process management
│   ├── tauri.conf.json     # Tauri configuration
│   └── fallback/           # Pioneer repository JSON templates (used as fallback)
└── README.md
```

---

## Getting Started

1. **Clone the repository**
   ```bash
   git clone https://github.com/your-org/PioneerGUI.git
   cd PioneerGUI
   ```

2. **Install frontend dependencies**
   ```bash
   npm install
   ```

3. **Run in development mode**
   ```bash
   npm run tauri:dev
   ```
   Tauri will spin up the Svelte dev server and open the desktop shell. Any code change reloads automatically.

4. **Build a production bundle**
   ```bash
   npm run tauri:build
   ```
   The compiled binaries for Windows, macOS, and Linux will appear under `src-tauri/target/release/` (plus platform-specific bundle folders).

---

## Pioneer Binary Expectations

- Pioneer must be available on the command line *before* launching the GUI. On each startup the backend executes:
  - `Pioneer params-predict <tmp_lib_dir> PreviewLibrary <tmp_fasta> --params-path <tmp_json>`
  - `Pioneer params-search <tmp_library> <tmp_ms_dir> <tmp_results_dir> --params-path <tmp_json>`
- If these commands succeed, their JSON output populates the editor. If either command fails (missing executable, permission issues, etc.), the GUI logs the error, displays a warning banner, and falls back to the checked-in JSON templates from `assets/example_config` in the Pioneer repo.
- When you press **Run BuildSpecLib** or **Run SearchDIA**, the backend writes your current parameters to a temporary JSON file and then launches `Pioneer predict` or `Pioneer search` respectively. Output is streamed to a timestamped log file that the GUI tails while also opening a native terminal window to display the full Pioneer session.

---

## Using the Interface

1. **Select a workflow tab** – `BuildSpecLib` for spectral library generation or `SearchDIA` for DIA analysis.
2. **Review highlighted parameters** – Fields marked with *Key parameter* badges correspond to Pioneer’s simplified defaults and are typically the most commonly tuned values.
3. **Load or edit parameters**
   - Click **Load JSON…** to import an existing Pioneer configuration (deep merged onto the defaults so missing keys remain populated).
   - Adjust values directly in the UI; numeric inputs accept decimals, booleans use toggles, and primitive arrays use one-value-per-line textareas.
4. **Save your configuration** – Use **Save JSON…** to persist the current parameters to disk at any time.
5. **Run Pioneer** – Press the corresponding **Run** button. The GUI shows the inferred progress stage (presearch, first search, quant search, etc.), streams the latest lines into the *Recent Pioneer output* panel, and opens a terminal window tailing the live log.
6. **Inspect status** – The status sidebar reports the last run outcome, log file location, and any terminal-launch warnings.

You can switch between tabs at any time; each tab maintains its own in-memory configuration and last-loaded file path.

---

## Troubleshooting

| Issue | Suggested fix |
|-------|----------------|
| GUI banner shows *Defaults loaded from the Pioneer.jl repository fallbacks* | Confirm `Pioneer` is on `PATH` and rerun the app. The fallback remains fully editable but may not include the latest upstream changes. |
| No external terminal opens when running Pioneer | Ensure a compatible terminal emulator is installed. The GUI tries common commands (`powershell`, `x-terminal-emulator`, `gnome-terminal`, `konsole`, `xfce4-terminal`, `mate-terminal`, `xterm`). A warning message appears in the status panel if spawning the terminal failed; the run will still execute headlessly and logs stream inside the GUI. |
| Pioneer exits immediately with a non-zero status | Check the *Recent Pioneer output* panel and the log file path displayed in the status panel. Adjust parameters and rerun. |
| Loading JSON removes unspecified keys | The loader deep-merges your file onto the active defaults so optional keys remain populated. If keys are missing, verify the source file is valid JSON. |

---

## Contributing

1. Ensure you can run `npm run tauri:dev` locally.
2. Make changes to the Svelte frontend (`src/`) or the Rust backend (`src-tauri/`).
3. Run the relevant checks:
   ```bash
   npm run check      # svelte-check
   npm run lint       # eslint + prettier formatting checks
   npm run tauri:dev  # quick smoke test
   ```
4. Submit a pull request with a clear description. Please keep platform-specific logic (terminal launching, etc.) inside the Rust backend for easier review.

---

## License

This repository follows the same license as Pioneer.jl. Refer to `LICENSE` for details.
