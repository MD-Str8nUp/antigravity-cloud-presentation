---
layout: center
class: text-center
---

<!-- S18 -->
<div class="space-y-4">
  <h1 class="text-4xl font-extrabold text-white">
    The Skilled Run (Part 2): Worktree TDD &amp; 2-Axis Review
  </h1>
  <p class="text-xl font-mono text-cyan-300">
    /implement-spec &middot; /tdd &middot; /code-review &middot; 41 passing tests
  </p>
  <p class="text-base text-gray-400 max-w-xl mx-auto">
    Executing 8 vertical slices in isolated git worktrees, auditing against Standards and Spec, and verifying HTMX in a real browser
  </p>
</div>

<!--
Now that our glossary, ADR, UI prototype, specification, and 8 vertical tickets are committed to disk, let's watch Sessions 9 and 10 of the Skilled Run execute the entire build.
-->

---
layout: cast-full
title: "Skilled Run (Sessions 9–10): /implement-spec Worktree TDD, /code-review & Browser Verification"
cast: /casts/02-skilled.cast
startChapter: 103
endChapter: 111
autoPlay: false
speed: 2.0
idleTimeLimit: 2.0
fit: both
---

<!-- S19 -->
<!--
Let's step through the final two sessions of the Skilled Run. At stop point 104, we run /implement-spec in a fresh context window. The orchestrator dispatches subagents in isolated git worktrees for each of the 8 tickets, running red-green-refactor TDD against in-memory Turso databases. At stop point 105, all 8 tickets merge cleanly and the orchestrator automatically launches two parallel /code-review subagents—one checking coding standards, the other checking spec compliance—finishing at stop point 107 with 41 passing tests. Finally, at stop points 109 to 111, we clear context and run live Chrome DevTools MCP browser verification.
-->

---
layout: default
---

<!-- S20 -->
<div class="flex items-center gap-3 mb-6">
  <h2 class="text-2xl font-bold m-0">Three Layers of Verification in the Skilled Run</h2>
</div>

<ol class="space-y-4 text-sm text-gray-200 list-decimal pl-6 mt-5 leading-relaxed">
  <li>
    <strong class="text-white">Layer 1: Worktree TDD Against In-Memory Turso (<code>/implement-spec</code> + <code>/tdd</code>)</strong>
    <ul class="list-disc pl-5 mt-1 space-y-1 text-gray-300">
      <li>Each ticket runs in an isolated git worktree with red-green-refactor integration tests against <code>Database::open(":memory:")</code> (41 passing tests across 5 deep domain modules).</li>
    </ul>
  </li>
  <li>
    <strong class="text-white">Layer 2: Parallel 2-Axis Automated Code Review (<code>/code-review</code>)</strong>
    <ul class="list-disc pl-5 mt-1 space-y-1 text-gray-300">
      <li>Spawns two read-only subagents in parallel: <strong>Standards Reviewer</strong> (module depth, error handling) and <strong>Spec Reviewer</strong> (checking every requirement in <code>spec.md</code> and <code>GLOSSARY.md</code>).</li>
      <li>Caught that custom shortage overrides in <code>shopping_list.rs</code> needed to persist in SQLite so HTMX inline edits survive page reloads.</li>
    </ul>
  </li>
  <li>
    <strong class="text-white">Layer 3: Live Browser Verification via Chrome DevTools MCP</strong>
    <ul class="list-disc pl-5 mt-1 space-y-1 text-gray-300">
      <li>Caught two subtle HTMX DOM wiring bugs (nested <code>hx-swap-oob</code> table wrappers and filter-bar <code>hx-trigger</code> scoping) that backend unit tests cannot see.</li>
    </ul>
  </li>
</ol>

<!--
Notice how each verification layer caught a different class of defect. Layer 1, worktree TDD, verified all the domain math—engineering notation parsing, multi-location stock lots, and cross-build net shortfall. Layer 2, the parallel 2-axis code review, caught a subtle spec gap where shopping list overrides weren't persisted across HTMX reloads. And Layer 3, live browser testing via Chrome DevTools MCP, caught two real client-side HTMX DOM swap bugs.
-->

---
layout: default
---

<!-- S21 -->
<div class="flex items-center gap-3 mb-4">
  <h2 class="text-2xl font-bold m-0">Head-to-Head: Baseline Run vs. Skilled Run</h2>
</div>

