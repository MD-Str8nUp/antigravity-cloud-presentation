---
layout: center
class: text-center
---

<!-- S06 -->
<div class="space-y-4">
  <h1 class="text-4xl font-extrabold text-white">
    The Baseline Run: Rapid Vibe Coding
  </h1>
  <p class="text-xl font-mono text-cyan-300">
    01-baseline.cast &middot; 18 stop points &middot; 2 sessions
  </p>
  <p class="text-base text-gray-400 max-w-xl mx-auto">
    What happens when Antigravity builds an application guided only by a global clarifying-question rule and ambient UI styling guidance
  </p>
</div>

<!--
Let's dive into Run 1: the Baseline Run. Here we start in a clean directory with zero project-specific workflow skills installed, and watch what happens when we ask Antigravity to build our audio electronics inventory and kit build tracker.
-->

---
layout: skill-breakdown
way: 1
title: Baseline Run Setup — Global Rule + Ambient Web Guidance
---

<!-- S07 -->
<div class="space-y-4">
  <div class="grid grid-cols-2 gap-4">
    <div class="p-3.5 rounded-xl bg-gray-900/90 border border-cyan-500/30 flex flex-col justify-between">
      <div>
        <div class="flex items-center justify-between text-xs font-mono text-cyan-400 uppercase tracking-wide mb-2">
          <span>~/.gemini/AGENTS.md</span>
          <span class="text-[10px] text-cyan-300/80 px-1.5 py-0.5 rounded bg-cyan-950/60 border border-cyan-500/20">Global User Rule</span>
        </div>
        <div class="text-xs font-mono text-gray-200 space-y-1 leading-relaxed">
          <p class="text-gray-300">You are a helpful assistant.</p>
          <p class="text-cyan-200 font-medium">When something is unclear, please ask clarifying questions, one at a time.</p>
        </div>
      </div>
    </div>

    <div class="p-3.5 rounded-xl bg-gray-900/90 border border-cyan-500/30 flex flex-col justify-between">
      <div>
        <div class="flex items-center justify-between text-xs font-mono text-cyan-400 uppercase tracking-wide mb-2">
          <span>Operator Prompt</span>
          <span class="text-[10px] text-cyan-300/80 px-1.5 py-0.5 rounded bg-cyan-950/60 border border-cyan-500/20">Initial Input</span>
        </div>
        <p class="text-xs font-mono text-gray-200 leading-relaxed">
          I'd like to build a web application to track my audio electronics
          components inventory and DIY kit builds.
        </p>
      </div>
    </div>
  </div>

  <div class="grid grid-cols-2 gap-4 text-xs pt-2">
    <div class="p-3 rounded-lg bg-gray-900/80 border border-gray-800">
      <span class="text-cyan-300 font-semibold">4 Surface-Level Questions:</span> Because the global rule says to ask one question at a time, the agent uses <code>ask_question</code> four times (storage stack, feature checklist + write-in, part allocation, and DIY focus) before jumping straight into code.
    </div>
    <div class="p-3 rounded-lg bg-gray-900/80 border border-gray-800">
      <span class="text-cyan-300 font-semibold">Ambient Skill Activation:</span> Without a domain-modeling or spec skill, the agent activates its built-in <code>modern-web-guidance</code> skill and builds <strong>PatchBay &amp; Bench</strong> in Node.js, Express, and <code>node:sqlite</code>.
    </div>
  </div>
</div>

<!--
In the Baseline Run, two things steer the agent. First, our global user rule tells it to ask clarifying questions one at a time. Second, once those questions are answered, it activates its built-in modern-web-guidance skill for dark-mode UI polish. Notice what is missing: nothing tells the agent to challenge domain terms, write a glossary, or separate specification from implementation.
-->

---
layout: cast-full
title: "Baseline Run (Part 1): 4 Clarifying Questions & 16-Minute Build"
cast: /casts/01-baseline.cast
startChapter: 1
endChapter: 11
autoPlay: false
speed: 2.0
idleTimeLimit: 2.0
fit: both
---

