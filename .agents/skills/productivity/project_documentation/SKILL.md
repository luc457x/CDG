---
name: project_documentation
description: Analyze codebase. Generate clean, modular Markdown documentation under doc/ folder with standard horizontal header navigation bar.
when_to_use: user requests project documentation, modular documentation pages, or updates to documentation structure.
metadata:
  category: documentation
---
# Project Documentation Generation

Generate comprehensive, clean, and linked documentation system under `doc/` directory at project root.

## Guidelines & Navigation Styling

To ensure premium-grade rendering across standard markdown viewer applications, follow these design standards:

1. **Header Navigation Bar**:
   - Do **NOT** use Markdown tables for sidebars. Headings nested inside table cells cause layout bugs.
   - Do use standard horizontal list of links at top of every documentation page. Example:
     ```markdown
     # Page Title

     [🏠 Home](../README.md) · [📖 Overview](README.md) · [🏗️ Architecture](architecture.md) · [🔧 Setup](installation_usage.md) · [🚀 Deployment](deployment.md) ...
     
     ---
     ```
2. **Relative Links Formatting**:
   - Always use simple relative paths.
   - For links inside same folder, write filename directly (e.g. `architecture.md`).
   - For links navigating to parent directories, use `../` prefixes (e.g. `../README.md`).

## Documentation Page Map

Structure documentation using following modular layout:

- **`doc/README.md` (Documentation Hub / Overview)**:
  - Central landing index. Contains project overview, features summary, and high-level directory map.
- **`doc/architecture.md` (System Architecture)**:
  - Component relationships, data flows, and visual Mermaid diagrams.
- **`doc/installation_usage.md` (Setup & Usage)**:
  - Quickstart steps, CLI parameters, config flags, and command reference.
- **`doc/[module_name].md` (Component Guides)**:
  - Create separate files for each major system module, library, API, or distinct codebase section.
- **`doc/deployment.md` (Deployment & Operations)**:
  - Environments setup, credential management, scheduler rules, and production deployment scripts.

## Workflow

1. **Scan Codebase**: Retrieve and analyze source files, scripts, package manifests, and existing deployment configurations.
2. **Identify Modules**: Group codebase features into clean categories matching page map.
3. **Verify Links**: Run checks to guarantee that all relative links resolve correctly between all pages and root README.
4. **Draft Walkthrough**: Document created pages and layout verification details.
