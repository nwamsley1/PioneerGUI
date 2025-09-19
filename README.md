# Pioneer.jl Tauri GUI Application Documentation

## Project Overview

This is a Tauri-based desktop application that provides a graphical user interface for Pioneer.jl, a Julia-based mass spectrometry data analysis tool. The application combines a React/TypeScript frontend with a Rust backend, leveraging Tauri's IPC (Inter-Process Communication) to manage Julia processes and handle terminal I/O.

### Key Features
- **Interactive Terminal**: Real-time Julia REPL integration with xterm.js
- **Parameter Configuration**: Comprehensive UI for configuring Pioneer.jl analysis parameters
- **File Management**: File selection dialogs for raw data (.arrow) and spectral libraries (.alt)
- **Dynamic UI**: Collapsible sections, resizable terminal, and responsive layout
- **Julia Process Management**: Spawns and manages Julia processes with 12 threads
- **Real-time Output**: Live streaming of Julia stdout/stderr to the terminal UI

### Technology Stack
- **Frontend**: React 18, TypeScript, Vite, xterm.js
- **Backend**: Rust with Tauri framework, Tokio for async operations
- **Process Management**: Julia runtime with Pioneer.jl package
- **Styling**: Custom CSS with dark theme support

## Project Structure

```
TestTauriApp/
├── src/                        # React frontend source
│   ├── App.tsx                # Main application component
│   ├── App.css                # Main application styles
│   ├── main.tsx               # React entry point
│   ├── Terminal.tsx           # Terminal component (unused legacy)
│   ├── CollapsibleSection.tsx # Collapsible UI sections
│   ├── CollapsibleTerminal.tsx# Resizable terminal component
│   ├── ParameterMenus.tsx     # Parameter menu wrapper
│   ├── menu_components.tsx    # Parameter configuration components
│   ├── NumberInput.tsx        # Numeric input component
│   ├── ArrayInput.tsx         # Array input component
│   ├── ToggleInput.tsx        # Toggle switch component
│   ├── OptionsInput.tsx       # Dropdown options component
│   ├── LabelWithToolTip.tsx   # Label with tooltip helper
│   ├── MenuComponents.css     # Component-specific styles
│   ├── utils.tsx              # Utility functions
│   └── public/
│       └── defaultJsonData.tsx # Default configuration values
├── src-tauri/                  # Rust backend source
│   ├── src/
│   │   ├── main.rs            # Main Rust entry point
│   │   └── terminal.rs        # Julia process management
│   ├── Cargo.toml             # Rust dependencies
│   └── tauri.conf.json        # Tauri configuration
├── package.json               # Node.js dependencies
├── tsconfig.json              # TypeScript configuration
├── vite.config.ts             # Vite build configuration
└── index.html                 # HTML entry point
```

## Architecture Overview

### Frontend-Backend Communication Flow
1. React frontend sends commands via Tauri's `invoke` API
2. Rust backend receives commands and manages Julia process
3. Julia output streams through Tokio channels
4. Backend emits events to frontend via Tauri's event system
5. Frontend updates terminal display with received output

### Key Components

#### Terminal Management
- Uses xterm.js for terminal emulation
- Bidirectional communication with Julia REPL
- Resizable terminal with drag functionality
- Real-time output streaming with trimming and formatting

#### Parameter Configuration
- Hierarchical JSON structure for Pioneer.jl parameters
- Dynamic form generation from configuration schema
- Real-time validation and state management
- Persistent configuration through JSON export

#### Process Lifecycle
1. Start terminal spawns Julia with Pioneer.jl
2. Commands sent through mpsc channels
3. Stdout/stderr captured and streamed
4. Graceful shutdown on process exit

---

## Complete Source Files

### Backend Files

#### src-tauri/Cargo.toml
```toml
[package]
name = "testtauriapp"
version = "0.0.0"
description = "A Tauri App"
authors = ["you"]
edition = "2021"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[build-dependencies]
tauri-build = { version = "1", features = [] }

[dependencies]
tauri = { version = "1", features = [ "shell-all", "http-all", "window-all", "fs-read-file", "fs-exists", "fs-read-dir", "dialog-all",] }
serde = { version = "1", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
tokio-tungstenite = "0.18"
futures-util = "0.3"
serde_json = "1"

[features]
# This feature is used for production builds or when a dev server is not specified, DO NOT REMOVE!!
custom-protocol = ["tauri/custom-protocol"]
```

#### src-tauri/tauri.conf.json
```json
{
  "build": {
    "beforeDevCommand": "pnpm dev",
    "beforeBuildCommand": "pnpm build",
    "devPath": "http://localhost:1420",
    "distDir": "../dist"
  },
  "package": {
    "productName": "testtauriapp",
    "version": "0.0.0"
  },
  "tauri": {
    "allowlist": {
      "all": false,
      "shell": {
        "all": true
      },

      "http": {
        "all": true,
        "request": true,
        "scope": ["http://*", "https://*", "ws://*", "wss://*"]
      },
      "dialog": {
        "all": true
      },
      "fs": {
        "readFile": true,
        "readDir": true,
        "exists": true,
        "scope":[
          "./public/*"
        ]
      },
      "window":{
        "all": true
      }
    },
    "windows": [
      {
        "titleBarStyle": "Overlay",
        "title": ""
      }
    ],
    "security": {
      "csp": null
    },
    "bundle": {
      "active": true,
      "targets": "all",
      "identifier": "com.tauri.dev",
      "icon": [
        "icons/32x32.png",
        "icons/128x128.png",
        "icons/128x128@2x.png",
        "icons/icon.icns",
        "icons/icon.ico"
      ]
    }
  }
}
```

#### src-tauri/src/main.rs
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod terminal;
use terminal::{TerminalState, start_terminal, send_input, send_commands, generate_json};

#[tauri::command]
fn greet() -> String {
    String::from("Hello from Rust!")
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up the tokio runtime
    let runtime = tokio::runtime::Runtime::new()?;
    // Use the runtime to run our async function
    runtime.block_on(async {
        // Set up any application-wide state here
        tauri::Builder::default()
            .manage(TerminalState(std::sync::Mutex::new(tokio::sync::mpsc::channel::<String>(100).0)))
            .invoke_handler(tauri::generate_handler![start_terminal, send_input, send_commands, greet, generate_json])
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
        Ok(())
    })
}
/*
 <input
                          type="text"
                          placeholder="Parameter 1"
                          value={jsonData.parameter1}
                          onChange={(e) =>
                            setJsonData({ ...jsonData, parameter1: e.target.value })
                          }
                        />

                          <select 
                            value={selectedOption}
                            onChange={handleSelectChange}
                            className="dropdown-select">
                              {options.map((option)=>
                              <option key={option.value} value={option.value}>
                                {option.label}
                              </option>
                              )}
                          </select>
                        <button
                          onClick={() => {
                            // Call the Rust backend to generate the JSON file
                            invoke("generate_json", { jsonData });
                          }}
                        >
                          Generate JSON
                        </button>
*/

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command

