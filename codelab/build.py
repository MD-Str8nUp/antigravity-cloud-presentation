#!/usr/bin/env python3
"""Static site builder and validator for the Antigravity Agent Skills Codelab."""

from __future__ import annotations

import argparse
import html
import re
import shutil
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import yaml
from markdown_it import MarkdownIt


ROOT_DIR = Path(__file__).resolve().parent


@dataclass
class PageMeta:
    source_path: Path
    rel_out_path: str
    title: str
    page_title: str
    nav_id: str
    nav_label: str
    badge: str
    subtitle: str
    card: dict[str, str]
    body_md: str
    lesson_num: str | None = None


def parse_frontmatter(raw_text: str, path: Path) -> tuple[dict[str, Any], str]:
    if not raw_text.startswith("---\n"):
        raise ValueError(f"{path}: Missing YAML frontmatter opening '---'")
    parts = raw_text.split("\n---\n", 1)
    if len(parts) != 2:
        raise ValueError(f"{path}: Missing YAML frontmatter closing '---'")
    meta = yaml.safe_load(parts[0][4:]) or {}
    return meta, parts[1]


def load_page_meta(path: Path, rel_out_path: str, lesson_num: str | None = None) -> PageMeta:
    raw = path.read_text(encoding="utf-8")
    meta, body = parse_frontmatter(raw, path)
    return PageMeta(
        source_path=path,
        rel_out_path=rel_out_path,
        title=str(meta.get("title", path.stem)),
        page_title=str(meta.get("page_title", meta.get("title", path.stem))),
        nav_id=str(meta.get("nav_id", path.stem)),
        nav_label=str(meta.get("nav_label", meta.get("title", path.stem))),
        badge=str(meta.get("badge", "")),
        subtitle=str(meta.get("subtitle", "")),
        card=dict(meta.get("card") or {}),
        body_md=body,
        lesson_num=lesson_num,
    )


def parse_attrs(attr_str: str) -> dict[str, str]:
    attrs: dict[str, str] = {}
    for match in re.finditer(r'([a-zA-Z0-9_-]+)="([^"]*)"', attr_str):
        attrs[match.group(1)] = match.group(2)
    return attrs


def render_inline_md(md: MarkdownIt, text: str) -> str:
    return md.renderInline(text.strip())


def rewrite_md_links(text: str) -> str:
    def _repl(match: re.Match[str]) -> str:
        prefix = match.group(1)
        url = match.group(2)
        if url.startswith(("http://", "https://", "mailto:", "#")):
            return match.group(0)
        anchor = ""
        if "#" in url:
            url, anchor = url.split("#", 1)
            anchor = "#" + anchor
        if url.endswith("GLOSSARY.md"):
            url = url[: -len("GLOSSARY.md")] + "glossary.html"
        elif url.endswith(".md"):
            url = url[:-3] + ".html"
        return f"{prefix}({url}{anchor})"

    return re.sub(r"(\[[^\]]+\])\(([^)]+)\)", _repl, text)


def convert_blank_tokens(escaped_code: str) -> str:
    pattern = re.compile(r"\{\{blank:([^}]+)\}\}")

    def _repl(match: re.Match[str]) -> str:
        raw = html.unescape(match.group(1))
        parts = raw.split("||")
        # Support '||' inside answer while keeping '|' as field separator
        # Format: answer1||answer2|error_hint|placeholder|width
        raw_fields = match.group(1)
        # Temporarily protect '||' before splitting on '|'
        sentinel = "\x00OR\x00"
        protected = html.unescape(raw_fields).replace("||", sentinel)
        fields = protected.split("|")
        answer = fields[0].replace(sentinel, "||") if len(fields) > 0 else ""
        error_hint = fields[1].replace(sentinel, "||") if len(fields) > 1 else ""
        placeholder = fields[2].replace(sentinel, "||") if len(fields) > 2 else "???"
        width = fields[3] if len(fields) > 3 else str(max(8, len(answer) + 2))
        return (
            f'<input type="text" class="code-blank" '
            f'data-answer="{html.escape(answer, quote=True)}" '
            f'data-error-hint="{html.escape(error_hint, quote=True)}" '
            f'placeholder="{html.escape(placeholder, quote=True)}" '
            f'style="width:{html.escape(width, quote=True)}ch" '
            f'aria-label="Fill in the blank: {html.escape(placeholder, quote=True)}" '
            f'autocomplete="off" spellcheck="false" />'
        )

    return pattern.sub(_repl, escaped_code)


