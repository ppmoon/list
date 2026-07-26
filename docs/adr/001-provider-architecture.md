# ADR 001 — Provider architecture for Alfred feature surfaces

## Status

Accepted

## Context

Alfred exposes many independent capabilities (apps, files, clipboard, workflows, …) behind one query box. A monolithic match/switch would couple ranking, IO, and UI.

## Decision

Each feature is a `Provider` implementing `search(query, config) -> Vec<ResultItem>`. A `ProviderSet` fans out, then a shared `Ranker` applies fuzzy + usage scoring. The egui UI and CLI both talk only to `Engine`.

## Consequences

- Features can be enabled/disabled in config without UI changes.
- Headless tests cover each provider and the feature matrix without a display.
- Workflow/clipboard/snippets persist under `~/.local/share/alfredrs/` independently.
