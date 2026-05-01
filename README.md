<h1 align="center"> ~ Google Open Images Dataset v5 ToolKit ~ </h1>
<h1 align="center"> ~ YOLO formatted Annotations Class Wise ~ </h1>
<h1 align="center"> ~ Object Detection Dataset ~ </h1>
<h1 align="center"> ~ Powered by Rust · Tauri · SvelteKit ~ </h1>

## About

**openimages_workbench** is a modern, cross-platform desktop application that rebuilds the original Python-based [OIDv5_ToolKit-YOLOv3](https://github.com/RajashekarY/OIDv5_ToolKit-YOLOv3) with a Rust backend and a SvelteKit UI delivered through [Tauri](https://tauri.app/).

It lets you download any of the 600 object-detection classes (or 19 000+ image-level label classes) from the [Open Images Dataset v5](https://storage.googleapis.com/openimages/web/index.html) — without pulling the whole multi-terabyte archive — and produces YOLO-formatted annotations ready for darknet / ultralytics training.

---

## Why this repo?

Do you want to build your own object detector but don't have enough images? Do you want a fast, native GUI instead of a Python CLI? Have you already discovered [Open Images Dataset v5](https://storage.googleapis.com/openimages/web/index.html) with its [600](https://storage.googleapis.com/openimages/2018_04/bbox_labels_600_hierarchy_visualizer/circle.html) classes and more than 1 700 000 annotated images, but you don't want to download gigabytes of data you don't need?

This workbench gives you the best of that dataset with a polished desktop experience — backed by the speed of Rust and a reactive SvelteKit interface.

---

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Backend | [Rust](https://www.rust-lang.org/) |
| Desktop shell | [Tauri v2](https://tauri.app/) |
| Frontend UI | [SvelteKit](https://kit.svelte.dev/) |
| Dataset | [Open Images v5](https://storage.googleapis.com/openimages/web/index.html) |

---

## Dataset Statistics

**Object Detection**

| | Train | Validation | Test | # Classes |
|---|---|---|---|---|
| Images | 1,743,042 | 41,620 | 125,436 | — |
| Boxes | 14,610,229 | 204,621 | 625,282 | 600 |

**Image Classification**

| | Train | Validation | Test | # Classes |
|---|---|---|---|---|
| Images | 9,011,219 | 41,620 | 125,436 | — |
| Machine-Generated Labels | 78,977,695 | 512,093 | 1,545,835 | 7,870 |
| Human-Verified Labels | 27,894,289 | 551,390 | 1,667,399 | 19,794 |

---

## Features

**(Object Detection)**

- Download any of the [600](https://storage.googleapis.com/openimages/2018_04/bbox_labels_600_hierarchy_visualizer/circle.html) classes individually, with automatic bounding-box creation for each image
- Download multiple classes into separated folders or a shared folder with a unified annotation file
- Filter by attributes: `IsOccluded`, `IsTruncated`, `IsGroupOf`, `IsDepiction`, `IsInside`
- Limit the number of images downloaded per class
- Resume interrupted downloads from the last downloaded image

**(Image Classification)**

- Download any of the [19,794](https://storage.googleapis.com/openimages/web/download.html#attributes) image-level label classes
- Choose between human-verified (`h`) or machine-generated (`m`) label sub-datasets
- Select train / validation / test splits

---

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain)
- [Node.js](https://nodejs.org/) ≥ 18 and npm
- [Tauri prerequisites](https://tauri.app/start/prerequisites/) for your OS

### Installation

1. Clone the repository
   ```bash
   git clone https://github.com/ry24aai-herts-ac-uk/openimages_workbench.git
   cd openimages_workbench
   ```

2. Install frontend dependencies
   ```bash
   npm install
   ```

3. Run in development mode (hot-reload UI + Rust backend)
   ```bash
   npm run tauri dev
   ```

4. Build a production binary
   ```bash
   npm run tauri build
   ```

---

## Annotation Format

Annotations are produced in **YOLO format** suitable for darknet / ultralytics training:

```
<class_index> <x_center> <y_center> <width> <height>
```

All coordinates are normalized (0 – 1) relative to image dimensions.

---

## Directory Structure (generated dataset)

```
openimages_workbench/
│
└───OID/
    ├───csv_folder/
    │    ├── class-descriptions-boxable.csv
    │    └── validation-annotations-bbox.csv
    │
    └───Dataset/
         ├── test/
         ├── train/
         └── validation/
              ├── Apple/
              │    ├── 0fdea8a716155a8e.jpg
              │    └── Labels/
              │         └── 0fdea8a716155a8e.txt
              └── Orange/
                   ├── 0b6f22bf3b586889.jpg
                   └── Labels/
                        └── 0b6f22bf3b586889.txt
```

---

## Optional Download Filters

| Flag | Description |
|------|-------------|
| `IsOccluded` | Include/exclude objects occluded by another object |
| `IsTruncated` | Include/exclude objects that extend beyond image boundaries |
| `IsGroupOf` | Include/exclude group annotations (5+ touching instances) |
| `IsDepiction` | Include/exclude cartoon / drawing depictions |
| `IsInside` | Include/exclude images taken from inside the object |
| `n_threads` | Number of parallel download threads |
| `limit` | Maximum number of images per class |

---

## Acknowledgements

This project is built on top of the excellent work by [RajashekarY](https://github.com/RajashekarY) and contributors to the original [OIDv5_ToolKit-YOLOv3](https://github.com/RajashekarY/OIDv5_ToolKit-YOLOv3) Python toolkit. Community contributors to the original project include:

- [Denis Zuenko](https://github.com/zuenko) — multithreading support
- [Skylion007](https://github.com/Skylion007) — O(n) label creation optimisation
- [Alex March](https://github.com/hosaka) — download limit option
- [Michael Baroody](https://github.com/mbaroody) — visualiser fix for multiword classes

---

## Citation

If you use this toolkit or the underlying dataset work, please cite the original repository:

```bibtex
@misc{OIDv5_ToolKit-YOLOv3,
  title   = {Toolkit to download and visualize single or multiple classes from the huge Open Images v5 dataset},
  author  = {RajashekarY, Vittorio, Angelo},
  year    = {2019},
  publisher = {Github},
  journal = {GitHub repository},
  howpublished = {\url{https://github.com/RajashekarY/OIDv5_ToolKit-YOLOv3}},
}
```

---

## Reference

"[We don't need no bounding-boxes: Training object class detectors using only human verification](https://arxiv.org/abs/1602.08405)" — Papadopoulos et al., CVPR 2016.

