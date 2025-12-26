+++
title = "Multi-Account"
weight = 1
+++

## Context Switching, perfected.

GitTop is built for the developer who wears multiple hats. Whether you're a freelancer with three clients, a maintainer with a bot account, or just keeping your day job separate from your side hustle, we handle the context switching for you.

### One App, Distinct Contexts

We do not believe in a "Unified Inbox". Mingling your work alerts with your hobby projects is a recipe for burnout.

*   **Work at Work:** Select your `@acme-corp` profile. See only what you're paid to see.
*   **Play at Home:** Click over to `@open-source-hero`. No distracting "Production Down" alerts from the day job.

**The Exception:**
There is one intentional loophole. If you use the [Rule Engine](/features/rules/) to mark a notification as **Important** (e.g., "Security Alerts"), it will break through the walls and appear in **all** inboxes, tagged with its origin (e.g., `@acme-corp`). You'll never miss a fire, no matter which profile you're looking at.

Switching contexts is instant. Click your name in the sidebar/topbar, pick a new identity, and the entire interface updates instantly.

## Enterprise-Grade Security

We treat your credentials with the respect they deserve. GitTop **never** stores your Personal Access Tokens in plain text.

We integrate directly with your operating system's native secure storage:
*   **Windows:** Windows Credential Manager
*   **macOS:** Keychain Services
*   **Linux:** Secret Service API

Your tokens stay encrypted and local to your machine, always.

## Common Setups

### The Contractor
Juggle tokens for `@client-a`, `@client-b`, and your personal agency account. Each operates in a completely isolated silo.

### The Maintainer
Process notifications for your main account, then switch to `@my-project-bot` to review automated actions or triage issues assigned to the bot.

### The Professional
Keep your `dotfiles` issues separate from `quarterly-planning`. Use the [Rule Engine](/features/rules/) to define strict boundaries for when each account is allowed to notify you.
