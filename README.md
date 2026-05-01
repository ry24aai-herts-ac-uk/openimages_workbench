# Open Images Workbench

A modern Rust + Tauri desktop workbench for downloading, exploring, converting, and exporting **Open Images** datasets across multiple versions, model families, and annotation styles. It evolves the original `OIDv5_ToolKit-YOLOv3`, which focused on downloading and visualizing one or more classes from Open Images v5 with YOLO-oriented workflows.

## Overview

Open Images Workbench is a rewrite of the original Python-based toolkit into a desktop-first application powered by **Rust**, **Tauri**, and a **SvelteKit** UI. The goal is to keep the practical strengths of the original project—class-based dataset download and inspection—while expanding support for newer Open Images releases, multiple detection and segmentation workflows, and a broader set of annotation formats.

The original repository positioned itself as a tool to download and visualize single or multiple classes from the Open Images v5 dataset. This new repository broadens that idea into a reusable workbench for dataset preparation, conversion, validation, and export, without locking the project to a single dataset version or to YOLOv3 alone.

## Why this rewrite exists

The original toolkit name and workflow were tightly coupled to **OIDv5** and **YOLOv3**. Open Images has later releases such as **V7**, and modern vision workflows commonly need flexible conversion pipelines that can target several model ecosystems and annotation conventions rather than one training format.

This rewrite is intended to provide:

- Multi-version Open Images support.
- Multiple algorithm targets beyond YOLOv3.
- Multiple annotation styles and export formats.
- A faster, cleaner, desktop-native experience.
- A UI that makes dataset operations easier to inspect and repeat.

## Planned capabilities

The scope of Open Images Workbench includes the following capabilities:

- Download one or more classes from supported Open Images releases, preserving the class-oriented workflow that made the original toolkit useful.
- Explore dataset metadata and inspect image/label structure before training.
- Export datasets for different model families, including YOLO-style workflows and future non-YOLO targets.
- Support different annotation styles, such as Open Images native annotations and normalized YOLO-style labels; similar Open Images toolkit forks have already shown the value of optional YOLO label export.
- Save repeatable dataset configurations so experiments can be reproduced more easily across projects.
- Provide desktop visualization and validation tools to catch issues early in the data-preparation pipeline.

## Design goals

Open Images Workbench is being built around a few core principles:

- **Version-agnostic**: avoid hardcoding assumptions around Open Images v5 when newer releases exist.
- **Model-agnostic**: support dataset preparation for multiple training stacks instead of binding the project name or UX to one detector family.
- **Format-aware**: make annotation conversion explicit, inspectable, and reversible wherever possible.
- **Desktop-first**: use Tauri to deliver a lightweight desktop shell with web-style UI ergonomics.
- **Reproducible**: treat dataset selection, filtering, conversion, and export as saved project state rather than ad hoc command history.

## Architecture

The repository is intended to use the following stack:

- **Rust** for core services, file operations, dataset processing, and high-performance conversion logic.
- **Tauri** for the cross-platform desktop application shell.
- **SvelteKit** for the frontend user interface and application flow.

This stack is a strong fit for a local desktop tool that needs efficient filesystem access, responsive UI behavior, and a lighter runtime footprint than many Electron-based alternatives.

## Migration from the original repo

This project is a spiritual and functional successor to [`OIDv5_ToolKit-YOLOv3`](https://github.com/RajashekarY/OIDv5_ToolKit-YOLOv3). The original repo centered on downloading and visualizing selected classes from Open Images v5, while community forks around the same toolkit pattern added options such as YOLO label export and more organized dataset structure.

Open Images Workbench keeps that practical dataset-tooling DNA, but shifts the repository toward a longer-term platform for:

- dataset acquisition,
- dataset inspection,
- annotation conversion,
- export to training-ready formats,
- and support for more than one computer-vision workflow.

## Initial roadmap

### Phase 1

- Project creation with Tauri + SvelteKit shell.
- Rust backend commands for dataset download and file indexing.
- Basic project-based UI for class selection and download jobs.
- Local dataset viewer for images and annotations.

### Phase 2

- Open Images version selector.
- Annotation conversion engine.
- YOLO-style export support.
- Saved configuration profiles and dataset manifests.

### Phase 3

- Additional export targets beyond YOLO.
- Validation rules for class maps, missing files, and malformed labels.
- Batch operations for merge, filter, split, and remap.
- Plugin-style architecture for future model families.

## Repository status

This repository is currently a rewrite and redesign effort, not a drop-in replacement for the original Python toolkit. Feature coverage will grow incrementally as the Rust core, Tauri integration, and SvelteKit UI stabilize.

## Inspiration and prior work

- Original project: [`OIDv5_ToolKit-YOLOv3`](https://github.com/RajashekarY/OIDv5_ToolKit-YOLOv3), focused on downloading and visualizing Open Images v5 classes.
- Related Open Images toolkit forks demonstrated useful extensions such as YOLO label export and improved dataset organization.
- Open Images documentation and downstream ecosystem tooling show that later dataset versions and broader training integrations are worth supporting.

## Contributing

Contributions are welcome, especially in these areas:

- Open Images parsers and version adapters.
- Annotation conversion logic.
- Desktop UX for dataset inspection.
- Export pipelines for additional model families.
- Test datasets and validation workflows.

Please open an issue before large changes so the repository structure and roadmap stay coherent.

## Naming

The repository name **Open Images Workbench** was chosen to reflect a broader, tool-oriented desktop app rather than a script collection tied to one dataset release or one detector architecture.

## License

Add the license that matches the intended reuse model for this rewrite.