<div class="grid grid-cols-2 gap-6 mt-6">
  <div class="p-5 rounded-xl bg-red-950/15 border border-red-500/30 space-y-3">
    <div class="text-xs font-mono text-red-400 uppercase">Run 1: Baseline Vibe-Coding</div>
    <h3 class="text-lg font-bold text-white">Fast Exploration, High Drift Risk</h3>
    <ul class="text-xs text-gray-300 space-y-2 list-disc pl-4">
      <li><strong>Upfront Questions:</strong> 4 broad multiple-choice questions</li>
      <li><strong>Time to Prototype:</strong> ~16 minutes (Node.js + SQLite)</li>
      <li><strong>Durable Docs on Disk:</strong> None (0 specs, 0 ADRs, 0 glossary)</li>
      <li><strong>Automated Tests:</strong> 0 tests</li>
      <li><strong>Post-<code>/clear</code> Audit:</strong> 8 functional &amp; security bugs found</li>
    </ul>
  </div>

  <div class="p-5 rounded-xl bg-emerald-950/15 border border-emerald-500/30 space-y-3">
    <div class="text-xs font-mono text-emerald-400 uppercase">Run 2: Skilled Engineering (mattpocock/skills)</div>
    <h3 class="text-lg font-bold text-white">Deliberate Upfront Design, Zero Context Drift</h3>
    <ul class="text-xs text-gray-300 space-y-2 list-disc pl-4">
      <li><strong>Upfront Questions:</strong> 33 domain-modeling questions across 6 sessions</li>
      <li><strong>Time to Verified Build:</strong> ~140m 1x across 10 <code>/clear</code> sessions + 11 subagents</li>
      <li><strong>Durable Docs on Disk:</strong> <code>GLOSSARY.md</code>, <code>ADR 0001</code>, <code>prototype-ui.html</code>, <code>spec.md</code>, 8 tickets</li>
      <li><strong>Automated Tests:</strong> 41 passing Rust integration &amp; unit tests</li>
      <li><strong>Post-<code>/clear</code> Audit:</strong> 1 spec gap caught in <code>/code-review</code> + 2 HTMX DOM fixes</li>
    </ul>
  </div>
</div>

<!--
Putting the two runs side by side makes the trade-off crystal clear. The Baseline Run got us a working UI in 16 minutes after just 4 questions, making it unbeatable for quick ideation, but left zero tests and zero documentation on disk. The Skilled Run invested 33 domain-modeling questions across 6 Wayfinder sessions, and in return produced a 41-test Rust and HTMX system with zero context drift across 10 /clear boundaries.
-->

---
layout: default
---

<!-- S22 -->
<div class="flex items-center gap-3 mb-6">
  <h2 class="text-2xl font-bold m-0">Decision Guide: When to Vibe-Code vs. Engineer with Skills</h2>
</div>

<ol class="space-y-4 text-sm text-gray-200 list-decimal pl-6 mt-5 leading-relaxed">
  <li>
    <strong class="text-white">Start with Baseline Vibe Coding When You Are Exploring</strong>
    <ul class="list-disc pl-5 mt-1 space-y-1 text-gray-300">
      <li>You have a loose idea, a throwaway weekend experiment, or an unfamiliar problem space where seeing a live screen in 15 minutes teaches you what you actually want.</li>
    </ul>
  </li>
  <li>
    <strong class="text-white">Graduate to <code>/wayfinder</code> &amp; <code>/domain-modeling</code> When the Project Matters</strong>
    <ul class="list-disc pl-5 mt-1 space-y-1 text-gray-300">
      <li>As soon as the codebase needs to survive <code>/clear</code>, handle real domain edge cases, or be maintained over weeks, commit <code>GLOSSARY.md</code>, ADRs, and <code>spec.md</code> to disk.</li>
    </ul>
  </li>
  <li>
    <strong class="text-white">Combine Both Workflows on Your Own Projects</strong>
    <ul class="list-disc pl-5 mt-1 space-y-1 text-gray-300">
      <li>Vibe-code a 15-minute spike to discover your requirements—then open a fresh directory with <code>mattpocock/skills</code> and feed what you learned into <code>/wayfinder</code>!</li>
    </ul>
  </li>
</ol>

<!--
The biggest takeaway is that these two workflows aren't enemies—they are stages of discovery. When you're not sure what you want to build, vibe-code a 15-minute spike and play with it. Once that spike shows you the real requirements you care about, switch to /wayfinder and /domain-modeling so your permanent codebase has the glossary, ADRs, deep modules, and tests to last.
-->