/*
#[tauri::command]
fn run_julia(window: tauri::Window) {
    std::thread::spawn(move || {
        let mut child = Command::new("julia")
            .args(&["--startup-file=no", "-i"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to start Julia");

        let mut stdin = child.stdin.take().expect("Failed to open stdin");
        let stdout = child.stdout.take().expect("Failed to open stdout");
        let stderr = child.stderr.take().expect("Failed to open stderr");

        let mut stdout_reader = BufReader::new(stdout);
        let mut stderr_reader = BufReader::new(stderr);

        let mut send_command = |cmd: &str| {
            window.emit("julia-output", format!("> {}", cmd)).unwrap();
            writeln!(stdin, "{}", cmd).expect("Failed to write command");
            writeln!(stdin, "flush(stdout); flush(stderr)").expect("Failed to flush");

            let mut output = String::new();
            let mut error = String::new();

            loop {
                output.clear();
                if stdout_reader.read_line(&mut output).expect("Failed to read line") == 0 {
                    break;
                }
                if !output.trim().is_empty() {
                    window.emit("julia-output", output.trim()).unwrap();
                }
                if output.contains("julia>") {
                    //break;
                }
            }

            loop {
                error.clear();
                if stderr_reader.read_line(&mut error).expect("Failed to read error") == 0 {
                    break;
                }
                if !error.trim().is_empty() {
                    window.emit("julia-output", format!("Error: {}", error.trim())).unwrap();
                }
            }
        };

        let commands = vec![
            "a = 15;",
            "b = 7;",
            "println(a + b)",
            "println(\"test\")",
            "using DataFrames, CSV",
            "println(\"test\")",
            "test = DataFrame(CSV.File(\"/Users/n.t.wamsley/Desktop/test_wide_quant.csv\"));",
            "first(test, 10)",
            "exit()"
        ];

        for cmd in commands {
            send_command(cmd);
            // Wait for the prompt to appear before sending the next command
            std::thread::sleep(std::time::Duration::from_millis(500));
        }

        let status = child.wait().expect("Julia process failed");
        window.emit("julia-finished", status.success()).unwrap();
    });
}
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn main() {


    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

*/
```

#### src-tauri/src/terminal.rs
```rust
use std::sync::Mutex;
use tauri::Manager;
use tokio::process::Command;
use tokio::io::{BufReader, AsyncBufReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
pub struct TerminalState(pub Mutex<mpsc::Sender<String>>);

#[tauri::command]
pub async fn start_terminal(app_handle: tauri::AppHandle, state: tauri::State<'_, TerminalState>) -> Result<(), String> {
    let (tx, mut rx) = mpsc::channel(100);
    *state.0.lock().unwrap() = tx;

    let mut child = Command::new("julia")
        .arg("--banner=no")
        .arg("-i")
        .arg("--threads=12")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    let mut line = String::new();
    let mut stderr = BufReader::new(child.stderr.take().unwrap());
    let mut error_line = String::new();
    
    // Task to handle input sending to Julia's stdin
    tokio::spawn(async move {
        while let Some(input) = rx.recv().await {
            if let Err(e) = stdin.write_all((input + "\n").as_bytes()).await {
                eprintln!("Failed to write to stdin: {}", e);
                break;
            }
            if let Err(e) = stdin.flush().await {
                eprintln!("Failed to flush stdin: {}", e);
                break;
            }
        }
    });



    // Clone the app_handle for stdout task
    let app_handle_stdout = app_handle.clone();
    tokio::spawn(async move {
        loop {
            match stdout.read_line(&mut line).await {
                Ok(0) => break, // EOF
                Ok(_) => {
                    let trimmed_line = line.trim_start().to_string();
                    if !trimmed_line.is_empty() {
                        app_handle_stdout.emit_all("julia-output", trimmed_line + "\n").unwrap();
                    }
                    line.clear();
                }
                Err(e) => {
                    eprintln!("Error reading stdout: {}", e);
                    break;
                }
            }
        }
    });

    // Clone the app_handle for stdout task
    /*
        let app_handle_stdout = app_handle.clone();
    tokio::spawn(async move {
        let mut buffer = [0; 1024];
        loop {
            match stdout.read(&mut buffer).await {
                Ok(0) => break, // EOF
                Ok(n) => {
                    let output = String::from_utf8_lossy(&buffer[..n]);
                    app_handle_stdout.emit_all("julia-output", output.to_string()).unwrap();
                }
                Err(e) => {
                    eprintln!("Error reading stdout: {}", e);
                    break;
                }
            }
        }
    });
     */

    // Clone the app_handle for stderr task
    let app_handle_stderr = app_handle.clone();

    tokio::spawn(async move {
        loop {
            match stderr.read_line(&mut error_line).await {
                Ok(0) => break, // EOF
                Ok(_) => {
                    if !error_line.trim().is_empty() {
                        app_handle_stderr.emit_all("julia-error", error_line.clone()).unwrap();
                    }
                    error_line.clear();
                }
                Err(e) => {
                    eprintln!("Error reading stderr: {}", e);
                    break;
                }
            }
        }
    });

    // Task to wait for Julia process to exit
    tokio::spawn(async move {
        let status = child.wait().await;
        match status {
            Ok(exit_status) => println!("Julia process exited with status: {:?}", exit_status),
            Err(e) => eprintln!("Error waiting for Julia process: {}", e),
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn send_input(input: String, state: tauri::State<'_, TerminalState>) -> Result<(), String> {
    let sender = state.0.lock().unwrap().clone();
    
    // Split the input into lines
    let lines: Vec<&str> = input.lines().collect();
    
    // Get the number of lines
    let line_count = lines.len();

    // Log the line count
    println!("Input received with {} lines", lines.len());

    // Take up to 50 lines
    let limited_lines: Vec<&str> = lines.iter().take(50).map(|&s| s).collect();
    
    // Join the limited lines back into a single string
    let limited_input = limited_lines.join("\n");
    
    // Send the limited input
    sender.send(limited_input + "\n").await.map_err(|e| e.to_string())?;
    
    // If there were more than 50 lines, send a message indicating that output was truncated
    if line_count > 50 {
        sender.send("\n... (output truncated after 50 lines)\n".to_string()).await.map_err(|e| e.to_string())?;
    }

    Ok(())

}


#[tauri::command]
pub async fn send_commands(state: tauri::State<'_, TerminalState>) -> Result<(), String> {
    let ascii_art = r#"println(
                                                                       
,-.----.                                                               
\    /  \                                                              
|   :    \   ,--,                                                      
|   |  .\ :,--.'|     ,---.        ,---,                       __  ,-. 
.   :  |: ||  |,     '   ,'\   ,-+-. /  |                    ,' ,'/ /| 
|   |   \ :`--'_    /   /   | ,--.'|'   |   ,---.     ,---.  '  | |' | 
|   : .   /,' ,'|  .   ; ,. :|   |  ,"' |  /     \   /     \ |  |   ,' 
;   | |`-' '  | |  '   | |: :|   | /  | | /    /  | /    /  |'  :  /   
|   | ;    |  | :  '   | .; :|   | |  | |.    ' / |.    ' / ||  | '    
:   ' |    '  : |__|   :    ||   | |  |/ '   ;   /|'   ;   /|;  : |    
:   : :    |  | '.'|\   \  / |   | |--'  '   |  / |'   |  / ||  , ;    
|   | :    ;  :    ; `----'  |   |/      |   :    ||   :    | ---'     
`---'.|    |  ,   /          '---'        \   \  /  \   \  /           
  `---`     ---`-'                         `----'    `----'            
                                                                       
)"#;

    let commands = vec![
        "println(\"Starting Pioneer...\")",
        "using Revise, Pkg",
        "cd(\"/Users/n.t.wamsley/Projects/Pioneer.jl/\")",
        "Pkg.activate(\".\")",
        "using Pioneer",
        "SearchDIA(\"./data/example_config/LibrarySearch.json\")",
        "exit()"
    ];


    let sender = state.0.lock().unwrap().clone();
    for cmd in commands {
        sender.send(cmd.to_string()).await.map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn generate_json(json_data: serde_json::Value) -> Result<(), String> {
  // Convert the JSON data to a string
  let json_string = serde_json::to_string_pretty(&json_data).unwrap();

  // Save the JSON string to a file
  std::fs::write("/Users/n.t.wamsley/Desktop/pre_search_parameters.json", json_string).unwrap();

  Ok(())
}


/*

  useEffect(() => {
    if (terminalRef.current && !term) {
      const newTerm = new Terminal({
        fontFamily: '"Courier New", monospace',
        fontSize: 14,
        lineHeight: 1.2,
        letterSpacing: 0,
        rendererType: 'canvas',
        theme: {
          background: "rgb(47, 47, 47)",
        },
      });
      const fitAddon = new FitAddon();
      newTerm.loadAddon(fitAddon);
      newTerm.open(terminalRef.current);
      fitAddon.fit();

      window.addEventListener('resize', () => {
        fitAddon.fit();
      });

      setTerm(newTerm);
      /*
      newTerm.onData(e => {
        if (e === '\r') { // Check if the enter key is pressed
          invoke('send_input', { input: '\n' });
        } else {
          invoke('send_input', { input: e });
        }
      });
      */
      invoke('start_terminal');

      const unlistenOutput = listen('julia-output', (event) => {
        const output = event.payload as string;
        // Trim any leading whitespace and ensure a single newline at the end
        const trimmedOutput = output.trimStart().replace(/\s*$/, '\r\n');
        newTerm.write(trimmedOutput);
      });
      
      const unlistenError = listen('julia-error', (event) => {
        const error = event.payload as string;
        // Trim any leading whitespace and ensure a single newline at the end
        const trimmedError = error.trimStart().replace(/\s*$/, '\r\n');
        newTerm.write(`Error: ${trimmedError}`);
      });

      return () => {
        newTerm.dispose();
        unlistenOutput.then(f => f());
        unlistenError.then(f => f());
      };

    }
  }, []);

*/
```

### Frontend Files

#### package.json
```json
{
  "name": "testtauriapp",
  "private": true,
  "version": "0.0.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "tauri": "tauri"
  },
  "dependencies": {
    "@tauri-apps/api": "^1",
    "@xterm/addon-fit": "^0.10.0",
    "@xterm/xterm": "^5.5.0",
    "node-pty": "^1.0.0",
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "xterm": "^5.3.0",
    "xterm-addon-fit": "^0.8.0",
    "xterm-for-react": "^1.0.4"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^1",
    "@types/react": "^18.2.15",
    "@types/react-dom": "^18.2.7",
    "@vitejs/plugin-react": "^4.2.1",
    "typescript": "^5.2.2",
    "vite": "^5.3.1"
  }
}
```

#### tsconfig.json
```json
{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,

    /* Bundler mode */
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx",

    /* Linting */
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true
  },
  "include": ["src"],
  "references": [{ "path": "./tsconfig.node.json" }],
}
```

#### vite.config.ts
```typescript
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [react()],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
```

#### index.html
```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <link rel="icon" type="image/svg+xml" href="/vite.svg" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Tauri + React + Typescript</title>
  </head>

  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
```

#### src/main.tsx
```typescript
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
```

#### src/App.tsx
```typescript
import { open } from '@tauri-apps/api/dialog';
import CollapsibleSection from './CollapsibleSection.tsx';
import ParameterMenuContent from './ParameterMenus.tsx';
import "./App.css";
import React, { useEffect, useRef, useState } from 'react';
import { Terminal } from 'xterm';
import { FitAddon } from 'xterm-addon-fit';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import 'xterm/css/xterm.css';
import CollapsibleTerminal from './CollapsibleTerminal';
import {PresearchParams} from './menu_components.tsx';
import {Content1, FirstSearchParams, QuantSearchParams, DeconvolutionParams, NormalizationParams, IrtMappingParams, SummarizeFirstSearchParams} from './menu_components.tsx';
import { readTextFile, BaseDirectory } from '@tauri-apps/api/fs';
import defaultJsonData from './public/defaultJsonData.tsx';

function App() {
  
  const [isTerminalOpen, setIsTerminalOpen] = useState(false);
  const [isProgramRunning, setIsProgramRunning] = useState(false);
  const [selectedOption, setSelectedOption] = useState('');
  const [selectedFiles, setSelectedFiles] = useState([]);
  const [selectedLibrary, setSelectedLibrary] = useState([]);
  const terminalRef = useRef<HTMLDivElement>(null);
  const lastSectionRef = useRef<HTMLDivElement>(null);
  const [jsonData, setJsonData] = useState<JsonData>(defaultJsonData);


    const terminalInstanceRef = useRef<Terminal | null>(null);
    const fitAddonRef = useRef<FitAddon | null>(null);
  

    useEffect(() => {
      let unlistenOutput: (() => void) | undefined;
      let unlistenError: (() => void) | undefined;
  
      const initializeTerminal = async () => {
        if (terminalRef.current && !terminalInstanceRef.current) {
          const newTerm = new Terminal({
            fontFamily: '"Courier New", monospace',
            fontSize: 14,
            lineHeight: 1.2,
            letterSpacing: 0,
            theme: {
              background: "rgb(47, 47, 47)",
            },
          });
          
          const fitAddon = new FitAddon();
          newTerm.loadAddon(fitAddon);
          newTerm.open(terminalRef.current);
          fitAddon.fit();
  
          terminalInstanceRef.current = newTerm;
          fitAddonRef.current = fitAddon;
  
          newTerm.onData(e => {
            invoke('send_input', { input: e === '\r' ? '\n' : e });
          });
          
          await invoke('start_terminal');
  
          unlistenOutput = await listen('julia-output', (event: any) => {
            const output = event.payload as string;
            // Trim any leading whitespace and ensure a single newline at the end
            const trimmedOutput = output.trimStart().replace(/\s*$/, '\r\n');
            newTerm.write(trimmedOutput);
          });
  
          unlistenError = await listen('julia-error', (event: any) => {
            const error = event.payload as string;
            // Trim any leading whitespace and ensure a single newline at the end
            const trimmedError = error.trimStart().replace(/\s*$/, '\r\n');
            newTerm.write(`Error: ${trimmedError}`);
          });
  
          window.addEventListener('resize', () => fitAddon.fit());
        }
      };
  
      initializeTerminal();
  
      return () => {
        if (terminalInstanceRef.current) {
          terminalInstanceRef.current.dispose();
          terminalInstanceRef.current = null;
        }
        if (unlistenOutput) unlistenOutput();
        if (unlistenError) unlistenError();
        window.removeEventListener('resize', () => fitAddonRef.current?.fit());
      };
    }, []);


    const handleSendCommands = () => {
      console.log("TEST");
      if (isProgramRunning) {
        setIsProgramRunning(false);
        setIsTerminalOpen(false);
      } else{
        setIsProgramRunning(true);
        setIsTerminalOpen(true);
        invoke('send_commands');
      }
    };

    const handleToggleTerminal = () => {
      setIsTerminalOpen(!isTerminalOpen);
    };

  async function handleFileSelection(){
    try{
      const selected = await open({
        multiple: true,
        filters:[{
          name: 'Image',
          extensions:['arrow']
        }]
      });
      setSelectedFiles([]);
      if (Array.isArray(selected) && selected.length > 0){
        //Selected multiple files
        setSelectedFiles(selected);
        //setGreetMsg(`Selected ${selected.length} files: ${selected.join(', ')}`);
      } else if (selected === null){
        //file selection cancelled
        //setGreetMsg("File selection was cancelled");
        setSelectedFiles([]);
      } else {
        //single file 
        setSelectedFiles([selected]);
        //setGreetMsg(`Selected file: ${selected}`);
      }
    } catch (error) {
      console.error("Error selected file:", error);
      //setGreetMsg("Error selecting file");
      setSelectedFiles([]);
    }
  }

  return (
    <div className="root-container">
      <div className="title-bar-containter">      
          <div data-tauri-drag-region className="title-bar"></div>
          <div data-tauri-drag-region className="main-content-header"></div>
      </div>
      <div className="app-container">
        <div className="sidebar">

            <div className="sidebar-content">
              <ul className="sidebar-menu">
                <li className="sidebar-menu-item"><a href="#">Pioneer</a></li>
                <li className="sidebar-menu-item"><a href="#">Translator</a></li>
                <li className="sidebar-menu-item"><a href="#">Altimiter</a></li>
                <li className="sidebar-menu-item"><a href="#">Horizon</a></li>
              </ul>
            </div>
          <div className="sidebar-footer">
                      <button
                          onClick={() => {
                            // Call the Rust backend to generate the JSON file
                            invoke("generate_json", { jsonData });
                          }}
                        >
                          Generate JSON
                        </button>

            <button className={`run-button ${isProgramRunning ? 'terminal-open' : ''}`}
                    onClick={handleSendCommands}>
              <span className="arrow-symbol">
                {!isProgramRunning ? "Run" : "Abort" }</span>
            </button>
          </div>
        </div>

        <div className="main-content-wrapper">
            
              <div className="main-content">
                <div className="scrollable-content">

                  
                  <button className="select-files-btn" onClick={handleFileSelection}>Raw Data (.arrow)</button>
                  <div className="file-selection-row">
                      <div className="file-selection-info">
                        {selectedFiles.length > 0 && (
                          <div className="selected-files-box">
                            {selectedFiles.map((file, index) => (
                              <div key={index} className="file-name" title={file}>{file}</div>
                            ))}
                          </div>
                        )}
                      </div>
                  </div>

                  <button className="select-files-btn" onClick={handleFileSelection}>Spectral Library (.alt)</button>
                  <div className="file-selection-row">
                    <div className="file-selection-info">
                      {selectedLibrary.length > 0 && (
                        <div className="selected-files-box">
                          {selectedLibrary.map((file, index) => (
                            <div key={index} className="file-name" title={file}>{file}</div>
                          ))}
                        </div>
                      )}
                    </div>
                  </div>

                  <CollapsibleSection title="Argument Presets">
                        <p className="preset-item">Select from default configurations</p>
                        <button className="select-presets-btn">OrbitrapAstralDIA.pion.config</button>
                        <button className="select-presets-btn">OrbitrapDIA.pion.config</button>
                        <button className="select-presets-btn">ZenoTOF.pion.config</button>
                        <button className="select-presets-btn">Browse Files... (.pion.config)</button>
                  </CollapsibleSection>


                  <div className="collapsible-sections-container">
                  <div className="collapsible-sections-container">
                  <div className="collapsible-sections-column">
                  <CollapsibleSection title="Presearch Parameters">
                    <ParameterMenuContent 
                        jsonData={jsonData}
                        setJsonData={setJsonData}
                        ContentComponent={PresearchParams}
                      />
                  </CollapsibleSection>
                  <CollapsibleSection title="First Search Params">
                  <ParameterMenuContent 
                        jsonData={jsonData}
                        setJsonData={setJsonData}
                        ContentComponent={FirstSearchParams}
                      />
                  </CollapsibleSection>
                  <CollapsibleSection title="Normalization Params">
                  <ParameterMenuContent 
                          jsonData={jsonData}
                          setJsonData={setJsonData}
                          ContentComponent={NormalizationParams}
                        />
                  </CollapsibleSection>
                  <CollapsibleSection title="Quant Search Params">
                    <ParameterMenuContent 
                          jsonData={jsonData}
                          setJsonData={setJsonData}
                          ContentComponent={QuantSearchParams}
                        />
                  </CollapsibleSection>  
                  </div>
                  <div className="collapsible-sections-column">
                  <CollapsibleSection title="Deconvolution Params">
                  <ParameterMenuContent 
                          jsonData={jsonData}
                          setJsonData={setJsonData}
                          ContentComponent={DeconvolutionParams}
                        />
                  </CollapsibleSection>
                  <CollapsibleSection title="RT Mapping Params">
                  <ParameterMenuContent 
                          jsonData={jsonData}
                          setJsonData={setJsonData}
                          ContentComponent={IrtMappingParams}
                        />
                  </CollapsibleSection>
                  <CollapsibleSection title="Summarize First Search Params">
                  <ParameterMenuContent 
                          jsonData={jsonData}
                          setJsonData={setJsonData}
                          ContentComponent={SummarizeFirstSearchParams}
                        />
                  </CollapsibleSection>
                  <CollapsibleSection title="RT and Mass Tolerance">
                    <ParameterMenuContent 
                      jsonData={jsonData}
                      setJsonData={setJsonData}
                      ContentComponent={Content1}
                    />
                    <div   ref={lastSectionRef} />
                  </CollapsibleSection>


                    </div>
                  </div>
                  </div>
                </div>
              <CollapsibleTerminal 
                isOpen={isTerminalOpen} 
                isRunning={isProgramRunning}
                previousSectionRef={lastSectionRef}
                onToggle={handleToggleTerminal} />       
            
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;
```

#### src/App.css
```css
.logo.vite:hover {
  filter: drop-shadow(0 0 2em #747bff);
}

.logo.react:hover {
  filter: drop-shadow(0 0 2em #61dafb);
}

:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  background-color: #f6f6f6;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

.titlebar {
  height: 30px;
  background: #329ea3;
  user-select: none;
  display: flex;
  justify-content: flex-end;
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
}

.titlebar-button {
  display: inline-flex;
  justify-content: center;
  align-items: center;
  width: 30px;
  height: 30px;
}

.titlebar-button:hover {
  background: #5bbec3;
}

.container {
  margin: 0;
  padding-top: 10vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
}

.logo {
  height: 6em;
  padding: 1.5em;
  will-change: filter;
  transition: 0.75s;
}

.logo.tauri:hover {
  filter: drop-shadow(0 0 2em #24c8db);
}

.row {
  display: flex;
  justify-content: center;
}

a {
  font-weight: 500;
  color: #646cff;
  text-decoration: inherit;
}

a:hover {
  color: #535bf2;
}

h1 {
  text-align: center;
}

input,
button {
  border-radius: 0px;
  border: 1px solid transparent;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: #0f0f0f;
  background-color: #ffffff;
  transition: border-color 0.25s;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
  outline: none;
  cursor: pointer;
}

button {
  cursor: pointer;
}

button:hover {
  border-color: #396cd8;
}
button:active {
  border-color: #396cd8;
  background-color: #e8e8e8;
}

input,
button {
  outline: none;
}

#greet-input {
  margin-right: 5px;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }

  a:hover {
    color: #24c8db;
  }

  input,
  button {
    color: #ffffff;
    background-color: #0f0f0f98;
  }
  button:active {
    background-color: #0f0f0f69;
  }
}
/*
.root-containter {
  height: 100vh;
  width: 150vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
*/
.root-container {
  height: 100vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.title-bar-containter{
  top: 0px;
  left: 0;
  right: 0;
  z-index: 1000;
  height:30px;
  display: flex;
  position:fixed;
}

.title-bar {
  height: 30px; /* Adjust this value to make the title bar taller */
  width: 150px;
  background-color: #f0f0f0;/* Makes buttons clickable background-color: #2c2c2c;*/
  -webkit-app-region: drag; /* Makes the title bar draggable */
  box-sizing: border-box;
}

.main-content-header {
  height: 100%; /* Adjust this value to make the title bar taller */
  background-color: #f0f0f0;
  -webkit-app-region: drag; /* Makes the title bar draggable */
  flex-grow: 1;
}

/*
.app-container {
  padding-top: 30px;
  height: calc(100vh - 30px);
  overflow-y: scroll;
  display: flex;
}
*/
.app-container {
  height: calc(100vh - 30px); /* Subtract the title bar height */
  flex: 1;
  display: flex;
  overflow: hidden;
}

.title-bar-text {
  color: white;
  font-size: 14px;
}

.title-bar-controls {
  display: flex;
  -webkit-app-region: no-drag; /* Makes buttons clickable */
}

.title-bar-controls button {
  background: none;
  border: none;
  color: white;
  font-size: 18px;
  padding: 5px 10px;
  cursor: pointer;
  transition: background-color 0.3s;
}

.title-bar-controls button:hover {
  background-color: rgba(255, 255, 255, 0.1);
}

.link-button {
  margin-right: 10px;
}

.main-content-wrapper {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  padding-left: 0px;
  padding-right: 0px;
  padding-top: 30px;
  box-sizing: border-box;
  width: 100%;
}

.sidebar {
  width: 150px;
  min-width: 150px;
  height: 100%;
  padding-top: 30px;
  padding-bottom: 10px;
  overflow-y: auto;
  background-color: #f0f0f0;
  box-shadow: 0px 0 5px rgba(0,0,0,0.1);
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
  position: relative;
}

.terminal-banner {
  background-color: #333;
  color: white;
  text-align: center;
  padding: 0px;
  cursor: pointer;
  font-size: 16px;
  height: 25px;
}

.terminal-banner:hover {
  background-color: #555;
}

.sidebar h2 {
  margin-bottom: 20px;
}

.sidebar ul {
  list-style-type: none;
  padding: 0;
}

.sidebar li {
  margin-bottom: 10px;
  font-size: 24px;
  cursor: pointer;
}

.sidebar li:hover {
  color: #0066cc;
}

.sidebar h2:hover{
  color: #0066cc;
  background-color: #ccc;
}

.sidebar-footer {
  position: sticky;
  bottom: 0;
  width: 130px;
  padding: 10px; /* Adjust as needed */
  background-color: inherit; /* Match the sidebar background */
}

.sidebar-content {
  flex-grow: 1;
  overflow: hidden;
}

.sidebar-menu {
  list-style-type: none;
  overflow: hidden;
  padding: 0;
  margin: 0;
}

.sidebar-menu-item {
  font-weight: bold;
  padding: 0;
}

.sidebar-menu-item a {
  display: block;
  font-weight: bold;
  padding: 10px 20px;
  text-decoration: none;
  color: #333;
  transition: background-color 0.3s;
}

.sidebar-menu-item a:hover {
  background-color: #ddd; /* Change this to your desired hover color */
}

.run-button {
  width: 100%;
  padding: 10px;
  background-color: #4CAF50;
  color: white;
  border: none;
  border-radius: 5px;
  font-size: 16px;
  cursor: pointer;
  align-items: center;
  justify-content: center;
  position: sticky;
  transition: background-color 0.3s;
}

.run-button.terminal-open {
  background-color: #2196F3; /* Blue color when terminal is open */
}

.run-button.terminal-open:hover {
  background-color: #1976D2;
}

.run-button:hover {
  background-color: #45a049;
}

.run-button:active {
  background-color: #3e8e41;
}

.arrow-symbol {
  margin-right: 8px;
  font-size: 18px;
}


.file-selection-container {
  max-width:100%;
}

.file-selection-column{
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.file-selection-row {
  display: flex;
  align-items: left;
  margin-top: 10px;
  margin-bottom:10px;
}

.greet-msg {
  flex-grow: 1;
  word-break: break-word;
}

.file-selection-row button {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  margin-top: 20px;
  white-space: nowrap;
}

.file-selection-info {
  flex-grow: 1;
  margin-right: 0px;
  margin-left: 0px;
  align-self: center;
  overflow-x: auto;
}

.greet-msg {
  margin-bottom: 100px;
}

.file-name {
  margin-bottom: 5px;
  white-space: nowrap; /* don't wrap text to next line*/
}

.file-selection-row button {
  white-space: nowrap;
  align-self: flex-start;
  justify-content: space-between;
  margin-top: 20px;
}

.select-files-btn {
  height: 50px;
  width: 100%; /* Increased width for better proportion */
  font-size: 16px;
  background-color: #2196F3;
  color: white;
  border: none;
  border-radius: 5px;
  cursor: pointer;
  display: flex;
  justify-content: center;
  align-items: center;
  align-self: center;
  text-align: center;
}

.select-files-btn:hover {
  background-color: #1976D2
}

.selected-files-box {
  max-height: 75px;
  overflow: auto;
  border: 3px solid #ccc;
  border-radius: 1px;
  padding: 3px;
  background-color: #f9f9f9;
  align-self: center;
  align-items: center;
  justify-content: center;
}

.select-presets-btn{
  height: 50px;
  width: 100%; /* Increased width for better proportion */
  font-size: 16px;
  background-color: #f0f0f0;
  color: black;
  border: 0.5 px solid #ccc;
  border-radius: 0px;
  cursor: pointer;
  display: flex;
  justify-content: center;
  align-items: center;
  align-self: center;
  text-align: center;
}

.select-presets-btn:hover {
  background-color: white;
}

.preset-item {
  display: flex;
  padding: 0px 15px;
  font-size: 16px;
  justify-content: space-between;
  align-items: center;
  font-weight: bold;
  text-decoration: none;
  color: #333;
}


.dropdown-container {
  width: 100%;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  justify-content: center;
  margin-bottom: 20px;
}

.dropdown-select {
  width: 200px;
  height: 40px;
  font-size: 16px;
  border: 1px solid #ccc;
  border-radius: 5px;
  background-color: white;
  cursor: pointer;
  padding: 0 10px;
}

.dropdown-select:focus {
  outline: none;
  border-color: #4CAF50;
}

html, body {
  height: 100%;
  margin: 0;
  padding: 0;
}

.collapsible-section {
  width: 100%;
  margin-bottom: 20px;
  border: 1px solid #ccc;
  border-radius: 5px;
  overflow: hidden;
}

.collapsible-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 15px;
  background-color: #f0f0f0;
  cursor: pointer;
  transition: background-color 0.3s;
}

.collapsible-header:hover {
  background-color: #e0e0e0;
}

.collapsible-header h3 {
  margin: 0;
}

.arrow {
  font-size: 12px;
  transition: transform 0.3s;
}

.collapsible-header.open .arrow {
  transform: rotate(180deg);
  transition: transform 0.5s ease-in-out; /* Add transition for arrow rotation */

}

.collapsible-content {
  max-height: 0;
  overflow: hidden;
  transition: max-height 0.5s ease-out;
}

.collapsible-content.open {
  max-height: 400px;
  min-height: 0px; /* Adjust this value based on your content */
  overflow-y: auto;
  transition: max-height 0.5s ease-in-out; /* Increased duration */
}


.collapsible-terminal {
  height: 300px; /* or whatever height you prefer */
  overflow: hidden;
  transition: height 0.3s ease;
  position: relative;
  background-color: rgb(47, 47, 47);
  margin-top: 10px;
  flex-shrink: 0; /* Prevent terminal from shrinking */
}

.collapsible-terminal-content {
  transition: height 0.3s ease-out;
  height: 100%;
  width: 100%;
  overflow: hidden; /* Changed from overflow-y: auto to hidden */
}

.collapsible-terminal.closed {
  height: 0;
}

.collapsible-terminal.open {
  border-top: 1px solid #444;
}


.scrollable-content {
  flex: 1;
  overflow-y: auto;

  padding: 20px;
}

.scrollable-content > * {
  max-width: 100%;
}



.scrollable-content::-webkit-scrollbar {
  width: 10px;
}

.scrollable-content::-webkit-scrollbar-track {
  background: #f1f1f1;
}

.scrollable-content::-webkit-scrollbar-thumb {
  background: #888;
}

.scrollable-content::-webkit-scrollbar-thumb:hover {
  background: #555;
}


input {
  border: none;
  outline: none;
  margin: 0;
  padding: 0;
  background-color: transparent;
  color: black;
}

input[type="text"] {
  border: none;
  border-bottom: 2px solid #3498db;
  background-color: transparent;
  padding: 10px 0;
  font-size: 16px;
  color: #333;
  transition: all 0.3s ease;
}

input[type="text"]:focus {
  outline: none;
  border-bottom-color: #2980b9;
  background-color: rgba(52, 152, 219, 0.1);
}



/* Ensure xterm.js terminal takes full size of its container */
.xterm {
  height: 100%;
  width: 100%;
}

/* Hide xterm.js scrollbar */
.xterm .xterm-scroll-area {
  -ms-overflow-style: none;  /* IE and Edge */
  scrollbar-width: none;  /* Firefox */
}

.xterm .xterm-scroll-area::-webkit-scrollbar {
  display: none;  /* Chrome, Safari and Opera */
}

.xterm-viewport {
  overflow-y: auto
}

.xterm-screen {
  width: 100%;
}



.xterm-viewport::-webkit-scrollbar {
  width: 10px;
}
.xterm-viewport::-webkit-scrollbar-track {
  background: #f1f1f1;
}

.xterm-viewport::-webkit-scrollbar-thumb {
  background: #888;
}

.xterm-viewport::-webkit-scrollbar-thumb:hover {
  background: #555;
}

.preset-description {
  font-size: 14px;
  color: #666;
  margin-bottom: 20px;
  line-height: 1.5;
}

.parameters-container {
  min-height: 400px;
  background-color: #f7f7f7;
  border-radius: 8px;
  padding: 20px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  overflow-x: auto; /* Allow horizontal scrolling if needed */
}

.parameter-columns {
  display: flex;
  flex-wrap: wrap; /* Allow columns to wrap */
  gap: 20px;
  justify-content: space-between;
}

.parameter-column {
  flex: 1 1 200px; /* Grow, shrink, and have a base width of 300px */
  min-width: 0; /* Allow shrinking below min-content width */
  max-width: 100%; /* Prevent expanding beyond container width */
}



@media (max-width: 768px) {
  .parameter-column {
    flex-basis: 100%; /* Full width on smaller screens */
  }
}

.collapsible-sections-container {
  display: flex;
  gap: 20px; /* Adjust the space between columns */
}

.collapsible-sections-column {
  flex: 1; /* Each column takes up equal width */
}

/* Responsive design for smaller screens */
@media (max-width: 768px) {
  .collapsible-sections-container {
    flex-direction: column;
  }
}

select {
  appearance: none;
  -webkit-appearance: none;
  -moz-appearance: none;
  background-color: #f0f0f0;
  border: none;
  border-bottom: 2px solid #3498db;
  border-radius: 0;
  padding: 10px 30px 10px 10px;
  font-size: 16px;
  color: #333;
  cursor: pointer;
  transition: all 0.3s ease;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12'%3E%3Cpath fill='%23333' d='M10.293 3.293L6 7.586 1.707 3.293A1 1 0 00.293 4.707l5 5a1 1 0 001.414 0l5-5a1 1 0 10-1.414-1.414z'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 10px center;
}

select:focus {
  outline: none;
  background-color: #e8e8e8;
  box-shadow: 0 2px 5px rgba(0, 0, 0, 0.1);
}

.collapsible-content::-webkit-scrollbar {
  width: 10px;
}
.collapsible-content::-webkit-scrollbar-track {
  background: #f1f1f1;
}

.collapsible-content::-webkit-scrollbar-thumb {
  background: #888;
}

.collapsible-content::-webkit-scrollbar-thumb:hover {
  background: #555;
}

.parameter-label {
  position: relative;
  display: inline-block;
}

.tooltip-trigger {
  margin-left: 5px;
  cursor: help;
  color: #3498db;
}

.tooltip {
  visibility: hidden;
  width: 200px;
  background-color: #555;
  color: #fff;
  text-align: center;
  border-radius: 6px;
  padding: 5px;
  position: absolute;
  z-index: 1;
  bottom: 125%;
  left: 50%;
  margin-left: -100px;
  opacity: 0;
  transition: opacity 0.3s;
}

.tooltip::after {
  content: "";
  position: absolute;
  top: 100%;
  left: 50%;
  margin-left: -5px;
  border-width: 5px;
  border-style: solid;
  border-color: #555 transparent transparent transparent;
}

.parameter-label:hover .tooltip {
  visibility: visible;
  opacity: 1;
}
.terminal-banner.dragging {
  background-color: #4a4a4a; /* or any color to indicate dragging */
}
```

#### src/MenuComponents.css
```css
* {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
  }
/* Common styles for both input types */
.array-input-group,
.toggle-input-group,
.options-input-group,
.number-input-group {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}

.toggle-input-group label,
.options-input-group label,
.number-input-group label {
  flex: 1;
  max-width: 60%;
  text-align: left;
  font-weight: 600;
  color: #333;
  padding-right: 10px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.toggle-input-wrapper,
.custom-dropdown,
.number-input-wrapper {
  width: 40%;
  max-width: 200px;
}

/* Options Input specific styles */
.dropdown-header {
  position: relative;
  border: none;
  border-bottom: 2px solid #3498db;
  background-color: transparent;
  padding: 5px 0;
  font-size: 16px;
  color: #333;
  transition: all 0.3s ease;
  width: 100%;
  text-align: left;
  cursor: pointer;
  height: 30px;
  display: flex;
  align-items: center;
}

.dropdown-arrow {
  position: absolute;
  right: 0;
  top: 50%;
  transform: translateY(-50%);
  width: 0;
  height: 0;
  border-left: 5px solid transparent;
  border-right: 5px solid transparent;
  border-top: 5px solid #333;
}

.dropdown-options {
  width: 100%;
  font-size: 16px;
  color: #333;
  transition: all 0.3s ease;
  background-color: white;
  border: 1px solid #3498db;
  border-top: none;
  max-height: 150px;
  overflow-y: auto;
}

.dropdown-option {
  padding: 5px;
  cursor: pointer;
}

.dropdown-option:hover {
  background-color: rgba(52, 152, 219, 0.1);
}

/* Number Input specific styles */
.number-input-wrapper input[type="number"] {
  width: 100%;
  border: none;
  border-bottom: 2px solid #3498db;
  background-color: transparent;
  padding: 5px 0;
  font-size: 16px;
  color: #333;
  transition: all 0.3s ease;
  height: 30px;
  box-sizing: border-box;
  -moz-appearance: textfield;
}

.number-input-wrapper input[type="number"]::-webkit-inner-spin-button,
.number-input-wrapper input[type="number"]::-webkit-outer-spin-button {
  /*-webkit-appearance: none;*/
  margin: 0;
}

.number-input-wrapper input[type="number"]:focus,
.number-input-wrapper input[type="number"]:hover,
.dropdown-header:hover {
  outline: none;
  border-bottom-color: #2980b9;
  background-color: rgba(52, 152, 219, 0.1);
}

/* Toggle Group*/

  .toggle-input-wrapper {
    display: flex;
    justify-content: flex-end;
  }
  

  .toggle-switch {
    position: relative;
    width: 78px;  /* Reduced width */
    height: 24px;  /* Reduced height */
  }
  
  .toggle-switch input {
    opacity: 0;
    width: 0;
    height: 0;
  }
  
  .toggle-switch label {
    position: absolute;
    cursor: pointer;
    top: 0;
    left: auto;
    right: 0;
    bottom: 0;
    background-color: #ccc;
    transition: .4s;
    border-radius: 24px;
    width: 60px;
  }


  .toggle-switch label:before {
    position: absolute;
    content: "";
    height: 18px;  /* Reduced height */
    width: 18px;  /* Reduced width */
    left: 3px;  /* Adjusted position */
    bottom: 3px;  /* Adjusted position */
    background-color: white;
    transition: .4s;
    border-radius: 50%;
  }
  
  .toggle-switch input:checked + label {
    background-color: #3498db;
  }
  
  .toggle-switch input:focus + label {
    box-shadow: 0 0 1px #3498db;
  }
  
  .toggle-switch input:checked + label:before {
    transform: translateX(22px);  /* Reduced movement */
  }

/* Array Input Group */

  
.array-input-group label {
    flex: 1;
    max-width: 60%;
    text-align: left;
    font-weight: 600;
    color: #333;
    padding-right: 10px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}
  
.array-input-wrapper {
    width: 40%;
    max-width: 200px;
    display: flex;
    justify-content: space-between;
    gap: 5px; /* Adds space between multiple inputs */
}
  
/* Styles for array input */
.array-input-wrapper input[type="number"] {
    width: 100%;
    border: none;
    border-bottom: 2px solid #3498db;
    background-color: transparent;
    padding: 5px 0;
    font-size: 16px;
    color: #333;
    transition: all 0.3s ease;
    height: 30px;
    box-sizing: border-box;
    -moz-appearance: textfield;
}
  
  
/* Hover and focus styles */
.array-input-wrapper input[type="number"]:focus,
.array-input-wrapper input[type="number"]:hover {
    outline: none;
    border-bottom-color: #2980b9;
    background-color: rgba(52, 152, 219, 0.1);
}
```

#### src/CollapsibleSection.tsx
```typescript
import React, { useState } from 'react';

function CollapsibleSection({ title, children }) {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <div className="collapsible-section">
      <div 
        className={`collapsible-header ${isOpen ? 'open' : ''}`}
        onClick={() => setIsOpen(!isOpen)}
      >
        <h3>{title}</h3>
        <span className="arrow">{isOpen ? '▲' : '▼'}</span>
      </div>
      <div className={`collapsible-content ${isOpen ? 'open' : ''}`}>
        {children}
      </div>
    </div>
  );
}

export default CollapsibleSection;
```

#### src/CollapsibleTerminal.tsx
```typescript
// CollapsibleTerminal.tsx
import React, { useEffect, useRef, useState, useCallback } from 'react';
import { Terminal } from 'xterm';
import { FitAddon } from 'xterm-addon-fit';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import 'xterm/css/xterm.css';
import "./App.css";


interface CollapsibleTerminalProps {
    isOpen: boolean;
    previousSectionRef: React.RefObject<HTMLElement>;
    onToggle: () => void;
    isRunning: boolean;
}

const CollapsibleTerminal: React.FC<CollapsibleTerminalProps> = ({ isOpen, isRunning, previousSectionRef, onToggle }) => {
  const terminalRef = useRef<HTMLDivElement>(null);
  const terminalInstanceRef = useRef<Terminal | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const [terminalHeight, setTerminalHeight] = useState('300px');
  const [isDragging, setIsDragging] = useState(false);

  const fitTerminal = useCallback(() => {
    if (fitAddonRef.current) {
      setTimeout(() => {
        fitAddonRef.current?.fit();
        if (terminalInstanceRef.current) {
          terminalInstanceRef.current.refresh(0, terminalInstanceRef.current.rows - 1);
        }
      }, 0);
    }
  }, []);
  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    setIsDragging(true);
  }, []);
  
  const handleMouseMove = useCallback((e: MouseEvent) => {
    if (isDragging) {
      const newHeight = window.innerHeight - e.clientY;
      setTerminalHeight(`${Math.max(newHeight, 100)}px`);
      fitTerminal();
    }
  }, [isDragging, fitTerminal]);
  
  const handleMouseUp = useCallback(() => {
    setIsDragging(false);
    fitTerminal();
  }, [fitTerminal]);
  useEffect(() => {
    const updateTerminalHeight = () => {
      if (previousSectionRef.current && terminalRef.current) {
        const previousSectionBottom = previousSectionRef.current.getBoundingClientRect().bottom;
        const mainContentBottom = document.querySelector('.main-content')?.getBoundingClientRect().bottom || window.innerHeight;
        const newHeight = mainContentBottom - previousSectionBottom;
        setTerminalHeight(`${Math.max(newHeight, 300)}px`);
      }
    };

    if (isOpen) {
      updateTerminalHeight();
      window.addEventListener('resize', updateTerminalHeight);
    }

    return () => window.removeEventListener('resize', updateTerminalHeight);
  }, [isOpen, previousSectionRef, fitTerminal]);

  useEffect(() => {
    let unlistenOutput: (() => void) | undefined;
    let unlistenError: (() => void) | undefined;

    const initializeTerminal = async () => {
      if (terminalRef.current && !terminalInstanceRef.current && isOpen) {
        const newTerm = new Terminal({
          fontFamily: '"Courier New", monospace',
          fontSize: 14,
          lineHeight: 1.2,
          theme: {
            background: "rgb(47, 47, 47)",
          },
        });
        
        const fitAddon = new FitAddon();
        newTerm.loadAddon(fitAddon);
        newTerm.open(terminalRef.current);
        fitAddon.fit();

        terminalInstanceRef.current = newTerm;
        fitAddonRef.current = fitAddon;

        setTimeout(() => {
            fitTerminal();
          }, 0);

        newTerm.onData(e => {
          invoke('send_input', { input: e === '\r' ? '\n' : e });
        });
        
        await invoke('start_terminal');

        unlistenOutput = await listen('julia-output', (event: any) => {
            const output = event.payload as string;
            // Trim any leading whitespace and ensure a single newline at the end
            const trimmedOutput = output.trimStart().replace(/\s*$/, '\r\n');
            newTerm.write(trimmedOutput);
        });

        unlistenError = await listen('julia-error', (event: any) => {
            const error = event.payload as string;
            // Trim any leading whitespace and ensure a single newline at the end
            const trimmedError = error.trimStart().replace(/\s*$/, '\r\n');
            newTerm.write(`Error: ${trimmedError}`);
        });

        window.addEventListener('resize', () => fitAddon.fit());
      }
    };

    if (isOpen) {
      initializeTerminal();
    }

    return () => {
      if (terminalInstanceRef.current) {
        terminalInstanceRef.current.dispose();
        terminalInstanceRef.current = null;
      }
      if (unlistenOutput) unlistenOutput();
      if (unlistenError) unlistenError();
      window.removeEventListener('resize', () => fitAddonRef.current?.fit());
    };
  }, [isOpen, fitTerminal]);
  useEffect(() => {
    if (isOpen) {
      window.addEventListener('mousemove', handleMouseMove);
      window.addEventListener('mouseup', handleMouseUp);
    }
    return () => {
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseup', handleMouseUp);
    };
  }, [isOpen, handleMouseMove, handleMouseUp]);
  useEffect(() => {
    if (isOpen) {
      // Wait for any CSS transitions to complete
      setTimeout(() => {
        fitTerminal();
      }, 300); // Adjust this value based on your transition duration
    }
  }, [isOpen, fitTerminal]);

  return (
    <div className="collapsible-terminal"
         style={{height: isRunning ? (isOpen ? terminalHeight : '25px') : '0px'}}>
<div 
  className={`terminal-banner ${isDragging ? 'dragging' : ''}`} 
  onClick={onToggle}
  onMouseDown={handleMouseDown}
  style={{ cursor: 'ns-resize' }}
>
  {isOpen ? 'Running...' : 'Terminal (Click to open)'}
</div>
<div 
  className={`collapsible-terminal-content ${isOpen ? 'open' : ''}`}
  style={{ height: isOpen ? terminalHeight : '0px', transition: isDragging ? 'none' : 'height 0.3s ease-in-out' }}
>
  <div ref={terminalRef} style={{ height: '100%', width: '100%' }}></div>
</div>
    </div>
  );
};

export default CollapsibleTerminal;
/*
    <div 
      className={`collapsible-terminal ${isOpen ? 'open' : ''}`}
      style={{ height: isOpen ? terminalHeight : '0px' }}
    >
     {isOpen && (
        <div className="terminal-banner" onClick={onClose}>
          Running...
        </div>
      )}
      <div className="collapsible-terminal-content">
        <div ref={terminalRef} style={{ height: '100%', width: '100%' }}></div>
      </div>
    </div>
*/


```

#### src/Terminal.tsx
```typescript
import React, { useState, useEffect, useRef } from 'react';

interface TerminalProps {
  children?: React.ReactNode;
}

const Terminal: React.FC<TerminalProps> = ({ children }) => {
  const [output, setOutput] = useState<string[]>([]);
  const terminalRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (children) {
      setOutput([children.toString()]);
    }
  }, [children]);

  useEffect(() => {
    if (terminalRef.current) {
      terminalRef.current.scrollTop = terminalRef.current.scrollHeight;
    }
  }, [output]);

  return (
    <div
      ref={terminalRef}
      className="terminal"
      style={{
        backgroundColor: '#000',
        color: '#00ff00',
        fontFamily: 'monospace',
        padding: '10px',
        height: '100px',
        overflowY: 'scroll',
      }}
    >
      {output.map((line, index) => (
        <div key={index}>{line}</div>
      ))}
    </div>
  );
};

export default Terminal;
```

#### src/ParameterMenus.tsx
```typescript
import React from 'react';


interface ParameterMenuContentProps {
    jsonData: JsonData;
    setJsonData: React.Dispatch<React.SetStateAction<JsonData>>;
    ContentComponent: React.ComponentType<{
      jsonData: JsonData;
      setJsonData: React.Dispatch<React.SetStateAction<JsonData>>;
    }>;
  }
  
  const ParameterMenuContent: React.FC<ParameterMenuContentProps> = ({ 
    jsonData, 
    setJsonData, 
    ContentComponent
  }) => {
    return (
      <>
        <ContentComponent jsonData={jsonData} setJsonData={setJsonData} />
      </>
    );
  };

export default ParameterMenuContent;
```

#### src/menu_components.tsx
```typescript
import React from 'react';
import CustomDropdown from './CustomDropdown';
import LabelWithTooltip from './LabelWithToolTip';
import NumberInput from './NumberInput.tsx';
import ArrayInput from './ArrayInput.tsx';
import OptionsInput from './OptionsInput.tsx';
import ToggleInput from './ToggleInput.tsx';

interface ContentProps {
    jsonData: JsonData;
    setJsonData: React.Dispatch<React.SetStateAction<JsonData>>;
  }

const Content1: React.FC<ContentProps> = ({jsonData, setJsonData}) => (
    <>
    <p className="preset-item">The presearch estimates the mass tolerance and library-to-empirical retention time alignment
                for each raw file based on a random sampling of scans
    </p>
    <div className="parameters-container">
        <div className="parameter-columns">
        <div className="parameter-column">
            <ToggleInput 
                            label="Min Index Search Score"
                            jsonData={jsonData}
                            setJsonData={setJsonData}
                            path={['parameter3']}
            />
            <NumberInput
            label="Min Index Search Score"
            jsonData={jsonData}
            setJsonData={setJsonData}
            path={['presearch_params', 'min_index_search_score']}
            step={1}
            allowDecimals={false}
            />
            <OptionsInput
                label="Parameter 1"
                description="This is a description for urmom."
                options={[
                { value: '', label: 'Select an Option' },
                { value: 'option1', label: 'Option 1' },
                { value: 'option2', label: 'Option 2' },
                ]}
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['parameter1']}
                placeholder="Select an onion"
            />
            <NumberInput
                label="Mass Tolerance"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['presearch_params', 'mass_tolerance']}
                step={0.1}
                allowDecimals={true}
                />
            <ArrayInput
            label="Min top N of M: N/M"
            jsonData={jsonData}
            setJsonData={setJsonData}
            path={['presearch_params', 'min_topn_of_m']}
            />
        </div>
        </div>
    </div>
    </>
);

