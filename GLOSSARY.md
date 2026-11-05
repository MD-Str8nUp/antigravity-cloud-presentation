# Antigravity Cloud Website Experiment

This repository orchestrates a three-run recorded experiment comparing how Antigravity (`agy`) designs and builds an audio electronics component and kit management web system across three levels of agent skill customization.

## Experiment Runs

**Baseline Run**:
Run 1 of the experiment, where `agy` builds the web system from an initial prompt guided only by the user's global rule (`When something is unclear, please ask clarifying questions, one at a time.`) with no repository skills installed.
_Avoid_: Unskilled run, vanilla run

**Spec Run**:
Run 2 of the experiment, where `agy` uses `mattpocock/skills` (`/wayfinder` through `/to-spec`) to organize requirements, model the domain, and produce a specification as its final output without implementing code.
_Avoid_: Skilled run, planning run

**Implementation Run**:
Run 3 of the experiment, cloned from the completed Spec Run with `google/skills` added, where `agy` shapes the specification into tickets (`/to-tickets`) informed by Google Cloud skills and implements the running web system (`/implement`).
_Avoid_: Comprehension run, Run 3

## Recording & Artifacts

**Terminal Cast**:
An `asciinema` `.cast` recording of an experiment run, captured in `recordings/raw/` and sanitized with chapter stop-point markers in `recordings/final/`.
_Avoid_: Video, screen recording

**Session Trace**:
The exported Markdown conversation log from Antigravity's brain directory for each session within an experiment run, stored under `artifacts/traces/`.
_Avoid_: Chat history, transcript dump

**Canonical Q&A**:
The pre-scripted set of operator answers used across runs when `agy` asks clarifying questions, ensuring consistent domain requirements between the Baseline Run and the Spec Run.
_Avoid_: Cheat sheet, prompt script

## Subject Domain (Audio Electronics Inventory)

**Component**:
An individual electronic part in the inventory, such as an op-amp (e.g., `TL074`), resistor, capacitor, potentiometer, or LED, characterized by package mounting style (`SMT` or `THT`), electrical specs, quantity on hand, and storage location.
_Avoid_: Item, product

**Kit**:
A DIY audio electronics project or PCB build tracked across varying stages of completion, alongside the components it requires or still needs.
_Avoid_: Bundle, assembly
