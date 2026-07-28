# Domain context — alfredrs

## Purpose

`alfredrs` is a Linux keyboard launcher inspired by Alfred for macOS. Users invoke a floating search box, type a query or keyword, pick a ranked result, and execute an action — open an app, search the web, run a workflow, paste a snippet, etc.

## Ubiquitous language

| Term | Meaning |
|---|---|
| **Launcher** | Floating search UI toggled by hotkey |
| **Query** | Raw input; may include a **keyword** + **argument** |
| **Provider** | Feature module that turns a query into **result items** |
| **Result item** | Ranked row with title, subtitle, kind, and **actions** |
| **Action** | Side effect: open path/URL, run command, copy, Large Type, etc. |
| **Ranking** | Fuzzy match score + usage-learned bonus |
| **Workflow** | JSON graph of keyword → script/filter/copy/open nodes |
| **Snippet** | Named text clip expanded by abbreviation |
| **File buffer** | Multi-select holding area for batch file actions |
| **Sync pack** | Exported JSON of config + usage for backup/sync |
| **Large Type** | Full-window oversized text display |

## Boundaries

- In scope: Alfred free + Powerpack *interaction model* on Linux
- Out of scope: Alfred Remote iOS, Apple-only integrations, binary compatibility with Alfred workflow packages

## Key ADRs

See `docs/adr/` (to be added as decisions harden). Initial choices:

1. Provider trait over hard-coded switch — each Alfred feature is a deep module.
2. egui for the launcher — fast to iterate; domain tested headlessly.
3. JSON workflows rather than Alfred's `.alfredworkflow` zip — portable and reviewable.