const PresearchParams: React.FC<ContentProps> = ({jsonData, setJsonData}) => (
    <>
    <p className="preset-item">The presearch estimates the mass tolerance and library-to-empirical retention time alignment
                for each raw file based on a random sampling of scans
    </p>
    <div className="parameters-container">
        <div className="parameter-columns">
        <div className="parameter-column">
            <ToggleInput 
                            label="Filter by Rank"
                            jsonData={jsonData}
                            setJsonData={setJsonData}
                            path={['presearch_params','filter_by_rank']}
            />
            <ToggleInput 
                label="Filter by Frag Count"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['presearch_params','filter_by_frag_count']}
            />
            <NumberInput
                label="Fragment Error Quantile"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['presearch_params', 'frag_err_quantile']}
                step={0.01}
                allowDecimals={true}
                min={0}
                max={1}
            />
            <NumberInput
                label="Fragment Tol. (ppm)"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['presearch_params', 'frag_tol_ppm']}
                step={1.0}
                allowDecimals={true}
                min={0}
                max={100}
            />
            <NumberInput
                label="Max Best Fragment Rank"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['presearch_params', 'max_best_rank']}
                step={1}
                allowDecimals={false}
                min={1}
                max={100}
            />
            <NumberInput
                label="Max Fragment Rank"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['presearch_params', 'max_frag_rank']}
                step={1}
                allowDecimals={false}
                min={1}
                max={100}
            />
            <NumberInput
                label="Max Presearch Iter."
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['presearch_params', 'max_presearch_iters']}
                step={1}
                allowDecimals={false}
                min={1}
                max={20}
            />
            <NumberInput
                label="Max Qval"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['presearch_params', 'max_qval']}
                step={0.01}
                allowDecimals={true}
                min={0}
                max={1}
            />
            <NumberInput
                label="Min Fragment Count"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['presearch_params', 'min_frag_count']}
                step={1}
                allowDecimals={false}
                min={1}
                max={100}
            />
            <NumberInput
                label="Min Index Search Score"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['presearch_params', 'min_index_search_score']}
                step={1}
                allowDecimals={false}
                min={1}
                max={255}
            />
            <NumberInput
                label="Min Log2 Matched Ratio"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['presearch_params', 'min_log2_matched_ratio']}
                step={0.1}
                allowDecimals={true}
            />
            <NumberInput
                label="Min Samples"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['presearch_params', 'min_samples']}
                step={1}
                allowDecimals={false}
                min={1}
            />
            <NumberInput
                label="Min Spectral Contrast"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['presearch_params', 'min_spectral_contrast']}
                step={0.1}
                allowDecimals={true}
                min={0}
                max={1}
            />
            <ArrayInput
                label="Min top N of M: N/M"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['presearch_params', 'min_topn_of_m']}
            />
            <NumberInput
                label="N fragment isotopes"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['presearch_params', 'n_frag_isotopes']}
                step={1}
                allowDecimals={false}
                min={0}
                max={3}
            />
        </div>
        </div>
    </div>
    </>
);