def build_code_or_mermaid_html(info: str, code_text: str, allow_blanks: bool = False) -> str:
    info = info.strip()
    parts = info.split(None, 1)
    lang = parts[0] if parts else "text"
    attrs = parse_attrs(parts[1]) if len(parts) > 1 else {}
    caption = attrs.get("caption", "")

    if lang == "mermaid":
        cap_html = (
            f'<figcaption class="mermaid-caption">{html.escape(caption)}</figcaption>'
            if caption
            else ""
        )
        return (
            f'<figure class="mermaid-figure">'
            f'<pre class="mermaid">{html.escape(code_text.strip())}</pre>'
            f"{cap_html}</figure>"
        )

    escaped = html.escape(code_text.rstrip("\n"))
    if allow_blanks:
        escaped = convert_blank_tokens(escaped)
    cap_html = f'<div class="code-caption">{html.escape(caption)}</div>' if caption else ""
    return (
        f'<div class="code-figure">{cap_html}'
        f'<pre><code class="language-{html.escape(lang)}">{escaped}</code></pre>'
        f"</div>"
    )


def build_lesson_cards_html(lessons: list[PageMeta], prefix: str) -> str:
    cards: list[str] = ['<div class="card-grid">']
    for lesson in lessons:
        num = lesson.lesson_num or "0001"
        cleaned_badge = re.sub(r"^Lesson\s+\d+\s*[•·-]\s*", "", lesson.badge, flags=re.IGNORECASE)
        meta_label = f"LESSON {num} • {cleaned_badge}" if cleaned_badge else f"LESSON {num}"
        raw_title = lesson.card.get("title", lesson.title).replace("**", "")
        summary = lesson.card.get("summary", lesson.subtitle)
        href = f"{prefix}{lesson.rel_out_path}"
        cards.append(
            f'<a class="lesson-card" href="{html.escape(href)}">'
            f'<div><div class="lesson-card-meta">{html.escape(meta_label)}</div>'
            f'<div class="lesson-card-title">{html.escape(raw_title)}</div>'
            f'<p class="lesson-card-summary">{html.escape(summary)}</p></div>'
            f"</a>"
        )
    cards.append("</div>")
    return "\n".join(cards)


def build_reference_cards_html(refs: list[PageMeta], prefix: str) -> str:
    cards: list[str] = ['<div class="card-grid">']
    for ref in refs:
        raw_title = ref.card.get("title", ref.title).replace("**", "")
        summary = ref.card.get("summary", ref.subtitle)
        href = f"{prefix}{ref.rel_out_path}"
        cards.append(
            f'<a class="ref-card" href="{html.escape(href)}">'
            f'<div><div class="lesson-card-meta">REFERENCE SHEET • {html.escape(ref.nav_label)}</div>'
            f'<div class="lesson-card-title">{html.escape(raw_title)}</div>'
            f'<p class="lesson-card-summary">{html.escape(summary)}</p></div>'
            f"</a>"
        )
    cards.append("</div>")
    return "\n".join(cards)


