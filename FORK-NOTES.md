# Fork and deployment notes

## Source and attribution

This is a hosting fork of [domesticmouse/antigravity-cloud-presentation](https://github.com/domesticmouse/antigravity-cloud-presentation), based on revision `7f103f41033d139913433084fd0d5085353e5e0c`.

The original presentation credits **Brett Morgan**, Developer Relations Engineer, Google. Original slide content, recordings, codelab, author credits, source links and QR codes are preserved. The existing follow-along QR code intentionally continues to identify the upstream presentation, not this fork.

The upstream [Apache License, Version 2.0](LICENSE) remains unchanged. This hosting fork does not claim authorship of the original work or endorsement by its authors or Google.

## Fork deployment

- Fork: https://github.com/MD-Str8nUp/antigravity-cloud-presentation
- Intended Pages URL: https://md-str8nup.github.io/antigravity-cloud-presentation/
- Required direct link: https://md-str8nup.github.io/antigravity-cloud-presentation/6
- Existing workflow: `.github/workflows/deploy-slides.yml`.

The workflow builds the static Python codelab, installs the locked Slidev dependencies, builds Slidev, and publishes `slides/dist` with GitHub Pages. It does not provision or execute the Google Cloud examples discussed in the presentation.

## Hosting-only changes

`slides/package.json` adds a `postbuild` step that copies the app entry point to `dist/6/index.html`. Static hosting can therefore redirect `/6` to `/6/` and serve the app successfully on direct navigation and refresh. The existing repository base path is retained. Slidev's generated `404.html` is retained for other client-side routes; these may use Pages' 404 fallback rather than return an initial HTTP 200.

The postbuild step also adds `.nojekyll` and includes this attribution note and the unchanged `LICENSE` in the deployed artifact. The workflow watches these two attribution files so subsequent changes trigger deployment.

## Local build

From the repository root, create a local Python environment, install `markdown-it-py` and `pyyaml`, then run `codelab/.venv/bin/python codelab/build.py --check`. In `slides`, run `npm ci` and `npm run build`. No skill installation, GCP account, paid service or global package install is required.

This preparation does not itself push, enable Pages, change GitHub settings, or verify a live deployment. Local verification evidence is kept separately in `.local-build/` (not committed).