const FirstSearchParams: React.FC<ContentProps> = ({jsonData, setJsonData}) => (
    <>
    <p className="preset-item">The presearch estimates the mass tolerance and library-to-empirical retention time alignment
                for each raw file based on a random sampling of scans
    </p>
    <div className="parameters-container">
        <div className="parameter-columns">
        <div className="parameter-column">
            <ToggleInput 
                            label="Filter by Rank"
                            jsonData={jsonData}
                            setJsonData={setJsonData}
                            path={['first_search_params','filter_by_rank']}
            />
            <ToggleInput 
                label="Filter by Frag Count"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['first_search_params','filter_on_frag_count']}
            />
            <NumberInput
                label="Max Best Fragment Rank"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['first_search_params', 'max_best_rank']}
                step={1}
                allowDecimals={false}
                min={1}
                max={100}
            />
            <NumberInput
                label="Max Iter Probit"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['first_search_params', 'max_iter_probit']}
                step={1}
                allowDecimals={false}
                min={1}
                max={100}
            />
            <NumberInput
                label="Max Precursors Passing"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['first_search_params', 'max_precursors_passing']}
                step={1}
                allowDecimals={false}
                min={1}
            />
            <NumberInput
                label="Max Qval Filter"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['first_search_params', 'max_qval_filter']}
                step={0.01}
                allowDecimals={true}
                min={0}
                max={1}
            />
            <NumberInput
                label="Max Qval Probit Rescore"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['first_search_params', 'max_qval_probit_rescore']}
                step={0.01}
                allowDecimals={true}
                min={0}
                max={1}
            />
            <NumberInput
                label="Min Fragment Count"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['first_search_params', 'min_frag_count']}
                step={1}
                allowDecimals={false}
                min={1}
                max={100}
            />
            <NumberInput
                label="Min Index Search Score"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['first_search_params', 'min_index_search_score']}
                step={1}
                allowDecimals={false}
                min={1}
                max={255}
            />
            <NumberInput
                label="Min Log2 Matched Ratio"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['first_search_params', 'min_log2_matched_ratio']}
                step={0.1}
                allowDecimals={true}
            />
            <NumberInput
                label="Min Spectral Contrast"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['first_search_params', 'min_spectral_contrast']}
                step={0.1}
                allowDecimals={true}
                min={0}
                max={1}
            />
            <ArrayInput
                label="Min top N of M: N/M"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['first_search_params', 'min_topn_of_m']}
            />
            <NumberInput
                label="N fragment isotopes"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['first_search_params', 'n_frag_isotopes']}
                step={1}
                allowDecimals={false}
                min={0}
                max={3}
            />
            <NumberInput
                label="N Train Rounds Probit"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['first_search_params', 'n_train_rounds_probit']}
                step={1}
                allowDecimals={false}
                min={0}
                max={3}
            />
        </div>
        </div>
    </div>
    </>
);

