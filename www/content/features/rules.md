+++
title = "Smart Rules"
weight = 1
+++

# Smart Rules

GitTop isn't just a notification viewer; it's a notification *manager*. The Rule Engine allows you to automatically process notifications as they arrive.

## Priorities

Every notification is assigned a priority:

- **High**: Urgent items. Mentioned, assigned, or review requested.
- **Normal**: Standard updates.
- **Low**: Merged PRs, CI updates, bot noises.
- **Ignored**: Muted or irrelevant items.

## Creating Rules

You can create rules based on:
- **Repository**: e.g., "Always mark `my-org/backend` as High"
- **Author**: e.g., "Ignore invalid-email-address[bot]"
- **Type**: e.g., "Treat all `ReviewRequested` events as Urgent"

## Power Mode

Enable **Power Mode** in settings to access the visual Rule Editor. This allows you to drag-and-drop priorities and inspect how rules are applied in real-time.