def build_simulator_html(sim_id: str) -> str:
    if sim_id == "run-comparison-explorer":
        return (
            '<section class="simulator-card" data-simulator="run-comparison-explorer" '
            'aria-label="Interactive Baseline vs. Skilled Run Explorer">'
            '<div class="widget-header">'
            '<h3 class="widget-title">Interactive Explorer: Baseline Vibe-Coding vs. Skill-Driven Engineering</h3>'
            '<span class="widget-badge">Click Any Stage to Compare</span>'
            "</div>"
            "<p>Select an engineering stage below to compare what happened in the recorded <strong>Baseline Run</strong> "
            "(<code>01-baseline.cast</code>) vs. the <strong>Skilled Run</strong> (<code>02-skilled.cast</code>)—and when each approach makes sense:</p>"
            '<div class="sim-grid">'
            '<div class="sim-tree-list" role="toolbar" aria-label="Comparison stages"></div>'
            '<div class="sim-inspector" aria-live="polite"></div>'
            "</div>"
            "</section>"
        )
    if sim_id == "cloud-seam-simulator":
        return (
            '<section class="simulator-card" data-simulator="cloud-seam-simulator" '
            'aria-label="Interactive Cloud Seam & FinOps Simulator">'
            '<div class="widget-header">'
            '<h3 class="widget-title">Interactive Explorer: Composing mattpocock/skills + google/skills</h3>'
            '<span class="widget-badge">Click Any Layer to Inspect</span>'
            "</div>"
            "<p>Select an architectural layer below to inspect how the local workbench adapts for scale-to-zero "
            "<strong>Cloud Run + Cloud Firestore</strong> using Google Cloud agent skills:</p>"
            '<div class="sim-grid">'
            '<div class="sim-tree-list" role="toolbar" aria-label="Cloud architecture layers"></div>'
            '<div class="sim-inspector" aria-live="polite"></div>'
            "</div>"
            "</section>"
        )
    raise ValueError(f"Unknown widget/simulator id: {sim_id}")