const QuantSearchParams: React.FC<ContentProps> = ({jsonData, setJsonData}) => (
    <>
    <p className="preset-item">The presearch estimates the mass tolerance and library-to-empirical retention time alignment
                for each raw file based on a random sampling of scans
    </p>
    <div className="parameters-container">
        <div className="parameter-columns">
        <div className="parameter-column">
            <ToggleInput 
                            label="Filter by Rank"
                            jsonData={jsonData}
                            setJsonData={setJsonData}
                            path={['quant_search_params','filter_by_rank']}
            />
            <ToggleInput 
                label="Filter by Frag Count"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['quant_search_params','filter_on_frag_count']}
            />
            <NumberInput
                label="Max Best Fragment Rank"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['quant_search_params', 'max_best_rank']}
                step={1}
                allowDecimals={false}
                min={1}
                max={100}
            />
            <NumberInput
                label="Min Fragment Count"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['quant_search_params', 'min_frag_count']}
                step={1}
                allowDecimals={false}
                min={1}
                max={100}
            />
            <NumberInput
                label="Min Log2 Matched Ratio"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['quant_search_params', 'min_log2_matched_ratio']}
                step={0.1}
                allowDecimals={true}
            />
            <NumberInput
                label="Min Spectral Contrast"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['quant_search_params', 'min_spectral_contrast']}
                step={0.1}
                allowDecimals={true}
                min={0}
                max={1}
            />
            <ArrayInput
                label="Min top N of M: N/M"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['quant_search_params', 'min_topn_of_m']}
            />
            <NumberInput
                label="N fragment isotopes"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['quant_search_params', 'n_frag_isotopes']}
                step={1}
                allowDecimals={false}
                min={0}
                max={3}
            />
        </div>
        </div>
    </div>
    </>
);

