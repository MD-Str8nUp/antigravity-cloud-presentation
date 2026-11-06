---
class: text-center
---

<!-- S01 -->
<div class="h-full flex flex-col items-center justify-center">

  <h1 class="text-5xl font-extrabold tracking-tight text-white mb-3">
    Vibe Coding vs. Skill-Driven Engineering
  </h1>
  <p class="text-2xl text-gray-300 font-light max-w-3xl">
    How Rules, Composable Agent Skills, and Google Cloud Skills Shape Antigravity Across Context Boundaries
  </p>

  <div class="mt-10 text-gray-400 text-base">
    <span class="font-semibold text-gray-200">Brett Morgan</span> &middot; Developer Relations Engineer, Google
  </div>

  <div class="mt-10 text-xs text-gray-500">
    Press <kbd>Space</kbd> or <kbd>&rarr;</kbd> to advance &middot; <kbd>[</kbd> <kbd>p</kbd> <kbd>]</kbd> on terminal recordings
  </div>
</div>

<!--
Welcome everyone! Today we are exploring two very different ways of working with an AI coding agent like Google Antigravity: rapid, exploratory vibe coding on one hand, and structured, skill-driven engineering on the other. Rather than arguing that one approach is always right and the other is always wrong, we recorded a transparent three-stage experiment with full terminal recordings and conversation traces so you can see exactly when each workflow shines and where it breaks down.
-->

---
layout: default
---

<!-- S02 -->
<div class="flex items-center gap-3 mb-4">
  <h2 class="text-2xl font-bold m-0">The Core Tension: Speed vs. Comprehension Debt</h2>
</div>

<div class="grid grid-cols-2 gap-6 mt-6">
  <div class="p-5 rounded-xl bg-red-950/15 border border-red-500/30 space-y-3">
    <div class="text-xs font-mono text-red-400 uppercase">The /clear Boundary Problem</div>
    <h3 class="text-lg font-bold text-white">When Intent Lives Only in Chat</h3>
    <ul class="text-xs text-gray-300 space-y-2 list-disc pl-4">
      <li>Modern agents can generate 2,000+ lines of working application code in 15 minutes from a three-sentence prompt.</li>
      <li>Every unasked domain assumption—schema trade-offs, edge cases, state transitions—is decided silently inside the model's context window.</li>
      <li><strong>Result:</strong> The moment you run <code>/clear</code> or return the next day, neither you nor the next agent session has a durable record of <em>what</em> was meant vs. <em>what</em> was accidentally coded.</li>
    </ul>
  </div>

  <div class="p-5 rounded-xl bg-emerald-950/15 border border-emerald-500/30 space-y-3">
    <div class="text-xs font-mono text-emerald-400 uppercase">Engineering Judgment</div>
    <h3 class="text-lg font-bold text-white">Choosing Ceremony Intentionally</h3>
    <ul class="text-xs text-gray-300 space-y-2 list-disc pl-4">
      <li><strong>Vibe Coding</strong> is genuinely great when you don't yet know what you want and need a throwaway prototype in 15 minutes to explore an idea.</li>
      <li><strong>Skill-Driven Engineering</strong> pays off as soon as the project matters and needs to survive across sessions, teammates, and production deployments.</li>
      <li><strong>Result:</strong> Treat ceremony as a dial you turn up when durability and maintenance justify it.</li>
    </ul>
  </div>
</div>

<!--
Here is the tension every developer hits with agentic coding tools. Generating two thousand lines of working code in fifteen minutes is now effortless. But when every design decision and domain edge case lives only inside a fleeting chat transcript, the moment you clear the context window or come back next week, you face comprehension debt. Our goal today is to look at how to treat engineering ceremony as a dial—using fast vibe coding when exploring, and turning up structured skills when building for longevity.
-->

---
layout: default
---

<!-- S03 -->
<div class="flex items-center gap-3 mb-6">
  <h2 class="text-2xl font-bold m-0">The Experiment: One Domain, Three Recorded Runs</h2>
</div>

<ol class="space-y-4 text-sm text-gray-200 list-decimal pl-6 mt-5 leading-relaxed">
  <li>
    <strong class="text-white">Subject Domain: Audio Electronics Component &amp; Kit Inventory</strong>
    <ul class="list-disc pl-5 mt-1 space-y-1 text-gray-300">
      <li>Track electronic <strong>Components</strong> (op-amps like <code>TL074</code>, resistors, capacitors, pots, <code>SMT</code> vs. <code>THT</code> packages, storage bins) and DIY <strong>Kit</strong> builds (BOM fulfillment, shortages, and build progress).</li>
      <li>Chosen because it looks like a simple CRUD app at first glance, but hides real domain nuance once you ask how parts are stored, allocated, and ordered.</li>
    </ul>
  </li>
  <li>
    <strong class="text-white">Controlled Operator Input: Identical Prompt + Canonical Q&amp;A</strong>
    <ul class="list-disc pl-5 mt-1 space-y-1 text-gray-300">
      <li>Every run starts from the exact same initial prompt and uses a pre-scripted <strong>Canonical Q&amp;A</strong> so differences come from the agent's skills, not extra human hints.</li>
    </ul>
  </li>
  <li>
    <strong class="text-white">Full Observability: Terminal Casts + Session Traces</strong>
    <ul class="list-disc pl-5 mt-1 space-y-1 text-gray-300">
      <li>Every terminal keystroke is recorded with <code>asciinema</code> and paired with exported Antigravity <strong>Session Traces</strong> across <code>/clear</code> boundaries.</li>
    </ul>
  </li>