def render_custom_blocks(
    md: MarkdownIt,
    text: str,
    source_path: Path,
    quiz_errors: list[str],
) -> tuple[str, dict[str, str]]:
    block_placeholders: dict[str, str] = {}
    counter = 0

    def next_token() -> str:
        nonlocal counter
        counter += 1
        return f"<!--CUSTOM_BLOCK_PLACEHOLDER_{counter}-->"

    # 1. :::exercise ... :::
    ex_pattern = re.compile(
        r"^:::exercise\s+([^\n]+)\n(.*?)^:::\s*$",
        re.MULTILINE | re.DOTALL,
    )

    def _replace_exercise(match: re.Match[str]) -> str:
        attrs = parse_attrs(match.group(1))
        body = match.group(2)
        ex_id = attrs.get("id", "ex-1")
        title = attrs.get("title", "Interactive Exercise")
        badge = attrs.get("badge", "Skill Practice")

        hint_text = ""
        success_text = "All blanks matched!"
        prompts_match = re.search(
            r"^:::prompts\s*\n(.*?)^:::\s*$",
            body,
            flags=re.MULTILINE | re.DOTALL,
        )
        if prompts_match:
            prompts_data = yaml.safe_load(prompts_match.group(1)) or {}
            hint_text = str(prompts_data.get("hint", "")).strip()
            success_text = str(prompts_data.get("success", success_text)).strip()
            body = body[: prompts_match.start()] + body[prompts_match.end() :]

        inner_html = render_markdown_fragment(md, body, allow_blanks=True)
        token = next_token()
        block_placeholders[token] = (
            f'<section class="exercise-card" id="{html.escape(ex_id)}" '
            f'data-hint="{html.escape(hint_text, quote=True)}" '
            f'data-success="{html.escape(success_text, quote=True)}">'
            f'<div class="widget-header">'
            f'<h3 class="widget-title">{html.escape(title)}</h3>'
            f'<span class="widget-badge">{html.escape(badge)}</span>'
            f"</div>"
            f"{inner_html}"
            f'<div class="widget-actions">'
            f'<button type="button" class="btn btn-primary" data-action="check-exercise">Check Answers</button>'
            f'<button type="button" class="btn" data-action="hint-exercise">Show Hint</button>'
            f"</div>"
            f'<div class="widget-feedback" role="status" aria-live="polite"></div>'
            f"</section>"
        )
        return token

    text = ex_pattern.sub(_replace_exercise, text)

    # 2. :::verify ... :::
    verify_pattern = re.compile(
        r"^:::verify\s+([^\n]+)\n(.*?)^:::\s*$",
        re.MULTILINE | re.DOTALL,
    )

    def _replace_verify(match: re.Match[str]) -> str:
        attrs = parse_attrs(match.group(1))
        body = match.group(2)
        v_id = attrs.get("id", "verify-1")
        title = attrs.get("title", "Verification Lab")
        badge = attrs.get("badge", "CLI Gate")

        checklist_items: list[str] = []
        prose_lines: list[str] = []
        for line in body.splitlines():
            m_chk = re.match(r"^\s*-\s*\[\s*\]\s+(.*)$", line)
            if m_chk:
                checklist_items.append(m_chk.group(1))
            else:
                prose_lines.append(line)

        inner_html = render_markdown_fragment(md, "\n".join(prose_lines), allow_blanks=False)
        chk_html = ""
        if checklist_items:
            items_rendered = []
            for idx, item_md in enumerate(checklist_items, start=1):
                storage_key = f"verify:{source_path.stem}:{v_id}:{idx}"
                item_inline = render_inline_md(md, rewrite_md_links(item_md))
                items_rendered.append(
                    f'<li class="verify-item">'
                    f'<input type="checkbox" id="{html.escape(v_id)}-chk-{idx}" '
                    f'data-storage-key="{html.escape(storage_key, quote=True)}" />'
                    f'<label for="{html.escape(v_id)}-chk-{idx}">{item_inline}</label>'
                    f"</li>"
                )
            chk_html = f'<ul class="verify-checklist">{"".join(items_rendered)}</ul>'

        token = next_token()
        block_placeholders[token] = (
            f'<section class="verify-card" id="{html.escape(v_id)}">'
            f'<div class="widget-header">'
            f'<h3 class="widget-title">{html.escape(title)}</h3>'
            f'<span class="widget-badge">{html.escape(badge)}</span>'
            f"</div>"
            f"{inner_html}{chk_html}"
            f"</section>"
        )
        return token

    text = verify_pattern.sub(_replace_verify, text)

    # 3. :::quiz ... :::
    quiz_pattern = re.compile(
        r"^:::quiz\s+([^\n]+)\n(.*?)^:::\s*$",
        re.MULTILINE | re.DOTALL,
    )

    def _replace_quiz(match: re.Match[str]) -> str:
        attrs = parse_attrs(match.group(1))
        body = match.group(2)
        q_id = attrs.get("id", "quiz-1")
        title = attrs.get("title", "Retrieval Practice")
        badge = attrs.get("badge", "Quiz")

        questions = yaml.safe_load(body) or []
        q_blocks_html: list[str] = []

        for q_idx, q in enumerate(questions, start=1):
            q_text = str(q.get("question", "")).strip()
            wrong_hint = str(q.get("wrong_hint", "")).strip()
            options = q.get("options") or []
            word_counts: list[int] = []
            opt_btns: list[str] = []

            for opt in options:
                letter = str(opt.get("letter", "A")).strip()
                opt_text = str(opt.get("text", "")).strip()
                is_correct = bool(opt.get("correct", False))
                explain = str(opt.get("explain", "")).strip()

                words = opt_text.split()
                word_counts.append(len(words))

                opt_btns.append(
                    f'<button type="button" class="quiz-option-btn" '
                    f'data-correct="{"true" if is_correct else "false"}" '
                    f'data-explain="{html.escape(explain, quote=True)}">'
                    f'<span class="quiz-option-letter">{html.escape(letter)}.</span>'
                    f'<span>{render_inline_md(md, opt_text)}</span>'
                    f"</button>"
                )

            if len(set(word_counts)) > 1:
                quiz_errors.append(
                    f"{source_path.name} ({q_id}, Q{q_idx}): Quiz options must have identical word counts, got {word_counts}"
                )

            q_blocks_html.append(
                f'<div class="quiz-question-block" data-wrong-hint="{html.escape(wrong_hint, quote=True)}">'
                f'<div class="quiz-question-title">{q_idx}. {render_inline_md(md, q_text)}</div>'
                f'<div class="quiz-options">{"".join(opt_btns)}</div>'
                f'<div class="widget-feedback" role="status" aria-live="polite"></div>'
                f"</div>"
            )

        token = next_token()
        block_placeholders[token] = (
            f'<section class="quiz-card" id="{html.escape(q_id)}">'
            f'<div class="widget-header">'
            f'<h3 class="widget-title">{html.escape(title)}</h3>'
            f'<span class="widget-badge">{html.escape(badge)}</span>'
            f"</div>"
            f'{"".join(q_blocks_html)}'
            f"</section>"
        )
        return token

    text = quiz_pattern.sub(_replace_quiz, text)

    # 4. :::callout* and :::teacher-prompt
    callout_pattern = re.compile(
        r"^:::(callout(?:-warn|-success)?|teacher-prompt)\s*\n(.*?)^:::\s*$",
        re.MULTILINE | re.DOTALL,
    )

    def _replace_callout(match: re.Match[str]) -> str:
        kind = match.group(1)
        body = match.group(2)
        css_class = "teacher-prompt" if kind == "teacher-prompt" else f"callout {kind}".strip()
        inner_html = render_markdown_fragment(md, body, allow_blanks=False)
        token = next_token()
        block_placeholders[token] = f'<aside class="{css_class}">{inner_html}</aside>'
        return token

    text = callout_pattern.sub(_replace_callout, text)
    return text, block_placeholders


