# Open Images Workbench

A desktop workbench for downloading, exploring, and exporting **Open Images** datasets — built with Rust and [GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui) (Zed's GPU-accelerated native UI framework). This project migrates and extends the original [`OIDv5_ToolKit-YOLOv3`](https://github.com/RajashekarY/OIDv5_ToolKit-YOLOv3) Python toolkit into a fully native Rust desktop application.

## Features

- **Class-based download**: Select one or more Open Images classes and download images with bounding-box annotations.
- **YOLO-format labels**: Generates YOLOv3-compatible label `.txt` files (class_idx cx cy w h, normalised).
- **Annotation filters**: Optionally filter by IsOccluded, IsTruncated, IsGroupOf, IsDepiction, IsInside flags.
- **Multi-class mode**: Merge multiple classes into a single dataset folder.
- **Dataset Viewer**: Browse locally downloaded images with bounding-box overlays.
- **Native UI**: No WebView, no Electron — pure Rust + GPUI GPU-accelerated rendering.

## Architecture

```
openimages_workbench/
└── src-gpui/           # GPUI desktop application
    ├── src/
    │   ├── main.rs     # Application entry point
    │   ├── state.rs    # Shared AppState entity
    │   ├── logic/      # Core business logic (no UI dependency)
    │   │   ├── models.rs    # Data models (Split, DownloadConfig, etc.)
    │   │   ├── csv_meta.rs  # CSV download & class listing
    │   │   ├── download.rs  # Image download + YOLO label generation
    │   │   └── index.rs     # Local dataset indexer
    │   └── views/      # GPUI views
    │       ├── home.rs      # Home / navigation screen
    │       ├── download.rs  # Download configuration form
    │       ├── jobs.rs      # Download progress monitor
    │       ├── viewer.rs    # Image viewer with bbox overlays
    │       └── theme.rs     # Colour palette and shared helpers
    └── Cargo.toml
```

## Prerequisites

- **Rust** 1.78+ with Cargo ([rustup.rs](https://rustup.rs))
- A **GPU** with Vulkan, Metal, or DirectX 12 support (required by GPUI's renderer)
- On Linux: `libxkbcommon`, `libGL`, and Vulkan loader (`libvulkan1`)

  ```bash
  # Ubuntu / Debian
  sudo apt-get install libxkbcommon-dev libgl1-mesa-dev libvulkan1 mesa-vulkan-drivers
  ```

## Build & Run

```bash
# Clone the repository
git clone https://github.com/ry24aai-herts-ac-uk/openimages_workbench
cd openimages_workbench

# Build the release binary
cargo build --release -p openimages-workbench-gpui

# Run directly
cargo run --release -p openimages-workbench-gpui

# Or run the compiled binary
./target/release/oidw
```

## Run Tests

```bash
cargo test -p openimages-workbench-gpui
```

## Usage

1. **Launch the app** — the Home screen shows three navigation cards.
2. **Download Dataset**:
   - Type a class name in the search box (min 2 characters) to filter classes.
   - Click a class to select it; selected classes appear as pills.
   - Choose a split (Train / Validation / Test / All).
   - Click **Fetch Metadata** to download the annotation CSV for the chosen split.
   - Click **Start Download** to begin downloading images and generating YOLO labels.
3. **Download Jobs** — monitor live progress for all active and completed jobs.
4. **Dataset Viewer** — click **Refresh Index** to scan the downloaded dataset, then browse images with bounding boxes.

## Data Layout

Downloaded data is stored in `~/.local/share/oidw/` (Linux/macOS) or the platform-specific data directory:

```
~/.local/share/oidw/
├── csv_folder/
│   ├── class-descriptions-boxable.csv
│   ├── train-annotations-bbox.csv
│   ├── validation-annotations-bbox.csv
│   └── test-annotations-bbox.csv
└── Dataset/
    ├── train/
    │   └── <ClassName>/
    │       ├── <image_id>.jpg
    │       └── Label/
    │           └── <image_id>.txt   # YOLO format: class_idx cx cy w h
    ├── validation/
    └── test/
```

## Migration from OIDv5_ToolKit-YOLOv3

This project is a Rust rewrite of [`RajashekarY/OIDv5_ToolKit-YOLOv3`](https://github.com/RajashekarY/OIDv5_ToolKit-YOLOv3). Key differences:

| Feature | Original (Python) | This project (Rust) |
|---|---|---|
| UI | CLI / argparse | Native desktop (GPUI) |
| Downloads | AWS CLI (`aws s3 cp`) | Direct HTTP (reqwest + S3) |
| Labels | Python script | Rust, same normalised format |
| Concurrency | `multiprocessing.dummy.Pool` | Tokio async + `futures::stream` |
| Dataset layout | Same structure | Same structure |

## License

Dual-licensed under **AGPL-3.0** (open source) and a commercial license. See `LICENSE` for details.
