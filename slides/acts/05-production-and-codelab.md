---
layout: center
class: text-center
---

<!-- S23 -->
<div class="space-y-4">
  <h1 class="text-4xl font-extrabold text-white">
    The Production Run: Google Cloud Skills &amp; Codelab
  </h1>
  <p class="text-xl font-mono text-cyan-300">
    google/skills &middot; Cloud Run &middot; Cloud Firestore &middot; /teach-code
  </p>
  <p class="text-base text-gray-400 max-w-xl mx-auto">
    Composing process skills with domain-specific cloud skills to take a local repository seam to scale-to-zero production
  </p>
</div>

<!--
Now we arrive at Stage 3 of our experiment: the Production Run. Having built a verified local application with Matt Pocock's engineering workflow skills, what happens when we compose those process skills with Google Cloud domain skills to take the application to serverless production?
-->

---
layout: default
---

<!-- S24 -->
<div class="flex items-center gap-3 mb-6">
  <h2 class="text-2xl font-bold m-0">Orthogonal Composition: Process Skills + Domain Skills</h2>
</div>

<ol class="space-y-4 text-sm text-gray-200 list-decimal pl-6 mt-5 leading-relaxed">
  <li>
    <strong class="text-white">Process Skills (<code>mattpocock/skills</code>) — <em>How</em> to Think &amp; Slice Work</strong>
    <ul class="list-disc pl-5 mt-1 space-y-1 text-gray-300">
      <li><code>/wayfinder</code>, <code>/grilling</code>, <code>/domain-modeling</code>, <code>/codebase-design</code>, <code>/tdd</code>, and <code>/code-review</code> govern how requirements are discovered, documented, and tested.</li>
    </ul>
  </li>
  <li>
    <strong class="text-white">Cloud Domain Skills (<code>google/skills</code>) — <em>What</em> Cloud Guardrails to Apply</strong>
    <ul class="list-disc pl-5 mt-1 space-y-1 text-gray-300">
      <li><code>cloud-databases-onboarding</code> &amp; <code>google-cloud-waf-cost-optimization</code>: Select a database that scales to zero with no idle VM hourly charges.</li>
      <li><code>cloud-run-basics</code> &amp; <code>gcloud</code>: Deploy stateless containers with <code>--min-instances=0</code> and non-interactive CLI safety flags.</li>
      <li><code>firestore-rules-creation</code>: Enforce least-privilege access and delegate rules authoring to <code>firestore-rules-author</code>.</li>
    </ul>
  </li>
</ol>

<!--
One of the best properties of Agent Skills is that they compose orthogonally. Matt Pocock's skills teach the agent how to structure engineering work—interviewing, writing ADRs, designing deep module boundaries, and running TDD. Google Cloud skills teach the agent domain-specific cloud architecture, FinOps cost optimization, and safe gcloud CLI execution.
-->

---
layout: default
---

<!-- S25 -->
<div class="flex items-center gap-3 mb-4">
  <h2 class="text-2xl font-bold m-0">Database Selection Guided by Cloud &amp; FinOps Skills</h2>
</div>

<div class="grid grid-cols-2 gap-6 mt-6">
  <div class="p-5 rounded-xl bg-red-950/15 border border-red-500/30 space-y-3">
    <div class="text-xs font-mono text-red-400 uppercase">Rejected for Intermittent Workbench Traffic</div>
    <h3 class="text-lg font-bold text-white">Always-On Instance Databases</h3>
    <ul class="text-xs text-gray-300 space-y-2 list-disc pl-4">
      <li><strong>Ephemeral SQLite on Cloud Run:</strong> Local container filesystem is wiped whenever Cloud Run scales instances down to zero.</li>
      <li><strong>Cloud SQL / AlloyDB / Cloud Spanner:</strong> Provisioned compute instances accrue continuous hourly billing even when the electronics bench sits idle all week.</li>
      <li><strong>Result:</strong> Paying 24/7 VM costs for a hobbyist or small-team workbench app.</li>
    </ul>
  </div>

  <div class="p-5 rounded-xl bg-emerald-950/15 border border-emerald-500/30 space-y-3">
    <div class="text-xs font-mono text-emerald-400 uppercase">Selected by cloud-databases-onboarding</div>
    <h3 class="text-lg font-bold text-white">Cloud Run + Cloud Firestore (Native Mode)</h3>
    <ul class="text-xs text-gray-300 space-y-2 list-disc pl-4">
      <li><strong>True Scale-to-Zero:</strong> Cloud Run (<code>--min-instances=0</code>) + Cloud Firestore Native Mode bill strictly per request and document operation.</li>
      <li><strong>Generous Free Tier:</strong> 1 GiB storage, 50,000 reads/day, and 20,000 writes/day at <strong>$0/month</strong> for typical bench usage.</li>
      <li><strong>Atomic Multi-Document Writes:</strong> Firestore transactions preserve Run 2's stock-deduction and BOM-allocation invariants.</li>
    </ul>
  </div>