const DeconvolutionParams: React.FC<ContentProps> = ({jsonData, setJsonData}) => (
    <>
    <p className="preset-item">The presearch estimates the mass tolerance and library-to-empirical retention time alignment
                for each raw file based on a random sampling of scans
    </p>
    <div className="parameters-container">
        <div className="parameter-columns">
        <div className="parameter-column">
            <NumberInput
                label="Lasso lambda"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['deconvolution_params', 'lambda']}
                step={1000.0}
                allowDecimals={true}
                min={0}
            />
            <NumberInput
                label="Huber Delta"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['deconvolution_params', 'huber_delta']}
                step={1000.0}
                allowDecimals={true}
                min={100.0}
            />
            <NumberInput
                label="Huber Delta Prop."
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['deconvolution_params', 'huber_delta_prop']}
                step={1000.0}
                allowDecimals={true}
                min={0.01}
            />
            <NumberInput
                label="Max Iter Newton"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['deconvolution_params', 'max_iter_newton']}
                step={1}
                allowDecimals={false}
                min={1}
                max={10000}
            />
            <NumberInput
                label="Max Iter Bisection"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['deconvolution_params', 'max_iter_bisection']}
                step={1}
                allowDecimals={false}
                min={1}
                max={10000}
            />
            <NumberInput
                label="Max Iter Outer"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['deconvolution_params', 'max_iter_outer']}
                step={1}
                allowDecimals={false}
                min={1}
                max={10000}
            />
            <NumberInput
                label="Accuracy Newton"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['deconvolution_params', 'acuracy_newton']}
                step={10}
                allowDecimals={true}
                min={1}
                max={100000}
            />
            <NumberInput
                label="Accuracy Bisection"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['deconvolution_params', 'acuracy_bisection']}
                step={10}
                allowDecimals={true}
                min={1}
                max={100000}
            />
            <NumberInput
                label="Max diff."
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['deconvolution_params', 'max_diff']}
                step={10}
                allowDecimals={true}
                min={1}
                max={100000}
            />
        </div>
        </div>
    </div>
    </>
);