def render_markdown_fragment(md: MarkdownIt, text: str, allow_blanks: bool = False) -> str:
    # CRITICAL RULE: Extract fenced code blocks into single-line placeholders BEFORE md.render()
    code_placeholders: dict[str, str] = {}
    code_counter = 0

    fenced_pattern = re.compile(
        r"^(`{3,}|~{3,})([^\n]*)\n(.*?)^\1\s*$",
        re.MULTILINE | re.DOTALL,
    )

    def _extract_code(match: re.Match[str]) -> str:
        nonlocal code_counter
        code_counter += 1
        placeholder = f"<!--FENCED_CODE_PLACEHOLDER_{code_counter}-->"
        info = match.group(2)
        code_body = match.group(3)
        code_placeholders[placeholder] = build_code_or_mermaid_html(
            info, code_body, allow_blanks=allow_blanks
        )
        return placeholder

    text_no_code = fenced_pattern.sub(_extract_code, text)

    # Rewrite internal .md links to .html
    text_no_code = rewrite_md_links(text_no_code)

    # Convert Table: captions before pipe tables into placeholders
    table_captions: dict[str, str] = {}
    cap_counter = 0

    def _extract_table_caption(match: re.Match[str]) -> str:
        nonlocal cap_counter
        cap_counter += 1
        marker = f"<!--TABLE_CAPTION_{cap_counter}-->"
        table_captions[marker] = match.group(1).strip()
        return f"{marker}\n"

    text_no_code = re.sub(
        r"^Table:\s*(.+)\n(?=\|)",
        _extract_table_caption,
        text_no_code,
        flags=re.MULTILINE,
    )

    # Convert {{sidenote:...}}
    def _replace_sidenote(match: re.Match[str]) -> str:
        note_md = match.group(1).strip()
        note_html = render_inline_md(md, note_md)
        return f'<span class="sidenote-number"></span><span class="sidenote">{note_html}</span>'

    text_no_code = re.sub(r"\{\{sidenote:([^}]+)\}\}", _replace_sidenote, text_no_code)

    rendered = md.render(text_no_code)

    # Support heading anchor suffixes {#id}
    rendered = re.sub(
        r"<(h[1-6])>(.*?)\s*\{#([a-zA-Z0-9_-]+)\}\s*</\1>",
        r'<\1 id="\3">\2</\1>',
        rendered,
    )

    # Style tables and attach captions + scope="col"
    rendered = re.sub(r"<th>", '<th scope="col">', rendered)

    for marker, cap_text in table_captions.items():
        rendered = re.sub(
            re.escape(marker) + r"\s*<table>",
            f'<div class="table-wrapper"><table class="ref-table"><caption>{html.escape(cap_text)}</caption>',
            rendered,
        )
    rendered = re.sub(r"<table>", '<div class="table-wrapper"><table class="ref-table">', rendered)
    rendered = re.sub(r"</table>", "</table></div>", rendered)

    # Restore fenced code placeholders AFTER md.render()
    for placeholder, code_html in code_placeholders.items():
        rendered = rendered.replace(f"<p>{placeholder}</p>", code_html)
        rendered = rendered.replace(placeholder, code_html)

    return rendered