<!-- S08 -->
<!--
Let's interact with the first half of the Baseline Run recording. Using the right-bracket key, we can step through the four clarifying questions: storage stack, key features where we write in project build progress tracking, part allocation, and guitar pedal focus. Then between stop points 9 and 10, the player automatically accelerates the 23-minute one-shot implementation to 29x speed so we can watch it generate the SQLite schema, Express server, and frontend in under 30 seconds.
-->

---
layout: default
---

<!-- S08b -->
<div class="flex items-center gap-3 mb-4">
  <h2 class="text-2xl font-bold m-0">The <code>/clear</code> Boundary: When Memory Resets to Zero</h2>
</div>

<div class="p-3.5 rounded-xl bg-gray-900/90 border border-amber-500/30 flex items-center justify-between mb-4">
  <div class="flex items-center gap-3">
    <span class="text-amber-400 font-mono text-sm font-bold bg-amber-950/80 px-2 py-0.5 rounded border border-amber-500/30">/clear</span>
    <span class="text-xs text-gray-300">Wiping conversation memory &middot; Transitioning from <strong>Session 1 (Build)</strong> to <strong>Session 2 (Audit)</strong></span>
  </div>
  <span class="text-[10px] text-amber-300/80 px-2 py-0.5 rounded bg-amber-950/60 border border-amber-500/20 font-mono">The Durability Test</span>
</div>

<div class="grid grid-cols-2 gap-6">
  <div class="p-4 rounded-xl bg-cyan-950/15 border border-cyan-500/30 space-y-2.5">
    <div class="flex items-center justify-between">
      <span class="text-xs font-mono text-cyan-400 uppercase tracking-wide">Session 1 On-Disk Artifacts</span>
      <span class="text-[10px] text-cyan-300/80 px-1.5 py-0.5 rounded bg-cyan-950/60 border border-cyan-500/20">What Survived</span>
    </div>
    <h3 class="text-base font-bold text-white">Working Code on Localhost</h3>
    <ul class="text-xs text-gray-300 space-y-1.5 list-disc pl-4">
      <li><strong>Node 22 + Express:</strong> <code>server.js</code>, <code>schema.sql</code>, vanilla frontend.</li>
      <li><strong>Realistic Seed Data:</strong> <code>TL072</code>, <code>PT2399</code>, <code>1N4148</code>, pedal kit records.</li>
      <li><strong>Hardware Styling:</strong> Resistor color-band generator, dark workbench UI.</li>
      <li><strong>Immediate Result:</strong> Looks functional and complete in browser on <code>localhost:3000</code>.</li>
    </ul>
  </div>

  <div class="p-4 rounded-xl bg-red-950/15 border border-red-500/30 space-y-2.5">
    <div class="flex items-center justify-between">
      <span class="text-xs font-mono text-red-400 uppercase tracking-wide">The /clear Amnesia Gap</span>
      <span class="text-[10px] text-red-300/80 px-1.5 py-0.5 rounded bg-red-950/60 border border-red-500/20">What Was Lost</span>
    </div>
    <h3 class="text-base font-bold text-white">Zero Written Intent or Tests</h3>
    <ul class="text-xs text-gray-300 space-y-1.5 list-disc pl-4">
      <li><strong>No Specifications:</strong> 0 <code>spec.md</code>, 0 <code>ADR</code>s, 0 <code>GLOSSARY.md</code>.</li>
      <li><strong>No Automated Verification:</strong> 0 unit tests, 0 integration tests.</li>
      <li><strong>Ephemeral Rationale:</strong> Every domain edge case lived only in prior chat context.</li>
      <li><strong>Next Session Reality:</strong> The fresh agent must reverse-engineer intent from raw code alone.</li>
    </ul>
  </div>
</div>

<div class="mt-4 p-3 rounded-lg bg-gray-900/80 border border-gray-800 text-xs text-gray-300 flex items-center justify-between">
  <div>
    <span class="text-cyan-300 font-semibold">The Core Question:</span> When a fresh agent reviews this codebase without conversation memory, what does it find?
  </div>
  <div class="text-[11px] font-mono text-gray-400">
    Proceed to Part 2 &rarr;
  </div>