const NormalizationParams: React.FC<ContentProps> = ({jsonData, setJsonData}) => (
    <>
    <p className="preset-item">The presearch estimates the mass tolerance and library-to-empirical retention time alignment
                for each raw file based on a random sampling of scans
    </p>
    <div className="parameters-container">
        <div className="parameter-columns">
        <div className="parameter-column">
            <NumberInput
                label="Max Qval"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['normalization_params', 'max_q_value']}
                step={0.01}
                allowDecimals={true}
                min={0}
                max={1}
            />
            <NumberInput
                label="Min Points Above FWHM"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['normalization_params', 'min_points_above_FWHM']}
                step={1}
                allowDecimals={false}
                min={0}
                max={100}
            />
            <NumberInput
                label="N RT Bins"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['normalization_params', 'n_rt_bins']}
                step={1}
                allowDecimals={false}
                min={0}
                max={1000}
            />
            <NumberInput
                label="N Spline Knots"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['normalization_params', 'spline_n_knots']}
                step={1}
                allowDecimals={false}
                min={0}
                max={100}
            />
        </div>
        </div>
    </div>
    </>
);

const IrtMappingParams: React.FC<ContentProps> = ({jsonData, setJsonData}) => (
    <>
    <p className="preset-item">The presearch estimates the mass tolerance and library-to-empirical retention time alignment
                for each raw file based on a random sampling of scans
    </p>
    <div className="parameters-container">
        <div className="parameter-columns">
        <div className="parameter-column">
            <NumberInput
                label="Smoothing Bandwidth"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['irt_mapping_params', 'bandwidth']}
                step={0.05}
                allowDecimals={true}
                min={0}
                max={10}
            />
            <NumberInput
                label="Min Probability"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['irt_mapping_params', 'min_prob']}
                step={0.01}
                allowDecimals={true}
                min={0}
                max={1}
            />
            <NumberInput
                label="Sigma Tol."
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['irt_mapping_params', 'n_sigma_tol']}
                step={0.1}
                allowDecimals={true}
                min={0}
                max={100}
            />
            <NumberInput
                label="N Bins"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['irt_mapping_params', 'n_bins']}
                step={1}
                allowDecimals={false}
                min={3}
                max={100}
            />
        </div>
        </div>
    </div>
    </>
);

const SummarizeFirstSearchParams: React.FC<ContentProps> = ({jsonData, setJsonData}) => (
    <>
    <p className="preset-item">The presearch estimates the mass tolerance and library-to-empirical retention time alignment
                for each raw file based on a random sampling of scans
    </p>
    <div className="parameters-container">
        <div className="parameter-columns">
        <div className="parameter-column">
            <NumberInput
                label="Max Precursors"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['summarize_first_search_params', 'max_precursors']}
                step={1000}
                allowDecimals={false}
                min={1}
            />
            <NumberInput 
                label="Max Prob To Impute RT"
                jsonData={jsonData}
                setJsonData={setJsonData}
                path={['summarize_first_search_params', 'max_prob_to_impute']}
                step={0.01}
                allowDecimals={true}
                min={0}
                max={1}
            />
        </div>
        </div>
    </div>
    </>
);
export {PresearchParams, 
        FirstSearchParams, 
        QuantSearchParams, 
        DeconvolutionParams, 
        NormalizationParams, 
        IrtMappingParams,
        SummarizeFirstSearchParams,
        Content1};


  
```

#### src/NumberInput.tsx
```typescript
// NumberInput.tsx
import React, { useState } from 'react';
import { getNestedValue, setNestedValue } from './utils';
import "./App.css";
import "./MenuComponents.css";
import LabelWithTooltip from './LabelWithToolTip';

interface NumberInputProps {
  label: string;
  description?: string;
  jsonData: JsonData;
  setJsonData: React.Dispatch<React.SetStateAction<JsonData>>;
  path: string[];
  step?: number;
  allowDecimals?: boolean;
  min?: number;
  max?: number;
}


const NumberInput: React.FC<NumberInputProps> = ({ 
  label, 
  description,
  jsonData, 
  setJsonData, 
  path, 
  step = 1, 
  allowDecimals = false,
  min = Number.MIN_SAFE_INTEGER,
  max = Number.MAX_SAFE_INTEGER
}) => {
  const initialValue = getNestedValue(jsonData, path) || 0;
  const [inputValue, setInputValue] = useState(initialValue.toString());


  const handleChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const newValue = event.target.value;
    setInputValue(newValue);

    if (newValue === '' || (allowDecimals ? !isNaN(parseFloat(newValue)) : !isNaN(parseInt(newValue)))) {
      const numericValue = allowDecimals ? parseFloat(newValue) : parseInt(newValue);
      
      if (!isNaN(numericValue) && numericValue >= min && numericValue <= max) {
        setJsonData((prevData) => {
          const newData = { ...prevData };
          setNestedValue(newData, path, numericValue);
          return newData;
        });
      }
    }
  };

  const handleBlur = () => {
    let finalValue = initialValue;
    if (inputValue !== '') {
      const numericValue = allowDecimals ? parseFloat(inputValue) : parseInt(inputValue);
      if (!isNaN(numericValue)) {
        finalValue = Math.min(Math.max(numericValue, min), max);
      }
    }
    setInputValue(finalValue.toString());
    setJsonData((prevData) => {
      const newData = { ...prevData };
      setNestedValue(newData, path, finalValue);
      return newData;
    });
  };

  return (
    <div className="number-input-group">
          <LabelWithTooltip 
          htmlFor="text"
          label={label} 
          description={description}
          />
        <div className="number-input-wrapper">
          <input
            type="number"
            value={inputValue}
            onChange={handleChange}
            onBlur={handleBlur}
            step={step}
            min={min}
            max={max}
            {...(allowDecimals ? {} : { pattern: "\\d*" })}
          />
        </div>
    </div>
  );
};

export default NumberInput;

/*
interface NumberInputProps {
  label: string;
  value: number;
  onChange: (value: number) => void;
}

const NumberInput: React.FC<NumberInputProps> = ({ label, value, onChange }) => (
  <div>
    <label>{label}</label>
    <input
      type="number"
      value={value}
      onChange={(e) => onChange(Number(e.target.value))}
    />
  </div>
);

export default NumberInput;
*/
```

#### src/ArrayInput.tsx
```typescript
import React from 'react';
import { getNestedValue, setNestedValue } from './utils';
import LabelWithTooltip from './LabelWithToolTip';
interface ArrayInputProps {
  label: string;
  description?: string;
  jsonData: JsonData;
  setJsonData: React.Dispatch<React.SetStateAction<JsonData>>;
  path: string[];
}