def build_nav_html(
    page: PageMeta,
    lessons: list[PageMeta],
    refs: list[PageMeta],
    prefix: str,
) -> str:
    # Left: contextual breadcrumb trail
    if page.rel_out_path == "index.html":
        breadcrumbs = '<span class="nav-current" aria-current="page">Index</span>'
    elif page.lesson_num is not None:
        breadcrumbs = (
            f'<a href="{prefix}index.html">Index</a>'
            f'<span class="nav-sep" aria-hidden="true">›</span>'
            f'<span class="nav-current" aria-current="page">Lesson {html.escape(page.lesson_num)}</span>'
        )
    else:
        breadcrumbs = (
            f'<a href="{prefix}index.html">Index</a>'
            f'<span class="nav-sep" aria-hidden="true">›</span>'
            f'<span class="nav-current" aria-current="page">{html.escape(page.nav_label)}</span>'
        )

    # Right: compact pager on lesson pages + reference sheet links + mission badge
    controls: list[str] = []
    if page.lesson_num is not None:
        idx = next((i for i, l in enumerate(lessons) if l.rel_out_path == page.rel_out_path), 0)
        prev_l = lessons[idx - 1] if idx > 0 else None
        next_l = lessons[idx + 1] if idx + 1 < len(lessons) else None

        prev_html = (
            f'<a class="nav-pager-link" href="{prefix}{prev_l.rel_out_path}" '
            f'title="Lesson {html.escape(prev_l.lesson_num or "")}: {html.escape(prev_l.title)}">← Prev</a>'
            if prev_l
            else '<span class="nav-pager-disabled" aria-disabled="true">← Prev</span>'
        )
        next_html = (
            f'<a class="nav-pager-link" href="{prefix}{next_l.rel_out_path}" '
            f'title="Lesson {html.escape(next_l.lesson_num or "")}: {html.escape(next_l.title)}">Next →</a>'
            if next_l
            else '<span class="nav-pager-disabled" aria-disabled="true">Next →</span>'
        )
        controls.append(
            f'<span class="nav-pager-group" role="group" aria-label="Adjacent lessons">'
            f'{prev_html}<span class="nav-sep" aria-hidden="true">·</span>{next_html}'
            f"</span>"
            f'<span class="nav-divider" aria-hidden="true">|</span>'
        )

    for ref in refs:
        is_current = ' aria-current="page"' if page.rel_out_path == ref.rel_out_path else ""
        controls.append(
            f'<a class="nav-ref-link" href="{prefix}{ref.rel_out_path}"{is_current}>'
            f"{html.escape(ref.nav_label)}</a>"
        )

    controls.append('<span class="mission-badge">Antigravity • Skills Codelab</span>')

    return (
        '<header class="site-header">'
        '<nav class="course-nav" aria-label="Course navigation">'
        f'<div class="nav-breadcrumbs">{breadcrumbs}</div>'
        f'<div class="nav-controls">{"".join(controls)}</div>'
        "</nav>"
        "</header>"
    )