</div>

<!--
We have just watched the agent generate a complete working prototype in sixteen minutes. Everything looks impressive on the surface. But now we hit the central test of agentic development: the /clear boundary. When we wipe the conversation context, all the ephemeral chat memory is gone. The next agent session knows only what was permanently written to disk. As we step into Part 2, watch what happens when a fresh agent inspects the code with no spec or glossary to guide it—and uncovers eight latent bugs.
-->

---
layout: cast-full
title: "Baseline Run (Part 2): /clear Audit, 8 Bug Fixes & Browser Verification"
cast: /casts/01-baseline.cast
startChapter: 12
endChapter: 17
autoPlay: false
speed: 2.0
idleTimeLimit: 2.0
fit: both
---

<!-- S09 -->
<!--
Now look at what happens in the second half of the Baseline Run after we run /clear to wipe the conversation context. At stop point 13, we ask a fresh agent session to review the codebase. Because there is no spec or glossary on disk, the agent has to reverse-engineer the entire app from source code—and at stop point 14 it uncovers 8 real bugs, including non-atomic stock deductions, broken modal Escape keys, and an innerHTML XSS sink. At stop points 16 and 17, we have it verify the fixes in a headless browser via Chrome DevTools MCP.
-->

---
layout: default
---

<!-- S10 -->
<div class="flex items-center gap-3 mb-4">
  <h2 class="text-2xl font-bold m-0">Baseline Run: What Worked vs. What Broke After <code>/clear</code></h2>
</div>

<div class="grid grid-cols-2 gap-6 mt-6">
  <div class="p-5 rounded-xl bg-emerald-950/15 border border-emerald-500/30 space-y-3">
    <div class="text-xs font-mono text-emerald-400 uppercase">Where Vibe Coding Shines</div>
    <h3 class="text-lg font-bold text-white">16 Minutes to a Working Prototype</h3>
    <ul class="text-xs text-gray-300 space-y-2 list-disc pl-4">
      <li>Zero setup friction: Node 22 built-in <code>node:sqlite</code> + Express with realistic seeded audio components (<code>TL072</code>, <code>PT2399</code>, <code>1N4148</code>) and pedal kits.</li>
      <li><code>modern-web-guidance</code> delivered an impressive hardware-inspired dark workbench UI with resistor color-band previews.</li>
      <li><strong>Best for:</strong> Discovering what you actually want when requirements are still fuzzy.</li>
    </ul>
  </div>

  <div class="p-5 rounded-xl bg-red-950/15 border border-red-500/30 space-y-3">
    <div class="text-xs font-mono text-red-400 uppercase">Where Vibe Coding Breaks Down</div>
    <h3 class="text-lg font-bold text-white">8 Bugs &amp; Zero Durable Intent Artifacts</h3>
    <ul class="text-xs text-gray-300 space-y-2 list-disc pl-4">
      <li><strong>No tests or docs:</strong> Zero unit/integration tests, no <code>GLOSSARY.md</code>, no <code>ADR</code>s, and no <code>spec.md</code>.</li>
      <li><strong>Shallow domain model:</strong> Flat string storage locations, binary 2-state kit status, and per-kit shortage math that double-counts shared stock across multiple planned builds.</li>
      <li><strong>8 functional bugs:</strong> Non-transactional BOM deduction, <code>parseInt</code> truncating <code>4.7k</code> to <code>4</code>, and unescaped <code>innerHTML</code> interpolation.</li>
    </ul>
  </div>
</div>

<!--
Let's take an honest look at both sides of this Baseline Run. On the left, getting a rich, seeded dark-mode prototype running on localhost in 16 minutes is genuinely impressive—if you are exploring a new idea, nothing beats that speed. On the right, once you clear the context window, you discover that the domain model is shallow, there are zero automated tests, and the fresh session had to fix 8 functional bugs just by reading the raw code.
-->