const ArrayInput: React.FC<ArrayInputProps> = ({ label, description, jsonData, setJsonData, path }) => {
  const value = getNestedValue(jsonData, path) || []; // Default to empty array if undefined

  const handleChange = (index: number, newValue: number) => {
    setJsonData((prevData) => {
      const newData = JSON.parse(JSON.stringify(prevData)); // Deep clone
      const newArray = [...getNestedValue(newData, path)];
      newArray[index] = newValue;
      setNestedValue(newData, path, newArray);
      return newData;
    });
  };

  return (
    <div>
      <div className="array-input-group">
      <LabelWithTooltip 
            htmlFor="text"
            label={label} 
            description={description}
            />
      <div className="array-input-wrapper">
      {value.map((val: number, index: number) => (
        <input
          key={index}
          type="number"
          value={val}
          onChange={(e) => handleChange(index, Number(e.target.value))}
        />
      ))}
      </div>
      </div>
    </div>
  );
};

export default ArrayInput;
```

#### src/ToggleInput.tsx
```typescript
// NumberInput.tsx
import React, { useState } from 'react';
import { getNestedValue, setNestedValue } from './utils';
import "./App.css";
import LabelWithTooltip from './LabelWithToolTip';
import "./MenuComponents.css";
interface ToggleInputProps {
  label: string;
  description?: string;
  jsonData: JsonData;
  setJsonData: React.Dispatch<React.SetStateAction<JsonData>>;
  path: string[];
}


const ToggleInput: React.FC<ToggleInputProps> = ({ 
  label, 
  description,
  jsonData, 
  setJsonData, 
  path, 
}) => {

  const value = getNestedValue(jsonData, path) || false;

  const handleChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const newValue = event.target.checked;
    setJsonData((prevData) => {
      const newData = { ...prevData };
      setNestedValue(newData, path, newValue);
      return newData;
    });
  };
  return (
    <div className="toggle-input-group">
            <LabelWithTooltip 
            htmlFor="text"
            label={label} 
            description={description}
            />
      <div className="toggle-input-wrapper">
        <div className="toggle-switch">
          <input
            id={`toggle-${path.join('-')}`}
            type="checkbox"
            checked={value}
            onChange={handleChange}
          />
          <label htmlFor={`toggle-${path.join('-')}`}></label>
        </div>
      </div>
    </div>

  );
};

export default ToggleInput;

/*

*/
```

#### src/OptionsInput.tsx
```typescript
import React, { useState, useRef, useEffect } from 'react';
import { getNestedValue, setNestedValue } from './utils';
import LabelWithTooltip from './LabelWithToolTip';
import "./MenuComponents.css"

interface OptionsInputProps {
  label: string;
  description?: string;
  options: { value: string; label: string }[];
  jsonData: JsonData;
  setJsonData: React.Dispatch<React.SetStateAction<JsonData>>;
  path: string[];
  placeholder?: string;
}

const OptionsInput: React.FC<OptionsInputProps> = ({ label,description,options, jsonData, setJsonData, path, placeholder }) => {
  const [isOpen, setIsOpen] = useState(false);
  const [selectedValue, setSelectedValue] = useState<string>('');
  const dropdownRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const currentValue = getNestedValue(jsonData, path);
    setSelectedValue(currentValue || '');
  }, [jsonData, path]);

  const handleToggle = () => setIsOpen(!isOpen);

  const handleOptionClick = (newValue: string) => {
    setJsonData((prevData) => {
      const newData = { ...prevData };
      setNestedValue(newData, path, newValue);
      return newData;
    });
    setIsOpen(false);
  };

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    };
    document.addEventListener('mousedown', handleClickOutside);
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, []);

  const selectedOption = options.find(option => option.value === selectedValue);

  return (
    <div className="options-input-group">
      <LabelWithTooltip 
            htmlFor="text"
            label={label} 
            description={description}
            />
      <div className="custom-dropdown" ref={dropdownRef}>
        <div className="dropdown-header" onClick={handleToggle}>
        {selectedOption ? selectedOption.label : placeholder || 'Select an option'}
        <span className="dropdown-arrow"></span>
        </div>
        {isOpen && (
        <div className="dropdown-options">
            {options.map((option) => (
                <li
                    key={option.value}
                    onClick={() => handleOptionClick(option.value)}
                    className="dropdown-option"
                >
                {option.label}
            </li>
            ))}
        </div>
        )}
      </div>
      </div>
)};
export default OptionsInput;
```

#### src/LabelWithToolTip.tsx
```typescript
import React from 'react';

function LabelWithTooltip({ htmlFor, label, description }) {
  return (
    <div className="parameter-label">
      <label htmlFor={htmlFor}>{label}</label>
      <span className="tooltip-trigger">ⓘ</span>
      <div className="tooltip">{description}</div>
    </div>
  );
}

export default LabelWithTooltip;
```

#### src/utils.tsx
```typescript
export function getNestedValue(obj: any, path: string[]) {
    return path.reduce((prev, curr) => prev && prev[curr], obj);
  }
  
export function setNestedValue(obj: any, path: string[], value: any) {
const [head, ...rest] = path;
if (rest.length === 0) {
    obj[head] = value;
} else {
    if (!(head in obj)) {
    obj[head] = {};
    }
    setNestedValue(obj[head], rest, value);
}
}
```

#### src/public/defaultJsonData.tsx
```typescript
const defaultJsonData: JsonData = {
    "expected_matches": 1000000,
    "isotope_err_bounds":[1, 0],
    "irt_err_sigma": 4,
    "choose_most_intense": false,
    "quadrupole_isolation_width": 8.0036, 
    "presearch_params":
    {
        "min_index_search_score": 22,
        "filter_on_frag_count": false,
        "filter_by_rank": false,
        "n_frag_isotopes": 1,
        "min_frag_count": 7,
        "min_log2_matched_ratio": 1.5,
        "min_spectral_contrast": 0.9,
        "min_topn_of_m": [3, 3],
        "max_best_rank": 1,
        "sample_rate": 0.02,
        "frag_tol_ppm": 30.0,
        "max_qval": 0.01,
        "min_samples": 3500,
        "max_frag_rank": 5,
        "frag_err_quantile": 0.01,
        "max_presearch_iters": 10
    },
    "first_search_params":
    {
        "min_index_search_score": 15,
        "filter_on_frag_count": false,
        "min_frag_count": 4,
        "filter_by_rank": false,
        "min_topn_of_m": [2, 3], 
        "n_frag_isotopes": 2,      
        "min_log2_matched_ratio": 0.0,
        "min_spectral_contrast": 0.5,
        "max_best_rank": 1,
        "n_train_rounds_probit": 2,
        "max_iter_probit":20,
        "max_q_value_probit_rescore": 0.01,
        "max_q_value_filter": 1.0,
        "max_precursors_passing": 500000
    },
    "summarize_first_search_params":
    {
        "max_precursors": 500000,
        "max_prob_to_impute": 0.5
    },
    "quant_search_params":
    {
        "filter_by_rank": false,
        "filter_on_frag_count": false,
        "WH_smoothing_strength": 1.0,
        "min_frag_count": 1,
        "min_log2_matched_ratio": -1.7,
        "min_spectral_contrast": 0.0,
        "min_topn_of_m": [1, 3],
        "n_frag_isotopes": 2,
        "max_best_rank": 3
    },
    "frag_tol_params":
    {
        "frag_tol_quantile": 0.99,
        "frag_tol_bounds":[10.0, 40.0]
    },
    "irt_mapping_params": 
    {
        "n_bins": 200,
        "bandwidth": 0.25,
        "n_sigma_tol":4,
        "min_prob": 0.95
    },
    "integration_params":
    {
        "n_quadrature_nodes": 100,
        "intensity_filter_threshold": 0.01
    },
    "deconvolution_params":         
    {
        "lambda": 0.0,
        "huber_delta": 0,
        "huber_delta_prop": 1.0,
        "max_iter_newton": 100,
        "max_iter_bisection": 100,
        "max_iter_outer": 100,
        "accuracy_newton": 100,
        "accuracy_bisection": 100,
        "max_diff": 0.01
    },
    "qc_plot_params":
    {
        "n_files_per_plot": 12
    },
    "normalization_params":
    {
        "n_rt_bins": 100,
        "spline_n_knots": 7,
        "max_q_value": 0.01,
        "min_points_above_FWHM": 2
    },
    "benchmark_params":
    {   
        "results_folder": "/Users/n.t.wamsley/Desktop/testresults"
    },
    "library_folder": "/Users/n.t.wamsley/RIS_temp/ASMS_2024/ASTRAL_THREE_PROTEOME/unispec_chronologer_1mc_1var_by_052724/spec_lib/pioneer_lib",
    "ms_data_dir":"/Users/n.t.wamsley/TEST_DATA/PXD046444/arrow/astral_test"
};
export default defaultJsonData;
```

---

## Build and Run Instructions

### Prerequisites
- Node.js (v16+)
- Rust (latest stable)
- Julia (with Pioneer.jl installed)
- pnpm or npm package manager

### Installation Steps

1. **Clone the project structure**
2. **Install frontend dependencies:**
   ```bash
   pnpm install
   # or
   npm install
   ```

3. **Install Rust dependencies:**
   ```bash
   cd src-tauri
   cargo build
   ```

4. **Configure Julia and Pioneer.jl paths:**
   - Update paths in `src-tauri/src/terminal.rs` line 176-178
   - Update default paths in `src/public/defaultJsonData.tsx`

### Development
```bash
pnpm tauri dev
# or
npm run tauri dev
```

### Build for Production
```bash
pnpm tauri build
# or
npm run tauri build
```

## Key Implementation Notes

1. **Terminal Integration**: The app uses xterm.js for terminal emulation with custom Rust backend for Julia process management
2. **State Management**: React state with nested object updates for complex parameter configuration
3. **IPC Communication**: Tauri's invoke/listen pattern for bidirectional communication
4. **Process Lifecycle**: Julia runs with 12 threads and automatic stdout/stderr streaming
5. **UI Components**: Custom input components with validation and real-time updates
6. **Configuration Export**: JSON configuration saved to desktop for Pioneer.jl consumption

## Future Enhancements Considerations
- Add parameter validation rules
- Implement configuration presets loading
- Add progress indicators for long-running analyses
- Implement result visualization
- Add multi-file batch processing
- Enhance error handling and recovery