def build_lesson_pager_html(page: PageMeta, lessons: list[PageMeta], prefix: str) -> str:
    if page.lesson_num is None:
        return ""
    idx = next((i for i, l in enumerate(lessons) if l.rel_out_path == page.rel_out_path), 0)
    prev_l = lessons[idx - 1] if idx > 0 else None
    next_l = lessons[idx + 1] if idx + 1 < len(lessons) else None

    cards: list[str] = []
    if prev_l:
        cards.append(
            f'<a class="pager-card pager-card-prev" href="{prefix}{prev_l.rel_out_path}">'
            f'<span class="pager-meta">← Previous • Lesson {html.escape(prev_l.lesson_num or "")}</span>'
            f'<span class="pager-title">{html.escape(prev_l.title)}</span>'
            f"</a>"
        )
    else:
        cards.append(
            f'<a class="pager-card pager-card-prev" href="{prefix}index.html">'
            f'<span class="pager-meta">← Codelab Index</span>'
            f'<span class="pager-title">All Lessons &amp; Reference Sheets</span>'
            f"</a>"
        )

    if next_l:
        cards.append(
            f'<a class="pager-card pager-card-next" href="{prefix}{next_l.rel_out_path}">'
            f'<span class="pager-meta">Next • Lesson {html.escape(next_l.lesson_num or "")} →</span>'
            f'<span class="pager-title">{html.escape(next_l.title)}</span>'
            f"</a>"
        )
    else:
        cards.append(
            f'<a class="pager-card pager-card-next" href="{prefix}reference/agent-skills-workflow-cheatsheet.html">'
            f'<span class="pager-meta">Reference Sheet →</span>'
            f'<span class="pager-title">Vibe Coding vs. Agent Skills Decision &amp; Workflow Cheat Sheet</span>'
            f"</a>"
        )

    return f'<nav class="lesson-pager" aria-label="Lesson pagination">{"".join(cards)}</nav>'


def render_full_page(
    md: MarkdownIt,
    page: PageMeta,
    lessons: list[PageMeta],
    refs: list[PageMeta],
    quiz_errors: list[str],
) -> str:
    depth = page.rel_out_path.count("/")
    prefix = "../" * depth

    body_with_blocks, custom_blocks = render_custom_blocks(
        md, page.body_md, page.source_path, quiz_errors
    )
    rendered_body = render_markdown_fragment(md, body_with_blocks, allow_blanks=False)

    for token, block_html in custom_blocks.items():
        rendered_body = rendered_body.replace(f"<p>{token}</p>", block_html)
        rendered_body = rendered_body.replace(token, block_html)

    # Replace widget directives
    rendered_body = rendered_body.replace(
        "<p>{{widget:lesson-cards}}</p>", build_lesson_cards_html(lessons, prefix)
    )
    rendered_body = rendered_body.replace(
        "{{widget:lesson-cards}}", build_lesson_cards_html(lessons, prefix)
    )
    rendered_body = rendered_body.replace(
        "<p>{{widget:reference-cards}}</p>", build_reference_cards_html(refs, prefix)
    )
    rendered_body = rendered_body.replace(
        "{{widget:reference-cards}}", build_reference_cards_html(refs, prefix)
    )

    def _replace_sim(match: re.Match[str]) -> str:
        return build_simulator_html(match.group(1))

    rendered_body = re.sub(
        r"(?:<p>)?\{\{widget:([a-zA-Z0-9_-]+)\}\}(?:</p>)?",
        _replace_sim,
        rendered_body,
    )

    nav_html = build_nav_html(page, lessons, refs, prefix)
    pager_html = build_lesson_pager_html(page, lessons, prefix)
    badge_html = (
        f'<div class="article-badge">{html.escape(page.badge)}</div>' if page.badge else ""
    )
    subtitle_html = (
        f'<p class="article-subtitle">{html.escape(page.subtitle)}</p>'
        if page.subtitle
        else ""
    )

    return f"""<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <meta name="color-scheme" content="dark" />
  <title>{html.escape(page.page_title)}</title>
  <link rel="stylesheet" href="{prefix}assets/style.css" />
  <script src="{prefix}assets/course.js" defer></script>
</head>
<body>
  <a class="skip-link" href="#main-content">Skip to main content</a>
  {nav_html}
  <main id="main-content" class="page-main">
    <article class="tufte-article">
      <header class="article-header">
        {badge_html}
        <h1>{html.escape(page.title)}</h1>
        {subtitle_html}
      </header>
      {rendered_body}
      {pager_html}
    </article>
  </main>
  <footer class="site-footer">
    <span>Shaping AI Coding Agents with Skills — Interactive Codelab (Baseline Vibe-Coding, mattpocock/skills &amp; google/skills)</span>
  </footer>
</body>
</html>
"""