</div>

<!--
When we consult cloud-databases-onboarding and google-cloud-waf-cost-optimization for a workbench inventory app that gets used intermittently on nights and weekends, the trade-off becomes clear immediately. Running local SQLite inside a Cloud Run container loses data on scale-to-zero, while Cloud SQL or AlloyDB charges for an always-on instance 24/7. Pairing Cloud Run with Cloud Firestore in Native Mode gives us true scale-to-zero execution and ACID multi-document transactions inside the free tier.
-->

---
layout: skill-breakdown
way: 3
title: Swapping the Repository Seam (ADR 0002)
---

<!-- S26 -->
<div class="space-y-4">

```rust
#[async_trait::async_trait]
pub trait InventoryRepository: Send + Sync {
    async fn list_components(&self, filter: &ComponentFilter) -> Result<Vec<ComponentSummary>, AppError>;
    async fn adjust_stock_lot(&self, lot_id: i64, delta: i64) -> Result<StockLot, AppError>;
    async fn allocate_bom_line(&self, req: &AllocationRequest) -> Result<BomLineState, AppError>;
    async fn compute_net_shortfall(&self, build_ids: &[i64]) -> Result<ShoppingList, AppError>;
}
```

  <div class="grid grid-cols-2 gap-4 text-xs pt-2">
    <div class="p-3 rounded-lg bg-gray-900/80 border border-gray-800">
      <span class="text-emerald-300 font-semibold">Deep Module Payoff:</span> Because Run 2 isolated persistence behind <code>src/db.rs</code>, the Axum + Maud + HTMX handlers (<code>web.rs</code>), value parser (<code>spec_parser.rs</code>), and shortfall math (<code>shopping_list.rs</code>) require <strong>zero changes</strong>.
    </div>
    <div class="p-3 rounded-lg bg-gray-900/80 border border-gray-800">
      <span class="text-emerald-300 font-semibold">Dual-Backend Verification:</span> Fast local TDD continues running against <code>TursoRepository</code> (<code>:memory:</code>), while cloud integration tests run against the local Firestore Emulator and deploy via Application Default Credentials.
    </div>
  </div>
</div>

<!--
Here is the architectural payoff from the Skilled Run's deep module design. Because Run 2 kept all SQL inside src/db.rs and kept the Axum HTMX routes, electrical value normalization, and net shortfall math in pure Rust domain modules, adapting to Cloud Firestore does not touch a single HTML template or route handler. We introduce the InventoryRepository trait seam in ADR 0002, keep TursoRepository for instant in-memory unit tests, and add FirestoreRepository for Cloud Run.
-->

---
layout: default
---

<!-- S27 -->
<div class="flex items-center gap-3 mb-6">
  <h2 class="text-2xl font-bold m-0">The 5-Lesson Interactive Codelab (<code>./codelab/</code>)</h2>
</div>

<ol class="space-y-3 text-sm text-gray-200 list-decimal pl-6 mt-4 leading-relaxed">
  <li>
    <strong class="text-white">
      <CodelabLink path="codelab/lessons/0001-baseline-vibe-coding-for-rapid-exploration.html" class="text-cyan-300 hover:underline">
        Lesson 0001: Baseline Vibe-Coding — Rapid Exploration &amp; Its Trade-Offs
      </CodelabLink>
    </strong>
    <ul class="list-disc pl-5 mt-0.5 text-gray-300">
      <li>Inspect <code>01-baseline.cast</code> &amp; traces, then run a 15-minute vibe-coding spike on <strong>your own project idea</strong>.</li>
    </ul>
  </li>
  <li>
    <strong class="text-white">
      <CodelabLink path="codelab/lessons/0002-wayfinding-and-domain-modeling-with-skills.html" class="text-cyan-300 hover:underline">
        Lesson 0002: Wayfinding &amp; Domain Modeling Across /clear Boundaries
      </CodelabLink>
    </strong>
    <ul class="list-disc pl-5 mt-0.5 text-gray-300">
      <li>Install <code>mattpocock/skills</code> and chart a <code>/wayfinder</code> map, <code>GLOSSARY.md</code>, and ADR for your project.</li>
    </ul>
  </li>
  <li>
    <strong class="text-white">
      <CodelabLink path="codelab/lessons/0003-prototyping-specs-and-vertical-slice-tickets.html" class="text-cyan-300 hover:underline">
        Lesson 0003: Visual Prototyping, Specifications &amp; Vertical Slice Tickets
      </CodelabLink>
    </strong>
    <ul class="list-disc pl-5 mt-0.5 text-gray-300">
      <li>Generate multi-variant HTML UI prototypes (<code>/prototype</code>), synthesize <code>spec.md</code> (<code>/to-spec</code>), and slice tickets (<code>/to-tickets</code>).</li>
    </ul>
  </li>
  <li>
    <strong class="text-white">
      <CodelabLink path="codelab/lessons/0004-worktree-tdd-code-review-and-browser-verification.html" class="text-cyan-300 hover:underline">
        Lesson 0004: Autonomous Worktree TDD, 2-Axis Code Review &amp; Browser Verification
      </CodelabLink>
    </strong>
    <ul class="list-disc pl-5 mt-0.5 text-gray-300">
      <li>Execute tickets with <code>/implement-spec</code>, audit with parallel <code>/code-review</code> subagents, and test in a headless browser.</li>
    </ul>
  </li>
  <li>
    <strong class="text-white">
      <CodelabLink path="codelab/lessons/0005-capstone-exercise-productionizing-with-google-cloud-skills.html" class="text-cyan-300 hover:underline">
        Lesson 0005: Capstone Exercise — Productionizing with Google Cloud Skills
      </CodelabLink>
    </strong>
    <ul class="list-disc pl-5 mt-0.5 text-gray-300">
      <li>Open-ended capstone: add <code>google/skills</code> and design a scale-to-zero Cloud Run + Firestore deployment for your own app.</li>
    </ul>
  </li>
