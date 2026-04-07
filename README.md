# Llama Monitor

Web dashboard for managing [llama.cpp](https://github.com/ggerganov/llama.cpp) servers with real-time GPU monitoring.

![screenshot](docs/screenshot.png)

## Features

- **Server Management** -- Start/stop llama.cpp server with configurable parameters
- **Real-time GPU Monitoring** -- Temperature, load, VRAM, power, clock speeds (AMD ROCm + NVIDIA)
- **Inference Metrics** -- Prompt/generation speed, KV cache usage, slot status via Prometheus endpoint
- **Customizable Presets** -- Create, edit, delete model presets with all llama.cpp parameters; persisted to disk
- **Model Auto-Discovery** -- Scan a directory for .gguf files with parsed model names and quant types
- **Integrated Chat** -- Streaming chat UI with reasoning/thinking block support
- **PWA Support** -- Installable as a standalone app on mobile and desktop

## Supported Hardware

| Vendor | Tool | Detection |
|--------|------|-----------|
| AMD | `rocm-smi` | Auto-detected |
| NVIDIA | `nvidia-smi` | Auto-detected |

GPU backend is auto-detected at startup. Override with `--gpu-backend rocm|nvidia|none`.

## Installation

### From Source

```bash
# Prerequisites: Rust 1.82+, llama.cpp (llama-server binary)
git clone <repo-url> && cd llama-monitor
cargo build --release
```

The binary is at `target/release/llama-monitor`. It's a single self-contained executable (frontend is embedded at compile time).

### Dependencies

- **llama.cpp** -- `llama-server` binary (with `--metrics` and `--jinja` support)
- **GPU monitoring** (optional) -- `rocm-smi` (AMD) or `nvidia-smi` (NVIDIA)

## Quick Start

```bash
# Basic usage (uses default llama-server path)
./llama-monitor

# Specify llama-server location and model directory
./llama-monitor \
  --llama-server-path /usr/local/bin/llama-server \
  --models-dir /path/to/models \
  --port 7778
```

Open `http://localhost:7778` in your browser.

## CLI Reference

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--llama-server-path` | `-s` | (hardcoded) | Path to `llama-server` binary |
| `--llama-server-cwd` | | (hardcoded) | Working directory for llama-server |
| `--port` | `-p` | `7778` | Monitor web UI port |
| `--models-dir` | `-m` | (none) | Directory to scan for .gguf model files |
| `--presets-file` | | `~/.config/llama-monitor/presets.json` | Custom presets file location |
| `--gpu-backend` | | `auto` | Force GPU backend: `auto`, `rocm`, `nvidia`, `none` |
| `--gpu-arch` | | (from config) | GPU architecture for ROCm (e.g. `gfx906`, `gfx1100`, `auto`) |
| `--gpu-devices` | | (from config) | Visible GPU device indices (e.g. `0,1,2,3`) |
| `--help` | `-h` | | Show help |

## Configuration

### GPU Environment

GPU architecture and device selection are configurable via the **GPU Environment** panel in the Server tab, or via CLI flags. Settings are persisted to `~/.config/llama-monitor/gpu-env.json`.

On startup, the monitor auto-detects GPUs via `rocminfo` (AMD) or `nvidia-smi` (NVIDIA) and pre-selects the detected architecture. You can override this in the UI dropdown.

**Supported AMD architectures:**

| GFX ID | HSA Version | GPU Models |
|--------|------------|------------|
| `auto` | (none) | Let ROCm detect natively |
| `gfx900` | 9.0.0 | Vega 56/64, MI25 |
| `gfx906` | 9.0.6 | Radeon VII, MI50/60 |
| `gfx908` | 9.0.8 | MI100 |
| `gfx90a` | 9.0.10 | MI210, MI250, MI250X |
| `gfx942` | 9.4.2 | MI300, MI300X, MI300A |
| `gfx1010` | 10.1.0 | RX 5700/5700 XT |
| `gfx1030` | 10.3.0 | RX 6800/6900 XT |
| `gfx1100` | 11.0.0 | RX 7900 XTX/XT |
| `gfx1101` | 11.0.1 | RX 7800/7700 XT |
| `gfx1102` | 11.0.2 | RX 7600 |
| `gfx1150` | 11.5.0 | Ryzen AI APU (890M) |
| `gfx1200` | 12.0.0 | RX 9070 XT |

When a specific architecture is selected, `HSA_OVERRIDE_GFX_VERSION` and `HCC_AMDGPU_TARGET` are set accordingly. When set to `auto`, these are not set, letting ROCm detect natively.

### Presets

Presets are stored in `~/.config/llama-monitor/presets.json` (or the path specified by `--presets-file`). The file is created automatically with built-in defaults on first run.

Each preset contains:

```json
{
  "id": "default-1",
  "name": "Model Name (context details)",
  "model_path": "/path/to/model.gguf",
  "context_size": 256000,
  "ctk": "f16",
  "ctv": "f16",
  "tensor_split": "7,8,8,8",
  "batch_size": 2048,
  "ubatch_size": 2048,
  "no_mmap": true,
  "ngram_spec": false,
  "parallel_slots": 1
}
```

Manage presets through the web UI (New / Edit / Delete buttons) or edit the JSON file directly.

### Parameter Reference

All parameters map directly to llama-server CLI flags. The preset editor groups them into collapsible sections.

**Model & Memory**

| Parameter | Flag | Description |
|-----------|------|-------------|
| `gpu_layers` | `-ngl` | Layers to offload to GPU (default: 99 = all). Lower values keep layers on CPU. |
| `no_mmap` | `--no-mmap` | Disable memory-mapped I/O. Required for models >16GB or network storage. |
| `mlock` | `--mlock` | Lock model in RAM, prevent OS swapping. Requires enough physical RAM. |

**Context & KV Cache**

| Parameter | Flag | Description |
|-----------|------|-------------|
| `context_size` | `-c` | Max context length in tokens. Auto-enables YaRN above 262,144. |
| `ctk` / `ctv` | `-ctk` / `-ctv` | KV cache quantization: `f16`, `q8_0`, `turbo3` (4.6x compression) |
| `flash_attn` | `-fa` | Flash Attention: `auto`, `on`, `off` |

**Batching & Slots**

| Parameter | Flag | Description |
|-----------|------|-------------|
| `batch_size` | `-b` | Logical batch size for prompt evaluation |
| `ubatch_size` | `-ub` | Physical micro-batch size (usually same as batch_size) |
| `parallel_slots` | `-np` | Concurrent inference slots. Each reserves `context_size` tokens of KV cache. |

**GPU Distribution**

| Parameter | Flag | Description |
|-----------|------|-------------|
| `tensor_split` | `-ts` | VRAM distribution across GPUs (e.g. `7,8,8,8`). Empty for single GPU. |
| `split_mode` | `--split-mode` | Multi-GPU strategy: `layer` (default) or `row` |
| `main_gpu` | `-mg` | GPU index for computation (0-based) |

**Threading**

| Parameter | Flag | Description |
|-----------|------|-------------|
| `threads` | `-t` | CPU threads for generation (empty = all CPUs) |
| `threads_batch` | `-tb` | CPU threads for batch processing (empty = same as threads) |

**Rope Scaling**

| Parameter | Flag | Description |
|-----------|------|-------------|
| `rope_scaling` | `--rope-scaling` | Method: `yarn`, `linear`, `none`. Auto-yarn for context >262144. |
| `rope_freq_base` | `--rope-freq-base` | Base frequency (model default if not set) |
| `rope_freq_scale` | `--rope-freq-scale` | Frequency scale (auto-calculated for YaRN) |

**Speculative Decoding**

| Parameter | Flag | Description |
|-----------|------|-------------|
| `ngram_spec` | `--spec-type ngram-mod` | Enable ngram speculative decoding (+47% on transformers) |
| `spec_ngram_size` | `--spec-ngram-size-n` | N-gram lookup size (default: 24) |
| `draft_min` / `draft_max` | `--draft-min` / `--draft-max` | Draft token range (default: 8-24) |
| `draft_model` | `-md` | Draft model path for model-based speculation |

**Advanced**

| Parameter | Flag | Description |
|-----------|------|-------------|
| `seed` | `-s` | Random seed (-1 = random) |
| `system_prompt_file` | `--system-prompt-file` | Path to system prompt text file |
| `extra_args` | | Arbitrary additional CLI flags (space-separated) |

## Web UI

### Server Tab
Configure and launch llama-server. Select from presets or customize parameters. View server logs in real-time.

### Monitor Tab
Live dashboard showing:
- **Inference metrics** -- prompt/generation tokens per second, context usage, slot status
- **GPU table** -- per-GPU temperature, load, VRAM, power (with capping alerts), clock speeds

### Chat Tab
Streaming chat interface that proxies to the running llama-server's `/v1/chat/completions` endpoint. Supports reasoning/thinking blocks and Markdown rendering.

## Architecture

```
src/
  main.rs              -- Entry point: CLI parsing, wiring, tokio::main
  cli.rs               -- Clap argument definitions
  config.rs            -- AppConfig resolved from CLI args
  state.rs             -- Shared AppState (Arc<Mutex<...>>)
  gpu/
    mod.rs             -- GpuMetrics, GpuBackend trait, auto-detection
    rocm.rs            -- AMD ROCm via rocm-smi JSON
    nvidia.rs          -- NVIDIA via nvidia-smi CSV
    dummy.rs           -- No-op backend for headless/testing
  llama/
    metrics.rs         -- Prometheus text format parser
    server.rs          -- Subprocess management (start/stop)
    poller.rs          -- Async polling loop for /health, /metrics, /slots
  presets/
    mod.rs             -- ModelPreset, CRUD, file persistence
  models/
    mod.rs             -- GGUF file discovery and filename parsing
  web/
    mod.rs             -- Warp route composition
    api.rs             -- REST API handlers
    ws.rs              -- WebSocket real-time metrics push
    static_assets.rs   -- Embedded frontend (include_str!)
static/
  index.html           -- Dashboard HTML
  style.css            -- Nord-themed CSS
  app.js               -- Frontend JavaScript
  manifest.json        -- PWA manifest
  sw.js                -- Service worker
  icon.svg             -- App icon
```

### Data Flow

```
GPU (rocm-smi/nvidia-smi)  -->  GPU Poller (500ms)  --> AppState
llama-server /metrics       -->  Llama Poller (1s)   --> AppState
                                                         |
                                                    WebSocket (500ms)
                                                         |
                                                      Browser
```

## API Reference

| Method | Path | Description |
|--------|------|-------------|
| GET | `/` | Dashboard HTML |
| GET | `/ws` | WebSocket (real-time metrics push) |
| POST | `/api/start` | Start llama-server with `ServerConfig` JSON body |
| POST | `/api/stop` | Stop running llama-server |
| GET | `/api/presets` | List all presets |
| POST | `/api/presets` | Create a new preset |
| PUT | `/api/presets/{id}` | Update a preset |
| DELETE | `/api/presets/{id}` | Delete a preset |
| POST | `/api/presets/reset` | Reset presets to defaults |
| GET | `/api/models` | List discovered .gguf models |
| POST | `/api/models/refresh` | Re-scan models directory |
| POST | `/api/chat` | Streaming proxy to llama-server `/v1/chat/completions` |

## Development

```bash
# Run in debug mode
cargo run

# Run tests
cargo test

# Lint
cargo clippy -- -D warnings

# Format
cargo fmt
```

### Project Structure

- Frontend files in `static/` are embedded at compile time via `include_str!`
- No Node.js or build tooling required
- Tests use fixture files in `tests/fixtures/`

## License

MIT