def validate_rendered_html(rel_out_path: str, html_text: str, out_dir: Path) -> list[str]:
    errors: list[str] = []

    # 1. Verify no <pre><code...> contains <p>, <h1..6>, or nested <pre> tags
    for match in re.finditer(r"<pre><code[^>]*>(.*?)</code></pre>", html_text, flags=re.DOTALL):
        inner = match.group(1)
        if re.search(r"<(?:p|h[1-6]|pre)\b", inner):
            errors.append(
                f"{rel_out_path}: Corrupted <pre><code> block contains nested HTML block tags (<p>, <h1..6>, or <pre>)"
            )

    # 2. Verify no raw {#anchor} tokens leaked
    if re.search(r"\{#[a-zA-Z0-9_-]+\}", html_text):
        errors.append(f"{rel_out_path}: Unprocessed heading anchor token '{{#...}}' leaked into HTML")

    # 3. Verify internal href links resolve inside out_dir
    page_dir = (out_dir / rel_out_path).parent
    for m_href in re.finditer(r'href="([^"]+)"', html_text):
        href = m_href.group(1)
        if href.startswith(("http://", "https://", "mailto:", "#")):
            continue
        clean_href = href.split("#", 1)[0]
        if not clean_href:
            continue
        target = (page_dir / clean_href).resolve()
        if not target.exists():
            errors.append(f"{rel_out_path}: Broken internal link href='{href}' -> {target}")

    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description="Build Codelab Markdown to HTML.")
    parser.add_argument(
        "--out-dir",
        type=Path,
        default=ROOT_DIR,
        help="Output directory for generated HTML (defaults to workspace root).",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="Validate quiz option word counts, internal links, and code blocks.",
    )
    args = parser.parse_args()
    out_dir: Path = args.out_dir.resolve()

    md = MarkdownIt("commonmark", {"html": True, "typographer": True}).enable("table")

    index_page = load_page_meta(ROOT_DIR / "index.md", "index.html")

    lessons: list[PageMeta] = []
    lessons_dir = ROOT_DIR / "lessons"
    if lessons_dir.exists():
        for l_path in sorted(lessons_dir.glob("*.md")):
            m_num = re.match(r"^(\d{4})-", l_path.name)
            l_num = m_num.group(1) if m_num else "0001"
            lessons.append(
                load_page_meta(l_path, f"lessons/{l_path.stem}.html", lesson_num=l_num)
            )

    refs: list[PageMeta] = []
    ref_dir = ROOT_DIR / "reference"
    if ref_dir.exists():
        for r_path in sorted(ref_dir.glob("*.md")):
            out_name = "glossary.html" if r_path.name == "GLOSSARY.md" else f"{r_path.stem}.html"
            refs.append(load_page_meta(r_path, f"reference/{out_name}"))

    # Copy assets if building into a separate _site directory
    if out_dir != ROOT_DIR:
        out_dir.mkdir(parents=True, exist_ok=True)
        (out_dir / ".nojekyll").write_text("", encoding="utf-8")
        dest_assets = out_dir / "assets"
        if dest_assets.exists():
            shutil.rmtree(dest_assets)
        shutil.copytree(ROOT_DIR / "assets", dest_assets)

    all_pages = [index_page, *lessons, *refs]
    quiz_errors: list[str] = []
    rendered_pages: dict[str, str] = {}

    for page in all_pages:
        html_out = render_full_page(md, page, lessons, refs, quiz_errors)
        dest_file = out_dir / page.rel_out_path
        dest_file.parent.mkdir(parents=True, exist_ok=True)
        dest_file.write_text(html_out, encoding="utf-8")
        rendered_pages[page.rel_out_path] = html_out

    if args.check:
        validation_errors = list(quiz_errors)
        for rel_path, html_text in rendered_pages.items():
            validation_errors.extend(validate_rendered_html(rel_path, html_text, out_dir))
        if validation_errors:
            for err in validation_errors:
                print(f"ERROR: {err}", file=sys.stderr)
            return 1

    print(
        f"Built {len(all_pages)} page(s) ({len(lessons)} lesson(s), {len(refs)} reference sheet(s)) -> {out_dir}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