</ol>

<!--
Everything we explored today is packaged into a self-contained 5-lesson interactive Codelab right inside this repository. Crucially, the Codelab does not ask you to re-type our audio electronics inventory app! Instead, it uses our recorded casts and session traces as a transparent field study, and guides you through testing both Baseline Vibe-Coding and Skill-Driven Engineering on your own project requirements, finishing with an open-ended Google Cloud capstone exercise.
-->

---
layout: default
---

<!-- S28 -->
<div class="flex items-center gap-3 mb-6">
  <h2 class="text-2xl font-bold m-0">Links &amp; Resources</h2>
</div>

<div class="space-y-3 mt-4">
  <div class="p-3.5 rounded-xl bg-gray-900 border border-gray-800">
    <div class="flex items-center justify-between">
      <div class="text-xs font-mono text-cyan-400 uppercase tracking-wide">Interactive 5-Lesson Codelab &amp; Printable Reference Sheets</div>
      <span class="text-xs font-mono text-gray-500">Built with /teach-code</span>
    </div>
    <div class="text-sm font-bold font-mono mt-1">
      <CodelabLink path="codelab/index.html" class="text-cyan-300 hover:underline">
        Open Interactive Codelab (/codelab/index.html)
      </CodelabLink>
    </div>
    <p class="text-xs text-gray-400 mt-1 m-0">
      Includes all 5 hands-on lessons, interactive quizzes, and printable cheatsheets (<code>GLOSSARY</code>, <code>mattpocock/skills</code>, and <code>google/skills</code>).
    </p>
  </div>

  <div class="p-3.5 rounded-xl bg-gray-900 border border-gray-800">
    <div class="flex items-center justify-between">
      <div class="text-xs font-mono text-amber-400 uppercase tracking-wide">Experiment Repository, Casts &amp; Session Traces</div>
      <span class="text-xs font-mono text-gray-500">github.com</span>
    </div>
    <div class="text-sm font-bold font-mono mt-1">
      <a href="https://github.com/domesticmouse/antigravity-cloud-presentation" target="_blank" class="text-cyan-300 hover:underline">github.com/domesticmouse/antigravity-cloud-presentation</a>
    </div>
    <p class="text-xs text-gray-400 mt-1 m-0">
      Contains <code>recordings/final/*.cast</code>, <code>artifacts/traces/</code>, <code>experiments/</code>, <code>codelab/</code>, and this Slidev deck in <code>slides/</code>.
    </p>
  </div>

  <div class="p-3.5 rounded-xl bg-gray-900 border border-gray-800">
    <div class="flex items-center justify-between">
      <div class="text-xs font-mono text-emerald-400 uppercase tracking-wide">Composable Agent Skill Catalogs</div>
      <span class="text-xs font-mono text-gray-500">npx skills@latest add</span>
    </div>
    <div class="text-sm font-bold font-mono mt-1 space-x-4">
      <a href="https://github.com/mattpocock/skills" target="_blank" class="text-cyan-300 hover:underline">github.com/mattpocock/skills</a>
      <span class="text-gray-600">&middot;</span>
      <a href="https://github.com/google/skills" target="_blank" class="text-cyan-300 hover:underline">github.com/google/skills</a>
    </div>
    <p class="text-xs text-gray-400 mt-1 m-0">
      Install engineering workflow skills (<code>/wayfinder</code>, <code>/tdd</code>, <code>/code-review</code>) and official Google Cloud architecture &amp; CLI skills.
    </p>
  </div>
</div>

<!--
Thank you all so much! On screen are the links to the interactive 5-lesson Codelab, the full GitHub repository with every terminal recording, session trace, and experiment codebase, and both the mattpocock/skills and google/skills repositories. Grab a project idea of your own, try a 15-minute vibe-coding spike, and then take it through /wayfinder and Google Cloud skills!
-->
