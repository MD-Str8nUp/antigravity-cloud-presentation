# Antigravity Cloud Website Experiment

This repository orchestrates a three-run recorded experiment comparing how Antigravity (`agy`) designs and builds an audio electronics component and kit management web system across three levels of agent skill customization.

## Experiment Runs

**Baseline Run**:
Run 1 of the experiment, where the agent builds a local web application from an initial prompt guided only by the user's global clarifying-question rule with no repository skills installed.
_Avoid_: Unskilled run, vanilla run

**Skilled Run**:
Run 2 of the experiment, where the agent uses engineering workflow skills to model the domain, specify requirements, slice work into tickets, and build a verified local web application.
_Avoid_: Spec run, planning run

**Production Run**:
Stage 3 of the experiment, where the completed Skilled Run architecture is adapted for low-cost serverless cloud deployment (Cloud Run and Cloud Firestore behind the repository seam) using cloud skills guidance, presented as an architectural Codelab and Slidev act without a live terminal recording.
_Avoid_: Implementation run, comprehension run, Run 3

## Recording & Artifacts

**Terminal Cast**:
A terminal recording of an experiment run, captured as a raw recording and sanitized with chapter stop-point markers for playback.
_Avoid_: Video, screen recording

**Session Trace**:
The exported conversation log from the agent's session storage for each session within an experiment run.
_Avoid_: Chat history, transcript dump

**Canonical Q&A**:
The pre-scripted set of operator answers used across runs when the agent asks clarifying questions, ensuring consistent domain requirements.
_Avoid_: Cheat sheet, prompt script

**Codelab**:
An interactive lesson that walks learners through the recorded experiment runs using the captured Terminal Casts, Session Traces, and resulting codebases.
_Avoid_: Tutorial page, handout, lab doc

## Subject Domain (Audio Electronics Inventory)

**Component**:
An individual electronic part in the inventory, such as an op-amp (e.g., `TL074`), resistor, capacitor, potentiometer, or LED, characterized by package mounting style (`SMT` or `THT`), electrical specs, quantity on hand, and storage location.
_Avoid_: Item, product

**Kit**:
A DIY audio electronics project or PCB build tracked across varying stages of completion, alongside the components it requires or still needs.
_Avoid_: Bundle, assembly
