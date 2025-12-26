+++
title = "Smart Rules"
weight = 2
+++

## Take Control of Your Focus

GitHub notifications are notorious for being noisy. A single busy repository can flood your inbox, burying the critical "Review Requested" on a production bug.

GitTop helps you build a dam against this flood. You decide what gets through, what waits quietly, and what shouldn't exist at all.

## The Golden Rule

**By default, everything is shown.**

GitTop starts completely transparent. If you don't touch the Rule Engine, it behaves exactly like the GitHub notifications page. You build the rules to filter the noise, not to enable the signal.

## The 4 Actions

When a notification arrives, your rules decide what to do with it. There are four possible actions:

1.  **Show** (Default)
    *   **What happens:** The notification appears in your list.
    *   **Desktop:** You get a system desktop notification (banner/sound).
    *   **Use for:** Standard work, PRs you're following, team updates.

2.  **Silent**
    *   **What happens:** The notification appears in your list, just like "Show".
    *   **Desktop:** **No** desktop notification. No sound. No banner.
    *   **Use for:** Things you want to see eventually (like "subscribed to repo" updates) but don't need to be interrupted for.

3.  **Suppress**
    *   **What happens:** The notification is **completely hidden**. It does not appear in your list.
    *   **Desktop:** No desktop notification.
    *   **Use for:** Bot noise, CI success messages, or high-traffic repos you only occasionally check manually.

4.  **Important**
    *   **What happens:** The notification is pinned to the top of your list.
    *   **Desktop:** Always triggers a desktop notification.
    *   **Superpower:** This action **overrides everything else**. It bypasses schedules, silent mode, and even account isolation. **Important items show up in EVERY account's inbox.**
    *   **Use for:** Mentions, Review Requests, Deployment failures.

## The 3 Ways to Filter

You can apply these actions using three types of rules:

### 1. Schedules (Account Rules)
*The "Work/Life Balance" Switch.*

Schedules let you define **Active Hours** for each GitHub account.
*   **Example:** Set your work account (`@acme-corp`) to be active only Mon-Fri, 9:00 AM - 5:00 PM.
*   **Behavior:** Outside these hours, you can choose to either **Suppress** (Hide) or **Defer** (Silence) notifications.
*   **Why use it:** Stop thinking about work code on Saturday morning.

### 2. Organization Rules
*The "Project Priority" Switch.*

Rules that apply to every repository within a specific organization.
*   **Example:**
    *   `my-company`: **Show** (Priority +50)
    *   `open-source-lib`: **Silent** (Priority 0)
*   **Why use it:** Separate your day job from your hobby projects.

### 3. Type Rules
*The "Context" Switch.*

Rules based on *why* you got the notification. This is the most powerful filter.
*   **Example:**
    *   Reason: `Mentioned` → **Important**
    *   Reason: `ReviewRequested` → **Important**
    *   Reason: `CiActivity` → **Hide**
*   **Why use it:** Never miss a direct question, but ignore the 100th "Build Succeeded" email.

## How It Decides

With all these rules, what happens when they conflict? GitTop follows a strict logic:

1.  **Important Wins:** If *any* matching rule says "Important", the notification is Important. Period.
2.  **Highest Priority Wins:** If nothing is Important, the rule with the highest numeric Priority score wins.
3.  **Tie-Breaker:** If priorities are equal, the "strictest" action wins (Hide > Silent > Show).

### The "Explain Decision" Tool

Unsure why a notification was hidden? In the Rule Engine settings, use the **Explain Decision** tab.
You can simulate a notification (e.g., "Mentioned in Repo X") and GitTop will show you exactly which rules matched and which one won.