</ol>

<!--
To make this comparison concrete rather than theoretical, we ran a controlled experiment around an audio electronics workbench inventory and DIY kit manager. At first glance, tracking op-amps, resistors, and pedal kits sounds like a simple two-table CRUD app. Underneath, though, are real domain questions: what happens when the same op-amp lives in two different drawer bins, or when three planned pedal builds compete for the same five chips? By holding the initial prompt and operator answers constant, we can watch exactly how different skill configurations change the agent's behavior.
-->

---
layout: default
---

<!-- S04 -->
<div class="flex items-center gap-3 mb-6">
  <h2 class="text-2xl font-bold m-0">Three Ways to Shape Antigravity (<code>agy</code>)</h2>
</div>

<ol class="space-y-4 text-sm text-gray-200 list-decimal pl-6 mt-5 leading-relaxed">
  <li>
    <strong class="text-white">Rules (<code>AGENTS.md</code> &amp; Global User Rules) — Always-On Guardrails</strong>
    <ul class="list-disc pl-5 mt-1 space-y-1 text-gray-300">
      <li>Loaded into every turn automatically; ideal for short behavioral invariants like <em>"Ask clarifying questions, one at a time."</em></li>
      <li>Rules tell the agent to pause and ask, but they do not teach a multi-stage methodology or define artifact formats.</li>
    </ul>
  </li>
  <li>
    <strong class="text-white">Skills (<code>.agents/skills/*/SKILL.md</code>) — On-Demand Procedural Playbooks</strong>
    <ul class="list-disc pl-5 mt-1 space-y-1 text-gray-300">
      <li>Discovered via YAML frontmatter or invoked as slash commands (<code>/wayfinder</code>, <code>/prototype</code>, <code>/to-spec</code>, <code>/implement-spec</code>).</li>
      <li>Load detailed templates, checklists, and subagent workflows into context only when needed.</li>
    </ul>
  </li>
  <li>
    <strong class="text-white">MCP Servers (Model Context Protocol) — Live External Capabilities</strong>
    <ul class="list-disc pl-5 mt-1 space-y-1 text-gray-300">
      <li>Give the agent hands and eyes—such as <code>chrome-devtools-mcp</code> to launch a real headless browser, click UI buttons, and inspect console errors.</li>
    </ul>
  </li>
</ol>

<!--
Before we look at the first recording, let's distinguish the three customization levers in Antigravity. First, Rules in AGENTS.md or your global config are always-on guardrails—great for a one-line habit like asking clarifying questions one at a time. Second, Skills in .agents/skills are procedural playbooks loaded on demand when a task matches their description or when you type a slash command. Third, MCP servers connect the agent to live tools like Chrome DevTools for real browser testing.
-->

---
layout: default
---

<!-- S05 -->
<div class="flex items-center gap-3 mb-6">
  <h2 class="text-2xl font-bold m-0">The Three-Run Progression</h2>
</div>

<ol class="space-y-4 text-sm text-gray-200 list-decimal pl-6 mt-5 leading-relaxed">
  <li>
    <strong class="text-white">Run 1: The Baseline Run (Rapid Vibe Coding — <code>01-baseline.cast</code>)</strong>
    <ul class="list-disc pl-5 mt-1 space-y-1 text-gray-300">
      <li>Guided only by the global one-question-at-a-time rule and ambient <code>modern-web-guidance</code>; no engineering workflow skills installed.</li>
      <li>Agent chooses Node.js + Express + <code>node:sqlite</code> + vanilla JS and builds a complete app in 16 minutes—then faces a fresh-context <code>/clear</code> review.</li>
    </ul>
  </li>
  <li>
    <strong class="text-white">Run 2: The Skilled Run (<code>mattpocock/skills</code> — <code>02-skilled.cast</code>)</strong>
    <ul class="list-disc pl-5 mt-1 space-y-1 text-gray-300">
      <li>Installs 38 engineering workflow skills (<code>/wayfinder</code>, <code>/grilling</code>, <code>/domain-modeling</code>, <code>/prototype</code>, <code>/to-spec</code>, <code>/to-tickets</code>, <code>/implement-spec</code>, <code>/code-review</code>).</li>
      <li>Executes across 10 <code>/clear</code> sessions and 11 subagents to build a 41-test Rust + Axum + Maud + HTMX + local Turso application.</li>
    </ul>
  </li>
  <li>
    <strong class="text-white">Run 3: The Production Run (<code>google/skills</code> + Interactive Codelab)</strong>
    <ul class="list-disc pl-5 mt-1 space-y-1 text-gray-300">
      <li>Composes <code>mattpocock/skills</code> with <code>google/skills</code> to adapt the <code>InventoryRepository</code> seam for scale-to-zero <strong>Cloud Run</strong> + <strong>Cloud Firestore</strong>, culminating in a 5-lesson hands-on <strong>Codelab</strong>.</li>
    </ul>
  </li>
</ol>

<!--
Here is our roadmap across the three runs. In Run 1, the Baseline Run, we let Antigravity vibe-code the application with only our global clarifying-question rule. In Run 2, the Skilled Run, we install Matt Pocock's workflow skills and watch how ten fresh-context sessions build a deeply modeled, test-driven Rust and HTMX application. Finally, in Run 3, we layer in Google Cloud skills to take that architecture to scale-to-zero Cloud Run and Firestore, and walk through the interactive Codelab where you can try both workflows on your own ideas.
-->
