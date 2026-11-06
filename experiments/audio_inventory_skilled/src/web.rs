use crate::builds::{
    AttachmentKind, BenchLogEntry, BenchLogEntryType, BenchPlacementProgress, BomCandidateMatch,
    BomLine, BuildAttachment, BuildReadinessSummary, CreateBenchLogEntry, CreateBomLine,
    CreateKitBuild, FulfillmentStatus, KitBuild, KitBuildStatus, UpdateKitBuild, add_bom_line,
    add_build_attachment_file, add_build_external_link, bom_line_placements,
    classify_bom_line_stage, clone_kit_build, compute_bench_placement_progress,
    create_bench_log_entry, create_component_from_bom_line, create_kit_build,
    find_bom_line_candidates, get_bom_line, get_build_attachment, get_kit_build,
    group_bom_lines_by_stage, link_bom_line_component, list_bench_log_entries,
    list_build_attachments, list_kit_builds, mark_all_bom_line_designators_installed,
    pick_all_reserved, read_build_attachment_bytes, reserve_all_available,
    shelve_kit_build_and_return_parts, split_bom_line, substitute_bom_line_component,
    toggle_bom_line_designator, transition_bom_line_status, update_kit_build,
    update_kit_build_status,
};
use crate::catalog::{
    Component, ComponentQueryFilter, CreateComponent, CreateInitialStockLot, CreateStockLot,
    CreateStorageLocation, LocationKind, StorageLocation, UpdateComponent, adjust_stock_lot_delta,
    category_spec_fields, create_component, create_storage_location, get_component,
    get_storage_location, list_components, list_storage_locations, list_subtree_location_ids,
    merge_component_stock_lots, move_stock_lot_to_location, move_storage_location,
    query_components, update_component, update_storage_location, upsert_or_merge_stock_lot,
};
use crate::db::Db;
use crate::imports::{
    BomImportMode, ColumnMapping, ImportBatchDetails, ImportChosenAction, ImportMatchStatus,
    ImportPreset, ImportRow, SaveImportPresetInput, StageBomImportInput, StageComponentImportInput,
    commit_bom_import_batch, commit_component_import_batch, get_import_batch, get_import_row,
    list_import_presets, save_import_preset, stage_bom_import, stage_component_import,
    update_import_row_resolution,
};
use crate::shopping_list::{
    ReceiveShoppingListLine, ShoppingListQuery, ShoppingListReport, compute_shopping_list,
    export_full_shopping_list_csv, export_supplier_quick_order_csv, receive_shopping_list_lines,
    upsert_shopping_list_override,
};
use crate::spec_parser::{Category, parse_spec};
use axum::Router;
use axum::body::Bytes;
use axum::extract::{Form, Multipart, Path, Query, RawQuery, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use maud::{DOCTYPE, Markup, html};
use serde::Deserialize;
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::collections::BTreeMap;

const HTMX_JS: &str = include_str!("../assets/htmx.min.js");
const STYLES_CSS: &str = include_str!("../assets/styles.css");

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceTab {
    Catalog,
    Builds,
    Bench,
    ShoppingList,
    Imports,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct CatalogQuery {
    pub q: Option<String>,
    pub category: Option<String>,
    pub location_id: Option<String>,
    pub package_or_mounting: Option<String>,
    pub voltage_rating_volts: Option<String>,
    pub inspect_component_id: Option<String>,
    pub inspect_location_id: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, String>,
}

impl CatalogQuery {
    pub fn to_filter(&self) -> ComponentQueryFilter {
        let mut spec_filters = BTreeMap::new();
        for (k, v) in &self.extra {
            if let Some(spec_key) = k.strip_prefix("spec_") {
                let trimmed = v.trim();
                if !trimmed.is_empty() {
                    spec_filters.insert(spec_key.to_string(), trimmed.to_string());
                }
            }
        }
        let voltage_rating_volts = self
            .voltage_rating_volts
            .as_deref()
            .and_then(|s| s.trim().parse::<f64>().ok());

        ComponentQueryFilter {
            q: self.q.clone(),
            category: self.category.clone(),
            location_id: self.location_id.clone(),
            package_or_mounting: self.package_or_mounting.clone(),
            voltage_rating_volts,
            spec_filters,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateLocationForm {
    pub name: String,
    pub kind: Option<String>,
    pub parent_id: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MoveLocationForm {
    pub parent_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLocationForm {
    pub name: String,
    pub kind: String,
    pub parent_id: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateComponentForm {
    pub name: Option<String>,
    pub category: String,
    pub subtype: Option<String>,
    pub package_or_mounting: Option<String>,
    pub raw_spec_text: String,
    pub canonical_value_display: Option<String>,
    pub resistance_ohms: Option<String>,
    pub capacitance_pf: Option<String>,
    pub inductance_uh: Option<String>,
    pub voltage_rating_volts: Option<String>,
    pub low_stock_threshold: Option<String>,
    pub datasheet_url: Option<String>,
    pub notes: Option<String>,
    pub initial_location_id: Option<String>,
    pub initial_quantity: Option<String>,
    pub initial_supplier: Option<String>,
    pub initial_supplier_part_no: Option<String>,
    pub initial_lot_notes: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateComponentForm {
    pub name: String,
    pub category: String,
    pub subtype: Option<String>,
    pub package_or_mounting: Option<String>,
    pub raw_spec_text: String,
    pub canonical_value_display: Option<String>,
    pub resistance_ohms: Option<String>,
    pub capacitance_pf: Option<String>,
    pub inductance_uh: Option<String>,
    pub voltage_rating_volts: Option<String>,
    pub low_stock_threshold: Option<String>,
    pub datasheet_url: Option<String>,
    pub notes: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct AddStockLotForm {
    pub location_id: Option<String>,
    pub quantity: Option<i64>,
    pub supplier: Option<String>,
    pub supplier_part_no: Option<String>,
    pub lot_notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LotDeltaForm {
    pub delta: Option<i64>,
    pub by: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct MoveStockLotForm {
    pub target_location_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MergeLotsForm {
    pub target_location_id: Option<String>,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct BuildsQuery {
    pub kit_id: Option<String>,
    pub inspect_bom_line_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateKitBuildForm {
    pub name: String,
    pub project_type: Option<String>,
    pub status: Option<String>,
    pub location_id: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateKitBuildForm {
    pub name: String,
    pub project_type: Option<String>,
    pub status: String,
    pub location_id: Option<String>,
    pub description: Option<String>,
    pub return_parts: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateKitBuildStatusForm {
    pub status: String,
    pub return_parts: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBomLineForm {
    pub raw_spec_text: String,
    pub category: Option<String>,
    pub package_or_mounting: Option<String>,
    pub quantity_required: Option<i64>,
    pub reference_designators: Option<String>,
    pub included_in_kit: Option<String>,
    pub component_id: Option<String>,
    pub substitution_note: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct TransitionBomStatusForm {
    pub status: Option<String>,
    pub source_lot_id: Option<String>,
    pub quantity_allocated: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct LinkBomComponentForm {
    pub component_id: Option<String>,
    pub substitution_note: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateComponentFromBomForm {
    pub initial_location_id: Option<String>,
    pub initial_quantity: Option<i64>,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct ImportsQuery {
    pub batch_id: Option<String>,
    pub target_kind: Option<String>,
    pub kit_build_id: Option<String>,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct ImportColumnMappingFormFields {
    pub col_raw_spec_text: Option<String>,
    pub col_name: Option<String>,
    pub col_category: Option<String>,
    pub col_subtype: Option<String>,
    pub col_package_or_mounting: Option<String>,
    pub col_quantity: Option<String>,
    pub col_location_path: Option<String>,
    pub col_supplier: Option<String>,
    pub col_supplier_part_no: Option<String>,
    pub col_datasheet_url: Option<String>,
    pub col_reference_designators: Option<String>,
    pub col_included_in_kit: Option<String>,
    pub col_notes: Option<String>,
}

impl ImportColumnMappingFormFields {
    pub fn into_column_mapping(self) -> ColumnMapping {
        ColumnMapping {
            raw_spec_text: self.col_raw_spec_text,
            name: self.col_name,
            category: self.col_category,
            subtype: self.col_subtype,
            package_or_mounting: self.col_package_or_mounting,
            quantity: self.col_quantity,
            location_path: self.col_location_path,
            supplier: self.col_supplier,
            supplier_part_no: self.col_supplier_part_no,
            datasheet_url: self.col_datasheet_url,
            reference_designators: self.col_reference_designators,
            included_in_kit: self.col_included_in_kit,
            notes: self.col_notes,
        }
    }

    pub fn apply_multipart_field(&mut self, name: &str, val: String) -> bool {
        match name {
            "col_raw_spec_text" => self.col_raw_spec_text = Some(val),
            "col_name" => self.col_name = Some(val),
            "col_category" => self.col_category = Some(val),
            "col_subtype" => self.col_subtype = Some(val),
            "col_package_or_mounting" => self.col_package_or_mounting = Some(val),
            "col_quantity" => self.col_quantity = Some(val),
            "col_location_path" => self.col_location_path = Some(val),
            "col_supplier" => self.col_supplier = Some(val),
            "col_supplier_part_no" => self.col_supplier_part_no = Some(val),
            "col_datasheet_url" => self.col_datasheet_url = Some(val),
            "col_reference_designators" => self.col_reference_designators = Some(val),
            "col_included_in_kit" => self.col_included_in_kit = Some(val),
            "col_notes" => self.col_notes = Some(val),
            _ => return false,
        }
        true
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct StageComponentImportForm {
    pub csv_text: Option<String>,
    pub default_location_id: Option<String>,
    pub default_supplier: Option<String>,
    pub preset_name: Option<String>,
    #[serde(flatten)]
    pub cols: ImportColumnMappingFormFields,
}

#[derive(Debug, Default, Deserialize)]
pub struct StageBomImportForm {
    pub kit_build_id: Option<String>,
    pub csv_text: Option<String>,
    pub import_mode: Option<String>,
    pub group_designators: Option<String>,
    pub preset_name: Option<String>,
    #[serde(flatten)]
    pub cols: ImportColumnMappingFormFields,
}

#[derive(Debug, Deserialize)]
pub struct SaveImportPresetForm {
    pub name: String,
    pub target_kind: Option<String>,
    #[serde(flatten)]
    pub cols: ImportColumnMappingFormFields,
}

#[derive(Debug, Deserialize)]
pub struct ResolveImportRowForm {
    pub chosen_action: String,
    pub matched_component_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SubstituteBomComponentForm {
    pub component_id: String,
    pub substitution_note: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct SplitBomLineForm {
    pub split_quantity: Option<i64>,
    pub split_designators: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct CloneKitBuildForm {
    pub name: Option<String>,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct BenchQuery {
    pub kit_id: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct ToggleDesignatorForm {
    pub designator: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct AddExternalLinkForm {
    pub kind: Option<String>,
    pub label: Option<String>,
    pub title: Option<String>,
    pub external_url: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct CreateBenchLogForm {
    pub entry_type: Option<String>,
    pub body: Option<String>,
}

pub enum CatalogInspectorState<'a> {
    Component(&'a Component),
    Location {
        location: &'a StorageLocation,
        excluded_subtree_ids: Vec<String>,
    },
    NewComponent,
}

pub fn app_router(db: Db) -> Router {
    let state = AppState { db };
    Router::new()
        .route("/", get(catalog_page))
        .route("/catalog", get(catalog_page))
        .route("/catalog/components", post(create_component_handler))
        .route(
            "/catalog/components/{id}",
            post(update_component_handler),
        )
        .route(
            "/catalog/components/{id}/inspector",
            get(inspect_component_handler),
        )
        .route(
            "/catalog/components/{id}/lots",
            post(add_stock_lot_handler),
        )
        .route(
            "/catalog/components/{id}/merge-lots",
            post(merge_component_lots_handler),
        )
        .route("/catalog/lots/{lot_id}/delta", post(adjust_lot_delta_handler))
        .route("/catalog/lots/{lot_id}/move", post(move_lot_handler))
        .route("/locations", get(locations_page).post(create_location_handler))
        .route("/locations/{id}/inspect", get(inspect_location_handler))
        .route("/locations/{id}/move", post(move_location_handler))
        .route("/locations/{id}/update", post(update_location_handler))
        .route("/builds", get(builds_page).post(create_kit_build_handler))
        .route(
            "/builds/{id}",
            get(build_detail_page).post(update_kit_build_handler),
        )
        .route(
            "/builds/{id}/status",
            post(update_kit_build_status_handler),
        )
        .route(
            "/builds/{id}/shelve",
            post(shelve_kit_build_handler),
        )
        .route(
            "/builds/{id}/clone",
            post(clone_kit_build_handler),
        )
        .route(
            "/builds/{id}/reserve-all",
            post(reserve_all_available_handler),
        )
        .route(
            "/builds/{id}/pick-all",
            post(pick_all_reserved_handler),
        )
        .route(
            "/builds/{id}/bom-lines",
            post(create_bom_line_handler),
        )
        .route(
            "/builds/{id}/bom",
            post(create_bom_line_handler),
        )
        .route(
            "/builds/bom-lines/{line_id}/inspector",
            get(inspect_bom_line_handler),
        )
        .route(
            "/builds/{kit_id}/bom/{line_id}/inspector",
            get(inspect_kit_bom_line_handler),
        )
        .route(
            "/builds/bom-lines/{line_id}/status",
            post(transition_bom_line_status_handler).patch(transition_bom_line_status_handler),
        )
        .route(
            "/builds/{kit_id}/bom/{line_id}/status",
            post(transition_kit_bom_line_status_handler)
                .patch(transition_kit_bom_line_status_handler),
        )
        .route(
            "/builds/bom-lines/{line_id}/link-component",
            post(link_bom_line_component_handler),
        )
        .route(
            "/builds/{kit_id}/bom/{line_id}/link-component",
            post(link_kit_bom_line_component_handler),
        )
        .route(
            "/builds/bom-lines/{line_id}/substitute-modal",
            get(substitute_bom_line_modal_handler),
        )
        .route(
            "/builds/{kit_id}/bom/{line_id}/substitute-modal",
            get(substitute_kit_bom_line_modal_handler),
        )
        .route(
            "/builds/bom-lines/{line_id}/substitute",
            post(substitute_bom_line_handler),
        )
        .route(
            "/builds/{kit_id}/bom/{line_id}/substitute",
            post(substitute_kit_bom_line_handler),
        )
        .route(
            "/builds/bom-lines/{line_id}/split",
            post(split_bom_line_handler),
        )
        .route(
            "/builds/{kit_id}/bom/{line_id}/split",
            post(split_kit_bom_line_handler),
        )
        .route(
            "/builds/bom-lines/{line_id}/create-component",
            post(create_component_from_bom_line_handler),
        )
        .route(
            "/builds/{kit_id}/bom/{line_id}/create-component",
            post(create_component_from_kit_bom_line_handler),
        )
        .route("/bench", get(bench_page))
        .route("/bench/{kit_build_id}", get(bench_detail_page))
        .route("/checklist", get(bench_page))
        .route("/checklist/{kit_build_id}", get(bench_detail_page))
        .route("/builds/{kit_build_id}/bench", get(bench_detail_page))
        .route(
            "/bench/bom-lines/{line_id}/toggle-designator",
            post(toggle_designator_form_handler).patch(toggle_designator_form_handler),
        )
        .route(
            "/bench/bom-lines/{line_id}/designator/{des}",
            post(toggle_designator_path_handler).patch(toggle_designator_path_handler),
        )
        .route(
            "/builds/{kit_id}/bench/{line_id}/designator/{des}",
            post(toggle_kit_designator_path_handler).patch(toggle_kit_designator_path_handler),
        )
        .route(
            "/bench/bom-lines/{line_id}/install-all",
            post(install_all_designators_handler).patch(install_all_designators_handler),
        )
        .route(
            "/builds/{kit_id}/bench/{line_id}/install-all",
            post(install_all_kit_designators_handler).patch(install_all_kit_designators_handler),
        )
        .route(
            "/bench/{kit_build_id}/attachments",
            post(upload_build_attachment_handler),
        )
        .route(
            "/builds/{kit_build_id}/attachments",
            post(upload_build_attachment_handler),
        )
        .route(
            "/bench/{kit_build_id}/links",
            post(add_build_link_handler),
        )
        .route(
            "/builds/{kit_build_id}/links",
            post(add_build_link_handler),
        )
        .route(
            "/bench/{kit_build_id}/logs",
            post(create_bench_log_handler),
        )
        .route(
            "/builds/{kit_build_id}/bench-logs",
            post(create_bench_log_handler),
        )
        .route(
            "/attachments/{attachment_id}/download",
            get(download_attachment_handler),
        )
        .route(
            "/bench/attachments/{attachment_id}",
            get(download_attachment_handler),
        )
        .route("/shopping-list", get(shopping_list_page))
        .route(
            "/shopping-list/override",
            post(shopping_list_override_handler),
        )
        .route(
            "/shopping-list/export/full.csv",
            get(export_full_shopping_list_csv_handler),
        )
        .route(
            "/shopping-list/export/quick-order.csv",
            get(export_quick_order_shopping_list_csv_handler),
        )
        .route("/shopping-list/receive", post(receive_shopping_list_handler))
        .route("/imports", get(imports_page))
        .route(
            "/imports/components/stage",
            post(stage_component_import_handler),
        )
        .route("/imports/bom/stage", post(stage_bom_import_handler))
        .route("/imports/presets", post(save_import_preset_handler))
        .route(
            "/imports/rows/{row_id}/resolve",
            post(resolve_import_row_handler),
        )
        .route(
            "/imports/batches/{batch_id}/commit",
            post(commit_import_batch_handler),
        )
        .route("/settings", get(settings_page))
        .route("/settings/backup.zip", get(download_backup_zip_handler))
        .route("/settings/restore", post(restore_backup_zip_handler))
        .route("/assets/htmx.min.js", get(serve_htmx_js))
        .route("/assets/styles.css", get(serve_styles_css))
        .with_state(state)
}

pub fn is_htmx_request(headers: &HeaderMap) -> bool {
    headers
        .get("HX-Request")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v == "true")
}

pub fn render_page_or_partial(
    headers: &HeaderMap,
    active_tab: SurfaceTab,
    content: Markup,
) -> Html<String> {
    if is_htmx_request(headers) {
        let hx_target = headers
            .get("HX-Target")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if hx_target == "app-root" {
            Html(
                html! {
                    (content)
                    (render_subnav_inner(active_tab, true))
                }
                .into_string(),
            )
        } else {
            Html(content.into_string())
        }
    } else {
        Html(workbench_shell(active_tab, content).into_string())
    }
}

pub fn workbench_shell(active_tab: SurfaceTab, workspace_content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="UTF-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                meta name="color-scheme" content="light dark";
                title { "Audio Electronics Inventory & Kit Builds" }
                link rel="stylesheet" href="/assets/styles.css";
                script src="/assets/htmx.min.js" defer {}
            }
            body {
                header class="proto-banner" {
                    div style="display:flex; align-items:center; gap:0.65rem; flex-wrap:wrap;" {
                        span class="proto-badge" { "WORKBENCH" }
                        strong { "Audio Electronics Inventory & Kit Builds" }
                    }
                    div style="display:flex; align-items:center; gap:0.65rem; flex-wrap:wrap;" {
                        a class="btn xs" href="/locations"
                            hx-get="/locations"
                            hx-target="#app-root"
                            hx-push-url="true" {
                            "Storage Locations"
                        }
                        a class="btn xs" href="/settings"
                            hx-get="/settings"
                            hx-target="#app-root"
                            hx-push-url="true" {
                            "Backup & Restore"
                        }
                        a class="btn xs" href="/settings/backup.zip" {
                            "Download Backup (.zip)"
                        }
                    }
                }
                div class="workspace" {
                    (render_subnav(active_tab))
                    main id="app-root" {
                        (workspace_content)
                    }
                }
                dialog id="bom-action-dialog" closedby="any" aria-labelledby="bom-dialog-title" {
                    h2 id="bom-dialog-title" style="margin-block-start:0; font-size:1.05rem;" { "Action" }
                    div id="bom-dialog-content" {}
                }
            }
        }
    }
}

pub fn render_subnav(active_tab: SurfaceTab) -> Markup {
    render_subnav_inner(active_tab, false)
}

fn render_subnav_inner(active_tab: SurfaceTab, oob: bool) -> Markup {
    let tabs = [
        (SurfaceTab::Catalog, "Component Catalog", "/catalog"),
        (SurfaceTab::Builds, "Kit Build BOM", "/builds"),
        (SurfaceTab::Bench, "Active Bench Checklist", "/checklist"),
        (SurfaceTab::ShoppingList, "Shopping List", "/shopping-list"),
        (SurfaceTab::Imports, "CSV Import", "/imports"),
    ];

    html! {
        nav id="workbench-subnav"
            class="va-subnav"
            aria-label="Workbench Surfaces"
            hx-swap-oob=[if oob { Some("true") } else { None }] {
            div class="va-tabs" role="tablist" {
                @for (tab, label, href) in tabs {
                    a class="btn"
                        href=(href)
                        role="tab"
                        aria-selected=(if active_tab == tab { "true" } else { "false" })
                        hx-get=(href)
                        hx-target="#app-root"
                        hx-push-url="true" {
                        (label)
                    }
                }
            }
            div style="display:flex; align-items:center; gap:0.5rem;" {
                span class="badge neutral" { "Local Turso SQLite · Foreign Keys ON" }
            }
        }
    }
}

async fn serve_htmx_js() -> Response {
    ([(header::CONTENT_TYPE, "application/javascript; charset=utf-8")], HTMX_JS).into_response()
}

async fn serve_styles_css() -> Response {
    ([(header::CONTENT_TYPE, "text/css; charset=utf-8")], STYLES_CSS).into_response()
}

fn parse_optional_f64(s: &Option<String>) -> Option<f64> {
    s.as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .and_then(|v| v.parse::<f64>().ok())
}

fn parse_optional_i64(s: &Option<String>) -> Option<i64> {
    s.as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .and_then(|v| v.parse::<i64>().ok())
}

fn extract_extra_specs_json(extra: &BTreeMap<String, String>) -> Option<JsonValue> {
    let mut map = JsonMap::new();
    for (k, v) in extra {
        if let Some(spec_key) = k.strip_prefix("spec_") {
            let trimmed = v.trim();
            if !trimmed.is_empty() {
                map.insert(spec_key.to_string(), JsonValue::String(trimmed.to_string()));
            }
        }
    }
    if map.is_empty() {
        None
    } else {
        Some(JsonValue::Object(map))
    }
}

async fn load_and_render_catalog_workspace(
    db: &Db,
    query: &CatalogQuery,
    flash_error: Option<&str>,
) -> Result<Markup, (StatusCode, String)> {
    let locations = list_storage_locations(db).await.map_err(internal_err)?;
    let filter = query.to_filter();
    let query_result = query_components(db, &filter).await.map_err(internal_err)?;

    let inspected_component = if let Some(ref cid) = query.inspect_component_id {
        get_component(db, cid).await.map_err(internal_err)?
    } else {
        None
    };

    let inspected_location = if inspected_component.is_none() {
        if let Some(ref loc_id) = query.inspect_location_id {
            get_storage_location(db, loc_id)
                .await
                .map_err(internal_err)?
        } else if let Some(ref loc_id) = query.location_id {
            get_storage_location(db, loc_id)
                .await
                .map_err(internal_err)?
        } else {
            None
        }
    } else {
        None
    };

    let excluded_subtree_ids = if let Some(ref loc) = inspected_location {
        list_subtree_location_ids(db, &loc.id)
            .await
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    let inspector_state = if let Some(ref comp) = inspected_component {
        CatalogInspectorState::Component(comp)
    } else if let Some(ref loc) = inspected_location {
        CatalogInspectorState::Location {
            location: loc,
            excluded_subtree_ids,
        }
    } else if let Some(first_comp) = query_result.components.first() {
        CatalogInspectorState::Component(first_comp)
    } else if let Some(first_loc) = locations.first() {
        let subtree_ids = list_subtree_location_ids(db, &first_loc.id)
            .await
            .unwrap_or_default();
        CatalogInspectorState::Location {
            location: first_loc,
            excluded_subtree_ids: subtree_ids,
        }
    } else {
        CatalogInspectorState::NewComponent
    };

    Ok(render_catalog_and_locations_workspace(
        query,
        &query_result.components,
        query_result.parsed_query.as_ref(),
        &locations,
        inspector_state,
        flash_error,
    ))
}

async fn catalog_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<CatalogQuery>,
) -> Result<Html<String>, (StatusCode, String)> {
    let content = load_and_render_catalog_workspace(&state.db, &query, None).await?;
    Ok(render_page_or_partial(&headers, SurfaceTab::Catalog, content))
}

async fn locations_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<CatalogQuery>,
) -> Result<Html<String>, (StatusCode, String)> {
    let content = load_and_render_catalog_workspace(&state.db, &query, None).await?;
    Ok(render_page_or_partial(&headers, SurfaceTab::Catalog, content))
}

async fn create_component_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<CreateComponentForm>,
) -> Result<Html<String>, (StatusCode, String)> {
    let initial_qty = parse_optional_i64(&form.initial_quantity).unwrap_or(0);
    let initial_loc = form
        .initial_location_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    let initial_lot = if initial_qty > 0 || initial_loc.is_some() {
        Some(CreateInitialStockLot {
            location_id: initial_loc,
            quantity: initial_qty.max(0),
            supplier: form.initial_supplier.unwrap_or_default(),
            supplier_part_no: form.initial_supplier_part_no.unwrap_or_default(),
            lot_notes: form.initial_lot_notes.unwrap_or_default(),
        })
    } else {
        None
    };

    let created = create_component(
        &state.db,
        CreateComponent {
            name: form.name.unwrap_or_default(),
            category: form.category,
            subtype: form.subtype.unwrap_or_default(),
            package_or_mounting: form.package_or_mounting.unwrap_or_default(),
            raw_spec_text: form.raw_spec_text,
            canonical_value_display: form.canonical_value_display,
            resistance_ohms: parse_optional_f64(&form.resistance_ohms),
            capacitance_pf: parse_optional_f64(&form.capacitance_pf),
            inductance_uh: parse_optional_f64(&form.inductance_uh),
            voltage_rating_volts: parse_optional_f64(&form.voltage_rating_volts),
            low_stock_threshold: parse_optional_i64(&form.low_stock_threshold).unwrap_or(0),
            datasheet_url: form.datasheet_url.unwrap_or_default(),
            notes: form.notes.unwrap_or_default(),
            specs_json: extract_extra_specs_json(&form.extra),
            initial_lot,
        },
    )
    .await;

    let (inspect_component_id, flash_err) = match created {
        Ok(c) => (Some(c.id), None),
        Err(e) => (None, Some(e.to_string())),
    };

    let query = CatalogQuery {
        inspect_component_id,
        ..Default::default()
    };
    let content =
        load_and_render_catalog_workspace(&state.db, &query, flash_err.as_deref()).await?;
    Ok(render_page_or_partial(&headers, SurfaceTab::Catalog, content))
}

async fn update_component_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Form(form): Form<UpdateComponentForm>,
) -> Result<Html<String>, (StatusCode, String)> {
    let updated = update_component(
        &state.db,
        &id,
        UpdateComponent {
            name: form.name,
            category: form.category,
            subtype: form.subtype.unwrap_or_default(),
            package_or_mounting: form.package_or_mounting.unwrap_or_default(),
            raw_spec_text: form.raw_spec_text,
            canonical_value_display: form.canonical_value_display,
            resistance_ohms: parse_optional_f64(&form.resistance_ohms),
            capacitance_pf: parse_optional_f64(&form.capacitance_pf),
            inductance_uh: parse_optional_f64(&form.inductance_uh),
            voltage_rating_volts: parse_optional_f64(&form.voltage_rating_volts),
            low_stock_threshold: parse_optional_i64(&form.low_stock_threshold).unwrap_or(0),
            datasheet_url: form.datasheet_url.unwrap_or_default(),
            notes: form.notes.unwrap_or_default(),
            specs_json: extract_extra_specs_json(&form.extra),
        },
    )
    .await;

    let flash_err = updated.err().map(|e| e.to_string());
    let query = CatalogQuery {
        inspect_component_id: Some(id),
        ..Default::default()
    };
    let content =
        load_and_render_catalog_workspace(&state.db, &query, flash_err.as_deref()).await?;
    Ok(render_page_or_partial(&headers, SurfaceTab::Catalog, content))
}

async fn inspect_component_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Html<String>, (StatusCode, String)> {
    let locations = list_storage_locations(&state.db)
        .await
        .map_err(internal_err)?;
    let comp = get_component(&state.db, &id)
        .await
        .map_err(internal_err)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Component '{id}' not found")))?;

    if is_htmx_request(&headers) {
        let drawer = render_catalog_inspector_drawer(
            CatalogInspectorState::Component(&comp),
            &locations,
            None,
        );
        Ok(Html(drawer.into_string()))
    } else {
        let query = CatalogQuery {
            inspect_component_id: Some(id),
            ..Default::default()
        };
        let content = load_and_render_catalog_workspace(&state.db, &query, None).await?;
        Ok(Html(
            workbench_shell(SurfaceTab::Catalog, content).into_string(),
        ))
    }
}

async fn add_stock_lot_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Form(form): Form<AddStockLotForm>,
) -> Result<Html<String>, (StatusCode, String)> {
    let res = upsert_or_merge_stock_lot(
        &state.db,
        CreateStockLot {
            component_id: id.clone(),
            location_id: form.location_id,
            quantity: form.quantity.unwrap_or(1),
            supplier: form.supplier.unwrap_or_default(),
            supplier_part_no: form.supplier_part_no.unwrap_or_default(),
            lot_notes: form.lot_notes.unwrap_or_default(),
        },
    )
    .await;

    let flash_err = res.err().map(|e| e.to_string());
    let query = CatalogQuery {
        inspect_component_id: Some(id),
        ..Default::default()
    };
    let content =
        load_and_render_catalog_workspace(&state.db, &query, flash_err.as_deref()).await?;
    Ok(render_page_or_partial(&headers, SurfaceTab::Catalog, content))
}

async fn adjust_lot_delta_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(lot_id): Path<String>,
    Query(q_delta): Query<LotDeltaForm>,
    Form(f_delta): Form<LotDeltaForm>,
) -> Result<Html<String>, (StatusCode, String)> {
    let delta = f_delta
        .delta
        .or(f_delta.by)
        .or(q_delta.delta)
        .or(q_delta.by)
        .unwrap_or(0);

    let comp = adjust_stock_lot_delta(&state.db, &lot_id, delta)
        .await
        .map_err(internal_err)?;

    let hx_target = headers
        .get("HX-Target")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if is_htmx_request(&headers) && hx_target != "catalog-workspace" && hx_target != "inspector" {
        Ok(Html(render_component_row(&comp, false).into_string()))
    } else if is_htmx_request(&headers) && hx_target == "inspector" {
        let locations = list_storage_locations(&state.db)
            .await
            .map_err(internal_err)?;
        let drawer = render_catalog_inspector_drawer(
            CatalogInspectorState::Component(&comp),
            &locations,
            None,
        );
        // Also include out-of-band swap for the component table row!
        let row_oob = render_component_row_oob(&comp);
        let combined = html! {
            (drawer)
            (row_oob)
        };
        Ok(Html(combined.into_string()))
    } else {
        let query = CatalogQuery {
            inspect_component_id: Some(comp.id),
            ..Default::default()
        };
        let content = load_and_render_catalog_workspace(&state.db, &query, None).await?;
        Ok(render_page_or_partial(&headers, SurfaceTab::Catalog, content))
    }
}

async fn move_lot_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(lot_id): Path<String>,
    Form(form): Form<MoveStockLotForm>,
) -> Result<Html<String>, (StatusCode, String)> {
    let res =
        move_stock_lot_to_location(&state.db, &lot_id, form.target_location_id.as_deref()).await;
    let (comp_id, flash_err) = match res {
        Ok(c) => (Some(c.id), None),
        Err(e) => (None, Some(e.to_string())),
    };
    let query = CatalogQuery {
        inspect_component_id: comp_id,
        ..Default::default()
    };
    let content =
        load_and_render_catalog_workspace(&state.db, &query, flash_err.as_deref()).await?;
    Ok(render_page_or_partial(&headers, SurfaceTab::Catalog, content))
}

async fn merge_component_lots_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Form(form): Form<MergeLotsForm>,
) -> Result<Html<String>, (StatusCode, String)> {
    let res =
        merge_component_stock_lots(&state.db, &id, form.target_location_id.as_deref()).await;
    let flash_err = res.err().map(|e| e.to_string());
    let query = CatalogQuery {
        inspect_component_id: Some(id),
        ..Default::default()
    };
    let content =
        load_and_render_catalog_workspace(&state.db, &query, flash_err.as_deref()).await?;
    Ok(render_page_or_partial(&headers, SurfaceTab::Catalog, content))
}

async fn create_location_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<CreateLocationForm>,
) -> Result<Html<String>, (StatusCode, String)> {
    let created = create_storage_location(
        &state.db,
        CreateStorageLocation {
            name: form.name,
            kind: form.kind.unwrap_or_else(|| "Other".to_string()),
            parent_id: form.parent_id,
            notes: form.notes.unwrap_or_default(),
        },
    )
    .await;

    let (inspect_location_id, flash_err) = match created {
        Ok(loc) => (Some(loc.id), None),
        Err(e) => (None, Some(e.to_string())),
    };

    let query = CatalogQuery {
        inspect_location_id,
        ..Default::default()
    };
    let content =
        load_and_render_catalog_workspace(&state.db, &query, flash_err.as_deref()).await?;
    Ok(render_page_or_partial(&headers, SurfaceTab::Catalog, content))
}

async fn inspect_location_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Html<String>, (StatusCode, String)> {
    let locations = list_storage_locations(&state.db)
        .await
        .map_err(internal_err)?;
    let inspected = get_storage_location(&state.db, &id)
        .await
        .map_err(internal_err)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Location '{id}' not found")))?;

    if is_htmx_request(&headers) {
        let subtree_ids = list_subtree_location_ids(&state.db, &id)
            .await
            .unwrap_or_default();
        let drawer = render_catalog_inspector_drawer(
            CatalogInspectorState::Location {
                location: &inspected,
                excluded_subtree_ids: subtree_ids,
            },
            &locations,
            None,
        );
        Ok(Html(drawer.into_string()))
    } else {
        let query = CatalogQuery {
            inspect_location_id: Some(id),
            ..Default::default()
        };
        let content = load_and_render_catalog_workspace(&state.db, &query, None).await?;
        Ok(Html(
            workbench_shell(SurfaceTab::Catalog, content).into_string(),
        ))
    }
}

async fn move_location_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Form(form): Form<MoveLocationForm>,
) -> Result<Html<String>, (StatusCode, String)> {
    let move_result = move_storage_location(&state.db, &id, form.parent_id.as_deref()).await;
    let flash_err = move_result.err().map(|e| e.to_string());

    let query = CatalogQuery {
        inspect_location_id: Some(id),
        ..Default::default()
    };
    let content =
        load_and_render_catalog_workspace(&state.db, &query, flash_err.as_deref()).await?;
    Ok(render_page_or_partial(&headers, SurfaceTab::Catalog, content))
}

async fn update_location_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Form(form): Form<UpdateLocationForm>,
) -> Result<Html<String>, (StatusCode, String)> {
    let update_result = update_storage_location(
        &state.db,
        &id,
        &form.name,
        &form.kind,
        form.parent_id.as_deref(),
        form.notes.as_deref().unwrap_or(""),
    )
    .await;
    let flash_err = update_result.err().map(|e| e.to_string());

    let query = CatalogQuery {
        inspect_location_id: Some(id),
        ..Default::default()
    };
    let content =
        load_and_render_catalog_workspace(&state.db, &query, flash_err.as_deref()).await?;
    Ok(render_page_or_partial(&headers, SurfaceTab::Catalog, content))
}

fn format_specs_summary(specs_json: &JsonValue) -> String {
    if let JsonValue::Object(map) = specs_json {
        let mut parts = Vec::new();
        for (k, v) in map {
            let val_str = match v {
                JsonValue::String(s) if !s.is_empty() => s.clone(),
                JsonValue::Number(n) => n.to_string(),
                JsonValue::Bool(b) => b.to_string(),
                _ => continue,
            };
            parts.push(format!("{k}: {val_str}"));
        }
        parts.join(" · ")
    } else {
        String::new()
    }
}

fn render_component_row(comp: &Component, is_selected: bool) -> Markup {
    render_component_row_inner(comp, is_selected, false)
}

fn render_component_row_oob(comp: &Component) -> Markup {
    html! {
        template {
            (render_component_row_inner(comp, true, true))
        }
    }
}

fn render_component_row_inner(comp: &Component, is_selected: bool, oob: bool) -> Markup {
    let single_lot = if comp.lots.len() == 1 {
        comp.lots.first()
    } else {
        None
    };
    let badge_class = if comp.available_stock == 0 && comp.low_stock_threshold > 0 {
        "badge err"
    } else if comp.is_low_stock {
        "badge warn"
    } else {
        "badge ok"
    };
    let specs_summary = format_specs_summary(&comp.specs_json);

    html! {
        tr id=(format!("comp-row-{}", comp.id))
            hx-swap-oob=[if oob { Some("true") } else { None }]
            style=(if is_selected { "outline: 2px solid var(--brand-color); outline-offset:-2px;" } else { "" }) {
            th scope="row" {
                a href=(format!("/catalog?inspect_component_id={}", comp.id))
                    hx-get=(format!("/catalog/components/{}/inspector", comp.id))
                    hx-target="#inspector"
                    hx-swap="outerHTML"
                    style="color:inherit; text-decoration:none; font-weight:600;" {
                    (comp.name)
                }
                @if !comp.subtype.is_empty() {
                    div style="font-size:0.76rem; color:var(--text-muted); font-weight:400;" {
                        (comp.subtype)
                    }
                }
            }
            td {
                div { (comp.category) }
                @if !comp.package_or_mounting.is_empty() {
                    small style="color:var(--text-muted);" { (comp.package_or_mounting) }
                }
            }
            td {
                strong class="mono" { (comp.canonical_value_display) }
                div style="color:var(--text-muted); font-size:0.8rem;" {
                    "raw: " code { (comp.raw_spec_text) }
                }
                @if !specs_summary.is_empty() {
                    div style="color:var(--text-muted); font-size:0.75rem;" {
                        (specs_summary)
                    }
                }
            }
            td {
                @if comp.lots.is_empty() {
                    span style="color:var(--text-muted); font-size:0.8rem;" { "No Stock Lots" }
                } @else {
                    @for lot in &comp.lots {
                        div style="font-size:0.82rem;" {
                            code { (lot.location_breadcrumb) }
                            " "
                            strong { "(" (lot.quantity) ")" }
                        }
                    }
                }
            }
            td {
                span class=(badge_class) {
                    (comp.available_stock) " Avail"
                }
                span style="color:var(--text-muted); font-size:0.8rem;" {
                    " / " (comp.total_on_hand) " Total"
                    @if comp.reserved_quantity > 0 {
                        " (" (comp.reserved_quantity) " Rsv)"
                    }
                }
                @if comp.is_low_stock {
                    div style="margin-block-start:0.2rem;" {
                        span class="badge warn" {
                            "Low Stock (≤ " (comp.low_stock_threshold) ")"
                        }
                    }
                }
            }
            td {
                div style="display:flex; align-items:center; gap:0.35rem; flex-wrap:wrap;" {
                    @if let Some(lot) = single_lot {
                        div class="stepper" {
                            button type="button"
                                hx-post=(format!("/catalog/lots/{}/delta", lot.id))
                                hx-vals=r#"{"delta": -1}"#
                                hx-target=(format!("#comp-row-{}", comp.id))
                                hx-swap="outerHTML"
                                aria-label=(format!("Decrement {}", comp.name)) {
                                "−"
                            }
                            span { (lot.quantity) }
                            button type="button"
                                hx-post=(format!("/catalog/lots/{}/delta", lot.id))
                                hx-vals=r#"{"delta": 1}"#
                                hx-target=(format!("#comp-row-{}", comp.id))
                                hx-swap="outerHTML"
                                aria-label=(format!("Increment {}", comp.name)) {
                                "+"
                            }
                        }
                        button type="button"
                            class="xs"
                            hx-get=(format!("/catalog/components/{}/inspector", comp.id))
                            hx-target="#inspector"
                            hx-swap="outerHTML" {
                            "Inspect ▸"
                        }
                    } @else if comp.lots.len() > 1 {
                        button type="button"
                            class="xs"
                            hx-get=(format!("/catalog/components/{}/inspector", comp.id))
                            hx-target="#inspector"
                            hx-swap="outerHTML" {
                            (comp.lots.len()) " Stock Lots (" (comp.total_on_hand) ") ▸"
                        }
                    } @else {
                        button type="button"
                            class="xs"
                            hx-get=(format!("/catalog/components/{}/inspector", comp.id))
                            hx-target="#inspector"
                            hx-swap="outerHTML" {
                            "Add Stock Lot ▸"
                        }
                    }
                }
            }
        }
    }
}

fn render_catalog_and_locations_workspace(
    query: &CatalogQuery,
    components: &[Component],
    parsed_query: Option<&crate::spec_parser::ParsedSpec>,
    locations: &[StorageLocation],
    inspector_state: CatalogInspectorState<'_>,
    flash_error: Option<&str>,
) -> Markup {
    let q_str = query.q.as_deref().unwrap_or("").trim();
    let cat_selected = query
        .category
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty() && *s != "All")
        .and_then(Category::from_label);

    let parsed_badge = parsed_query
        .and_then(|p| p.summary_badge())
        .or_else(|| {
            if !q_str.is_empty() {
                parse_spec(q_str, cat_selected).summary_badge()
            } else {
                None
            }
        });

    let selected_comp_id = match inspector_state {
        CatalogInspectorState::Component(c) => Some(c.id.as_str()),
        _ => None,
    };

    html! {
        div id="catalog-workspace" {
            @if let Some(err) = flash_error {
                div class="card" style="background:var(--err-bg); color:var(--err-text); margin-block-end:0.85rem; font-weight:600;" role="alert" {
                    (err)
                }
            }

            form class="va-filter-ribbon"
                method="get"
                action="/catalog"
                hx-get="/catalog"
                hx-trigger="input changed delay:150ms from:find input, change from:find select, search, submit"
                hx-target="#catalog-workspace"
                hx-push-url="true" {
                input type="search"
                    name="q"
                    value=(q_str)
                    placeholder="Search spec shorthand (e.g. 4k7, 4700R, 0.047uF, 47n, 100kB, TL072)..."
                    style="min-inline-size: 260px; flex: 1;"
                    hx-get="/catalog"
                    hx-trigger="input changed delay:150ms, search"
                    hx-target="#catalog-workspace"
                    hx-include="closest form"
                    hx-push-url="true"
                    aria-label="Search components and specs";

                @if let Some(ref badge) = parsed_badge {
                    span class="badge info" id="spec-parser-badge" {
                        "spec_parser: " (badge)
                    }
                }

                select name="category" aria-label="Filter by Component Category" {
                    option value="" { "All Categories" }
                    @for cat in Category::ALL {
                        option value=(cat.as_str())
                            selected[cat_selected == Some(*cat)] {
                            (cat.as_str())
                        }
                    }
                }

                select name="location_id" aria-label="Filter by Storage Location" {
                    option value="" { "All Storage Locations" }
                    @for loc in locations {
                        option value=(loc.id)
                            selected[query.location_id.as_deref() == Some(loc.id.as_str())] {
                            (loc.breadcrumb_path) " (" (loc.kind) ")"
                        }
                    }
                }

                @if let Some(cat) = cat_selected {
                    @let fields = category_spec_fields(cat);
                    @if !fields.is_empty() {
                        div style="display:flex; gap:0.4rem; flex-wrap:wrap; inline-size:100%; padding-block-start:0.35rem; border-block-start:1px dashed var(--border-color);" {
                            span style="font-size:0.78rem; font-weight:600; color:var(--text-muted); align-self:center;" {
                                (cat.as_str()) " Attributes:"
                            }
                            @for &(field_key, field_label) in fields {
                                @let param_name = format!("spec_{field_key}");
                                @let current_val = query.extra.get(&param_name).map(String::as_str).unwrap_or("");
                                input type="text"
                                    name=(param_name)
                                    value=(current_val)
                                    placeholder=(field_label)
                                    aria-label=(format!("Filter by {field_label}"))
                                    style="inline-size: 135px; font-size:0.8rem;";
                            }
                            @if cat == Category::Capacitors {
                                input type="text"
                                    name="voltage_rating_volts"
                                    value=(query.voltage_rating_volts.as_deref().unwrap_or(""))
                                    placeholder="Voltage (V)"
                                    aria-label="Filter by Voltage Rating (V)"
                                    style="inline-size: 110px; font-size:0.8rem;";
                            }
                        }
                    }
                }

                button type="submit" class="xs" { "Filter" }
            }

            div class="va-grid-with-drawer" {
                div style="display:flex; flex-direction:column; gap:1rem;" {
                    div class="table-wrapper" role="region" aria-labelledby="catalog-table-caption" tabindex="0" {
                        table style="--label-2:'Category'; --label-3:'Canonical & Verbatim Spec'; --label-4:'Storage Locations'; --label-5:'Available Stock'; --label-6:'Quick Adjust';" {
                            caption id="catalog-table-caption" {
                                "Component Catalog (" (components.len()) " matching components)"
                            }
                            thead {
                                tr {
                                    th scope="col" { "Component" }
                                    th scope="col" { "Category & Package" }
                                    th scope="col" { "Canonical & Verbatim Spec" }
                                    th scope="col" { "Storage Locations (Stock Lots)" }
                                    th scope="col" { "Available / Total Stock" }
                                    th scope="col" { "Quick Adjust (HTMX)" }
                                }
                            }
                            tbody id="catalog-table-body" {
                                @if components.is_empty() {
                                    tr {
                                        td colspan="6" style="color:var(--text-muted); text-align:center; padding:1.25rem;" {
                                            "No matching Components found. Create a Component in the right-hand Inspector Drawer or clear active filters."
                                        }
                                    }
                                } @else {
                                    @for comp in components {
                                        (render_component_row(comp, selected_comp_id == Some(comp.id.as_str())))
                                    }
                                }
                            }
                        }
                    }

                    div class="table-wrapper" role="region" aria-labelledby="locations-table-caption" tabindex="0" {
                        table {
                            caption id="locations-table-caption" {
                                "Hierarchical Storage Locations (" (locations.len()) ")"
                            }
                            thead {
                                tr {
                                    th scope="col" { "Storage Location" }
                                    th scope="col" { "Kind" }
                                    th scope="col" { "Full Breadcrumb Path" }
                                    th scope="col" { "Nested Containers" }
                                    th scope="col" { "Notes" }
                                    th scope="col" { "Actions" }
                                }
                            }
                            tbody {
                                @if locations.is_empty() {
                                    tr {
                                        td colspan="6" style="color:var(--text-muted); text-align:center; padding:1.25rem;" {
                                            "No Storage Locations yet. Add a Crate, Box, Binder, Drawer, or Bag in the right-hand Inspector Drawer."
                                        }
                                    }
                                } @else {
                                    @for loc in locations {
                                        tr id=(format!("location-row-{}", loc.id)) {
                                            th scope="row" {
                                                span style=(format!("padding-inline-start: {}rem;", loc.depth as f64 * 0.85)) {
                                                    @if loc.depth > 0 { "↳ " }
                                                    (loc.name)
                                                }
                                            }
                                            td {
                                                span class="badge neutral" { (loc.kind) }
                                            }
                                            td class="mono" {
                                                (loc.breadcrumb_path)
                                            }
                                            td {
                                                (loc.child_count) " child"
                                            }
                                            td style="color:var(--text-muted);" {
                                                (loc.notes)
                                            }
                                            td {
                                                button type="button"
                                                    class="xs"
                                                    hx-get=(format!("/locations/{}/inspect", loc.id))
                                                    hx-target="#inspector"
                                                    hx-swap="outerHTML" {
                                                    "Inspect / Move ▸"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                (render_catalog_inspector_drawer(inspector_state, locations, None))
            }
        }
    }
}

fn render_catalog_inspector_drawer(
    state: CatalogInspectorState<'_>,
    all_locations: &[StorageLocation],
    flash_error: Option<&str>,
) -> Markup {
    html! {
        aside id="inspector" class="card" aria-label="Catalog & Storage Inspector Drawer" {
            @if let Some(err) = flash_error {
                div style="background:var(--err-bg); color:var(--err-text); padding:0.5rem; border-radius:4px; margin-block-end:0.65rem; font-size:0.82rem; font-weight:600;" {
                    (err)
                }
            }

            @match state {
                CatalogInspectorState::Component(comp) => {
                    div style="border-block-end:1px solid var(--border-color); padding-block-end:0.65rem; margin-block-end:0.75rem;" {
                        div style="display:flex; justify-content:space-between; align-items:start; gap:0.5rem;" {
                            h3 style="margin:0; font-size:0.98rem;" { (comp.name) }
                            span class="badge info mono" { (comp.canonical_value_display) }
                        }
                        div style="font-size:0.8rem; color:var(--text-muted); margin-block-start:0.3rem;" {
                            "Verbatim Spec: " code { (comp.raw_spec_text) }
                            br;
                            "Category: " strong { (comp.category) }
                            @if !comp.package_or_mounting.is_empty() {
                                " · Package: " (comp.package_or_mounting)
                            }
                        }
                        div style="margin-block-start:0.4rem; display:flex; gap:0.35rem; flex-wrap:wrap;" {
                            span class=(if comp.is_low_stock { "badge warn" } else { "badge ok" }) {
                                (comp.available_stock) " Avail / " (comp.total_on_hand) " On Hand"
                            }
                            @if comp.reserved_quantity > 0 {
                                span class="badge neutral" { (comp.reserved_quantity) " Reserved" }
                            }
                            @if !comp.datasheet_url.is_empty() {
                                a class="btn xs" href=(comp.datasheet_url) target="_blank" rel="noopener" { "Datasheet ↗" }
                            }
                        }
                    }

                    // Multi-lot / Per-lot Steppers & Merge Duplicate Stock Lots
                    h4 style="margin:0 0 0.45rem 0; font-size:0.82rem; text-transform:uppercase; color:var(--text-muted);" {
                        "Stock Lots (" (comp.lots.len()) ")"
                    }
                    @for lot in &comp.lots {
                        div style="border:1px solid var(--border-color); border-radius:5px; padding:0.5rem; margin-block-end:0.5rem; background:var(--bg-subtle);" {
                            div style="font-size:0.8rem; font-weight:600;" {
                                code { (lot.location_breadcrumb) }
                            }
                            div style="display:flex; justify-content:space-between; align-items:center; margin-block-start:0.35rem; gap:0.4rem;" {
                                small style="color:var(--text-muted);" {
                                    @if !lot.supplier.is_empty() || !lot.supplier_part_no.is_empty() {
                                        (lot.supplier) " #" (lot.supplier_part_no)
                                    } @else {
                                        "No supplier info"
                                    }
                                    @if !lot.lot_notes.is_empty() {
                                        " · " (lot.lot_notes)
                                    }
                                }
                                div class="stepper" {
                                    button type="button"
                                        hx-post=(format!("/catalog/lots/{}/delta", lot.id))
                                        hx-vals=r#"{"delta": -1}"#
                                        hx-target="#inspector"
                                        hx-swap="outerHTML"
                                        aria-label="Decrement lot quantity" {
                                        "−"
                                    }
                                    span { (lot.quantity) }
                                    button type="button"
                                        hx-post=(format!("/catalog/lots/{}/delta", lot.id))
                                        hx-vals=r#"{"delta": 1}"#
                                        hx-target="#inspector"
                                        hx-swap="outerHTML"
                                        aria-label="Increment lot quantity" {
                                        "+"
                                    }
                                }
                            }
                        }
                    }

                    @if comp.lots.len() > 1 {
                        form method="post"
                            action=(format!("/catalog/components/{}/merge-lots", comp.id))
                            hx-post=(format!("/catalog/components/{}/merge-lots", comp.id))
                            hx-target="#catalog-workspace"
                            hx-swap="outerHTML"
                            style="margin-block-end:0.85rem; padding-block-end:0.75rem; border-block-end:1px solid var(--border-color);" {
                            div class="form-field" style="margin-block-end:0.4rem;" {
                                label for="merge-target-loc" { "Consolidate into Storage Location" }
                                select id="merge-target-loc" name="target_location_id" {
                                    @for lot in &comp.lots {
                                        @if let Some(ref lid) = lot.location_id {
                                            option value=(lid) { (lot.location_breadcrumb) " (" (lot.quantity) " pcs)" }
                                        }
                                    }
                                    @for loc in all_locations {
                                        option value=(loc.id) { (loc.breadcrumb_path) }
                                    }
                                }
                            }
                            button type="submit" class="primary xs" style="inline-size:100%; justify-content:center;" {
                                "Merge Duplicate Stock Lots"
                            }
                        }
                    }

                    // Add Stock Lot Form
                    details style="margin-block-end:0.75rem; border-block-end:1px solid var(--border-color); padding-block-end:0.65rem;" {
                        summary style="cursor:pointer; font-weight:600; font-size:0.84rem;" {
                            "+ Add / Receive Stock Lot for " (comp.canonical_value_display)
                        }
                        form class="form-grid"
                            method="post"
                            action=(format!("/catalog/components/{}/lots", comp.id))
                            hx-post=(format!("/catalog/components/{}/lots", comp.id))
                            hx-target="#catalog-workspace"
                            hx-swap="outerHTML"
                            style="margin-block-start:0.5rem;" {
                            div class="form-field" {
                                label { "Storage Location" }
                                select name="location_id" {
                                    option value="" { "— Unassigned —" }
                                    @for loc in all_locations {
                                        option value=(loc.id) { (loc.breadcrumb_path) }
                                    }
                                }
                            }
                            div class="form-field" {
                                label { "Quantity" }
                                input type="number" name="quantity" value="10" min="0" required;
                            }
                            div class="form-field" {
                                label { "Supplier & Part #" }
                                div style="display:flex; gap:0.35rem;" {
                                    input type="text" name="supplier" placeholder="Supplier (e.g. Mouser)" style="flex:1;";
                                    input type="text" name="supplier_part_no" placeholder="Part #" style="flex:1;";
                                }
                            }
                            div class="form-field" {
                                label { "Lot Notes" }
                                input type="text" name="lot_notes" placeholder="Optional lot note...";
                            }
                            button type="submit" class="primary xs" { "Add / Merge Stock Lot" }
                        }
                    }

                    // Edit Component Form
                    details style="margin-block-end:0.75rem; border-block-end:1px solid var(--border-color); padding-block-end:0.65rem;" {
                        summary style="cursor:pointer; font-weight:600; font-size:0.84rem;" {
                            "Edit Component Metadata & Specs"
                        }
                        form class="form-grid"
                            method="post"
                            action=(format!("/catalog/components/{}", comp.id))
                            hx-post=(format!("/catalog/components/{}", comp.id))
                            hx-target="#catalog-workspace"
                            hx-swap="outerHTML"
                            style="margin-block-start:0.5rem;" {
                            div class="form-field" {
                                label { "Name" }
                                input type="text" name="name" value=(comp.name) required;
                            }
                            div class="form-field" {
                                label { "Category" }
                                select name="category" {
                                    @for cat in Category::ALL {
                                        option value=(cat.as_str()) selected[comp.category == cat.as_str()] {
                                            (cat.as_str())
                                        }
                                    }
                                }
                            }
                            div class="form-field" {
                                label { "Verbatim Spec (raw_spec_text)" }
                                input type="text" name="raw_spec_text" value=(comp.raw_spec_text) required;
                            }
                            div style="display:flex; gap:0.35rem;" {
                                div class="form-field" style="flex:1;" {
                                    label { "Subtype" }
                                    input type="text" name="subtype" value=(comp.subtype);
                                }
                                div class="form-field" style="flex:1;" {
                                    label { "Package / Mounting" }
                                    input type="text" name="package_or_mounting" value=(comp.package_or_mounting);
                                }
                            }
                            div style="display:flex; gap:0.35rem;" {
                                div class="form-field" style="flex:1;" {
                                    label { "Low-Stock Threshold" }
                                    input type="number" name="low_stock_threshold" value=(comp.low_stock_threshold) min="0";
                                }
                                div class="form-field" style="flex:1;" {
                                    label { "Datasheet URL" }
                                    input type="text" name="datasheet_url" value=(comp.datasheet_url);
                                }
                            }
                            div class="form-field" {
                                label { "Notes" }
                                input type="text" name="notes" value=(comp.notes);
                            }
                            button type="submit" class="primary xs" { "Save Component Changes" }
                        }
                    }
                }
                CatalogInspectorState::Location { location: loc, ref excluded_subtree_ids } => {
                    div style="border-block-end:1px solid var(--border-color); padding-block-end:0.65rem; margin-block-end:0.75rem;" {
                        div style="display:flex; justify-content:space-between; align-items:center; gap:0.5rem;" {
                            span class="badge info" { (loc.kind) }
                            span class="mono" style="font-size:0.75rem; color:var(--text-muted);" { (loc.id) }
                        }
                        h3 style="margin:0.35rem 0 0.2rem 0; font-size:1rem;" { (loc.name) }
                        div class="mono" style="font-size:0.82rem; color:var(--text-muted);" {
                            (loc.breadcrumb_path)
                        }
                    }

                    form class="form-grid"
                        method="post"
                        action=(format!("/locations/{}/move", loc.id))
                        hx-post=(format!("/locations/{}/move", loc.id))
                        hx-target="#catalog-workspace"
                        hx-swap="outerHTML"
                        style="margin-block-end:1rem; padding-block-end:0.85rem; border-block-end:1px solid var(--border-color);" {
                        div class="form-field" {
                            label for="move-parent-select" { "Move Storage Location to Parent Container" }
                            select id="move-parent-select" name="parent_id" {
                                option value="" selected[loc.parent_id.is_none()] { "— Top-Level Root Container —" }
                                @for candidate in all_locations {
                                    @if candidate.id != loc.id && !excluded_subtree_ids.contains(&candidate.id) {
                                        option value=(candidate.id)
                                            selected[loc.parent_id.as_deref() == Some(candidate.id.as_str())] {
                                            (candidate.breadcrumb_path) " (" (candidate.kind) ")"
                                        }
                                    }
                                }
                            }
                        }
                        button type="submit" class="primary xs" { "Move Container & Update Breadcrumbs" }
                    }
                }
                CatalogInspectorState::NewComponent => {}
            }

            // Always render Create Component & Create Storage Location forms in the drawer
            details open[matches!(state, CatalogInspectorState::NewComponent)]
                style="margin-block-end:0.75rem; border-block-end:1px solid var(--border-color); padding-block-end:0.65rem;" {
                summary style="cursor:pointer; font-weight:600; font-size:0.88rem;" {
                    "+ New Catalog Component"
                }
                form class="form-grid"
                    method="post"
                    action="/catalog/components"
                    hx-post="/catalog/components"
                    hx-target="#catalog-workspace"
                    hx-swap="outerHTML"
                    style="margin-block-start:0.5rem;" {
                    div class="form-field" {
                        label for="new-comp-spec" { "Specification Shorthand (raw_spec_text)" }
                        input id="new-comp-spec" type="text" name="raw_spec_text" required placeholder="e.g. 4k7 1% 1/4W Metal Film, 0.047uF 63V Box Film";
                    }
                    div class="form-field" {
                        label for="new-comp-name" { "Display Name (Optional — auto-derived if blank)" }
                        input id="new-comp-name" type="text" name="name" placeholder="e.g. 4.7kΩ 1% Metal Film Resistor";
                    }
                    div class="form-field" {
                        label for="new-comp-cat" { "Category" }
                        select id="new-comp-cat" name="category" {
                            @for cat in Category::ALL {
                                option value=(cat.as_str()) { (cat.as_str()) }
                            }
                        }
                    }
                    div style="display:flex; gap:0.35rem;" {
                        div class="form-field" style="flex:1;" {
                            label { "Subtype" }
                            input type="text" name="subtype" placeholder="e.g. Fixed, Film, Op-Amp";
                        }
                        div class="form-field" style="flex:1;" {
                            label { "Package / Mounting" }
                            input type="text" name="package_or_mounting" placeholder="e.g. Through-Hole, DIP-8";
                        }
                    }
                    div style="display:flex; gap:0.35rem;" {
                        div class="form-field" style="flex:1;" {
                            label { "Initial Location" }
                            select name="initial_location_id" {
                                option value="" { "— No Initial Lot —" }
                                @for loc in all_locations {
                                    option value=(loc.id) { (loc.breadcrumb_path) }
                                }
                            }
                        }
                        div class="form-field" style="inline-size:90px;" {
                            label { "Initial Qty" }
                            input type="number" name="initial_quantity" value="0" min="0";
                        }
                    }
                    div style="display:flex; gap:0.35rem;" {
                        div class="form-field" style="flex:1;" {
                            label { "Supplier" }
                            input type="text" name="initial_supplier" placeholder="e.g. Tayda, Mouser";
                        }
                        div class="form-field" style="flex:1;" {
                            label { "Supplier Part #" }
                            input type="text" name="initial_supplier_part_no" placeholder="e.g. A-2214";
                        }
                    }
                    div class="form-field" {
                        label { "Low-Stock Threshold" }
                        input type="number" name="low_stock_threshold" value="0" min="0";
                    }
                    button type="submit" class="primary" { "Create Component" }
                }
            }

            details open[matches!(state, CatalogInspectorState::Location { .. })] {
                summary style="cursor:pointer; font-weight:600; font-size:0.88rem;" {
                    "+ New Storage Location"
                }
                form class="form-grid"
                    method="post"
                    action="/locations"
                    hx-post="/locations"
                    hx-target="#catalog-workspace"
                    hx-swap="outerHTML"
                    style="margin-block-start:0.5rem;" {
                    div class="form-field" {
                        label for="new-loc-name" { "Name" }
                        input id="new-loc-name" type="text" name="name" required placeholder="e.g. Crate 2, Passives Bag, Binder A";
                    }
                    div class="form-field" {
                        label for="new-loc-kind" { "Container Kind" }
                        select id="new-loc-kind" name="kind" {
                            @for kind in LocationKind::ALL {
                                option value=(kind.as_str()) { (kind.as_str()) }
                            }
                        }
                    }
                    div class="form-field" {
                        label for="new-loc-parent" { "Parent Storage Location (Optional)" }
                        select id="new-loc-parent" name="parent_id" {
                            option value="" { "— Top-Level Root Container —" }
                            @for candidate in all_locations {
                                option value=(candidate.id) {
                                    (candidate.breadcrumb_path) " (" (candidate.kind) ")"
                                }
                            }
                        }
                    }
                    div class="form-field" {
                        label for="new-loc-notes" { "Notes" }
                        input id="new-loc-notes" type="text" name="notes" placeholder="Optional container notes...";
                    }
                    button type="submit" class="primary" { "Add Storage Location" }
                }
            }
        }
    }
}

fn parse_bool_checkbox(val: Option<&str>) -> bool {
    matches!(
        val.map(str::trim).map(str::to_lowercase).as_deref(),
        Some("1" | "true" | "on" | "yes")
    )
}

async fn load_and_render_builds_workspace(
    db: &Db,
    selected_kit_id: Option<&str>,
    selected_bom_line_id: Option<&str>,
    flash_error: Option<&str>,
) -> Result<Markup, (StatusCode, String)> {
    let builds = list_kit_builds(db).await.map_err(internal_err)?;
    let locations = list_storage_locations(db).await.map_err(internal_err)?;

    let active_build = if let Some(kid) = selected_kit_id {
        get_kit_build(db, kid).await.map_err(internal_err)?
    } else if let Some(first) = builds.first() {
        get_kit_build(db, &first.id).await.map_err(internal_err)?
    } else {
        None
    };

    let inspected_line = if let Some(ref b) = active_build {
        if let Some(lid) = selected_bom_line_id {
            b.bom_lines
                .iter()
                .find(|l| l.id == lid)
                .cloned()
                .or_else(|| b.bom_lines.first().cloned())
        } else {
            b.bom_lines.first().cloned()
        }
    } else {
        None
    };

    let candidates = if let Some(ref line) = inspected_line {
        find_bom_line_candidates(db, &line.id)
            .await
            .map_err(internal_err)?
    } else {
        Vec::new()
    };

    Ok(render_builds_workspace(
        &builds,
        active_build.as_ref(),
        inspected_line.as_ref(),
        &candidates,
        &locations,
        flash_error,
    ))
}

async fn builds_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<BuildsQuery>,
) -> Result<Html<String>, (StatusCode, String)> {
    let content = load_and_render_builds_workspace(
        &state.db,
        query.kit_id.as_deref(),
        query.inspect_bom_line_id.as_deref(),
        None,
    )
    .await?;
    Ok(render_page_or_partial(&headers, SurfaceTab::Builds, content))
}

async fn build_detail_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(query): Query<BuildsQuery>,
) -> Result<Html<String>, (StatusCode, String)> {
    let content = load_and_render_builds_workspace(
        &state.db,
        Some(&id),
        query.inspect_bom_line_id.as_deref(),
        None,
    )
    .await?;
    Ok(render_page_or_partial(&headers, SurfaceTab::Builds, content))
}

async fn create_kit_build_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<CreateKitBuildForm>,
) -> Result<Html<String>, (StatusCode, String)> {
    let res = create_kit_build(
        &state.db,
        CreateKitBuild {
            name: form.name,
            project_type: form.project_type.unwrap_or_default(),
            status: form.status,
            location_id: form.location_id,
            description: form.description.unwrap_or_default(),
        },
    )
    .await;

    let (kit_id, flash_err) = match res {
        Ok(b) => (Some(b.id), None),
        Err(e) => (None, Some(e.to_string())),
    };

    let content = load_and_render_builds_workspace(
        &state.db,
        kit_id.as_deref(),
        None,
        flash_err.as_deref(),
    )
    .await?;
    Ok(render_page_or_partial(&headers, SurfaceTab::Builds, content))
}

async fn update_kit_build_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Form(form): Form<UpdateKitBuildForm>,
) -> Result<Html<String>, (StatusCode, String)> {
    let return_parts = parse_bool_checkbox(form.return_parts.as_deref());
    let mut flash_err = None;

    if let Err(e) = update_kit_build(
        &state.db,
        &id,
        UpdateKitBuild {
            name: form.name,
            project_type: form.project_type.unwrap_or_default(),
            status: form.status.clone(),
            location_id: form.location_id,
            description: form.description.unwrap_or_default(),
        },
    )
    .await
    {
        flash_err = Some(e.to_string());
    } else if return_parts {
        if let Err(e) = update_kit_build_status(&state.db, &id, &form.status, true).await {
            flash_err = Some(e.to_string());
        }
    }

    let content =
        load_and_render_builds_workspace(&state.db, Some(&id), None, flash_err.as_deref()).await?;
    Ok(render_page_or_partial(&headers, SurfaceTab::Builds, content))
}

async fn update_kit_build_status_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Form(form): Form<UpdateKitBuildStatusForm>,
) -> Result<Html<String>, (StatusCode, String)> {
    let return_parts = parse_bool_checkbox(form.return_parts.as_deref());
    let res = update_kit_build_status(&state.db, &id, &form.status, return_parts).await;
    let flash_err = res.err().map(|e| e.to_string());
    let content =
        load_and_render_builds_workspace(&state.db, Some(&id), None, flash_err.as_deref()).await?;
    Ok(render_page_or_partial(&headers, SurfaceTab::Builds, content))
}

async fn shelve_kit_build_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Html<String>, (StatusCode, String)> {
    let res = shelve_kit_build_and_return_parts(&state.db, &id).await;
    let flash_err = res.err().map(|e| e.to_string());
    let content =
        load_and_render_builds_workspace(&state.db, Some(&id), None, flash_err.as_deref()).await?;
    Ok(render_page_or_partial(&headers, SurfaceTab::Builds, content))
}

async fn clone_kit_build_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Form(form): Form<CloneKitBuildForm>,
) -> Result<Html<String>, (StatusCode, String)> {
    let res = clone_kit_build(&state.db, &id, form.name.as_deref()).await;
    let (target_kit_id, flash_err) = match res {
        Ok(cloned) => (cloned.id, None),
        Err(e) => (id, Some(e.to_string())),
    };
    let content = load_and_render_builds_workspace(
        &state.db,
        Some(&target_kit_id),
        None,
        flash_err.as_deref(),
    )
    .await?;
    Ok(render_page_or_partial(&headers, SurfaceTab::Builds, content))
}

async fn reserve_all_available_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Html<String>, (StatusCode, String)> {
    let res = reserve_all_available(&state.db, &id).await;
    let flash_err = res.err().map(|e| e.to_string());
    let content =
        load_and_render_builds_workspace(&state.db, Some(&id), None, flash_err.as_deref()).await?;
    Ok(render_page_or_partial(&headers, SurfaceTab::Builds, content))
}

async fn pick_all_reserved_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Html<String>, (StatusCode, String)> {
    let res = pick_all_reserved(&state.db, &id).await;
    let flash_err = res.err().map(|e| e.to_string());
    let content =
        load_and_render_builds_workspace(&state.db, Some(&id), None, flash_err.as_deref()).await?;
    Ok(render_page_or_partial(&headers, SurfaceTab::Builds, content))
}

async fn create_bom_line_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Form(form): Form<CreateBomLineForm>,
) -> Result<Html<String>, (StatusCode, String)> {
    let included_in_kit = parse_bool_checkbox(form.included_in_kit.as_deref());
    let res = add_bom_line(
        &state.db,
        CreateBomLine {
            kit_build_id: id.clone(),
            raw_spec_text: form.raw_spec_text,
            category: form.category,
            package_or_mounting: form.package_or_mounting,
            quantity_required: form.quantity_required.unwrap_or(1).max(1),
            reference_designators: form.reference_designators.unwrap_or_default(),
            included_in_kit,
            component_id: form.component_id,
            substitution_note: form.substitution_note.unwrap_or_default(),
            notes: form.notes.unwrap_or_default(),
        },
    )
    .await;

    let (inspect_lid, flash_err) = match res {
        Ok(line) => (Some(line.id), None),
        Err(e) => (None, Some(e.to_string())),
    };

    let content = load_and_render_builds_workspace(
        &state.db,
        Some(&id),
        inspect_lid.as_deref(),
        flash_err.as_deref(),
    )
    .await?;
    Ok(render_page_or_partial(&headers, SurfaceTab::Builds, content))
}

async fn inspect_bom_line_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(line_id): Path<String>,
) -> Result<Html<String>, (StatusCode, String)> {
    let line = get_bom_line(&state.db, &line_id)
        .await
        .map_err(internal_err)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("BOM Line '{line_id}' not found")))?;

    if is_htmx_request(&headers) {
        let locations = list_storage_locations(&state.db)
            .await
            .map_err(internal_err)?;
        let candidates = find_bom_line_candidates(&state.db, &line_id)
            .await
            .map_err(internal_err)?;
        let drawer = render_bom_line_inspector_drawer(Some(&line), &candidates, &locations, None);
        Ok(Html(drawer.into_string()))
    } else {
        let content = load_and_render_builds_workspace(
            &state.db,
            Some(&line.kit_build_id),
            Some(&line_id),
            None,
        )
        .await?;
        Ok(Html(
            workbench_shell(SurfaceTab::Builds, content).into_string(),
        ))
    }
}

async fn inspect_kit_bom_line_handler(
    state: State<AppState>,
    headers: HeaderMap,
    Path((_kit_id, line_id)): Path<(String, String)>,
) -> Result<Html<String>, (StatusCode, String)> {
    inspect_bom_line_handler(state, headers, Path(line_id)).await
}

async fn transition_bom_line_status_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(line_id): Path<String>,
    Query(q_form): Query<TransitionBomStatusForm>,
    Form(f_form): Form<TransitionBomStatusForm>,
) -> Result<Html<String>, (StatusCode, String)> {
    let status = f_form
        .status
        .or(q_form.status)
        .unwrap_or_else(|| "Unallocated".to_string());
    let source_lot_id = f_form.source_lot_id.or(q_form.source_lot_id);
    let quantity_allocated = f_form.quantity_allocated.or(q_form.quantity_allocated);

    let updated_line = transition_bom_line_status(
        &state.db,
        &line_id,
        &status,
        source_lot_id.as_deref(),
        quantity_allocated,
    )
    .await
    .map_err(internal_err)?;

    let build = get_kit_build(&state.db, &updated_line.kit_build_id)
        .await
        .map_err(internal_err)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Kit Build not found".to_string()))?;

    let hx_target = headers
        .get("HX-Target")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if is_htmx_request(&headers)
        && hx_target != "builds-workspace"
        && hx_target != "app-root"
        && hx_target != "inspector"
    {
        let row_markup = render_bom_line_row(&updated_line, false);
        let oob_summary = render_build_readiness_summary_badge(&build.readiness, true);
        let combined = html! {
            (row_markup)
            (oob_summary)
        };
        Ok(Html(combined.into_string()))
    } else if is_htmx_request(&headers) && hx_target == "inspector" {
        let locations = list_storage_locations(&state.db)
            .await
            .map_err(internal_err)?;
        let candidates = find_bom_line_candidates(&state.db, &line_id)
            .await
            .map_err(internal_err)?;
        let drawer =
            render_bom_line_inspector_drawer(Some(&updated_line), &candidates, &locations, None);
        let row_oob = render_bom_line_row_oob(&updated_line);
        let oob_summary = render_build_readiness_summary_badge(&build.readiness, true);
        let combined = html! {
            (drawer)
            (row_oob)
            (oob_summary)
        };
        Ok(Html(combined.into_string()))
    } else {
        let content = load_and_render_builds_workspace(
            &state.db,
            Some(&updated_line.kit_build_id),
            Some(&line_id),
            None,
        )
        .await?;
        Ok(render_page_or_partial(&headers, SurfaceTab::Builds, content))
    }
}

async fn transition_kit_bom_line_status_handler(
    state: State<AppState>,
    headers: HeaderMap,
    Path((_kit_id, line_id)): Path<(String, String)>,
    query: Query<TransitionBomStatusForm>,
    form: Form<TransitionBomStatusForm>,
) -> Result<Html<String>, (StatusCode, String)> {
    transition_bom_line_status_handler(state, headers, Path(line_id), query, form).await
}

async fn link_bom_line_component_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(line_id): Path<String>,
    Form(form): Form<LinkBomComponentForm>,
) -> Result<Html<String>, (StatusCode, String)> {
    let updated = link_bom_line_component(
        &state.db,
        &line_id,
        form.component_id.as_deref(),
        form.substitution_note.as_deref(),
    )
    .await
    .map_err(internal_err)?;

    let content = load_and_render_builds_workspace(
        &state.db,
        Some(&updated.kit_build_id),
        Some(&line_id),
        None,
    )
    .await?;
    Ok(render_page_or_partial(&headers, SurfaceTab::Builds, content))
}

async fn link_kit_bom_line_component_handler(
    state: State<AppState>,
    headers: HeaderMap,
    Path((_kit_id, line_id)): Path<(String, String)>,
    form: Form<LinkBomComponentForm>,
) -> Result<Html<String>, (StatusCode, String)> {
    link_bom_line_component_handler(state, headers, Path(line_id), form).await
}

async fn substitute_bom_line_modal_handler(
    State(state): State<AppState>,
    Path(line_id): Path<String>,
) -> Result<Html<String>, (StatusCode, String)> {
    let line = get_bom_line(&state.db, &line_id)
        .await
        .map_err(internal_err)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("BOM Line '{line_id}' not found")))?;
    let all_components = list_components(&state.db).await.map_err(internal_err)?;
    let dialog = render_substitute_bom_line_dialog(&line, &all_components);
    Ok(Html(dialog.into_string()))
}

async fn substitute_kit_bom_line_modal_handler(
    state: State<AppState>,
    Path((_kit_id, line_id)): Path<(String, String)>,
) -> Result<Html<String>, (StatusCode, String)> {
    substitute_bom_line_modal_handler(state, Path(line_id)).await
}

async fn substitute_bom_line_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(line_id): Path<String>,
    Form(form): Form<SubstituteBomComponentForm>,
) -> Result<Html<String>, (StatusCode, String)> {
    let updated = substitute_bom_line_component(
        &state.db,
        &line_id,
        &form.component_id,
        form.substitution_note.as_deref().unwrap_or(""),
    )
    .await
    .map_err(internal_err)?;

    let content = load_and_render_builds_workspace(
        &state.db,
        Some(&updated.kit_build_id),
        Some(&line_id),
        None,
    )
    .await?;
    Ok(render_page_or_partial(&headers, SurfaceTab::Builds, content))
}

async fn substitute_kit_bom_line_handler(
    state: State<AppState>,
    headers: HeaderMap,
    Path((_kit_id, line_id)): Path<(String, String)>,
    form: Form<SubstituteBomComponentForm>,
) -> Result<Html<String>, (StatusCode, String)> {
    substitute_bom_line_handler(state, headers, Path(line_id), form).await
}

async fn split_bom_line_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(line_id): Path<String>,
    Form(form): Form<SplitBomLineForm>,
) -> Result<Html<String>, (StatusCode, String)> {
    let existing = get_bom_line(&state.db, &line_id)
        .await
        .map_err(internal_err)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("BOM Line '{line_id}' not found")))?;
    let default_split_qty = (existing.quantity_required / 2).max(1);
    let split_qty = form.split_quantity.unwrap_or(default_split_qty);

    let res = split_bom_line(
        &state.db,
        &line_id,
        split_qty,
        form.split_designators.as_deref(),
    )
    .await;

    let (inspect_lid, flash_err) = match res {
        Ok((_orig, sibling)) => (Some(sibling.id), None),
        Err(e) => (Some(line_id), Some(e.to_string())),
    };

    let content = load_and_render_builds_workspace(
        &state.db,
        Some(&existing.kit_build_id),
        inspect_lid.as_deref(),
        flash_err.as_deref(),
    )
    .await?;
    Ok(render_page_or_partial(&headers, SurfaceTab::Builds, content))
}

async fn split_kit_bom_line_handler(
    state: State<AppState>,
    headers: HeaderMap,
    Path((_kit_id, line_id)): Path<(String, String)>,
    form: Form<SplitBomLineForm>,
) -> Result<Html<String>, (StatusCode, String)> {
    split_bom_line_handler(state, headers, Path(line_id), form).await
}

async fn create_component_from_bom_line_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(line_id): Path<String>,
    Form(form): Form<CreateComponentFromBomForm>,
) -> Result<Html<String>, (StatusCode, String)> {
    let initial_loc = form
        .initial_location_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let initial_qty = form.initial_quantity.unwrap_or(0).max(0);

    let (_comp, updated_line) =
        create_component_from_bom_line(&state.db, &line_id, initial_loc, initial_qty)
            .await
            .map_err(internal_err)?;

    let content = load_and_render_builds_workspace(
        &state.db,
        Some(&updated_line.kit_build_id),
        Some(&line_id),
        None,
    )
    .await?;
    Ok(render_page_or_partial(&headers, SurfaceTab::Builds, content))
}

async fn create_component_from_kit_bom_line_handler(
    state: State<AppState>,
    headers: HeaderMap,
    Path((_kit_id, line_id)): Path<(String, String)>,
    form: Form<CreateComponentFromBomForm>,
) -> Result<Html<String>, (StatusCode, String)> {
    create_component_from_bom_line_handler(state, headers, Path(line_id), form).await
}

fn render_fulfillment_status_badge(status: &str) -> Markup {
    let badge_class = match status {
        "Installed" => "badge ok",
        "Picked" => "badge info",
        "Reserved" => "badge warn",
        "Unallocated" => "badge err",
        _ => "badge neutral",
    };
    html! {
        span class=(badge_class) { (status) }
    }
}

fn render_build_readiness_summary_badge(readiness: &BuildReadinessSummary, oob: bool) -> Markup {
    let badge_class = if readiness.shortfall_lines == 0 && readiness.total_lines > 0 {
        "badge ok"
    } else if readiness.shortfall_lines > 0 {
        "badge warn"
    } else {
        "badge neutral"
    };
    html! {
        span id="build-readiness-summary"
            class=(badge_class)
            hx-swap-oob=[if oob { Some("true") } else { None }] {
            (readiness.display_label())
        }
    }
}

fn render_bom_line_row(line: &BomLine, is_selected: bool) -> Markup {
    render_bom_line_row_inner(line, is_selected, false)
}

fn render_bom_line_row_oob(line: &BomLine) -> Markup {
    html! {
        template {
            (render_bom_line_row_inner(line, true, true))
        }
    }
}

fn render_substitute_bom_line_dialog(line: &BomLine, all_components: &[Component]) -> Markup {
    html! {
        dialog id="bom-action-dialog" closedby="any" open aria-labelledby="bom-dialog-title" {
            div style="display:flex; justify-content:space-between; align-items:center; gap:0.75rem; margin-block-end:0.65rem;" {
                h2 id="bom-dialog-title" style="margin:0; font-size:1.05rem;" {
                    "Substitute Catalog Component (" (line.reference_designators) ")"
                }
                button type="button"
                    class="xs"
                    onclick="this.closest('dialog')?.close()" {
                    "Close ✕"
                }
            }
            div id="bom-dialog-content" {
                div style="font-size:0.84rem; color:var(--text-muted); margin-block-end:0.65rem;" {
                    "Original Schematic Specification (preserved verbatim): "
                    code { (line.raw_spec_text) }
                    " · Canonical: "
                    strong class="mono" { (line.canonical_value_display) }
                }
                form class="form-grid"
                    method="post"
                    action=(format!("/builds/bom-lines/{}/substitute", line.id))
                    hx-post=(format!("/builds/bom-lines/{}/substitute", line.id))
                    hx-target="#builds-workspace"
                    hx-swap="outerHTML"
                    onsubmit="this.closest('dialog')?.close()" {
                    div class="form-field" {
                        label for="sub-component-select" { "Replacement Catalog Component" }
                        select id="sub-component-select" name="component_id" required {
                            @for comp in all_components {
                                option value=(comp.id)
                                    selected[line.component_id.as_deref() == Some(comp.id.as_str())] {
                                    (comp.name) " [" (comp.canonical_value_display) " · " (comp.available_stock) " Avail]"
                                }
                            }
                        }
                    }
                    div class="form-field" {
                        label for="sub-note-input" { "Substitution Note (Reason / Pinout / Mod)" }
                        input id="sub-note-input"
                            type="text"
                            name="substitution_note"
                            value=(line.substitution_note)
                            placeholder="e.g. pinout rotated, higher voltage rating";
                    }
                    div style="display:flex; justify-content:flex-end; gap:0.45rem;" {
                        button type="button"
                            class="xs"
                            onclick="this.closest('dialog')?.close()" {
                            "Cancel"
                        }
                        button type="submit" class="primary xs" {
                            "Apply Component Substitution"
                        }
                    }
                }
            }
        }
    }
}

fn render_bom_line_row_inner(line: &BomLine, is_selected: bool, oob: bool) -> Markup {
    let has_shortfall = line.shortfall_quantity > 0;

    html! {
        tr id=(format!("bom-row-{}", line.id))
            hx-swap-oob=[if oob { Some("true") } else { None }]
            style=(if is_selected { "outline: 2px solid var(--brand-color); outline-offset:-2px;" } else { "" }) {
            th scope="row" class="mono" {
                a href=(format!("/builds/{}?inspect_bom_line_id={}", line.kit_build_id, line.id))
                    hx-get=(format!("/builds/bom-lines/{}/inspector", line.id))
                    hx-target="#inspector"
                    hx-swap="outerHTML"
                    style="color:inherit; text-decoration:none; font-weight:600;" {
                    @if line.reference_designators.is_empty() {
                        "—"
                    } @else {
                        (line.reference_designators)
                    }
                }
            }
            td {
                div {
                    strong class="mono" { (line.canonical_value_display) }
                    " · "
                    code { (line.raw_spec_text) }
                    " "
                    small style="color:var(--text-muted);" { "(" (line.category) ")" }
                }
                @if line.included_in_kit {
                    div style="font-size:0.8rem; color:var(--text-muted);" {
                        "↳ Included in Kit (vendor-supplied)"
                    }
                } @else if let Some(ref comp) = line.linked_component {
                    div style="font-size:0.8rem; color:var(--text-muted);" {
                        "↳ Linked: " strong { (comp.name) }
                        @if !line.substitution_note.is_empty() {
                            " · "
                            span class="badge warn" {
                                (comp.canonical_value_display) " — " (line.substitution_note)
                            }
                        }
                    }
                } @else {
                    div style="font-size:0.8rem; color:var(--warn-text);" {
                        "↳ " span class="badge warn" { "Unlinked" } " (Click Inspect ▸ for candidates / 1-click create)"
                    }
                }
                @if !line.substitution_note.is_empty() && line.linked_component.is_none() {
                    div style="font-size:0.78rem; color:var(--warn-text); margin-block-start:2px;" {
                        "⚡ " (line.substitution_note)
                    }
                }
            }
            td {
                strong class="mono" {
                    (line.quantity_allocated) " / " (line.quantity_required)
                }
                @if has_shortfall {
                    span class="badge err" style="margin-inline-start:4px;" {
                        "Shortfall: " (line.shortfall_quantity)
                    }
                }
                @if let Some(ref comp) = line.linked_component {
                    div style="font-size:0.76rem; color:var(--text-muted);" {
                        "Catalog Avail: " (comp.available_stock) " / " (comp.total_on_hand) " On Hand"
                    }
                }
            }
            td {
                @if let Some(ref loc_path) = line.source_lot_location_breadcrumb {
                    code style="font-size:0.78rem;" {
                        (loc_path)
                        @if let Some(qty) = line.source_lot_quantity {
                            " (" (qty) ")"
                        }
                    }
                } @else if let Some(ref comp) = line.linked_component {
                    @if let Some(first_lot) = comp.lots.first() {
                        span style="color:var(--text-muted); font-size:0.78rem;" {
                            (first_lot.location_breadcrumb) " (" (first_lot.quantity) ")"
                        }
                    } @else {
                        span style="color:var(--text-muted);" { "—" }
                    }
                } @else {
                    span style="color:var(--text-muted);" { "—" }
                }
            }
            td {
                (render_fulfillment_status_badge(&line.fulfillment_status))
            }
            td {
                div style="display:flex; gap:0.25rem; flex-wrap:wrap; align-items:center;" {
                    @for st in FulfillmentStatus::ALL {
                        @let status = st.as_str();
                        button type="button"
                            class=(if line.fulfillment_status == status { "xs primary" } else { "xs" })
                            hx-patch=(format!("/builds/bom-lines/{}/status", line.id))
                            hx-vals=(format!(r#"{{"status": "{status}"}}"#))
                            hx-target=(format!("#bom-row-{}", line.id))
                            hx-swap="outerHTML" {
                            (status)
                        }
                    }
                    button type="button"
                        class="xs"
                        hx-get=(format!("/builds/bom-lines/{}/substitute-modal", line.id))
                        hx-target="#bom-action-dialog"
                        hx-swap="outerHTML" {
                        "Substitute ▸"
                    }
                    button type="button"
                        class="xs"
                        hx-get=(format!("/builds/bom-lines/{}/inspector", line.id))
                        hx-target="#inspector"
                        hx-swap="outerHTML" {
                        "Inspect ▸"
                    }
                }
            }
        }
    }
}

fn render_builds_workspace(
    builds: &[KitBuild],
    active_build: Option<&KitBuild>,
    inspected_line: Option<&BomLine>,
    candidates: &[BomCandidateMatch],
    locations: &[StorageLocation],
    flash_error: Option<&str>,
) -> Markup {
    let selected_line_id = inspected_line.map(|l| l.id.as_str());

    html! {
        div id="builds-workspace" {
            @if let Some(err) = flash_error {
                div class="card" style="background:var(--err-bg); color:var(--err-text); margin-block-end:0.85rem; font-weight:600;" role="alert" {
                    (err)
                }
            }

            // Top Kit Build selector & New Kit Build collapsible bar
            div class="va-filter-ribbon" style="justify-content:space-between;" {
                div style="display:flex; align-items:center; gap:0.5rem; flex-wrap:wrap;" {
                    strong { "Kit Builds (" (builds.len()) "):" }
                    @if builds.is_empty() {
                        span style="color:var(--text-muted); font-size:0.85rem;" {
                            "No Kit Builds yet — create one below."
                        }
                    } @else {
                        @for b in builds {
                            @let is_active = active_build.is_some_and(|ab| ab.id == b.id);
                            a class=(if is_active { "btn xs primary" } else { "btn xs" })
                                href=(format!("/builds/{}", b.id))
                                hx-get=(format!("/builds/{}", b.id))
                                hx-target="#builds-workspace"
                                hx-swap="outerHTML"
                                hx-push-url="true" {
                                (b.name) " · " (b.status)
                            }
                        }
                    }
                }

                details open[builds.is_empty()] {
                    summary class="btn xs" style="cursor:pointer;" { "+ New Kit Build" }
                    form class="form-grid"
                        method="post"
                        action="/builds"
                        hx-post="/builds"
                        hx-target="#builds-workspace"
                        hx-swap="outerHTML"
                        style="margin-block-start:0.5rem; padding:0.65rem; border:1px solid var(--border-color); border-radius:6px; background:var(--bg-subtle); min-inline-size:320px;" {
                        div class="form-field" {
                            label { "Build Name" }
                            input type="text" name="name" required placeholder="e.g. Klon Centaur Overdrive (#1)";
                        }
                        div style="display:flex; gap:0.35rem;" {
                            div class="form-field" style="flex:1;" {
                                label { "Project Type" }
                                input type="text" name="project_type" placeholder="e.g. Overdrive Pedal, Eurorack VCO";
                            }
                            div class="form-field" style="flex:1;" {
                                label { "Initial Status" }
                                select name="status" {
                                    @for st in KitBuildStatus::ALL {
                                        option value=(st.as_str()) { (st.as_str()) }
                                    }
                                }
                            }
                        }
                        div class="form-field" {
                            label { "Physical Storage Location (Optional Project Box)" }
                            select name="location_id" {
                                option value="" { "— Unassigned —" }
                                @for loc in locations {
                                    option value=(loc.id) { (loc.breadcrumb_path) " (" (loc.kind) ")" }
                                }
                            }
                        }
                        div class="form-field" {
                            label { "Description / Build Notes" }
                            input type="text" name="description" placeholder="Optional schematic or revision note...";
                        }
                        button type="submit" class="primary xs" { "Create Kit Build" }
                    }
                }
            }

            @if let Some(build) = active_build {
                // Active Kit Build Header Card + Batch Kitting Actions
                div class="card" style="margin-block-end:0.85rem; display:flex; flex-direction:column; gap:0.65rem;" {
                    div style="display:flex; justify-content:space-between; align-items:start; flex-wrap:wrap; gap:0.75rem;" {
                        div {
                            div style="display:flex; align-items:center; gap:0.5rem; flex-wrap:wrap;" {
                                h2 style="margin:0; font-size:1.15rem;" { (build.name) }
                                span class="badge info" { (build.status) }
                                (render_build_readiness_summary_badge(&build.readiness, false))
                            }
                            div style="font-size:0.82rem; color:var(--text-muted); margin-block-start:0.25rem;" {
                                "Storage Location: " code { (build.location_breadcrumb) }
                                @if !build.project_type.is_empty() {
                                    " · Type: " (build.project_type)
                                }
                                @if !build.description.is_empty() {
                                    " · " (build.description)
                                }
                            }
                        }

                        div style="display:flex; align-items:center; gap:0.45rem; flex-wrap:wrap;" {
                            form method="post"
                                action=(format!("/builds/{}/reserve-all", build.id))
                                hx-post=(format!("/builds/{}/reserve-all", build.id))
                                hx-target="#builds-workspace"
                                hx-swap="outerHTML"
                                style="margin:0;" {
                                button type="submit" class="xs primary" {
                                    "Reserve All Available"
                                }
                            }
                            form method="post"
                                action=(format!("/builds/{}/pick-all", build.id))
                                hx-post=(format!("/builds/{}/pick-all", build.id))
                                hx-target="#builds-workspace"
                                hx-swap="outerHTML"
                                style="margin:0;" {
                                button type="submit" class="xs primary" {
                                    "Pick All Reserved to Project Box"
                                }
                            }
                            form method="post"
                                action=(format!("/builds/{}/shelve", build.id))
                                hx-post=(format!("/builds/{}/shelve", build.id))
                                hx-target="#builds-workspace"
                                hx-swap="outerHTML"
                                style="margin:0;" {
                                button type="submit" class="xs" {
                                    "Shelve Build & Return Picked/Reserved Parts"
                                }
                            }
                            form method="post"
                                action=(format!("/builds/{}/clone", build.id))
                                hx-post=(format!("/builds/{}/clone", build.id))
                                hx-target="#builds-workspace"
                                hx-swap="outerHTML"
                                style="margin:0;" {
                                button type="submit" class="xs" {
                                    "Clone Kit Build"
                                }
                            }
                        }
                    }

                    // Edit Kit Build Lifecycle Status & Storage Location inline form
                    details {
                        summary style="cursor:pointer; font-size:0.82rem; font-weight:600; color:var(--text-muted);" {
                            "Update Kit Build Lifecycle Status, Project Box Location, or Metadata ▾"
                        }
                        form style="display:flex; gap:0.5rem; flex-wrap:wrap; align-items:end; margin-block-start:0.5rem;"
                            method="post"
                            action=(format!("/builds/{}", build.id))
                            hx-post=(format!("/builds/{}", build.id))
                            hx-target="#builds-workspace"
                            hx-swap="outerHTML" {
                            div class="form-field" {
                                label { "Build Name" }
                                input type="text" name="name" value=(build.name) required;
                            }
                            div class="form-field" {
                                label { "Project Type" }
                                input type="text" name="project_type" value=(build.project_type);
                            }
                            div class="form-field" {
                                label { "Lifecycle Status" }
                                select name="status" {
                                    @for st in KitBuildStatus::ALL {
                                        option value=(st.as_str()) selected[build.status == st.as_str()] {
                                            (st.as_str())
                                        }
                                    }
                                }
                            }
                            div class="form-field" {
                                label { "Storage Location" }
                                select name="location_id" {
                                    option value="" selected[build.location_id.is_none()] { "— Unassigned —" }
                                    @for loc in locations {
                                        option value=(loc.id)
                                            selected[build.location_id.as_deref() == Some(loc.id.as_str())] {
                                            (loc.breadcrumb_path) " (" (loc.kind) ")"
                                        }
                                    }
                                }
                            }
                            div class="form-field" {
                                label { "Description" }
                                input type="text" name="description" value=(build.description);
                            }
                            label style="display:inline-flex; align-items:center; gap:0.3rem; font-size:0.8rem;" {
                                input type="checkbox" name="return_parts" value="1";
                                "Return Picked & release Reserved parts"
                            }
                            button type="submit" class="primary xs" { "Save Build" }
                        }
                    }
                }

                // Two-column Workbench Grid: BOM Table (left) + BOM Line Inspector Drawer (right)
                div class="va-grid-with-drawer" {
                    div style="display:flex; flex-direction:column; gap:0.85rem;" {
                        // Add BOM Line inline bar
                        form class="va-filter-ribbon"
                            method="post"
                            action=(format!("/builds/{}/bom-lines", build.id))
                            hx-post=(format!("/builds/{}/bom-lines", build.id))
                            hx-target="#builds-workspace"
                            hx-swap="outerHTML"
                            style="margin-block-end:0;" {
                            strong style="font-size:0.84rem;" { "+ Add BOM Line:" }
                            input type="text"
                                name="reference_designators"
                                placeholder="Ref Des (e.g. R1, R2, C4)"
                                aria-label="Reference Designators"
                                style="inline-size:150px;";
                            input type="text"
                                name="raw_spec_text"
                                required
                                placeholder="Spec shorthand (e.g. 4k7 1% 1/4W, 0.047uF Film, TL072)"
                                aria-label="Schematic Specification"
                                style="flex:1; min-inline-size:210px;";
                            input type="number"
                                name="quantity_required"
                                value="1"
                                min="1"
                                aria-label="Quantity Required"
                                style="inline-size:70px;";
                            select name="category" aria-label="Category (Auto-inferred if blank)" {
                                option value="" { "Auto-Infer Category" }
                                @for cat in Category::ALL {
                                    option value=(cat.as_str()) { (cat.as_str()) }
                                }
                            }
                            label style="display:inline-flex; align-items:center; gap:0.25rem; font-size:0.8rem;" {
                                input type="checkbox" name="included_in_kit" value="1";
                                "Included in Kit"
                            }
                            button type="submit" class="primary xs" { "Add Line" }
                        }

                        div class="table-wrapper" role="region" aria-labelledby="bom-table-caption" tabindex="0" {
                            table style="--label-2:'Spec & Linked Component'; --label-3:'Alloc / Req'; --label-4:'Source Stock Lot'; --label-5:'Status'; --label-6:'Actions';" {
                                caption id="bom-table-caption" {
                                    (build.name) " — BOM Readiness (" (build.readiness.display_label()) ")"
                                }
                                thead {
                                    tr {
                                        th scope="col" { "Ref Designators" }
                                        th scope="col" { "Schematic Spec & Linked Component" }
                                        th scope="col" { "Alloc / Req" }
                                        th scope="col" { "Source Stock Lot" }
                                        th scope="col" { "Fulfillment Status" }
                                        th scope="col" { "HTMX Row Transitions" }
                                    }
                                }
                                tbody id="bom-table-body" {
                                    @if build.bom_lines.is_empty() {
                                        tr {
                                            td colspan="6" style="color:var(--text-muted); text-align:center; padding:1.25rem;" {
                                                "No BOM Lines on this Kit Build yet. Add a BOM Line above to parse specifications and auto-link catalog Components."
                                            }
                                        }
                                    } @else {
                                        @for line in &build.bom_lines {
                                            (render_bom_line_row(line, selected_line_id == Some(line.id.as_str())))
                                        }
                                    }
                                }
                            }
                        }
                    }

                    (render_bom_line_inspector_drawer(inspected_line, candidates, locations, None))
                }
            }
        }
    }
}

fn render_bom_line_inspector_drawer(
    inspected_line: Option<&BomLine>,
    candidates: &[BomCandidateMatch],
    all_locations: &[StorageLocation],
    flash_error: Option<&str>,
) -> Markup {
    html! {
        aside id="inspector" class="card" aria-label="BOM Line Inspector Drawer" {
            @if let Some(err) = flash_error {
                div style="background:var(--err-bg); color:var(--err-text); padding:0.5rem; border-radius:4px; margin-block-end:0.65rem; font-size:0.82rem; font-weight:600;" {
                    (err)
                }
            }

            @if let Some(line) = inspected_line {
                div style="border-block-end:1px solid var(--border-color); padding-block-end:0.65rem; margin-block-end:0.75rem;" {
                    div style="display:flex; justify-content:space-between; align-items:center; gap:0.5rem;" {
                        strong class="mono" style="font-size:0.95rem;" {
                            @if line.reference_designators.is_empty() {
                                "BOM Line"
                            } @else {
                                (line.reference_designators)
                            }
                        }
                        (render_fulfillment_status_badge(&line.fulfillment_status))
                    }
                    div style="font-size:0.82rem; color:var(--text-muted); margin-block-start:0.35rem;" {
                        "Schematic Spec: " code { (line.raw_spec_text) }
                        br;
                        "Canonical Value: " strong class="mono" { (line.canonical_value_display) }
                        " · Category: " (line.category)
                        br;
                        "Allocated: " strong { (line.quantity_allocated) " / " (line.quantity_required) }
                        @if line.shortfall_quantity() > 0 {
                            " "
                            span class="badge err" { "Shortfall: " (line.shortfall_quantity()) }
                        }
                    }
                }

                // Linked Component, Partial Allocation & Source Lot Picker
                @if let Some(ref comp) = line.linked_component {
                    div style="border:1px solid var(--border-color); border-radius:5px; padding:0.55rem; margin-block-end:0.75rem; background:var(--bg-subtle);" {
                        div style="font-size:0.82rem; font-weight:600;" {
                            "Linked Catalog Component:"
                        }
                        div style="font-size:0.85rem; margin-block-start:0.2rem;" {
                            strong { (comp.name) }
                            " "
                            span class="badge ok" { (comp.available_stock) " Avail / " (comp.total_on_hand) " On Hand" }
                        }
                        @if !line.substitution_note.is_empty() {
                            div style="font-size:0.78rem; color:var(--warn-text); margin-block-start:0.25rem;" {
                                "Substitution: " strong { (comp.canonical_value_display) " — " (line.substitution_note) }
                            }
                        }

                        form style="margin-block-start:0.5rem;"
                            method="post"
                            action=(format!("/builds/bom-lines/{}/status", line.id))
                            hx-post=(format!("/builds/bom-lines/{}/status", line.id))
                            hx-target="#inspector"
                            hx-swap="outerHTML" {
                            div style="display:flex; gap:0.35rem; margin-block-end:0.35rem;" {
                                div class="form-field" style="flex:1;" {
                                    label { "Fulfillment Status" }
                                    select name="status" {
                                        option value="Picked" selected[line.fulfillment_status == "Picked"] { "Picked" }
                                        option value="Reserved" selected[line.fulfillment_status == "Reserved"] { "Reserved" }
                                    }
                                }
                                div class="form-field" style="inline-size:95px;" {
                                    label { "Alloc Qty" }
                                    input type="number"
                                        name="quantity_allocated"
                                        value=(if line.quantity_allocated > 0 { line.quantity_allocated } else { line.quantity_required })
                                        min="1"
                                        max=(line.quantity_required);
                                }
                            }
                            @if !comp.lots.is_empty() {
                                div class="form-field" style="margin-block-end:0.35rem;" {
                                    label { "Pick from Specific Stock Lot" }
                                    select name="source_lot_id" {
                                        @for lot in &comp.lots {
                                            option value=(lot.id)
                                                selected[line.source_lot_id.as_deref() == Some(lot.id.as_str())] {
                                                (lot.location_breadcrumb) " (" (lot.quantity) " pcs)"
                                            }
                                        }
                                    }
                                }
                            }
                            button type="submit" class="primary xs" style="inline-size:100%; justify-content:center;" {
                                "Apply Allocation / Pick from Lot"
                            }
                        }
                    }
                }

                // Substitute Catalog Component modal trigger
                div style="margin-block-end:0.75rem;" {
                    button type="button"
                        class="xs"
                        style="inline-size:100%; justify-content:center;"
                        hx-get=(format!("/builds/bom-lines/{}/substitute-modal", line.id))
                        hx-target="#bom-action-dialog"
                        hx-swap="outerHTML" {
                        "Substitute Catalog Component ▸"
                    }
                }

                // 1-Click "Split BOM Line" action (when quantity_required > 1)
                @if line.quantity_required > 1 {
                    div style="border:1px solid var(--border-color); border-radius:5px; padding:0.55rem; margin-block-end:0.75rem; background:var(--bg-subtle);" {
                        h4 style="margin:0 0 0.35rem 0; font-size:0.82rem; text-transform:uppercase; color:var(--text-muted);" {
                            "1-Click Split BOM Line (" (line.quantity_required) " pcs)"
                        }
                        form class="form-grid"
                            method="post"
                            action=(format!("/builds/bom-lines/{}/split", line.id))
                            hx-post=(format!("/builds/bom-lines/{}/split", line.id))
                            hx-target="#builds-workspace"
                            hx-swap="outerHTML" {
                            div style="display:flex; gap:0.35rem;" {
                                div class="form-field" style="inline-size:95px;" {
                                    label { "Split Qty" }
                                    input type="number"
                                        name="split_quantity"
                                        value=((line.quantity_required / 2).max(1))
                                        min="1"
                                        max=(line.quantity_required - 1);
                                }
                                div class="form-field" style="flex:1;" {
                                    label { "Sibling Designators (Auto if blank)" }
                                    input type="text"
                                        name="split_designators"
                                        placeholder="e.g. C3, C4";
                                }
                            }
                            button type="submit" class="primary xs" style="inline-size:100%; justify-content:center;" {
                                "Split BOM Line"
                            }
                        }
                    }
                }

                // Compatible Catalog Candidates
                div style="margin-block-end:0.75rem;" {
                    h4 style="margin:0 0 0.4rem 0; font-size:0.82rem; text-transform:uppercase; color:var(--text-muted);" {
                        "Compatible Catalog Candidates (" (candidates.len()) ")"
                    }
                    @if candidates.is_empty() {
                        p style="font-size:0.8rem; color:var(--text-muted); margin:0 0 0.5rem 0;" {
                            "No matching Components in catalog yet."
                        }
                    } @else {
                        @for cand in candidates {
                            div style="border:1px solid var(--border-color); border-radius:5px; padding:0.5rem; margin-block-end:0.45rem; background:var(--bg-subtle);" {
                                div style="display:flex; justify-content:space-between; align-items:start; gap:0.35rem;" {
                                    strong style="font-size:0.82rem;" { (cand.component.name) }
                                    span class=(if cand.component.available_stock > 0 { "badge ok" } else { "badge neutral" }) {
                                        (cand.component.available_stock) " Avail"
                                    }
                                }
                                div style="font-size:0.76rem; color:var(--text-muted); margin:0.2rem 0 0.4rem 0;" {
                                    (cand.match_reason)
                                    br;
                                    "Spec: " code { (cand.component.raw_spec_text) }
                                }
                                form method="post"
                                    action=(format!("/builds/bom-lines/{}/link-component", line.id))
                                    hx-post=(format!("/builds/bom-lines/{}/link-component", line.id))
                                    hx-target="#builds-workspace"
                                    hx-swap="outerHTML"
                                    style="margin:0;" {
                                    input type="hidden" name="component_id" value=(cand.component.id);
                                    button type="submit" class="xs primary" {
                                        "Link Candidate Component"
                                    }
                                }
                            }
                        }
                    }
                }

                // 1-Click "Create Component from BOM Line" action
                @if !line.included_in_kit {
                    div style="border-block-start:1px solid var(--border-color); padding-block-start:0.65rem;" {
                        h4 style="margin:0 0 0.4rem 0; font-size:0.82rem; text-transform:uppercase; color:var(--text-muted);" {
                            "Create Component from BOM Line"
                        }
                        form class="form-grid"
                            method="post"
                            action=(format!("/builds/bom-lines/{}/create-component", line.id))
                            hx-post=(format!("/builds/bom-lines/{}/create-component", line.id))
                            hx-target="#builds-workspace"
                            hx-swap="outerHTML" {
                            div style="display:flex; gap:0.35rem;" {
                                div class="form-field" style="flex:1;" {
                                    label { "Initial Location (Optional)" }
                                    select name="initial_location_id" {
                                        option value="" { "— No Initial Lot —" }
                                        @for loc in all_locations {
                                            option value=(loc.id) { (loc.breadcrumb_path) }
                                        }
                                    }
                                }
                                div class="form-field" style="inline-size:85px;" {
                                    label { "Initial Qty" }
                                    input type="number" name="initial_quantity" value="0" min="0";
                                }
                            }
                            button type="submit" class="primary xs" style="inline-size:100%; justify-content:center;" {
                                "+ Create Component from BOM Line"
                            }
                        }
                    }
                }
            } @else {
                p style="color:var(--text-muted); font-size:0.85rem; margin:0;" {
                    "Select or add a BOM Line to inspect compatible catalog candidates, pick from specific Stock Lots, or create a catalog Component in 1 click."
                }
            }
        }
    }
}

async fn load_and_render_bench_workspace(
    db: &Db,
    selected_kit_id: Option<&str>,
    flash_error: Option<&str>,
) -> Result<Markup, (StatusCode, String)> {
    let all_builds = list_kit_builds(db).await.map_err(internal_err)?;

    let active_build = if let Some(kid) = selected_kit_id
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        get_kit_build(db, kid).await.map_err(internal_err)?
    } else {
        all_builds
            .iter()
            .find(|b| b.status == "On the Bench (In Progress)")
            .or_else(|| all_builds.first())
            .cloned()
    };

    let (attachments, log_entries) = if let Some(ref build) = active_build {
        let atts = list_build_attachments(db, &build.id)
            .await
            .map_err(internal_err)?;
        let logs = list_bench_log_entries(db, &build.id)
            .await
            .map_err(internal_err)?;
        (atts, logs)
    } else {
        (Vec::new(), Vec::new())
    };

    Ok(render_bench_workspace(
        &all_builds,
        active_build.as_ref(),
        &attachments,
        &log_entries,
        flash_error,
    ))
}

async fn bench_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<BenchQuery>,
) -> Result<Html<String>, (StatusCode, String)> {
    let content =
        load_and_render_bench_workspace(&state.db, query.kit_id.as_deref(), None).await?;
    Ok(render_page_or_partial(&headers, SurfaceTab::Bench, content))
}

async fn bench_detail_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(kit_build_id): Path<String>,
) -> Result<Html<String>, (StatusCode, String)> {
    let content = load_and_render_bench_workspace(&state.db, Some(&kit_build_id), None).await?;
    Ok(render_page_or_partial(&headers, SurfaceTab::Bench, content))
}

async fn render_bench_line_update_response(
    db: &Db,
    headers: &HeaderMap,
    updated_line: &BomLine,
) -> Result<Html<String>, (StatusCode, String)> {
    let hx_target = headers
        .get("HX-Target")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .trim_start_matches('#');

    if is_htmx_request(headers) && hx_target.starts_with("bench-row-") {
        let build = get_kit_build(db, &updated_line.kit_build_id)
            .await
            .map_err(internal_err)?
            .ok_or_else(|| {
                (
                    StatusCode::NOT_FOUND,
                    format!("Kit Build '{}' not found", updated_line.kit_build_id),
                )
            })?;
        let progress = compute_bench_placement_progress(&build.bom_lines);
        let stage_groups = group_bom_lines_by_stage(&build.bom_lines);
        let row = render_bench_bom_line_row(updated_line);
        let progress_oob = render_bench_progress_banner(&progress, true);
        let readiness_oob = render_build_readiness_summary_badge(&build.readiness, true);

        let combined = html! {
            (row)
            (progress_oob)
            (readiness_oob)
            @for group in &stage_groups {
                span id=(format!("bench-stage-badge-{}", group.stage.slug()))
                    class=(if group.soldered_placements == group.total_placements && group.total_placements > 0 { "badge ok" } else { "badge info" })
                    hx-swap-oob="true" {
                    (group.soldered_placements) " / " (group.total_placements) " Soldered"
                }
            }
        };
        Ok(Html(combined.into_string()))
    } else {
        let content =
            load_and_render_bench_workspace(db, Some(&updated_line.kit_build_id), None).await?;
        Ok(render_page_or_partial(headers, SurfaceTab::Bench, content))
    }
}

async fn toggle_designator_form_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(line_id): Path<String>,
    Query(q_form): Query<ToggleDesignatorForm>,
    Form(f_form): Form<ToggleDesignatorForm>,
) -> Result<Html<String>, (StatusCode, String)> {
    let des = f_form
        .designator
        .or(q_form.designator)
        .unwrap_or_default();
    let updated_line = toggle_bom_line_designator(&state.db, &line_id, &des)
        .await
        .map_err(internal_err)?;
    render_bench_line_update_response(&state.db, &headers, &updated_line).await
}

async fn toggle_designator_path_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((line_id, des)): Path<(String, String)>,
) -> Result<Html<String>, (StatusCode, String)> {
    let updated_line = toggle_bom_line_designator(&state.db, &line_id, &des)
        .await
        .map_err(internal_err)?;
    render_bench_line_update_response(&state.db, &headers, &updated_line).await
}

async fn toggle_kit_designator_path_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((_kit_id, line_id, des)): Path<(String, String, String)>,
) -> Result<Html<String>, (StatusCode, String)> {
    let updated_line = toggle_bom_line_designator(&state.db, &line_id, &des)
        .await
        .map_err(internal_err)?;
    render_bench_line_update_response(&state.db, &headers, &updated_line).await
}

async fn install_all_designators_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(line_id): Path<String>,
) -> Result<Html<String>, (StatusCode, String)> {
    let updated_line = mark_all_bom_line_designators_installed(&state.db, &line_id)
        .await
        .map_err(internal_err)?;
    render_bench_line_update_response(&state.db, &headers, &updated_line).await
}

async fn install_all_kit_designators_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((_kit_id, line_id)): Path<(String, String)>,
) -> Result<Html<String>, (StatusCode, String)> {
    let updated_line = mark_all_bom_line_designators_installed(&state.db, &line_id)
        .await
        .map_err(internal_err)?;
    render_bench_line_update_response(&state.db, &headers, &updated_line).await
}

async fn render_bench_panel_or_workspace(
    db: &Db,
    headers: &HeaderMap,
    kit_build_id: &str,
    flash_error: Option<&str>,
) -> Result<Html<String>, (StatusCode, String)> {
    let hx_target = headers
        .get("HX-Target")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .trim_start_matches('#');

    if is_htmx_request(headers) && (hx_target == "bench-right-panel" || hx_target == "inspector") {
        let attachments = list_build_attachments(db, kit_build_id)
            .await
            .map_err(internal_err)?;
        let log_entries = list_bench_log_entries(db, kit_build_id)
            .await
            .map_err(internal_err)?;
        let panel = render_bench_right_panel(kit_build_id, &attachments, &log_entries, flash_error);
        Ok(Html(panel.into_string()))
    } else if is_htmx_request(headers) && hx_target == "bench-log-list" {
        let log_entries = list_bench_log_entries(db, kit_build_id)
            .await
            .map_err(internal_err)?;
        let list = render_bench_log_list(&log_entries);
        Ok(Html(list.into_string()))
    } else {
        let content =
            load_and_render_bench_workspace(db, Some(kit_build_id), flash_error).await?;
        Ok(render_page_or_partial(headers, SurfaceTab::Bench, content))
    }
}

async fn upload_build_attachment_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(kit_build_id): Path<String>,
    mut multipart: Multipart,
) -> Result<Html<String>, (StatusCode, String)> {
    let mut kind = String::from("Schematic PDF");
    let mut bench_log_entry_id: Option<String> = None;
    let mut file_name = String::new();
    let mut mime_type = String::from("application/octet-stream");
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut external_url = String::new();
    let mut label = String::new();

    while let Some(field) = multipart.next_field().await.map_err(internal_err)? {
        let field_name = field.name().unwrap_or("").to_string();
        if field_name == "file" || field.file_name().is_some() {
            if let Some(fname) = field.file_name() {
                file_name = fname.to_string();
            }
            if let Some(ct) = field.content_type() {
                mime_type = ct.to_string();
            }
            let data = field.bytes().await.map_err(internal_err)?;
            file_bytes = Some(data.to_vec());
        } else {
            let text = field.text().await.map_err(internal_err)?;
            match field_name.as_str() {
                "kind" => kind = text,
                "bench_log_entry_id" if !text.trim().is_empty() => {
                    bench_log_entry_id = Some(text.trim().to_string())
                }
                "external_url" | "url" => external_url = text,
                "label" | "title" | "original_filename" => {
                    if file_name.is_empty() {
                        file_name = text.clone();
                    }
                    label = text;
                }
                _ => {}
            }
        }
    }

    let res = if let Some(bytes) = file_bytes {
        add_build_attachment_file(
            &state.db,
            &kit_build_id,
            bench_log_entry_id.as_deref(),
            &kind,
            &file_name,
            &mime_type,
            &bytes,
        )
        .await
    } else if !external_url.trim().is_empty() {
        add_build_external_link(&state.db, &kit_build_id, Some(&kind), &label, &external_url).await
    } else {
        Err(anyhow::anyhow!("No attachment file or external URL provided"))
    };

    let flash_err = res.err().map(|e| e.to_string());
    render_bench_panel_or_workspace(&state.db, &headers, &kit_build_id, flash_err.as_deref()).await
}

async fn add_build_link_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(kit_build_id): Path<String>,
    Form(form): Form<AddExternalLinkForm>,
) -> Result<Html<String>, (StatusCode, String)> {
    let url = form
        .external_url
        .or(form.url)
        .unwrap_or_default();
    let label = form
        .label
        .or(form.title)
        .unwrap_or_default();

    let res = add_build_external_link(
        &state.db,
        &kit_build_id,
        form.kind.as_deref(),
        &label,
        &url,
    )
    .await;
    let flash_err = res.err().map(|e| e.to_string());
    render_bench_panel_or_workspace(&state.db, &headers, &kit_build_id, flash_err.as_deref()).await
}

async fn create_bench_log_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(kit_build_id): Path<String>,
    body_bytes: Bytes,
) -> Result<Html<String>, (StatusCode, String)> {
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let mut entry_type = String::from("Note");
    let mut body_text = String::new();
    let mut photo_file: Option<(String, String, Vec<u8>)> = None;

    if content_type.starts_with("multipart/form-data") {
        let req = axum::http::Request::builder()
            .method("POST")
            .header(header::CONTENT_TYPE, content_type)
            .body(axum::body::Body::from(body_bytes))
            .map_err(internal_err)?;
        use axum::extract::FromRequest;
        let mut multipart = Multipart::from_request(req, &())
            .await
            .map_err(internal_err)?;
        while let Some(field) = multipart.next_field().await.map_err(internal_err)? {
            let fname = field.name().unwrap_or("").to_string();
            if fname == "photo" || fname == "file" || field.file_name().is_some() {
                let orig = field.file_name().unwrap_or("bench_photo.jpg").to_string();
                let ct = field.content_type().unwrap_or("image/jpeg").to_string();
                let data = field.bytes().await.map_err(internal_err)?.to_vec();
                if !data.is_empty() && !orig.trim().is_empty() {
                    photo_file = Some((orig, ct, data));
                }
            } else {
                let val = field.text().await.map_err(internal_err)?;
                match fname.as_str() {
                    "entry_type" | "type" => entry_type = val,
                    "body" | "note" => body_text = val,
                    _ => {}
                }
            }
        }
    } else {
        let req = axum::http::Request::builder()
            .method("POST")
            .header(
                header::CONTENT_TYPE,
                if content_type.is_empty() {
                    "application/x-www-form-urlencoded"
                } else {
                    content_type
                },
            )
            .body(axum::body::Body::from(body_bytes))
            .map_err(internal_err)?;
        use axum::extract::FromRequest;
        let Form(form) = Form::<CreateBenchLogForm>::from_request(req, &())
            .await
            .map_err(internal_err)?;
        if let Some(et) = form.entry_type {
            entry_type = et;
        }
        if let Some(b) = form.body {
            body_text = b;
        }
    }

    let created = create_bench_log_entry(
        &state.db,
        CreateBenchLogEntry {
            kit_build_id: kit_build_id.clone(),
            entry_type,
            body: body_text,
        },
    )
    .await;

    let flash_err = match created {
        Ok(entry) => {
            if let Some((orig_name, mime, bytes)) = photo_file {
                add_build_attachment_file(
                    &state.db,
                    &kit_build_id,
                    Some(&entry.id),
                    "Photos",
                    &orig_name,
                    &mime,
                    &bytes,
                )
                .await
                .err()
                .map(|e| e.to_string())
            } else {
                None
            }
        }
        Err(e) => Some(e.to_string()),
    };

    render_bench_panel_or_workspace(&state.db, &headers, &kit_build_id, flash_err.as_deref()).await
}

async fn download_attachment_handler(
    State(state): State<AppState>,
    Path(attachment_id): Path<String>,
) -> Result<Response, (StatusCode, String)> {
    let att = get_build_attachment(&state.db, &attachment_id)
        .await
        .map_err(internal_err)?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("Attachment '{attachment_id}' not found"),
            )
        })?;

    let bytes = read_build_attachment_bytes(&state.db, &att).map_err(internal_err)?;
    let disposition = format!(
        "inline; filename=\"{}\"",
        att.original_filename.replace('"', "_")
    );
    Ok((
        [
            (header::CONTENT_TYPE, att.mime_type),
            (header::CONTENT_DISPOSITION, disposition),
        ],
        bytes,
    )
        .into_response())
}

fn render_bench_progress_banner(progress: &BenchPlacementProgress, is_oob: bool) -> Markup {
    html! {
        div id="bench-progress-banner"
            class="card"
            style="margin-block-end:0.75rem; display:flex; justify-content:space-between; align-items:center; flex-wrap:wrap; gap:0.5rem;"
            hx-swap-oob=[if is_oob { Some("true") } else { None }] {
            div {
                strong { "Bench Populate Progress: " (progress.display_label()) }
                div style="font-size:0.8rem; color:var(--text-muted);" {
                    "Click individual Reference Designators as you solder them onto the PCB, or mark the entire BOM Line Installed."
                }
            }
            progress max=(progress.total_placements.max(1)) value=(progress.soldered_placements) style="inline-size:180px;" {}
        }
    }
}

fn render_bench_bom_line_row(line: &BomLine) -> Markup {
    let stage = classify_bom_line_stage(line);
    let placements = bom_line_placements(line);

    html! {
        tr id=(format!("bench-row-{}", line.id)) {
            th scope="row" style="font-size:0.78rem; color:var(--text-muted);" {
                (stage.as_str())
            }
            td {
                strong { (line.raw_spec_text) }
                " "
                small style="color:var(--text-muted);" { "(" (line.canonical_value_display) ")" }
                @if let Some(ref comp) = line.linked_component {
                    div style="font-size:0.78rem; color:var(--text-muted);" {
                        (comp.name)
                    }
                } @else if line.included_in_kit {
                    div style="font-size:0.78rem; color:var(--text-muted);" {
                        "Included in Kit"
                    }
                }
                @if !line.substitution_note.is_empty() {
                    div style="font-size:0.76rem; color:var(--warn-text);" {
                        "Substitution: " (line.substitution_note)
                    }
                }
            }
            td {
                @for des in &placements {
                    @let is_done = line.fulfillment_status == "Installed" || line.installed_designators.contains(des);
                    form method="post"
                        action=(format!("/bench/bom-lines/{}/toggle-designator", line.id))
                        hx-post=(format!("/bench/bom-lines/{}/toggle-designator", line.id))
                        hx-target=(format!("#bench-row-{}", line.id))
                        hx-swap="outerHTML"
                        style="display:inline;" {
                        input type="hidden" name="designator" value=(des);
                        button type="submit"
                            class=(if is_done { "designator-chip done" } else { "designator-chip" }) {
                            @if is_done { "✓ " } @else { "○ " }
                            (des)
                        }
                    }
                }
            }
            td {
                div style="display:flex; align-items:center; gap:0.35rem; flex-wrap:wrap;" {
                    (render_fulfillment_status_badge(&line.fulfillment_status))
                    @if line.fulfillment_status != "Installed" {
                        form method="post"
                            action=(format!("/bench/bom-lines/{}/install-all", line.id))
                            hx-post=(format!("/bench/bom-lines/{}/install-all", line.id))
                            hx-target=(format!("#bench-row-{}", line.id))
                            hx-swap="outerHTML"
                            style="display:inline; margin:0;" {
                            button type="submit" class="xs primary" {
                                "Mark All Installed"
                            }
                        }
                    }
                }
            }
        }
    }
}

fn render_bench_log_list(log_entries: &[BenchLogEntry]) -> Markup {
    html! {
        div id="bench-log-list" {
            @if log_entries.is_empty() {
                p style="font-size:0.8rem; color:var(--text-muted); margin:0;" {
                    "No Bench Log Entries recorded yet. Log IC pin voltages, audio probe measurements, or circuit modifications above."
                }
            } @else {
                @for entry in log_entries {
                    div style="border-block-start:1px solid var(--border-color); padding:0.45rem 0; font-size:0.82rem;" {
                        div style="display:flex; justify-content:space-between; color:var(--text-muted); font-size:0.75rem;" {
                            strong { (entry.entry_type) }
                            span class="mono" { (entry.created_at) }
                        }
                        div style="margin-block-start:0.15rem;" { (entry.body) }
                        @if !entry.attachments.is_empty() {
                            div style="display:flex; gap:0.3rem; flex-wrap:wrap; margin-block-start:0.25rem;" {
                                @for att in &entry.attachments {
                                    a class="badge info"
                                        href=(format!("/attachments/{}/download", att.id))
                                        target="_blank" {
                                        "📷 " (att.original_filename)
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn render_bench_right_panel(
    kit_build_id: &str,
    attachments: &[BuildAttachment],
    log_entries: &[BenchLogEntry],
    flash_error: Option<&str>,
) -> Markup {
    html! {
        aside id="bench-right-panel" class="card" aria-label="Bench Log & Reference Docs" {
            @if let Some(err) = flash_error {
                div style="background:var(--err-bg); color:var(--err-text); padding:0.5rem; border-radius:4px; margin-block-end:0.65rem; font-size:0.82rem; font-weight:600;" {
                    (err)
                }
            }

            h3 style="margin-block-start:0; font-size:0.95rem;" { "Bench Attachments & External Links" }

            div style="display:flex; flex-direction:column; gap:0.35rem; margin-block-end:0.65rem;" {
                @if attachments.is_empty() {
                    p style="font-size:0.8rem; color:var(--text-muted); margin:0;" {
                        "No schematics, drill templates, or links attached yet."
                    }
                } @else {
                    @for att in attachments {
                        div style="display:flex; justify-content:space-between; align-items:center; gap:0.35rem; font-size:0.8rem; border:1px solid var(--border-color); border-radius:4px; padding:0.3rem 0.45rem; background:var(--bg-subtle);" {
                            @if !att.external_url.is_empty() {
                                a href=(att.external_url) target="_blank" rel="noopener noreferrer" style="font-weight:600;" {
                                    "🔗 " (att.original_filename)
                                }
                            } @else {
                                a href=(format!("/attachments/{}/download", att.id)) target="_blank" style="font-weight:600;" {
                                    "📄 " (att.original_filename)
                                }
                            }
                            span class="badge neutral" { (att.kind) }
                        }
                    }
                }
            }

            form class="form-grid"
                method="post"
                enctype="multipart/form-data"
                action=(format!("/bench/{kit_build_id}/attachments"))
                hx-post=(format!("/bench/{kit_build_id}/attachments"))
                hx-encoding="multipart/form-data"
                hx-target="#bench-right-panel"
                hx-swap="outerHTML"
                style="border-block-start:1px solid var(--border-color); padding-block-start:0.55rem; margin-block-end:0.6rem;" {
                div class="form-field" {
                    label { "Upload Schematic / Drill Doc / Photo" }
                    select name="kind" {
                        @for k in AttachmentKind::ALL_FILE_KINDS {
                            option value=(k.as_str()) { (k.as_str()) }
                        }
                    }
                }
                div style="display:flex; gap:0.35rem; align-items:center;" {
                    input type="file" name="file" required style="flex:1; font-size:0.78rem;";
                    button type="submit" class="xs primary" { "Upload" }
                }
            }

            form class="form-grid"
                method="post"
                action=(format!("/bench/{kit_build_id}/links"))
                hx-post=(format!("/bench/{kit_build_id}/links"))
                hx-target="#bench-right-panel"
                hx-swap="outerHTML"
                style="border-block-start:1px solid var(--border-color); padding-block-start:0.55rem; margin-block-end:0.75rem;" {
                div class="form-field" {
                    label { "Attach External Project / Forum URL" }
                    input type="text" name="label" placeholder="Label (e.g. Aion Build Guide)";
                }
                div style="display:flex; gap:0.35rem;" {
                    input type="text" name="external_url" placeholder="https://..." required style="flex:1;";
                    button type="submit" class="xs primary" { "Add Link" }
                }
            }

            div style="border-block-start:1px solid var(--border-color); padding-block-start:0.65rem;" {
                h3 style="margin:0 0 0.45rem 0; font-size:0.95rem;" { "Timestamped Bench Log Entries" }

                form id="bench-log-form"
                    class="form-grid"
                    method="post"
                    action=(format!("/bench/{kit_build_id}/logs"))
                    hx-post=(format!("/bench/{kit_build_id}/logs"))
                    hx-target="#bench-right-panel"
                    hx-swap="outerHTML"
                    style="margin-block-end:0.65rem;" {
                    div class="form-field" {
                        label { "Entry Category" }
                        select name="entry_type" aria-label="Bench Log Entry Type" {
                            @for et in BenchLogEntryType::ALL {
                                option value=(et.as_str()) { (et.as_str()) }
                            }
                        }
                    }
                    div style="display:flex; gap:0.35rem;" {
                        input type="text"
                            name="body"
                            placeholder="e.g. IC1 Pin 7 = 4.51V..."
                            required
                            style="flex:1;"
                            aria-label="Bench Log Entry Text";
                        button type="submit" class="primary xs" { "Add Entry" }
                    }
                }

                (render_bench_log_list(log_entries))
            }
        }
    }
}

fn render_bench_workspace(
    all_builds: &[KitBuild],
    active_build: Option<&KitBuild>,
    attachments: &[BuildAttachment],
    log_entries: &[BenchLogEntry],
    flash_error: Option<&str>,
) -> Markup {
    html! {
        div id="bench-workspace" {
            div class="va-filter-ribbon" style="justify-content:space-between;" {
                div style="display:flex; gap:0.45rem; align-items:center; flex-wrap:wrap;" {
                    strong style="font-size:0.85rem;" { "Active Bench Kit Build:" }
                    @if all_builds.is_empty() {
                        span style="color:var(--text-muted); font-size:0.82rem;" {
                            "No Kit Builds created yet."
                        }
                    } @else {
                        @for b in all_builds {
                            @let is_active = active_build.is_some_and(|ab| ab.id == b.id);
                            a class=(if is_active { "btn xs primary" } else { "btn xs" })
                                href=(format!("/bench/{}", b.id))
                                hx-get=(format!("/bench/{}", b.id))
                                hx-target="#bench-workspace"
                                hx-swap="outerHTML"
                                hx-push-url="true" {
                                (b.name)
                                " ("
                                (b.status)
                                ")"
                            }
                        }
                    }
                }

                @if let Some(build) = active_build {
                    div style="display:flex; align-items:center; gap:0.5rem; flex-wrap:wrap;" {
                        span style="font-size:0.8rem; color:var(--text-muted);" {
                            "Project Box: " code { (build.location_breadcrumb) }
                        }
                        (render_build_readiness_summary_badge(&build.readiness, false))
                    }
                }
            }

            @if let Some(build) = active_build {
                @let progress = compute_bench_placement_progress(&build.bom_lines);
                @let stage_groups = group_bom_lines_by_stage(&build.bom_lines);

                div class="va-grid-with-drawer" {
                    div {
                        (render_bench_progress_banner(&progress, false))

                        @for group in &stage_groups {
                            div class="card" style="margin-block-end:0.85rem; padding:0; overflow:hidden;" {
                                div style="padding:0.55rem 0.75rem; background:var(--bg-subtle); border-block-end:1px solid var(--border-color); display:flex; justify-content:space-between; align-items:center;" {
                                    strong style="font-size:0.86rem;" { (group.stage.as_str()) }
                                    span id=(format!("bench-stage-badge-{}", group.stage.slug()))
                                        class=(if group.soldered_placements == group.total_placements && group.total_placements > 0 { "badge ok" } else { "badge info" }) {
                                        (group.soldered_placements) " / " (group.total_placements) " Soldered"
                                    }
                                }
                                div tabindex="0" role="region" aria-label=(group.stage.as_str()) class="table-wrapper" style="border:none; border-radius:0;" {
                                    table style="--label-2:'Value & Component'; --label-3:'Placement Designators'; --label-4:'Line Status';" {
                                        caption { (group.stage.as_str()) }
                                        thead {
                                            tr {
                                                th scope="col" { "Assembly Stage" }
                                                th scope="col" { "Schematic Spec & Part" }
                                                th scope="col" { "Interactive Reference Designators" }
                                                th scope="col" { "Fulfillment Status" }
                                            }
                                        }
                                        tbody {
                                            @if group.lines.is_empty() {
                                                tr {
                                                    td colspan="4" style="color:var(--text-muted); font-size:0.82rem; text-align:center; padding:0.75rem;" {
                                                        "No BOM Lines in this assembly stage."
                                                    }
                                                }
                                            } @else {
                                                @for line in &group.lines {
                                                    (render_bench_bom_line_row(line))
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    (render_bench_right_panel(&build.id, attachments, log_entries, flash_error))
                }
            } @else {
                div class="card" {
                    h2 style="margin-block-start:0;" { "Active Bench Checklist" }
                    p style="color:var(--text-muted);" {
                        "Create a Kit Build in the Kit Build BOM tab to start populating its PCB on the bench."
                    }
                }
            }
        }
    }
}

fn percent_decode(input: &str) -> String {
    let bytes = input.replace('+', " ").into_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or("");
            if let Ok(val) = u8::from_str_radix(hex, 16) {
                out.push(val);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn percent_encode(input: &str) -> String {
    let mut out = String::new();
    for b in input.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

fn parse_urlencoded_pairs(raw: &str) -> Vec<(String, String)> {
    raw.split('&')
        .filter(|s| !s.is_empty())
        .map(|pair| {
            if let Some((k, v)) = pair.split_once('=') {
                (percent_decode(k), percent_decode(v))
            } else {
                (percent_decode(pair), String::new())
            }
        })
        .collect()
}

fn parse_shopping_list_params(
    raw: &str,
    all_builds: &[KitBuild],
) -> (ShoppingListQuery, Vec<(String, String)>) {
    let pairs = parse_urlencoded_pairs(raw);

    let mut kit_build_ids = Vec::new();
    let mut include_low_stock_restock = false;
    let mut supplier_filter = None;
    let mut has_explicit_selection = false;

    for (k, v) in &pairs {
        match k.as_str() {
            "kit_build_id" | "kit_build_ids" | "kit_id" => {
                has_explicit_selection = true;
                for part in v.split(',') {
                    let trimmed = part.trim();
                    if !trimmed.is_empty() && !kit_build_ids.iter().any(|x| x == trimmed) {
                        kit_build_ids.push(trimmed.to_string());
                    }
                }
            }
            "submitted" => {
                has_explicit_selection = true;
            }
            "include_low_stock_restock" => {
                let vt = v.trim();
                if vt == "true" || vt == "1" || vt == "on" || vt == "yes" {
                    include_low_stock_restock = true;
                }
            }
            "supplier" | "supplier_filter" => {
                let vt = v.trim();
                if !vt.is_empty() && !vt.eq_ignore_ascii_case("All") {
                    supplier_filter = Some(vt.to_string());
                }
            }
            _ => {}
        }
    }

    if !has_explicit_selection && kit_build_ids.is_empty() {
        for b in all_builds {
            if !matches!(b.status.as_str(), "Completed" | "Shelved / Abandoned") {
                kit_build_ids.push(b.id.clone());
            }
        }
        if kit_build_ids.is_empty() {
            for b in all_builds {
                kit_build_ids.push(b.id.clone());
            }
        }
    }

    (
        ShoppingListQuery {
            kit_build_ids,
            include_low_stock_restock,
            supplier_filter,
        },
        pairs,
    )
}

fn build_shopping_list_query_string(query: &ShoppingListQuery) -> String {
    let mut parts = vec!["submitted=1".to_string()];
    for id in &query.kit_build_ids {
        parts.push(format!("kit_build_id={}", percent_encode(id)));
    }
    if query.include_low_stock_restock {
        parts.push("include_low_stock_restock=true".to_string());
    }
    if let Some(ref sup) = query.supplier_filter
        && !sup.trim().is_empty()
    {
        parts.push(format!("supplier={}", percent_encode(sup.trim())));
    }
    parts.join("&")
}

async fn load_and_render_shopping_list_workspace(
    db: &Db,
    raw_query_or_body: &str,
    flash_message: Option<&str>,
    flash_error: Option<&str>,
) -> Result<Markup, (StatusCode, String)> {
    let all_builds = list_kit_builds(db).await.map_err(internal_err)?;
    let all_locations = list_storage_locations(db).await.map_err(internal_err)?;
    let (sl_query, _) = parse_shopping_list_params(raw_query_or_body, &all_builds);
    let report = compute_shopping_list(db, &sl_query)
        .await
        .map_err(internal_err)?;

    Ok(render_shopping_list_workspace(
        &all_builds,
        &all_locations,
        &sl_query,
        &report,
        flash_message,
        flash_error,
    ))
}

fn render_shopping_list_workspace(
    all_builds: &[KitBuild],
    all_locations: &[StorageLocation],
    query: &ShoppingListQuery,
    report: &ShoppingListReport,
    flash_message: Option<&str>,
    flash_error: Option<&str>,
) -> Markup {
    let qs = build_shopping_list_query_string(query);
    let full_csv_href = format!("/shopping-list/export/full.csv?{qs}");
    let quick_csv_href = format!("/shopping-list/export/quick-order.csv?{qs}");
    let quick_order_preview =
        export_supplier_quick_order_csv(report, query.supplier_filter.as_deref());

    html! {
        div id="shopping-list-workspace" {
            @if let Some(err) = flash_error {
                div class="card" style="background:var(--err-bg); color:var(--err-text); margin-block-end:0.85rem; font-weight:600;" role="alert" {
                    (err)
                }
            }
            @if let Some(msg) = flash_message {
                div class="card" style="background:var(--ok-bg); color:var(--ok-text); margin-block-end:0.85rem; font-weight:600;" role="status" {
                    (msg)
                }
            }

            form class="va-filter-ribbon"
                method="get"
                action="/shopping-list"
                hx-get="/shopping-list"
                hx-trigger="change, submit"
                hx-target="#shopping-list-workspace"
                hx-swap="outerHTML"
                hx-push-url="true" {
                input type="hidden" name="submitted" value="1";

                div style="display:flex; align-items:center; gap:0.6rem; flex-wrap:wrap; flex:1;" {
                    strong style="font-size:0.82rem;" { "Selected Kit Builds:" }
                    @if all_builds.is_empty() {
                        span style="color:var(--text-muted); font-size:0.82rem;" {
                            "No Kit Builds created yet."
                        }
                    } @else {
                        @for build in all_builds {
                            @let is_checked = query.kit_build_ids.iter().any(|id| id == &build.id);
                            label style="display:inline-flex; align-items:center; gap:0.3rem; font-size:0.82rem; padding:0.2rem 0.5rem; border:1px solid var(--border-color); border-radius:4px; background:var(--bg-subtle); cursor:pointer;" {
                                input type="checkbox"
                                    name="kit_build_id"
                                    value=(build.id)
                                    checked[is_checked];
                                span { (build.name) }
                                span class="badge neutral" style="font-size:0.7rem;" { (build.status) }
                            }
                        }
                    }
                }

                label style="display:inline-flex; align-items:center; gap:0.35rem; font-size:0.82rem; font-weight:600; cursor:pointer;" {
                    input type="checkbox"
                        name="include_low_stock_restock"
                        value="true"
                        checked[query.include_low_stock_restock];
                    "Include catalog low-stock restocking"
                }

                select name="supplier" aria-label="Filter by Supplier" {
                    option value="" { "All Suppliers" }
                    @for sup in &report.available_suppliers {
                        option value=(sup)
                            selected[query.supplier_filter.as_deref().is_some_and(|s| s.eq_ignore_ascii_case(sup))] {
                            (sup)
                        }
                    }
                }

                button type="submit" class="xs" { "Refresh Shopping List" }
                a class="btn xs" href=(full_csv_href) download="shopping-list-full.csv" {
                    "Export Full Shopping List CSV"
                }
                a class="btn primary xs" href=(quick_csv_href) download="supplier-quick-order.csv" {
                    "Export Supplier Quick-Order CSV"
                }
            }

            div class="va-grid-with-drawer" {
                div style="display:flex; flex-direction:column; gap:1rem;" {
                    div class="table-wrapper" role="region" aria-labelledby="shopping-list-caption" tabindex="0" {
                        table {
                            caption id="shopping-list-caption" {
                                div style="display:flex; justify-content:space-between; align-items:center; flex-wrap:wrap; gap:0.5rem;" {
                                    span {
                                        "Live Multi-Build Shopping List (" (report.lines.len()) " lines · "
                                        (report.total_shortfall_lines) " shortfalls · "
                                        (report.total_order_quantity) " total parts to order)"
                                    }
                                    span class="badge info" {
                                        "Net Shortfall = max(0, sum(unallocated_needed) - Available Stock)"
                                    }
                                }
                            }
                            thead {
                                tr {
                                    th scope="col" { "Component / Normalized Spec" }
                                    th scope="col" { "Category & Package" }
                                    th scope="col" { "Total Needed" }
                                    th scope="col" { "Available Stock" }
                                    th scope="col" { "Net Shortfall / Restock" }
                                    th scope="col" { "Contributing Kit Builds & Designators" }
                                    th scope="col" { "Inline Override (Order Qty / Supplier / Part #)" }
                                }
                            }
                            tbody {
                                @if report.lines.is_empty() {
                                    tr {
                                        td colspan="7" style="text-align:center; color:var(--text-muted); padding:1.25rem;" {
                                            "No shortfalls or low-stock restock items for the selected Kit Builds."
                                        }
                                    }
                                } @else {
                                    @for line in &report.lines {
                                        tr {
                                            th scope="row" {
                                                div style="font-weight:700;" { (line.spec_or_component_name) }
                                                div style="font-size:0.75rem; color:var(--text-muted); font-family:ui-monospace, monospace;" {
                                                    (line.raw_spec_text)
                                                }
                                                @if line.component_id.is_some() {
                                                    span class="badge ok" style="font-size:0.7rem;" { "Catalog Linked" }
                                                } @else {
                                                    span class="badge warn" style="font-size:0.7rem;" { "Unlinked BOM Spec" }
                                                }
                                            }
                                            td data-label="Category & Package" {
                                                div { (line.category) }
                                                @if !line.package_or_mounting.is_empty() {
                                                    span class="badge neutral" { (line.package_or_mounting) }
                                                }
                                            }
                                            td data-label="Total Needed" style="font-weight:700;" {
                                                (line.total_needed)
                                            }
                                            td data-label="Available Stock" {
                                                (line.available_stock)
                                            }
                                            td data-label="Net Shortfall / Restock" {
                                                @if line.net_shortfall > 0 {
                                                    span class="badge err" { "Shortfall: " (line.net_shortfall) }
                                                }
                                                @if line.restock_top_up > 0 {
                                                    span class="badge warn" style="margin-inline-start:0.25rem;" {
                                                        "Restock +" (line.restock_top_up)
                                                    }
                                                }
                                            }
                                            td data-label="Kit Builds & Designators" {
                                                @if line.build_breakdown.is_empty() {
                                                    span class="badge neutral" {
                                                        "Catalog Low-Stock Restock (Threshold: " (line.low_stock_threshold) ")"
                                                    }
                                                } @else {
                                                    details open {
                                                        summary style="cursor:pointer; font-size:0.8rem; font-weight:600;" {
                                                            (line.build_breakdown.len()) " Kit Build(s)"
                                                        }
                                                        ul style="margin:0.35rem 0 0 1rem; padding:0; font-size:0.78rem;" {
                                                            @for bd in &line.build_breakdown {
                                                                li {
                                                                    strong { (bd.kit_build_name) }
                                                                    ": " (bd.quantity_needed) "x "
                                                                    @if !bd.reference_designators.is_empty() {
                                                                        code { "(" (bd.reference_designators) ")" }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            td data-label="Inline Override" {
                                                form method="post"
                                                    action="/shopping-list/override"
                                                    hx-post="/shopping-list/override"
                                                    hx-target="#shopping-list-workspace"
                                                    hx-swap="outerHTML"
                                                    style="display:flex; gap:0.3rem; flex-wrap:wrap; align-items:center;" {
                                                    input type="hidden" name="submitted" value="1";
                                                    @for bid in &query.kit_build_ids {
                                                        input type="hidden" name="kit_build_id" value=(bid);
                                                    }
                                                    @if query.include_low_stock_restock {
                                                        input type="hidden" name="include_low_stock_restock" value="true";
                                                    }
                                                    @if let Some(ref sf) = query.supplier_filter {
                                                        input type="hidden" name="supplier_filter" value=(sf);
                                                    }
                                                    input type="hidden" name="group_key" value=(line.group_key);

                                                    input type="number"
                                                        name="order_quantity"
                                                        value=(line.order_quantity)
                                                        min="0"
                                                        aria-label="Order Quantity"
                                                        style="inline-size:68px; font-size:0.8rem;";
                                                    input type="text"
                                                        name="supplier"
                                                        value=(line.supplier)
                                                        placeholder="Supplier"
                                                        aria-label="Supplier"
                                                        style="inline-size:95px; font-size:0.8rem;";
                                                    input type="text"
                                                        name="supplier_part_no"
                                                        value=(line.supplier_part_no)
                                                        placeholder="Part #"
                                                        aria-label="Supplier Part #"
                                                        style="inline-size:115px; font-size:0.8rem;";
                                                    button type="submit" class="xs" { "Save" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                aside id="inspector" class="card" aria-label="Order Receiving & Quick-Order Clipboard" {
                    h2 style="margin-block-start:0; font-size:1rem;" {
                        "Receive Selected Lines into Stock"
                    }
                    p style="font-size:0.8rem; color:var(--text-muted); margin-block-start:0;" {
                        "Creates or increments Stock Lots at the chosen Storage Location and auto-creates & links catalog Components for any unlinked BOM Lines."
                    }
                    form class="form-grid"
                        method="post"
                        action="/shopping-list/receive"
                        hx-post="/shopping-list/receive"
                        hx-target="#shopping-list-workspace"
                        hx-swap="outerHTML" {
                        input type="hidden" name="submitted" value="1";
                        @for bid in &query.kit_build_ids {
                            input type="hidden" name="kit_build_id" value=(bid);
                        }
                        @if query.include_low_stock_restock {
                            input type="hidden" name="include_low_stock_restock" value="true";
                        }
                        @if let Some(ref sf) = query.supplier_filter {
                            input type="hidden" name="supplier_filter" value=(sf);
                        }

                        div class="form-field" {
                            label for="receive-target-location" { "Target Storage Location" }
                            select id="receive-target-location" name="target_location_id" {
                                option value="" { "— Unassigned Location —" }
                                @for loc in all_locations {
                                    option value=(loc.id) { (loc.breadcrumb_path) }
                                }
                            }
                        }

                        div style="display:flex; flex-direction:column; gap:0.35rem; max-block-size:220px; overflow-y:auto; padding:0.35rem; border:1px solid var(--border-color); border-radius:4px;" {
                            @for line in &report.lines {
                                label style="display:flex; align-items:center; justify-content:space-between; gap:0.4rem; font-size:0.8rem; cursor:pointer;" {
                                    span style="display:inline-flex; align-items:center; gap:0.35rem;" {
                                        input type="checkbox"
                                            name="group_key"
                                            value=(line.group_key)
                                            checked;
                                        span { (line.spec_or_component_name) }
                                    }
                                    span class="badge neutral" { "+" (line.order_quantity) }
                                }
                            }
                        }

                        button type="submit" class="primary xs" style="inline-size:100%; justify-content:center;" {
                            "Receive Selected Lines into Stock"
                        }
                    }

                    hr style="border:none; border-block-start:1px solid var(--border-color); margin-block:0.85rem;";

                    h3 style="margin-block:0 0.35rem; font-size:0.88rem;" {
                        "Supplier Quick-Order Clipboard Format"
                    }
                    textarea readonly rows="6"
                        aria-label="Supplier Quick-Order CSV Clipboard"
                        style="inline-size:100%; font-family:ui-monospace, monospace; font-size:0.75rem;" {
                        (quick_order_preview)
                    }
                }
            }
        }
    }
}

async fn shopping_list_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    RawQuery(raw_query): RawQuery,
) -> Result<Html<String>, (StatusCode, String)> {
    let raw = raw_query.unwrap_or_default();
    let content = load_and_render_shopping_list_workspace(&state.db, &raw, None, None).await?;
    Ok(render_page_or_partial(
        &headers,
        SurfaceTab::ShoppingList,
        content,
    ))
}

async fn shopping_list_override_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: String,
) -> Result<Html<String>, (StatusCode, String)> {
    let pairs = parse_urlencoded_pairs(&body);

    let mut group_key = String::new();
    let mut order_quantity: Option<i64> = None;
    let mut supplier: Option<String> = None;
    let mut supplier_part_no: Option<String> = None;
    let mut notes: Option<String> = None;

    for (k, v) in &pairs {
        match k.as_str() {
            "group_key" => group_key = v.trim().to_string(),
            "order_quantity" => {
                if let Ok(q) = v.trim().parse::<i64>() {
                    order_quantity = Some(q);
                }
            }
            "supplier" => supplier = Some(v.trim().to_string()),
            "supplier_part_no" => supplier_part_no = Some(v.trim().to_string()),
            "notes" => notes = Some(v.trim().to_string()),
            _ => {}
        }
    }

    let res = upsert_shopping_list_override(
        &state.db,
        &group_key,
        order_quantity,
        supplier.as_deref(),
        supplier_part_no.as_deref(),
        notes.as_deref(),
    )
    .await;

    let flash_err = res.err().map(|e| e.to_string());
    let content = load_and_render_shopping_list_workspace(
        &state.db,
        &body,
        None,
        flash_err.as_deref(),
    )
    .await?;
    Ok(render_page_or_partial(
        &headers,
        SurfaceTab::ShoppingList,
        content,
    ))
}

async fn export_full_shopping_list_csv_handler(
    State(state): State<AppState>,
    RawQuery(raw_query): RawQuery,
) -> Result<Response, (StatusCode, String)> {
    let raw = raw_query.unwrap_or_default();
    let all_builds = list_kit_builds(&state.db).await.map_err(internal_err)?;
    let (sl_query, _) = parse_shopping_list_params(&raw, &all_builds);
    let report = compute_shopping_list(&state.db, &sl_query)
        .await
        .map_err(internal_err)?;
    let csv_body = export_full_shopping_list_csv(&report);

    Ok((
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"shopping-list-full.csv\"",
            ),
        ],
        csv_body,
    )
        .into_response())
}

async fn export_quick_order_shopping_list_csv_handler(
    State(state): State<AppState>,
    RawQuery(raw_query): RawQuery,
) -> Result<Response, (StatusCode, String)> {
    let raw = raw_query.unwrap_or_default();
    let all_builds = list_kit_builds(&state.db).await.map_err(internal_err)?;
    let (sl_query, _) = parse_shopping_list_params(&raw, &all_builds);
    let report = compute_shopping_list(&state.db, &sl_query)
        .await
        .map_err(internal_err)?;
    let csv_body =
        export_supplier_quick_order_csv(&report, sl_query.supplier_filter.as_deref());

    Ok((
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"supplier-quick-order.csv\"",
            ),
        ],
        csv_body,
    )
        .into_response())
}

async fn receive_shopping_list_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: String,
) -> Result<Html<String>, (StatusCode, String)> {
    let all_builds = list_kit_builds(&state.db).await.map_err(internal_err)?;
    let (sl_query, pairs) = parse_shopping_list_params(&body, &all_builds);
    let report = compute_shopping_list(&state.db, &sl_query)
        .await
        .map_err(internal_err)?;

    let mut target_location_id: Option<String> = None;
    let mut selected_group_keys: Vec<String> = Vec::new();
    let mut explicit_quantities: BTreeMap<String, i64> = BTreeMap::new();

    for (k, v) in &pairs {
        match k.as_str() {
            "target_location_id" | "location_id" => {
                let vt = v.trim();
                if !vt.is_empty() {
                    target_location_id = Some(vt.to_string());
                }
            }
            "group_key" | "group_keys" => {
                let vt = v.trim();
                if !vt.is_empty() && !selected_group_keys.iter().any(|g| g == vt) {
                    selected_group_keys.push(vt.to_string());
                }
            }
            _ => {
                if let Some(gkey) = k.strip_prefix("qty_")
                    && let Ok(q) = v.trim().parse::<i64>()
                {
                    explicit_quantities.insert(gkey.to_string(), q);
                }
            }
        }
    }

    let mut lines_to_receive = Vec::new();
    for gkey in &selected_group_keys {
        if let Some(line) = report.lines.iter().find(|l| &l.group_key == gkey) {
            let mut recv = ReceiveShoppingListLine::from_shopping_list_line(line);
            if let Some(&q_override) = explicit_quantities.get(gkey) {
                recv.quantity_received = q_override;
            }
            lines_to_receive.push(recv);
        } else {
            let qty = explicit_quantities.get(gkey).copied().unwrap_or(1);
            lines_to_receive.push(ReceiveShoppingListLine {
                group_key: gkey.clone(),
                component_id: gkey.strip_prefix("comp:").map(str::to_string),
                category: String::new(),
                raw_spec_text: String::new(),
                canonical_value_display: String::new(),
                package_or_mounting: String::new(),
                quantity_received: qty,
                supplier: String::new(),
                supplier_part_no: String::new(),
                bom_line_ids: Vec::new(),
            });
        }
    }

    let res = receive_shopping_list_lines(
        &state.db,
        target_location_id.as_deref(),
        &lines_to_receive,
    )
    .await;

    let (flash_msg, flash_err) = match res {
        Ok(r) => (
            Some(format!(
                "Received {} Stock Lot(s); created {} new Component(s) and linked {} BOM Line(s).",
                r.updated_lots.len(),
                r.created_components.len(),
                r.linked_bom_line_ids.len()
            )),
            None,
        ),
        Err(e) => (None, Some(e.to_string())),
    };

    let content = load_and_render_shopping_list_workspace(
        &state.db,
        &body,
        flash_msg.as_deref(),
        flash_err.as_deref(),
    )
    .await?;
    Ok(render_page_or_partial(
        &headers,
        SurfaceTab::ShoppingList,
        content,
    ))
}

async fn parse_component_stage_request(
    state: &AppState,
    headers: &HeaderMap,
    req: axum::extract::Request,
) -> Result<StageComponentImportForm, String> {
    use axum::extract::FromRequest;
    let is_multipart = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|ct| ct.starts_with("multipart/form-data"));

    if is_multipart {
        let mut mp = Multipart::from_request(req, state)
            .await
            .map_err(|e| e.to_string())?;
        let mut form = StageComponentImportForm::default();
        let mut file_text = String::new();
        while let Some(field) = mp.next_field().await.map_err(|e| e.to_string())? {
            let name = field.name().unwrap_or("").to_string();
            let val = field.text().await.map_err(|e| e.to_string())?;
            match name.as_str() {
                "csv_file" => {
                    if !val.trim().is_empty() {
                        file_text = val;
                    }
                }
                "csv_text" => form.csv_text = Some(val),
                "default_location_id" => form.default_location_id = Some(val),
                "default_supplier" => form.default_supplier = Some(val),
                "preset_name" => form.preset_name = Some(val),
                other => {
                    let _ = form.cols.apply_multipart_field(other, val);
                }
            }
        }
        if !file_text.trim().is_empty() {
            form.csv_text = Some(file_text);
        }
        Ok(form)
    } else {
        let Form(form) = Form::<StageComponentImportForm>::from_request(req, state)
            .await
            .map_err(|e| e.to_string())?;
        Ok(form)
    }
}

async fn parse_bom_stage_request(
    state: &AppState,
    headers: &HeaderMap,
    req: axum::extract::Request,
) -> Result<StageBomImportForm, String> {
    use axum::extract::FromRequest;
    let is_multipart = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|ct| ct.starts_with("multipart/form-data"));

    if is_multipart {
        let mut mp = Multipart::from_request(req, state)
            .await
            .map_err(|e| e.to_string())?;
        let mut form = StageBomImportForm::default();
        let mut file_text = String::new();
        while let Some(field) = mp.next_field().await.map_err(|e| e.to_string())? {
            let name = field.name().unwrap_or("").to_string();
            let val = field.text().await.map_err(|e| e.to_string())?;
            match name.as_str() {
                "csv_file" => {
                    if !val.trim().is_empty() {
                        file_text = val;
                    }
                }
                "csv_text" => form.csv_text = Some(val),
                "kit_build_id" => form.kit_build_id = Some(val),
                "import_mode" => form.import_mode = Some(val),
                "group_designators" => form.group_designators = Some(val),
                "preset_name" => form.preset_name = Some(val),
                other => {
                    let _ = form.cols.apply_multipart_field(other, val);
                }
            }
        }
        if !file_text.trim().is_empty() {
            form.csv_text = Some(file_text);
        }
        Ok(form)
    } else {
        let Form(form) = Form::<StageBomImportForm>::from_request(req, state)
            .await
            .map_err(|e| e.to_string())?;
        Ok(form)
    }
}

async fn load_and_render_imports_workspace(
    db: &Db,
    query: &ImportsQuery,
    flash_message: Option<&str>,
    flash_error: Option<&str>,
) -> Result<Markup, (StatusCode, String)> {
    let locations = list_storage_locations(db).await.map_err(internal_err)?;
    let builds = list_kit_builds(db).await.map_err(internal_err)?;
    let presets = list_import_presets(db, None).await.map_err(internal_err)?;

    let batch = if let Some(ref bid) = query.batch_id {
        get_import_batch(db, bid).await.map_err(internal_err)?
    } else {
        None
    };

    Ok(render_imports_workspace(
        query,
        &locations,
        &builds,
        &presets,
        batch.as_ref(),
        flash_message,
        flash_error,
    ))
}

async fn imports_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ImportsQuery>,
) -> Result<Html<String>, (StatusCode, String)> {
    let content = load_and_render_imports_workspace(&state.db, &query, None, None).await?;
    Ok(render_page_or_partial(&headers, SurfaceTab::Imports, content))
}

async fn stage_component_import_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    req: axum::extract::Request,
) -> Result<Html<String>, (StatusCode, String)> {
    let form = match parse_component_stage_request(&state, &headers, req).await {
        Ok(f) => f,
        Err(e) => {
            let content = load_and_render_imports_workspace(
                &state.db,
                &ImportsQuery::default(),
                None,
                Some(&e),
            )
            .await?;
            return Ok(render_page_or_partial(&headers, SurfaceTab::Imports, content));
        }
    };

    let mapping = form.cols.into_column_mapping();

    let staged = stage_component_import(
        &state.db,
        StageComponentImportInput {
            csv_text: form.csv_text.unwrap_or_default(),
            default_location_id: form.default_location_id,
            default_supplier: form.default_supplier,
            preset_name: form.preset_name,
            column_mapping: Some(mapping),
        },
    )
    .await;

    let (batch_id, flash_err) = match staged {
        Ok(b) => (Some(b.id), None),
        Err(e) => (None, Some(e.to_string())),
    };

    let query = ImportsQuery {
        batch_id,
        target_kind: Some("components".into()),
        kit_build_id: None,
    };
    let content =
        load_and_render_imports_workspace(&state.db, &query, None, flash_err.as_deref()).await?;
    Ok(render_page_or_partial(&headers, SurfaceTab::Imports, content))
}

async fn stage_bom_import_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    req: axum::extract::Request,
) -> Result<Html<String>, (StatusCode, String)> {
    let form = match parse_bom_stage_request(&state, &headers, req).await {
        Ok(f) => f,
        Err(e) => {
            let content = load_and_render_imports_workspace(
                &state.db,
                &ImportsQuery {
                    target_kind: Some("bom".into()),
                    ..Default::default()
                },
                None,
                Some(&e),
            )
            .await?;
            return Ok(render_page_or_partial(&headers, SurfaceTab::Imports, content));
        }
    };

    let kit_build_id = form.kit_build_id.unwrap_or_default();
    let import_mode = form
        .import_mode
        .as_deref()
        .map(BomImportMode::from_label)
        .unwrap_or(BomImportMode::Append);
    let group_designators = match form.group_designators.as_deref().map(str::trim) {
        Some("false") | Some("0") | Some("off") => false,
        Some(_) => true,
        None => false, // unchecked HTML checkbox
    };

    let mapping = form.cols.into_column_mapping();

    let staged = stage_bom_import(
        &state.db,
        StageBomImportInput {
            kit_build_id: kit_build_id.clone(),
            csv_text: form.csv_text.unwrap_or_default(),
            import_mode,
            group_designators,
            preset_name: form.preset_name,
            column_mapping: Some(mapping),
        },
    )
    .await;

    let (batch_id, flash_err) = match staged {
        Ok(b) => (Some(b.id), None),
        Err(e) => (None, Some(e.to_string())),
    };

    let query = ImportsQuery {
        batch_id,
        target_kind: Some("bom".into()),
        kit_build_id: Some(kit_build_id),
    };
    let content =
        load_and_render_imports_workspace(&state.db, &query, None, flash_err.as_deref()).await?;
    Ok(render_page_or_partial(&headers, SurfaceTab::Imports, content))
}

async fn save_import_preset_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<SaveImportPresetForm>,
) -> Result<Html<String>, (StatusCode, String)> {
    let target_kind = form.target_kind.unwrap_or_else(|| "components".into());
    let res = save_import_preset(
        &state.db,
        SaveImportPresetInput {
            name: form.name,
            target_kind: target_kind.clone(),
            mapping: form.cols.into_column_mapping(),
        },
    )
    .await;

    let (flash_msg, flash_err) = match res {
        Ok(p) => (Some(format!("Saved preset '{}'", p.name)), None),
        Err(e) => (None, Some(e.to_string())),
    };
    let query = ImportsQuery {
        target_kind: Some(target_kind),
        ..Default::default()
    };
    let content = load_and_render_imports_workspace(
        &state.db,
        &query,
        flash_msg.as_deref(),
        flash_err.as_deref(),
    )
    .await?;
    Ok(render_page_or_partial(&headers, SurfaceTab::Imports, content))
}

async fn resolve_import_row_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(row_id): Path<String>,
    Form(form): Form<ResolveImportRowForm>,
) -> Result<Html<String>, (StatusCode, String)> {
    let existing = get_import_row(&state.db, &row_id)
        .await
        .map_err(internal_err)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Import row '{row_id}' not found")))?;

    let action = ImportChosenAction::from_label(&form.chosen_action);
    let updated_res = update_import_row_resolution(
        &state.db,
        &row_id,
        action,
        form.matched_component_id.as_deref(),
    )
    .await;

    let hx_target = headers
        .get("HX-Target")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if let Ok(ref row) = updated_res
        && is_htmx_request(&headers)
        && hx_target == format!("import-row-{row_id}")
    {
        let batch = get_import_batch(&state.db, &row.batch_id)
            .await
            .map_err(internal_err)?
            .ok_or_else(|| (StatusCode::NOT_FOUND, "Batch not found".to_string()))?;
        return Ok(Html(
            render_staged_import_row(&batch.target_kind, &batch.status, row).into_string(),
        ));
    }

    let flash_err = updated_res.err().map(|e| e.to_string());
    let query = ImportsQuery {
        batch_id: Some(existing.batch_id),
        ..Default::default()
    };
    let content =
        load_and_render_imports_workspace(&state.db, &query, None, flash_err.as_deref()).await?;
    Ok(render_page_or_partial(&headers, SurfaceTab::Imports, content))
}

async fn commit_import_batch_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(batch_id): Path<String>,
) -> Result<Html<String>, (StatusCode, String)> {
    let batch = get_import_batch(&state.db, &batch_id)
        .await
        .map_err(internal_err)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Import batch '{batch_id}' not found")))?;

    let (flash_msg, flash_err) = if batch.target_kind == "bom" {
        match commit_bom_import_batch(&state.db, &batch_id).await {
            Ok(sum) => (
                Some(format!(
                    "Committed BOM Import: {} BOM Lines created ({} auto-linked, {} unallocated lines replaced).",
                    sum.bom_lines_created, sum.bom_lines_linked, sum.unallocated_lines_replaced
                )),
                None,
            ),
            Err(e) => (None, Some(e.to_string())),
        }
    } else {
        match commit_component_import_batch(&state.db, &batch_id).await {
            Ok(sum) => (
                Some(format!(
                    "Committed Component Import: {} new Components created, {} Stock Lots created/merged, {} new Storage Locations created.",
                    sum.components_created,
                    sum.stock_lots_merged_or_created,
                    sum.locations_created
                )),
                None,
            ),
            Err(e) => (None, Some(e.to_string())),
        }
    };

    let query = ImportsQuery {
        batch_id: Some(batch_id),
        target_kind: Some(batch.target_kind),
        kit_build_id: batch.kit_build_id,
    };
    let content = load_and_render_imports_workspace(
        &state.db,
        &query,
        flash_msg.as_deref(),
        flash_err.as_deref(),
    )
    .await?;
    Ok(render_page_or_partial(&headers, SurfaceTab::Imports, content))
}

fn render_match_status_badge(status: ImportMatchStatus) -> Markup {
    let cls = match status {
        ImportMatchStatus::ExactMatch => "badge ok",
        ImportMatchStatus::NearOrAmbiguousMatch => "badge warn",
        ImportMatchStatus::NoMatch => "badge info",
    };
    html! {
        span class=(cls) { (status.display_label()) }
    }
}

fn render_staged_import_row(target_kind: &str, batch_status: &str, row: &ImportRow) -> Markup {
    let is_bom = target_kind == "bom";
    let is_committed = batch_status == "committed";

    html! {
        tr id=(format!("import-row-{}", row.id)) {
            th scope="row" class="mono" {
                @if is_bom {
                    @if !row.reference_designators.is_empty() {
                        (row.reference_designators)
                    } @else {
                        (format!("#{}", row.row_index + 1))
                    }
                } @else {
                    (format!("#{}", row.row_index + 1))
                }
            }
            td {
                div {
                    strong class="mono" { (row.parsed_spec.canonical_value_display) }
                    @if let Some(badge) = row.parsed_spec.summary_badge() {
                        span class="badge neutral" style="margin-inline-start:0.35rem;" { (badge) }
                    }
                }
                div style="font-size:0.78rem; color:var(--text-muted);" {
                    "raw: " code { (row.raw_spec_text) }
                }
            }
            td {
                div { (row.category) }
                @if !row.package_or_mounting.is_empty() {
                    small style="color:var(--text-muted);" { (row.package_or_mounting) }
                }
            }
            td class="mono" {
                strong { (row.quantity) }
                @if is_bom && row.included_in_kit {
                    div { span class="badge neutral" { "Included in Kit" } }
                }
            }
            td {
                @if is_bom {
                    @if !row.notes.is_empty() {
                        span style="font-size:0.8rem;" { (row.notes) }
                    } @else {
                        span style="color:var(--text-muted); font-size:0.8rem;" { "—" }
                    }
                } @else {
                    @if !row.supplier.is_empty() || !row.supplier_part_no.is_empty() {
                        div style="font-size:0.8rem;" {
                            strong { (row.supplier) }
                            @if !row.supplier_part_no.is_empty() {
                                " · " code { (row.supplier_part_no) }
                            }
                        }
                    }
                    @if !row.location_path.is_empty() {
                        div style="font-size:0.78rem; color:var(--text-muted);" {
                            "Location: " code { (row.location_path) }
                        }
                    }
                }
            }
            td {
                (render_match_status_badge(row.match_status))
                @if let Some(ref mname) = row.matched_component_name {
                    div style="font-size:0.78rem; margin-block-start:0.2rem;" {
                        "Matched: " strong { (mname) }
                    }
                } @else if !row.candidates.is_empty() {
                    div style="font-size:0.76rem; color:var(--text-muted); margin-block-start:0.2rem;" {
                        (format!("{} candidate(s) found", row.candidates.len()))
                    }
                }
            }
            td {
                @if is_committed {
                    span class="badge ok" { (row.chosen_action.as_str()) }
                } @else {
                    form style="display:flex; gap:0.35rem; align-items:center; flex-wrap:wrap;"
                        method="post"
                        action=(format!("/imports/rows/{}/resolve", row.id))
                        hx-post=(format!("/imports/rows/{}/resolve", row.id))
                        hx-target=(format!("#import-row-{}", row.id))
                        hx-swap="outerHTML" {
                        select name="chosen_action" aria-label="Reconciliation Action" style="font-size:0.78rem;" {
                            @if is_bom {
                                option value="link_component" selected[row.chosen_action == ImportChosenAction::LinkComponent || row.chosen_action == ImportChosenAction::AddStockLot] {
                                    "Link Catalog Component"
                                }
                                option value="keep_unlinked" selected[row.chosen_action == ImportChosenAction::KeepUnlinked] {
                                    "Keep Unlinked"
                                }
                                option value="create_new" selected[row.chosen_action == ImportChosenAction::CreateNew] {
                                    "Create & Link New Component"
                                }
                                option value="skip" selected[row.chosen_action == ImportChosenAction::Skip] {
                                    "Skip Row"
                                }
                            } @else {
                                option value="add_stock_lot" selected[row.chosen_action == ImportChosenAction::AddStockLot] {
                                    "Add / Increment Stock Lot"
                                }
                                option value="create_new" selected[row.chosen_action == ImportChosenAction::CreateNew] {
                                    "Create New Component + Stock Lot"
                                }
                                option value="skip" selected[row.chosen_action == ImportChosenAction::Skip] {
                                    "Skip Row"
                                }
                            }
                        }
                        @if !row.candidates.is_empty() {
                            select name="matched_component_id" aria-label="Candidate Component" style="font-size:0.78rem; max-inline-size:210px;" {
                                option value="" { "— Select Candidate —" }
                                @for cand in &row.candidates {
                                    option value=(cand.component_id)
                                        selected[row.matched_component_id.as_deref() == Some(cand.component_id.as_str())] {
                                        (format!("{} ({} Avail)", cand.name, cand.available_stock))
                                    }
                                }
                            }
                        } @else if let Some(ref cid) = row.matched_component_id {
                            input type="hidden" name="matched_component_id" value=(cid);
                        }
                        button type="submit" class="xs" { "Apply" }
                    }
                }
            }
        }
    }
}

fn render_imports_workspace(
    query: &ImportsQuery,
    locations: &[StorageLocation],
    builds: &[KitBuild],
    presets: &[ImportPreset],
    batch: Option<&ImportBatchDetails>,
    flash_message: Option<&str>,
    flash_error: Option<&str>,
) -> Markup {
    let exact_count = batch
        .map(|b| {
            b.rows
                .iter()
                .filter(|r| r.match_status == ImportMatchStatus::ExactMatch)
                .count()
        })
        .unwrap_or(0);
    let near_count = batch
        .map(|b| {
            b.rows
                .iter()
                .filter(|r| r.match_status == ImportMatchStatus::NearOrAmbiguousMatch)
                .count()
        })
        .unwrap_or(0);
    let no_match_count = batch
        .map(|b| {
            b.rows
                .iter()
                .filter(|r| r.match_status == ImportMatchStatus::NoMatch)
                .count()
        })
        .unwrap_or(0);

    html! {
        div id="imports-workspace" {
            @if let Some(err) = flash_error {
                div class="card" style="border-color:var(--err-text); background:var(--err-bg); color:var(--err-text); margin-block-end:0.75rem;" role="alert" {
                    strong { "Import Error: " } (err)
                }
            }
            @if let Some(msg) = flash_message {
                div class="card" style="border-color:var(--ok-text); background:var(--ok-bg); color:var(--ok-text); margin-block-end:0.75rem;" role="status" {
                    strong { (msg) }
                }
            }

            @if let Some(b) = batch {
                div class="card" style="margin-block-end:1rem;" {
                    div style="display:flex; justify-content:space-between; align-items:center; flex-wrap:wrap; gap:0.75rem; margin-block-end:0.75rem;" {
                        div {
                            h2 style="margin:0; font-size:1.05rem;" {
                                @if b.target_kind == "bom" {
                                    "Staged Kit Build BOM Import Preview"
                                } @else {
                                    "Staged Component & Stock Lot Import Preview"
                                }
                            }
                            div style="display:flex; gap:0.5rem; align-items:center; flex-wrap:wrap; margin-block-start:0.35rem;" {
                                span class="badge neutral" { (format!("Batch Status: {}", b.status)) }
                                span class="badge ok" { (format!("{exact_count} Exact Match")) }
                                span class="badge warn" { (format!("{near_count} Near / Ambiguous Match")) }
                                span class="badge info" { (format!("{no_match_count} No Match")) }
                            }
                        }
                        div style="display:flex; gap:0.5rem; align-items:center;" {
                            @if b.status != "committed" {
                                form method="post"
                                    action=(format!("/imports/batches/{}/commit", b.id))
                                    hx-post=(format!("/imports/batches/{}/commit", b.id))
                                    hx-target="#imports-workspace"
                                    hx-swap="outerHTML" {
                                    button type="submit" class="primary" {
                                        "Commit Reconciled Batch Atomically"
                                    }
                                }
                            } @else {
                                span class="badge ok" { "Committed" }
                                @if b.target_kind == "bom" {
                                    @if let Some(ref kbid) = b.kit_build_id {
                                        a class="btn xs" href=(format!("/builds/{kbid}")) { "View Kit Build BOM ▸" }
                                    }
                                } @else {
                                    a class="btn xs" href="/catalog" { "View Component Catalog ▸" }
                                }
                            }
                        }
                    }

                    div tabindex="0" role="region" aria-label="Staged Import Reconciliation Table" class="table-wrapper" {
                        table {
                            caption { (format!("Staged Import Rows ({} rows)", b.rows.len())) }
                            thead {
                                tr {
                                    th scope="col" {
                                        @if b.target_kind == "bom" { "Designators" } @else { "Row" }
                                    }
                                    th scope="col" { "Parsed & Verbatim Spec" }
                                    th scope="col" { "Category & Package" }
                                    th scope="col" { "Qty" }
                                    th scope="col" {
                                        @if b.target_kind == "bom" { "Notes" } @else { "Supplier & Location" }
                                    }
                                    th scope="col" { "Catalog Match Status" }
                                    th scope="col" { "Reconciliation Action" }
                                }
                            }
                            tbody {
                                @for row in &b.rows {
                                    (render_staged_import_row(&b.target_kind, &b.status, row))
                                }
                            }
                        }
                    }
                }
            }

            div style="display:grid; grid-template-columns:repeat(auto-fit, minmax(380px, 1fr)); gap:1rem; align-items:start;" {
                // 1. Component & Stock Lot CSV / Paste Import Card
                section class="card" aria-labelledby="comp-import-heading" {
                    h2 id="comp-import-heading" style="margin-block-start:0; font-size:1.05rem;" {
                        "1. Component & Stock Lot Import"
                    }
                    p style="color:var(--text-muted); font-size:0.82rem; margin-block-start:0;" {
                        "Upload a supplier CSV or paste tabular rows. Auto-detects Mouser, DigiKey, Tayda, and LCSC headers and previews Exact, Near / Ambiguous, and No Match rows before committing."
                    }
                    form class="form-grid"
                        method="post"
                        action="/imports/components/stage"
                        enctype="multipart/form-data"
                        hx-post="/imports/components/stage"
                        hx-target="#imports-workspace"
                        hx-swap="outerHTML" {
                        div style="display:flex; gap:0.5rem; flex-wrap:wrap;" {
                            div class="form-field" style="flex:1;" {
                                label for="comp-preset-select" { "Supplier / Mapping Preset" }
                                select id="comp-preset-select" name="preset_name" {
                                    option value="" { "Auto-Detect from Headers (Mouser / DigiKey / Tayda / LCSC)" }
                                    @for p in presets {
                                        option value=(p.name) {
                                            @if p.is_builtin {
                                                (format!("{} (Built-in Supplier Preset)", p.name))
                                            } @else {
                                                (format!("{} (Saved Custom Preset)", p.name))
                                            }
                                        }
                                    }
                                }
                            }
                            div class="form-field" style="flex:1;" {
                                label for="comp-default-location" { "Default Target Storage Location" }
                                select id="comp-default-location" name="default_location_id" {
                                    option value="" { "— Unassigned / From Row Location Column —" }
                                    @for loc in locations {
                                        option value=(loc.id) { (loc.breadcrumb_path) }
                                    }
                                }
                            }
                        }
                        div style="display:flex; gap:0.5rem; flex-wrap:wrap;" {
                            div class="form-field" style="flex:1;" {
                                label for="comp-default-supplier" { "Default Supplier (Optional)" }
                                input id="comp-default-supplier" type="text" name="default_supplier" placeholder="e.g. Mouser, DigiKey, Tayda, LCSC";
                            }
                            div class="form-field" style="flex:1;" {
                                label for="comp-csv-file" { "Upload CSV File (Optional)" }
                                input id="comp-csv-file" type="file" name="csv_file" accept=".csv,.tsv,.txt";
                            }
                        }
                        div class="form-field" {
                            label for="comp-csv-text" { "Or Paste CSV / Tabular Data" }
                            textarea id="comp-csv-text" name="csv_text" rows="6"
                                placeholder="Supplier,Supplier Part #,Spec,Category,Package,Qty,Location\nTayda,A-2214,4k7 1% 1/4W Metal Film,Resistors,Through-Hole,50,Binder A > Resistors" {}
                        }
                        details {
                            summary style="cursor:pointer; font-size:0.8rem; color:var(--text-muted);" {
                                "Optional Custom Column Header Overrides"
                            }
                            div style="display:grid; grid-template-columns:1fr 1fr; gap:0.4rem; margin-block-start:0.4rem;" {
                                div class="form-field" {
                                    label { "Spec / Value Column" }
                                    input type="text" name="col_raw_spec_text" placeholder="e.g. Description or Value";
                                }
                                div class="form-field" {
                                    label { "Category Column" }
                                    input type="text" name="col_category" placeholder="e.g. Category";
                                }
                                div class="form-field" {
                                    label { "Package Column" }
                                    input type="text" name="col_package_or_mounting" placeholder="e.g. Package";
                                }
                                div class="form-field" {
                                    label { "Quantity Column" }
                                    input type="text" name="col_quantity" placeholder="e.g. Qty";
                                }
                                div class="form-field" {
                                    label { "Location Path Column" }
                                    input type="text" name="col_location_path" placeholder="e.g. Location";
                                }
                                div class="form-field" {
                                    label { "Supplier Part # Column" }
                                    input type="text" name="col_supplier_part_no" placeholder="e.g. SKU";
                                }
                            }
                        }
                        button type="submit" class="primary" {
                            "Stage Component Import & Preview Matches"
                        }
                    }
                }

                // 2. Kit Build BOM CSV / Paste Import Card
                section class="card" aria-labelledby="bom-import-heading" {
                    h2 id="bom-import-heading" style="margin-block-start:0; font-size:1.05rem;" {
                        "2. Kit Build BOM Import"
                    }
                    p style="color:var(--text-muted); font-size:0.82rem; margin-block-start:0;" {
                        "Import a KiCad, Eagle, or pedal build-doc BOM into a Kit Build. Infers category from reference designator prefixes (R, C, D/Q, U/IC, RV/VR, SW/J, L/T) and groups identical-spec rows."
                    }
                    form class="form-grid"
                        method="post"
                        action="/imports/bom/stage"
                        enctype="multipart/form-data"
                        hx-post="/imports/bom/stage"
                        hx-target="#imports-workspace"
                        hx-swap="outerHTML" {
                        div style="display:flex; gap:0.5rem; flex-wrap:wrap;" {
                            div class="form-field" style="flex:1;" {
                                label for="bom-kit-select" { "Target Kit Build" }
                                select id="bom-kit-select" name="kit_build_id" required {
                                    @for kb in builds {
                                        option value=(kb.id)
                                            selected[query.kit_build_id.as_deref() == Some(kb.id.as_str())] {
                                            (format!("{} ({})", kb.name, kb.status))
                                        }
                                    }
                                }
                            }
                            div class="form-field" style="flex:1;" {
                                label for="bom-import-mode" { "Import Mode" }
                                select id="bom-import-mode" name="import_mode" {
                                    option value="append" { "Append to Existing BOM" }
                                    option value="replace_unallocated" { "Replace Unallocated Lines" }
                                }
                            }
                        }
                        div style="display:flex; gap:0.75rem; align-items:center; flex-wrap:wrap;" {
                            label style="display:flex; align-items:center; gap:0.4rem; font-size:0.85rem; cursor:pointer;" {
                                input type="checkbox" name="group_designators" value="true" checked;
                                strong { "Automatic Designator Grouping" }
                                span style="color:var(--text-muted); font-size:0.78rem;" {
                                    "(combines identical Category + Spec + Package rows e.g. R1, R2, R5)"
                                }
                            }
                        }
                        div class="form-field" {
                            label for="bom-csv-file" { "Upload BOM CSV File (Optional)" }
                            input id="bom-csv-file" type="file" name="csv_file" accept=".csv,.tsv,.txt";
                        }
                        div class="form-field" {
                            label for="bom-csv-text" { "Or Paste BOM CSV / Tabular Data" }
                            textarea id="bom-csv-text" name="csv_text" rows="6"
                                placeholder="Reference,Value,Footprint,Qty,Notes\nR1,4k7,Through-Hole,1,\nR2,4.7k,Through-Hole,1,\nC1,0.047uF,Through-Hole 5mm,1,Box Film" {}
                        }
                        button type="submit" class="primary" {
                            "Stage BOM Import & Preview Auto-Matching"
                        }
                    }
                }
            }
        }
    }
}

fn internal_err(e: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

async fn settings_page(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Html<String>, (StatusCode, String)> {
    let content = render_settings_workspace(&state.db, None, None)
        .await
        .map_err(internal_err)?;
    Ok(render_page_or_partial(&headers, SurfaceTab::Catalog, content))
}

async fn download_backup_zip_handler(
    State(state): State<AppState>,
) -> Result<Response, (StatusCode, String)> {
    let zip_bytes = crate::backup::create_backup_zip(&state.db)
        .await
        .map_err(internal_err)?;
    Ok((
        [
            (header::CONTENT_TYPE, "application/zip".to_string()),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"audio-inventory-backup.zip\"".to_string(),
            ),
        ],
        Bytes::from(zip_bytes),
    )
        .into_response())
}

async fn restore_backup_zip_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<Response, (StatusCode, String)> {
    let mut zip_bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid multipart upload: {e}")))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" || name == "backup_zip" || field.file_name().is_some() {
            let bytes = field
                .bytes()
                .await
                .map_err(|e| (StatusCode::BAD_REQUEST, format!("Failed to read upload bytes: {e}")))?;
            if !bytes.is_empty() {
                zip_bytes = Some(bytes.to_vec());
            }
        }
    }

    let Some(zip_bytes) = zip_bytes else {
        let err_msg = "Please select a .zip backup archive file to restore.";
        let content = render_settings_workspace(&state.db, None, Some(err_msg))
            .await
            .map_err(internal_err)?;
        let html = render_page_or_partial(&headers, SurfaceTab::Catalog, content);
        return Ok((StatusCode::BAD_REQUEST, html).into_response());
    };

    match crate::backup::restore_backup_zip_into_db(&state.db, &zip_bytes).await {
        Ok(summary) => {
            let ok_msg = format!(
                "Backup Restored: inventory.db ({} bytes) and {} attachment file(s) restored into {}.",
                summary.database_bytes,
                summary.attachment_files_restored,
                state.db.data_dir().display()
            );
            let content = render_settings_workspace(&state.db, Some(&ok_msg), None)
                .await
                .map_err(internal_err)?;
            let html = render_page_or_partial(&headers, SurfaceTab::Catalog, content);
            Ok((StatusCode::OK, html).into_response())
        }
        Err(err) => {
            let err_msg = format!("Restore failed: {err}");
            let content = render_settings_workspace(&state.db, None, Some(&err_msg))
                .await
                .map_err(internal_err)?;
            let html = render_page_or_partial(&headers, SurfaceTab::Catalog, content);
            Ok((StatusCode::BAD_REQUEST, html).into_response())
        }
    }
}

async fn render_settings_workspace(
    db: &Db,
    flash_ok: Option<&str>,
    flash_error: Option<&str>,
) -> anyhow::Result<Markup> {
    let locations = list_storage_locations(db).await?;
    let components = list_components(db).await?;
    let builds = list_kit_builds(db).await?;
    let total_lots: usize = components.iter().map(|c| c.lots.len()).sum();

    Ok(html! {
        div id="settings-workspace" {
            @if let Some(ok) = flash_ok {
                div class="badge ok" role="status" style="display:block; padding:0.65rem 0.9rem; margin-block-end:0.75rem; font-size:0.85rem;" {
                    (ok)
                }
            }
            @if let Some(err) = flash_error {
                div class="badge err" role="alert" style="display:block; padding:0.65rem 0.9rem; margin-block-end:0.75rem; font-size:0.85rem;" {
                    (err)
                }
            }

            div class="va-filter-ribbon" {
                div style="display:flex; justify-content:space-between; align-items:center; gap:0.75rem; flex-wrap:wrap; width:100%;" {
                    div {
                        strong style="font-size:1rem;" { "Unified Data Directory Backup & Restore" }
                        div style="font-size:0.82rem; color:var(--text-muted);" {
                            "Active Data Directory: "
                            code class="mono" { (db.data_dir().display().to_string()) }
                            " · "
                            (format!(
                                "{} Components · {} Stock Lots · {} Storage Locations · {} Kit Builds",
                                components.len(),
                                total_lots,
                                locations.len(),
                                builds.len()
                            ))
                        }
                    }
                    div style="display:flex; gap:0.5rem; align-items:center;" {
                        a class="btn primary" href="/settings/backup.zip" {
                            "Download Backup (.zip)"
                        }
                    }
                }
            }

            div style="display:grid; grid-template-columns:repeat(auto-fit, minmax(360px, 1fr)); gap:1rem; align-items:start;" {
                section class="card" aria-labelledby="backup-export-heading" {
                    h2 id="backup-export-heading" style="margin-block-start:0; font-size:1.05rem;" {
                        "1. Download Consistent Snapshot (.zip)"
                    }
                    p style="color:var(--text-muted); font-size:0.85rem;" {
                        "Exports a consistent snapshot of the SQLite database ("
                        code class="mono" { "inventory.db" }
                        ") together with all uploaded Kit Build schematics, build guides, and bench photos under "
                        code class="mono" { "attachments/<kit_build_id>/" }
                        "."
                    }
                    a class="btn primary" href="/settings/backup.zip" {
                        "Download Backup (.zip)"
                    }
                }

                section class="card" aria-labelledby="backup-restore-heading" {
                    h2 id="backup-restore-heading" style="margin-block-start:0; font-size:1.05rem;" {
                        "2. Restore from Backup (.zip)"
                    }
                    p style="color:var(--text-muted); font-size:0.85rem;" {
                        "Validates the archive structure and restores both "
                        code class="mono" { "inventory.db" }
                        " and "
                        code class="mono" { "attachments/" }
                        " into the configured data directory ("
                        code class="mono" { "--data-dir" }
                        " / "
                        code class="mono" { "AUDIO_INVENTORY_DATA_DIR" }
                        ")."
                    }
                    form class="form-grid"
                        method="post"
                        action="/settings/restore"
                        enctype="multipart/form-data"
                        hx-post="/settings/restore"
                        hx-encoding="multipart/form-data"
                        hx-target="#settings-workspace"
                        hx-swap="outerHTML" {
                        div class="form-field" {
                            label for="restore-zip-file" { "Select Backup Archive (.zip)" }
                            input id="restore-zip-file" type="file" name="file" accept=".zip,application/zip" required;
                        }
                        button type="submit" class="primary" {
                            "Validate & Restore Backup (.zip)"
                        }
                    }
                    div style="margin-block-start:0.85rem; font-size:0.78rem; color:var(--text-muted);" {
                        "CLI Restore Command: "
                        code class="mono" { "audio_inventory --data-dir ./data --restore audio-inventory-backup.zip" }
                    }
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_memory_db;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn body_string(body: Body) -> String {
        let bytes = body.collect().await.expect("collect body").to_bytes();
        String::from_utf8(bytes.to_vec()).expect("utf8 body")
    }

    #[tokio::test]
    async fn serves_workbench_shell_vendored_assets_htmx_partials_and_storage_location_moves() {
        let db = init_memory_db().await.expect("memory db");
        let app = app_router(db.clone());

        // 1. Full page request renders DOCTYPE, color-scheme meta, vendored asset links, and all 5 surface tabs
        let res = app
            .clone()
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let html = body_string(res.into_body()).await;
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains(r#"name="color-scheme" content="light dark""#));
        assert!(html.contains("/assets/htmx.min.js"));
        assert!(html.contains("/assets/styles.css"));
        assert!(html.contains("Component Catalog"));
        assert!(html.contains("Kit Build BOM"));
        assert!(html.contains("Active Bench Checklist"));
        assert!(html.contains("Shopping List"));
        assert!(html.contains("CSV Import"));

        // 2. HX-Request returns partial fragment without <!DOCTYPE html>
        let hx_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/catalog")
                    .header("HX-Request", "true")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(hx_res.status(), StatusCode::OK);
        let hx_html = body_string(hx_res.into_body()).await;
        assert!(!hx_html.contains("<!DOCTYPE html>"));
        assert!(hx_html.contains("Storage Locations"));

        // 3. Vendored assets are served with 200 OK
        let css_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/assets/styles.css")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(css_res.status(), StatusCode::OK);
        let css_text = body_string(css_res.into_body()).await;
        assert!(css_text.contains("color-scheme: light dark"));

        let js_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/assets/htmx.min.js")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(js_res.status(), StatusCode::OK);
        let js_text = body_string(js_res.into_body()).await;
        assert!(js_text.contains("htmx"));

        // 4. Create Storage Locations via POST /locations and move parent via POST /locations/{id}/move
        let c1 = crate::catalog::create_storage_location(
            &db,
            crate::catalog::CreateStorageLocation {
                name: "Crate 1".into(),
                kind: "Crate".into(),
                parent_id: None,
                notes: "".into(),
            },
        )
        .await
        .unwrap();
        let c2 = crate::catalog::create_storage_location(
            &db,
            crate::catalog::CreateStorageLocation {
                name: "Crate 2".into(),
                kind: "Crate".into(),
                parent_id: None,
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        // Create Passives Bag inside Crate 1 via HTTP POST
        let create_bag_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/locations")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .header("HX-Request", "true")
                    .body(Body::from(format!(
                        "name=Passives+Bag&kind=Bag&parent_id={}&notes=Resistor+and+cap+bags",
                        c1.id
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(create_bag_res.status(), StatusCode::OK);
        let bag_html = body_string(create_bag_res.into_body()).await;
        assert!(
            bag_html.contains("Crate 1 &gt; Passives Bag")
                || bag_html.contains("Crate 1 > Passives Bag")
        );

        let locations = crate::catalog::list_storage_locations(&db).await.unwrap();
        let bag = locations.iter().find(|l| l.name == "Passives Bag").unwrap();

        // Create Film Caps Sub-bag inside Passives Bag
        let sub_bag = crate::catalog::create_storage_location(
            &db,
            crate::catalog::CreateStorageLocation {
                name: "Film Caps Sub-bag".into(),
                kind: "Bag".into(),
                parent_id: Some(bag.id.clone()),
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        // Inspect Film Caps Sub-bag via GET /locations/{id}/inspect
        let inspect_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/locations/{}/inspect", sub_bag.id))
                    .header("HX-Request", "true")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(inspect_res.status(), StatusCode::OK);
        let inspect_html = body_string(inspect_res.into_body()).await;
        assert!(inspect_html.contains("Film Caps Sub-bag"));

        // Move Passives Bag to Crate 2 via POST /locations/{id}/move and verify nested breadcrumb updates
        let move_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/locations/{}/move", bag.id))
                    .header("content-type", "application/x-www-form-urlencoded")
                    .header("HX-Request", "true")
                    .body(Body::from(format!("parent_id={}", c2.id)))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(move_res.status(), StatusCode::OK);
        let move_html = body_string(move_res.into_body()).await;
        assert!(
            move_html.contains("Crate 2 &gt; Passives Bag &gt; Film Caps Sub-bag")
                || move_html.contains("Crate 2 > Passives Bag > Film Caps Sub-bag")
        );
    }

    #[tokio::test]
    async fn catalog_web_routes_support_component_creation_parametric_search_inline_stepper_inspector_and_lot_merging() {
        let db = init_memory_db().await.expect("memory db");
        let app = app_router(db.clone());

        let binder = crate::catalog::create_storage_location(
            &db,
            crate::catalog::CreateStorageLocation {
                name: "Binder A".into(),
                kind: "Binder".into(),
                parent_id: None,
                notes: "".into(),
            },
        )
        .await
        .unwrap();
        let crate_b = crate::catalog::create_storage_location(
            &db,
            crate::catalog::CreateStorageLocation {
                name: "Crate B".into(),
                kind: "Crate".into(),
                parent_id: None,
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        // 1. Create a Resistor component with initial stock lot via POST /catalog/components
        let create_comp_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/catalog/components")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .header("HX-Request", "true")
                    .body(Body::from(format!(
                        "name=4.7k+Metal+Film+Resistor&category=Resistors&subtype=Fixed&package_or_mounting=Through-Hole&raw_spec_text=4k7+1%25+1%2F4W+Metal+Film&low_stock_threshold=20&initial_location_id={}&initial_quantity=42&initial_supplier=Tayda&initial_supplier_part_no=A-2214",
                        binder.id
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(create_comp_res.status(), StatusCode::OK);
        let create_comp_html = body_string(create_comp_res.into_body()).await;
        assert!(create_comp_html.contains("4.7kΩ"));
        assert!(create_comp_html.contains("4k7 1% 1/4W Metal Film"));
        assert!(create_comp_html.contains("Binder A"));
        assert!(create_comp_html.contains("42 Avail"));

        let comps = crate::catalog::list_components(&db).await.unwrap();
        assert_eq!(comps.len(), 1);
        let comp = &comps[0];
        let lot_id = &comp.lots[0].id;

        // 2. Single-lot inline stepper: POST /catalog/lots/{lot_id}/delta adjusts quantity and returns updated #comp-row-{id}
        let delta_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/catalog/lots/{lot_id}/delta"))
                    .header("content-type", "application/x-www-form-urlencoded")
                    .header("HX-Request", "true")
                    .body(Body::from("delta=-1"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(delta_res.status(), StatusCode::OK);
        let delta_html = body_string(delta_res.into_body()).await;
        assert!(delta_html.contains(&format!("id=\"comp-row-{}\"", comp.id)));
        assert!(delta_html.contains("41 Avail"));

        // 3. Debounced shorthand search & category dynamic filters: GET /catalog?q=4700R&category=Resistors&spec_tolerance=1%25
        let search_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/catalog?q=4700R&category=Resistors&spec_tolerance=1%25")
                    .header("HX-Request", "true")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(search_res.status(), StatusCode::OK);
        let search_html = body_string(search_res.into_body()).await;
        assert!(search_html.contains("spec_parser: 4.7kΩ"));
        assert!(search_html.contains("4.7k Metal Film Resistor"));
        assert!(search_html.contains("spec_tolerance"));
        assert!(search_html.contains("spec_composition"));

        // 4. Add a second Stock Lot in Crate B via POST /catalog/components/{id}/lots
        let add_lot_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/catalog/components/{}/lots", comp.id))
                    .header("content-type", "application/x-www-form-urlencoded")
                    .header("HX-Request", "true")
                    .body(Body::from(format!(
                        "location_id={}&quantity=15&supplier=Mouser&supplier_part_no=271-4.7K-RC&lot_notes=Surplus",
                        crate_b.id
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(add_lot_res.status(), StatusCode::OK);
        let add_lot_html = body_string(add_lot_res.into_body()).await;
        assert!(add_lot_html.contains("2 Stock Lots (56)"));

        // 5. Inspect multi-lot component in right-hand Inspector Drawer via GET /catalog/components/{id}/inspector
        let inspector_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/catalog/components/{}/inspector", comp.id))
                    .header("HX-Request", "true")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(inspector_res.status(), StatusCode::OK);
        let inspector_html = body_string(inspector_res.into_body()).await;
        assert!(inspector_html.contains("Merge Duplicate Stock Lots"));
        assert!(inspector_html.contains("Binder A"));
        assert!(inspector_html.contains("Crate B"));

        // 6. Merge duplicate lots via POST /catalog/components/{id}/merge-lots
        let merge_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/catalog/components/{}/merge-lots", comp.id))
                    .header("content-type", "application/x-www-form-urlencoded")
                    .header("HX-Request", "true")
                    .body(Body::from(format!("target_location_id={}", binder.id)))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(merge_res.status(), StatusCode::OK);
        let merge_html = body_string(merge_res.into_body()).await;
        assert!(merge_html.contains("56 Avail"));
        let after_merge = crate::catalog::get_component(&db, &comp.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(after_merge.lots.len(), 1);
        assert_eq!(after_merge.total_on_hand, 56);
    }

    #[tokio::test]
    async fn builds_web_routes_render_bom_readiness_hx_swap_oob_row_transitions_inspector_candidates_and_batch_kitting()
     {
        let db = init_memory_db().await.expect("memory db");
        let app = app_router(db.clone());

        let box_loc = crate::catalog::create_storage_location(
            &db,
            crate::catalog::CreateStorageLocation {
                name: "Project Box #4".into(),
                kind: "Box".into(),
                parent_id: None,
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        // Create two 47nF film capacitors so "0.047uF Film" is ambiguous (2 candidates)
        let cap1 = crate::catalog::create_component(
            &db,
            crate::catalog::CreateComponent {
                name: "47nF 63V WIMA Film Cap".into(),
                category: "Capacitors".into(),
                subtype: "Film".into(),
                package_or_mounting: "Through-Hole 5mm".into(),
                raw_spec_text: "0.047uF 63V Box Film".into(),
                canonical_value_display: None,
                resistance_ohms: None,
                capacitance_pf: None,
                inductance_uh: None,
                voltage_rating_volts: None,
                low_stock_threshold: 2,
                datasheet_url: "".into(),
                notes: "".into(),
                specs_json: None,
                initial_lot: Some(crate::catalog::CreateInitialStockLot {
                    location_id: Some(box_loc.id.clone()),
                    quantity: 10,
                    supplier: "Mouser".into(),
                    supplier_part_no: "WIMA-47N".into(),
                    lot_notes: "".into(),
                }),
            },
        )
        .await
        .unwrap();

        let _cap2 = crate::catalog::create_component(
            &db,
            crate::catalog::CreateComponent {
                name: "47nF 100V KemEt Film Cap".into(),
                category: "Capacitors".into(),
                subtype: "Film".into(),
                package_or_mounting: "Through-Hole 5mm".into(),
                raw_spec_text: "47n 100V Box Film".into(),
                canonical_value_display: None,
                resistance_ohms: None,
                capacitance_pf: None,
                inductance_uh: None,
                voltage_rating_volts: None,
                low_stock_threshold: 2,
                datasheet_url: "".into(),
                notes: "".into(),
                specs_json: None,
                initial_lot: Some(crate::catalog::CreateInitialStockLot {
                    location_id: Some(box_loc.id.clone()),
                    quantity: 5,
                    supplier: "DigiKey".into(),
                    supplier_part_no: "KEM-47N".into(),
                    lot_notes: "".into(),
                }),
            },
        )
        .await
        .unwrap();

        // 1. Create a Kit Build via POST /builds
        let create_build_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/builds")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .header("HX-Request", "true")
                    .body(Body::from(format!(
                        "name=Klon+Centaur&project_type=Overdrive&status=Sourcing+%2F+Waiting+on+Parts&location_id={}&description=Transparent+overdrive",
                        box_loc.id
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(create_build_res.status(), StatusCode::OK);
        let build_html = body_string(create_build_res.into_body()).await;
        assert!(build_html.contains("Klon Centaur"));
        assert!(build_html.contains("Project Box #4"));
        assert!(build_html.contains("0/0 Lines Ready · 0 Shortfalls"));

        let builds = crate::builds::list_kit_builds(&db).await.unwrap();
        assert_eq!(builds.len(), 1);
        let build_id = &builds[0].id;

        // 2. Add an ambiguous BOM Line ("0.047uF Film", C1, C2) via POST /builds/{id}/bom-lines
        let add_line_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/builds/{build_id}/bom-lines"))
                    .header("content-type", "application/x-www-form-urlencoded")
                    .header("HX-Request", "true")
                    .body(Body::from(
                        "reference_designators=C1%2C+C2&raw_spec_text=0.047uF+Film&quantity_required=2",
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(add_line_res.status(), StatusCode::OK);
        let add_line_html = body_string(add_line_res.into_body()).await;
        assert!(add_line_html.contains("0/1 Lines Ready · 1 Shortfalls"));
        assert!(add_line_html.contains("Unlinked"));

        let lines = crate::builds::list_bom_lines_for_build(&db, build_id)
            .await
            .unwrap();
        assert_eq!(lines.len(), 1);
        let line_id = &lines[0].id;

        // 3. Inspect BOM Line in right-hand Inspector Drawer via GET /builds/bom-lines/{line_id}/inspector
        let inspect_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/builds/bom-lines/{line_id}/inspector"))
                    .header("HX-Request", "true")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(inspect_res.status(), StatusCode::OK);
        let inspect_html = body_string(inspect_res.into_body()).await;
        assert!(inspect_html.contains("47nF 63V WIMA Film Cap"));
        assert!(inspect_html.contains("47nF 100V KemEt Film Cap"));
        assert!(inspect_html.contains("Create Component from BOM Line"));

        // 4. Link candidate cap1 via POST /builds/bom-lines/{line_id}/link-component
        let link_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/builds/bom-lines/{line_id}/link-component"))
                    .header("content-type", "application/x-www-form-urlencoded")
                    .header("HX-Request", "true")
                    .body(Body::from(format!("component_id={}", cap1.id)))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(link_res.status(), StatusCode::OK);
        let link_html = body_string(link_res.into_body()).await;
        assert!(link_html.contains("47nF 63V WIMA Film Cap"));
        assert!(link_html.contains(&format!("hx-patch=\"/builds/bom-lines/{line_id}/status\"")));

        // 5. Row-level status transition via PATCH /builds/bom-lines/{line_id}/status returns updated <tr> AND hx-swap-oob readiness counter
        let status_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/builds/bom-lines/{line_id}/status"))
                    .header("content-type", "application/x-www-form-urlencoded")
                    .header("HX-Request", "true")
                    .header("HX-Target", format!("bom-row-{line_id}"))
                    .body(Body::from("status=Reserved"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(status_res.status(), StatusCode::OK);
        let status_html = body_string(status_res.into_body()).await;
        assert!(status_html.contains(&format!("id=\"bom-row-{line_id}\"")));
        assert!(status_html.contains("id=\"build-readiness-summary\""));
        assert!(status_html.contains("hx-swap-oob=\"true\""));
        assert!(status_html.contains("1/1 Lines Ready · 0 Shortfalls"));

        // 6. Batch Pick All Reserved to Project Box via POST /builds/{id}/pick-all
        let pick_all_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/builds/{build_id}/pick-all"))
                    .header("HX-Request", "true")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(pick_all_res.status(), StatusCode::OK);
        let pick_all_html = body_string(pick_all_res.into_body()).await;
        assert!(pick_all_html.contains("Picked"));
        assert_eq!(
            crate::catalog::get_component(&db, &cap1.id)
                .await
                .unwrap()
                .unwrap()
                .total_on_hand,
            8
        );

        // 7. Add an unlinked BOM Line with 0 candidates ("500kA 16mm PCB") and trigger 1-click "Create Component from BOM Line"
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/builds/{build_id}/bom-lines"))
                    .header("content-type", "application/x-www-form-urlencoded")
                    .header("HX-Request", "true")
                    .body(Body::from(
                        "reference_designators=VR1&raw_spec_text=500kA+16mm+PCB&quantity_required=1",
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        let all_lines = crate::builds::list_bom_lines_for_build(&db, build_id)
            .await
            .unwrap();
        let vr1_line = all_lines
            .iter()
            .find(|l| l.reference_designators == "VR1")
            .unwrap();

        let create_from_bom_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/builds/bom-lines/{}/create-component",
                        vr1_line.id
                    ))
                    .header("content-type", "application/x-www-form-urlencoded")
                    .header("HX-Request", "true")
                    .body(Body::from(format!(
                        "initial_location_id={}&initial_quantity=3",
                        box_loc.id
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(create_from_bom_res.status(), StatusCode::OK);
        let create_from_bom_html = body_string(create_from_bom_res.into_body()).await;
        assert!(create_from_bom_html.contains("500kΩ"));
    }

    #[tokio::test]
    async fn builds_web_routes_support_substitution_modal_partial_allocation_line_splitting_and_kit_cloning()
     {
        let db = init_memory_db().await.expect("memory db");
        let app = app_router(db.clone());

        let drawer = crate::catalog::create_storage_location(
            &db,
            crate::catalog::CreateStorageLocation {
                name: "Drawer 1".into(),
                kind: "Drawer".into(),
                parent_id: None,
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        let j201 = crate::catalog::create_component(
            &db,
            crate::catalog::CreateComponent {
                name: "J201 N-Channel JFET".into(),
                category: "Semiconductors — Discrete".into(),
                subtype: "JFET".into(),
                package_or_mounting: "TO-92".into(),
                raw_spec_text: "J201 TO-92".into(),
                canonical_value_display: None,
                resistance_ohms: None,
                capacitance_pf: None,
                inductance_uh: None,
                voltage_rating_volts: None,
                low_stock_threshold: 1,
                datasheet_url: "".into(),
                notes: "".into(),
                specs_json: None,
                initial_lot: None,
            },
        )
        .await
        .unwrap();

        let j5457 = crate::catalog::create_component(
            &db,
            crate::catalog::CreateComponent {
                name: "2N5457 N-Channel JFET".into(),
                category: "Semiconductors — Discrete".into(),
                subtype: "JFET".into(),
                package_or_mounting: "TO-92".into(),
                raw_spec_text: "2N5457 TO-92".into(),
                canonical_value_display: None,
                resistance_ohms: None,
                capacitance_pf: None,
                inductance_uh: None,
                voltage_rating_volts: None,
                low_stock_threshold: 1,
                datasheet_url: "".into(),
                notes: "".into(),
                specs_json: None,
                initial_lot: Some(crate::catalog::CreateInitialStockLot {
                    location_id: Some(drawer.id.clone()),
                    quantity: 8,
                    supplier: "Mouser".into(),
                    supplier_part_no: "2N5457".into(),
                    lot_notes: "".into(),
                }),
            },
        )
        .await
        .unwrap();

        let cap_100n = crate::catalog::create_component(
            &db,
            crate::catalog::CreateComponent {
                name: "100nF 63V Box Film Cap".into(),
                category: "Capacitors".into(),
                subtype: "Film".into(),
                package_or_mounting: "Through-Hole 5mm".into(),
                raw_spec_text: "100nF 63V Box Film".into(),
                canonical_value_display: None,
                resistance_ohms: None,
                capacitance_pf: None,
                inductance_uh: None,
                voltage_rating_volts: None,
                low_stock_threshold: 2,
                datasheet_url: "".into(),
                notes: "".into(),
                specs_json: None,
                initial_lot: Some(crate::catalog::CreateInitialStockLot {
                    location_id: Some(drawer.id.clone()),
                    quantity: 6,
                    supplier: "Mouser".into(),
                    supplier_part_no: "100N".into(),
                    lot_notes: "".into(),
                }),
            },
        )
        .await
        .unwrap();

        let build = crate::builds::create_kit_build(
            &db,
            crate::builds::CreateKitBuild {
                name: "BSIAB Overdrive".into(),
                project_type: "Overdrive".into(),
                status: Some("On the Bench (In Progress)".into()),
                location_id: Some(drawer.id.clone()),
                description: "Brown sound in a box".into(),
            },
        )
        .await
        .unwrap();

        let q1_line = crate::builds::add_bom_line(
            &db,
            crate::builds::CreateBomLine {
                kit_build_id: build.id.clone(),
                raw_spec_text: "J201".into(),
                category: None,
                package_or_mounting: None,
                quantity_required: 1,
                reference_designators: "Q1".into(),
                included_in_kit: false,
                component_id: None,
                substitution_note: "".into(),
                notes: "".into(),
            },
        )
        .await
        .unwrap();
        assert_eq!(q1_line.component_id.as_deref(), Some(j201.id.as_str()));

        let c_line = crate::builds::add_bom_line(
            &db,
            crate::builds::CreateBomLine {
                kit_build_id: build.id.clone(),
                raw_spec_text: "100nF".into(),
                category: None,
                package_or_mounting: None,
                quantity_required: 4,
                reference_designators: "C1, C2, C3, C4".into(),
                included_in_kit: false,
                component_id: Some(cap_100n.id.clone()),
                substitution_note: "".into(),
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        // 1. Open native <dialog closedby="any"> substitution modal via GET /builds/bom-lines/{line_id}/substitute-modal
        let modal_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/builds/bom-lines/{}/substitute-modal", q1_line.id))
                    .header("HX-Request", "true")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(modal_res.status(), StatusCode::OK);
        let modal_html = body_string(modal_res.into_body()).await;
        assert!(modal_html.contains("closedby=\"any\""));
        assert!(modal_html.contains("J201"));
        assert!(modal_html.contains("2N5457 N-Channel JFET"));

        // 2. Submit substitution via POST /builds/bom-lines/{line_id}/substitute
        let sub_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/builds/bom-lines/{}/substitute", q1_line.id))
                    .header("content-type", "application/x-www-form-urlencoded")
                    .header("HX-Request", "true")
                    .body(Body::from(format!(
                        "component_id={}&substitution_note=pinout+rotated",
                        j5457.id
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(sub_res.status(), StatusCode::OK);
        let sub_html = body_string(sub_res.into_body()).await;
        // Renders both original schematic spec (J201) and substituted part + note (2N5457 — pinout rotated) side by side
        assert!(sub_html.contains("J201"));
        assert!(sub_html.contains("2N5457 — pinout rotated"));

        // 3. Partially allocate C1..C4 (2 of 4 Reserved) and verify Shortfall: 2 badge in table & Inspector Drawer
        let partial_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/builds/bom-lines/{}/status", c_line.id))
                    .header("content-type", "application/x-www-form-urlencoded")
                    .header("HX-Request", "true")
                    .header("HX-Target", "inspector")
                    .body(Body::from("status=Reserved&quantity_allocated=2"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(partial_res.status(), StatusCode::OK);
        let partial_html = body_string(partial_res.into_body()).await;
        assert!(partial_html.contains("2 / 4"));
        assert!(partial_html.contains("Shortfall: 2"));

        // 4. Trigger 1-click Split BOM Line via POST /builds/bom-lines/{line_id}/split
        let split_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/builds/bom-lines/{}/split", c_line.id))
                    .header("content-type", "application/x-www-form-urlencoded")
                    .header("HX-Request", "true")
                    .body(Body::from("split_quantity=2"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(split_res.status(), StatusCode::OK);
        let split_html = body_string(split_res.into_body()).await;
        assert!(split_html.contains("C1, C2"));
        assert!(split_html.contains("C3, C4"));

        // 5. Trigger 1-click Clone Kit Build via POST /builds/{id}/clone
        let clone_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/builds/{}/clone", build.id))
                    .header("content-type", "application/x-www-form-urlencoded")
                    .header("HX-Request", "true")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(clone_res.status(), StatusCode::OK);
        let clone_html = body_string(clone_res.into_body()).await;
        assert!(clone_html.contains("BSIAB Overdrive (Copy)"));
        assert!(clone_html.contains("Wishlist / Planned"));
        assert!(clone_html.contains("Unassigned Location"));
        assert!(clone_html.contains("2N5457 — pinout rotated"));
    }

    #[tokio::test]
    async fn shopping_list_web_routes_render_multi_build_shortfall_overrides_csv_exports_and_order_receiving()
     {
        let db = init_memory_db().await.expect("memory db");
        let app = app_router(db.clone());

        let binder = crate::catalog::create_storage_location(
            &db,
            crate::catalog::CreateStorageLocation {
                name: "Binder A".into(),
                kind: "Binder".into(),
                parent_id: None,
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        let r_4k7 = crate::catalog::create_component(
            &db,
            crate::catalog::CreateComponent {
                name: "4.7kΩ 1% Metal Film Resistor".into(),
                category: "Resistors".into(),
                subtype: "Metal Film".into(),
                package_or_mounting: "Through-Hole".into(),
                raw_spec_text: "4k7 1% 1/4W Metal Film".into(),
                canonical_value_display: None,
                resistance_ohms: None,
                capacitance_pf: None,
                inductance_uh: None,
                voltage_rating_volts: None,
                low_stock_threshold: 10,
                datasheet_url: "".into(),
                notes: "".into(),
                specs_json: None,
                initial_lot: Some(crate::catalog::CreateInitialStockLot {
                    location_id: Some(binder.id.clone()),
                    quantity: 5,
                    supplier: "Tayda".into(),
                    supplier_part_no: "A-2214".into(),
                    lot_notes: "".into(),
                }),
            },
        )
        .await
        .unwrap();

        let b1 = crate::builds::create_kit_build(
            &db,
            crate::builds::CreateKitBuild {
                name: "Klon Centaur".into(),
                project_type: "Overdrive".into(),
                status: Some("Sourcing / Waiting on Parts".into()),
                location_id: None,
                description: "".into(),
            },
        )
        .await
        .unwrap();

        let b2 = crate::builds::create_kit_build(
            &db,
            crate::builds::CreateKitBuild {
                name: "Tube Screamer".into(),
                project_type: "Overdrive".into(),
                status: Some("Wishlist / Planned".into()),
                location_id: None,
                description: "".into(),
            },
        )
        .await
        .unwrap();

        crate::builds::add_bom_line(
            &db,
            crate::builds::CreateBomLine {
                kit_build_id: b1.id.clone(),
                raw_spec_text: "4k7 1%".into(),
                category: Some("Resistors".into()),
                package_or_mounting: Some("Through-Hole".into()),
                quantity_required: 4,
                reference_designators: "R1, R2, R3, R4".into(),
                included_in_kit: false,
                component_id: Some(r_4k7.id.clone()),
                substitution_note: "".into(),
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        crate::builds::add_bom_line(
            &db,
            crate::builds::CreateBomLine {
                kit_build_id: b2.id.clone(),
                raw_spec_text: "4.7k 1%".into(),
                category: Some("Resistors".into()),
                package_or_mounting: Some("Through-Hole".into()),
                quantity_required: 4,
                reference_designators: "R10, R11, R12, R13".into(),
                included_in_kit: false,
                component_id: Some(r_4k7.id.clone()),
                substitution_note: "".into(),
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        let unlinked_cap = crate::builds::add_bom_line(
            &db,
            crate::builds::CreateBomLine {
                kit_build_id: b1.id.clone(),
                raw_spec_text: "0.047uF 63V Film".into(),
                category: Some("Capacitors".into()),
                package_or_mounting: Some("Through-Hole 5mm".into()),
                quantity_required: 2,
                reference_designators: "C1, C2".into(),
                included_in_kit: false,
                component_id: None,
                substitution_note: "".into(),
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        // 1. GET /shopping-list with both builds and include_low_stock_restock=true
        let page_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/shopping-list?kit_build_id={}&kit_build_id={}&include_low_stock_restock=true",
                        b1.id, b2.id
                    ))
                    .header("HX-Request", "true")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(page_res.status(), StatusCode::OK);
        let page_html = body_string(page_res.into_body()).await;
        assert!(page_html.contains("4.7kΩ 1% Metal Film Resistor"));
        assert!(page_html.contains("Klon Centaur"));
        assert!(page_html.contains("Tube Screamer"));
        assert!(page_html.contains("R1, R2, R3, R4"));
        assert!(page_html.contains("R10, R11, R12, R13"));
        assert!(page_html.contains("A-2214"));

        // 2. POST /shopping-list/override on the unlinked 47nF capacitor line
        let override_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/shopping-list/override")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .header("HX-Request", "true")
                    .body(Body::from(
                        "group_key=unlinked%3ACapacitors%3A47nF%3AThrough-Hole+5mm&order_quantity=10&supplier=Mouser&supplier_part_no=505-MKS2.047%2F63%2F5",
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(override_res.status(), StatusCode::OK);
        let override_html = body_string(override_res.into_body()).await;
        assert!(override_html.contains("505-MKS2.047/63/5"));

        // 3. GET /shopping-list/export/full.csv and /shopping-list/export/quick-order.csv
        let full_csv_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/shopping-list/export/full.csv?kit_build_id={}&kit_build_id={}",
                        b1.id, b2.id
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(full_csv_res.status(), StatusCode::OK);
        let full_csv_text = body_string(full_csv_res.into_body()).await;
        assert!(full_csv_text.contains("Category,Spec / Component,Package,Total Needed,Available Stock,Net Shortfall"));
        assert!(full_csv_text.contains("A-2214"));
        assert!(full_csv_text.contains("505-MKS2.047/63/5"));

        let quick_csv_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/shopping-list/export/quick-order.csv?kit_build_id={}&kit_build_id={}&supplier=Mouser",
                        b1.id, b2.id
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(quick_csv_res.status(), StatusCode::OK);
        let quick_csv_text = body_string(quick_csv_res.into_body()).await;
        assert!(quick_csv_text.starts_with("Supplier Part #,Order Quantity,Customer Reference\n"));
        assert!(quick_csv_text.contains("505-MKS2.047/63/5,10,\"Klon Centaur [C1, C2]\""));
        assert!(!quick_csv_text.contains("A-2214"));

        // 4. POST /shopping-list/receive to receive both lines into Binder A
        let receive_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/shopping-list/receive")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .header("HX-Request", "true")
                    .body(Body::from(format!(
                        "target_location_id={}&kit_build_id={}&kit_build_id={}&group_key=comp%3A{}&group_key=unlinked%3ACapacitors%3A47nF%3AThrough-Hole+5mm",
                        binder.id, b1.id, b2.id, r_4k7.id
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(receive_res.status(), StatusCode::OK);

        let updated_r = crate::catalog::get_component(&db, &r_4k7.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated_r.total_on_hand, 8); // 5 + 3 shortfall received

        let updated_bom = crate::builds::get_bom_line(&db, &unlinked_cap.id)
            .await
            .unwrap()
            .unwrap();
        assert!(updated_bom.component_id.is_some());
    }

    #[tokio::test]
    async fn active_bench_checklist_web_routes_render_stages_toggle_designator_chips_upload_attachments_and_record_bench_logs()
     {
        let tmp = tempfile::tempdir().expect("tempdir");
        let db = crate::db::init_data_dir(tmp.path()).await.expect("data dir db");
        let app = app_router(db.clone());

        let drawer = crate::catalog::create_storage_location(
            &db,
            crate::catalog::CreateStorageLocation {
                name: "Drawer A".into(),
                kind: "Drawer".into(),
                parent_id: None,
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        let r_comp = crate::catalog::create_component(
            &db,
            crate::catalog::CreateComponent {
                name: "4.7kΩ Metal Film Resistor".into(),
                category: "Resistors".into(),
                subtype: "Fixed".into(),
                package_or_mounting: "Through-Hole".into(),
                raw_spec_text: "4k7 1% 1/4W Metal Film".into(),
                canonical_value_display: None,
                resistance_ohms: None,
                capacitance_pf: None,
                inductance_uh: None,
                voltage_rating_volts: None,
                low_stock_threshold: 2,
                datasheet_url: "".into(),
                notes: "".into(),
                specs_json: None,
                initial_lot: Some(crate::catalog::CreateInitialStockLot {
                    location_id: Some(drawer.id.clone()),
                    quantity: 10,
                    supplier: "Tayda".into(),
                    supplier_part_no: "R-4K7".into(),
                    lot_notes: "".into(),
                }),
            },
        )
        .await
        .unwrap();

        let build = crate::builds::create_kit_build(
            &db,
            crate::builds::CreateKitBuild {
                name: "Klon Centaur (#1)".into(),
                project_type: "Overdrive".into(),
                status: Some("On the Bench (In Progress)".into()),
                location_id: Some(drawer.id.clone()),
                description: "".into(),
            },
        )
        .await
        .unwrap();

        let line_r = crate::builds::add_bom_line(
            &db,
            crate::builds::CreateBomLine {
                kit_build_id: build.id.clone(),
                raw_spec_text: "4k7 1%".into(),
                category: None,
                package_or_mounting: None,
                quantity_required: 2,
                reference_designators: "R1, R2".into(),
                included_in_kit: false,
                component_id: None,
                substitution_note: "".into(),
                notes: "".into(),
            },
        )
        .await
        .unwrap();
        crate::builds::transition_bom_line_status(&db, &line_r.id, "Reserved", None, None)
            .await
            .unwrap();

        let line_ic = crate::builds::add_bom_line(
            &db,
            crate::builds::CreateBomLine {
                kit_build_id: build.id.clone(),
                raw_spec_text: "TL072IP DIP-8".into(),
                category: None,
                package_or_mounting: None,
                quantity_required: 1,
                reference_designators: "IC1".into(),
                included_in_kit: false,
                component_id: None,
                substitution_note: "".into(),
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        let _line_pot = crate::builds::add_bom_line(
            &db,
            crate::builds::CreateBomLine {
                kit_build_id: build.id.clone(),
                raw_spec_text: "100kB 16mm PCB".into(),
                category: None,
                package_or_mounting: None,
                quantity_required: 1,
                reference_designators: "VR1".into(),
                included_in_kit: false,
                component_id: None,
                substitution_note: "".into(),
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        let bench_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/bench/{}", build.id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(bench_res.status(), StatusCode::OK);
        let bench_html = body_string(bench_res.into_body()).await;
        assert!(bench_html.contains("Stage 1: Low-Profile Resistors &amp; Diodes") || bench_html.contains("Stage 1: Low-Profile Resistors & Diodes"));
        assert!(bench_html.contains("Stage 2: IC Sockets &amp; Film Capacitors") || bench_html.contains("Stage 2: IC Sockets & Film Capacitors"));
        assert!(bench_html.contains("Stage 3: Electrolytics, Pots &amp; Off-Board Hardware") || bench_html.contains("Stage 3: Electrolytics, Pots & Off-Board Hardware"));
        assert!(bench_html.contains("0 / 4 Placements Soldered"));
        assert!(bench_html.contains("<progress"));
        assert!(bench_html.contains("○ R1"));
        assert!(bench_html.contains("○ R2"));

        let toggle_r1_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/bench/bom-lines/{}/toggle-designator", line_r.id))
                    .header("content-type", "application/x-www-form-urlencoded")
                    .header("HX-Request", "true")
                    .header("HX-Target", format!("bench-row-{}", line_r.id))
                    .body(Body::from("designator=R1"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(toggle_r1_res.status(), StatusCode::OK);
        let toggle_r1_html = body_string(toggle_r1_res.into_body()).await;
        assert!(toggle_r1_html.contains("✓ R1"));
        assert!(toggle_r1_html.contains("○ R2"));
        assert!(toggle_r1_html.contains("1 / 4 Placements Soldered"));
        assert!(toggle_r1_html.contains("hx-swap-oob=\"true\""));

        let toggle_r2_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/bench/bom-lines/{}/toggle-designator", line_r.id))
                    .header("content-type", "application/x-www-form-urlencoded")
                    .header("HX-Request", "true")
                    .header("HX-Target", format!("bench-row-{}", line_r.id))
                    .body(Body::from("designator=R2"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(toggle_r2_res.status(), StatusCode::OK);
        let toggle_r2_html = body_string(toggle_r2_res.into_body()).await;
        assert!(toggle_r2_html.contains("✓ R1"));
        assert!(toggle_r2_html.contains("✓ R2"));
        assert!(toggle_r2_html.contains("Installed"));
        assert!(toggle_r2_html.contains("2 / 4 Placements Soldered"));
        assert_eq!(
            crate::catalog::get_component(&db, &r_comp.id)
                .await
                .unwrap()
                .unwrap()
                .total_on_hand,
            8
        );

        let install_ic_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/bench/bom-lines/{}/install-all", line_ic.id))
                    .header("HX-Request", "true")
                    .header("HX-Target", format!("bench-row-{}", line_ic.id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(install_ic_res.status(), StatusCode::OK);
        let install_ic_html = body_string(install_ic_res.into_body()).await;
        assert!(install_ic_html.contains("✓ IC1"));
        assert!(install_ic_html.contains("3 / 4 Placements Soldered"));

        let boundary = "----BenchAttachmentBoundary123";
        let multipart_body = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"kind\"\r\n\r\nSchematic PDF\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"klon_schematic.pdf\"\r\nContent-Type: application/pdf\r\n\r\n%PDF-1.4-klon-test\r\n--{boundary}--\r\n"
        );
        let upload_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/bench/{}/attachments", build.id))
                    .header(
                        "content-type",
                        format!("multipart/form-data; boundary={boundary}"),
                    )
                    .header("HX-Request", "true")
                    .body(Body::from(multipart_body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(upload_res.status(), StatusCode::OK);
        let upload_html = body_string(upload_res.into_body()).await;
        assert!(upload_html.contains("klon_schematic.pdf"));
        assert!(upload_html.contains("Schematic PDF"));

        let atts = crate::builds::list_build_attachments(&db, &build.id)
            .await
            .unwrap();
        assert_eq!(atts.len(), 1);
        let dl_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/attachments/{}/download", atts[0].id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(dl_res.status(), StatusCode::OK);
        let dl_bytes = body_string(dl_res.into_body()).await;
        assert_eq!(dl_bytes, "%PDF-1.4-klon-test");

        let link_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/bench/{}/links", build.id))
                    .header("content-type", "application/x-www-form-urlencoded")
                    .header("HX-Request", "true")
                    .body(Body::from(
                        "label=Aion+Refractor+Docs&external_url=https%3A%2F%2Faionfx.com%2Fproject%2Frefractor",
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(link_res.status(), StatusCode::OK);
        let link_html = body_string(link_res.into_body()).await;
        assert!(link_html.contains("Aion Refractor Docs"));
        assert!(link_html.contains("https://aionfx.com/project/refractor"));

        let log_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/bench/{}/logs", build.id))
                    .header("content-type", "application/x-www-form-urlencoded")
                    .header("HX-Request", "true")
                    .body(Body::from(
                        "entry_type=Measurement+%2F+Voltage+Reading&body=IC1+Pin+8+%3D+%2B9.14V%2C+Pin+4+%3D+0.00V",
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(log_res.status(), StatusCode::OK);
        let log_html = body_string(log_res.into_body()).await;
        assert!(log_html.contains("Measurement / Voltage Reading"));
        assert!(log_html.contains("IC1 Pin 8 = +9.14V, Pin 4 = 0.00V"));
    }

    #[tokio::test]
    async fn imports_web_routes_stage_reconcile_and_commit_component_and_bom_csv_imports() {
        let db = init_memory_db().await.expect("memory db");
        let app = app_router(db.clone());

        let binder = crate::catalog::create_storage_location(
            &db,
            crate::catalog::CreateStorageLocation {
                name: "Binder A".into(),
                kind: "Binder".into(),
                parent_id: None,
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        let existing_r = crate::catalog::create_component(
            &db,
            crate::catalog::CreateComponent {
                name: "4.7kΩ 1% 1/4W Metal Film Resistor".into(),
                category: "Resistors".into(),
                subtype: "Fixed".into(),
                package_or_mounting: "Through-Hole".into(),
                raw_spec_text: "4k7 1% 1/4W Metal Film".into(),
                canonical_value_display: None,
                resistance_ohms: None,
                capacitance_pf: None,
                inductance_uh: None,
                voltage_rating_volts: None,
                low_stock_threshold: 10,
                datasheet_url: "".into(),
                notes: "".into(),
                specs_json: None,
                initial_lot: Some(crate::catalog::CreateInitialStockLot {
                    location_id: Some(binder.id.clone()),
                    quantity: 20,
                    supplier: "Tayda".into(),
                    supplier_part_no: "A-2214".into(),
                    lot_notes: "".into(),
                }),
            },
        )
        .await
        .unwrap();

        // 1. GET /imports renders Component & Stock Lot Import and Kit Build BOM Import forms + supplier presets
        let get_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/imports")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(get_res.status(), StatusCode::OK);
        let get_html = body_string(get_res.into_body()).await;
        assert!(get_html.contains("Component &amp; Stock Lot Import") || get_html.contains("Component & Stock Lot Import"));
        assert!(get_html.contains("Kit Build BOM Import"));
        assert!(get_html.contains("Mouser"));
        assert!(get_html.contains("DigiKey"));
        assert!(get_html.contains("Tayda"));
        assert!(get_html.contains("LCSC"));

        // 2. Stage Component CSV import via POST /imports/components/stage
        let encoded_csv = "Tayda+SKU%2CProduct+Name%2CQty%2CLocation%0AA-2214%2C4k7+1%25+1%2F4W+Metal+Film%2C15%2CBinder+A%0AA-9999%2C1N4148+Switching+Diode+DO-35+Axial%2C50%2CCrate+1+%3E+Diodes+Bag%0A";
        let stage_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/imports/components/stage")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .header("HX-Request", "true")
                    .body(Body::from(format!(
                        "default_location_id={}&default_supplier=Tayda&preset_name=Tayda&csv_text={}",
                        binder.id, encoded_csv
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(stage_res.status(), StatusCode::OK);
        let stage_html = body_string(stage_res.into_body()).await;
        assert!(stage_html.contains("Exact Match"));
        assert!(stage_html.contains("No Match"));
        assert!(stage_html.contains("1N4148 Switching Diode DO-35 Axial"));

        // Find the staged batch ID from DB
        let conn = db.connect().await.unwrap();
        let mut b_rows = conn
            .query(
                "SELECT id FROM import_batches WHERE target_kind = 'components' ORDER BY created_at DESC LIMIT 1",
                (),
            )
            .await
            .unwrap();
        let comp_batch_id = b_rows.next().await.unwrap().unwrap().get::<String>(0).unwrap();
        drop(b_rows);
        let comp_batch = crate::imports::get_import_batch(&db, &comp_batch_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(comp_batch.rows.len(), 2);

        // 3. Override row resolution via POST /imports/rows/{row_id}/resolve
        let row0_id = &comp_batch.rows[0].id;
        let resolve_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/imports/rows/{row0_id}/resolve"))
                    .header("content-type", "application/x-www-form-urlencoded")
                    .header("HX-Request", "true")
                    .body(Body::from(format!(
                        "chosen_action=add_stock_lot&matched_component_id={}",
                        existing_r.id
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resolve_res.status(), StatusCode::OK);

        // 4. Commit the staged Component batch via POST /imports/batches/{batch_id}/commit
        let commit_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/imports/batches/{comp_batch_id}/commit"))
                    .header("HX-Request", "true")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(commit_res.status(), StatusCode::OK);
        let commit_html = body_string(commit_res.into_body()).await;
        assert!(commit_html.contains("Committed"));

        let r_after = crate::catalog::get_component(&db, &existing_r.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(r_after.total_on_hand, 35); // 20 + 15

        // 5. Stage & Commit a Kit Build BOM Import via POST /imports/bom/stage and POST /imports/batches/{batch_id}/commit
        let build = crate::builds::create_kit_build(
            &db,
            crate::builds::CreateKitBuild {
                name: "Big Muff Pi".into(),
                project_type: "Fuzz".into(),
                status: None,
                location_id: None,
                description: "".into(),
            },
        )
        .await
        .unwrap();

        let encoded_bom = "Reference%2CValue%2CFootprint%0AR1%2C4k7%2CThrough-Hole%0AR2%2C4.7k%2CThrough-Hole%0AD1%2C1N4148%2CDO-35+Axial%0A";
        let stage_bom_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/imports/bom/stage")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .header("HX-Request", "true")
                    .body(Body::from(format!(
                        "kit_build_id={}&import_mode=replace_unallocated&group_designators=true&csv_text={}",
                        build.id, encoded_bom
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(stage_bom_res.status(), StatusCode::OK);
        let stage_bom_html = body_string(stage_bom_res.into_body()).await;
        assert!(
            !stage_bom_html.contains("Import Error:"),
            "unexpected error in stage_bom_html: {stage_bom_html}"
        );
        assert!(stage_bom_html.contains("R1, R2"));

        let conn2 = db.connect().await.unwrap();
        let mut bom_b_rows = conn2
            .query(
                "SELECT id FROM import_batches WHERE target_kind = 'bom' ORDER BY created_at DESC LIMIT 1",
                (),
            )
            .await
            .unwrap();
        let bom_batch_id = bom_b_rows
            .next()
            .await
            .unwrap()
            .unwrap()
            .get::<String>(0)
            .unwrap();

        let commit_bom_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/imports/batches/{bom_batch_id}/commit"))
                    .header("HX-Request", "true")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(commit_bom_res.status(), StatusCode::OK);

        let bom_lines = crate::builds::list_bom_lines_for_build(&db, &build.id)
            .await
            .unwrap();
        assert_eq!(bom_lines.len(), 2); // R1, R2 grouped + D1
        assert_eq!(bom_lines[0].reference_designators, "R1, R2");
        assert_eq!(bom_lines[0].quantity_required, 2);
        assert_eq!(
            bom_lines[0].component_id.as_deref(),
            Some(existing_r.id.as_str())
        );
    }

    #[tokio::test]
    async fn settings_backup_zip_download_and_restore_upload_preserve_all_records_and_attachments_byte_for_byte()
     {
        use http_body_util::BodyExt;

        let src_tmp = tempfile::tempdir().expect("src tempdir");
        let src_dir = src_tmp.path().join("workshop_source");
        let src_db = crate::db::init_data_dir(&src_dir).await.expect("src db");
        let src_app = app_router(src_db.clone());

        // Populate all domain entities in source data directory
        let crate_loc = crate::catalog::create_storage_location(
            &src_db,
            crate::catalog::CreateStorageLocation {
                name: "Bench Crate".into(),
                kind: "Crate".into(),
                parent_id: None,
                notes: "Top shelf".into(),
            },
        )
        .await
        .unwrap();
        let drawer_loc = crate::catalog::create_storage_location(
            &src_db,
            crate::catalog::CreateStorageLocation {
                name: "IC Drawer".into(),
                kind: "Drawer".into(),
                parent_id: Some(crate_loc.id.clone()),
                notes: "ESD foam".into(),
            },
        )
        .await
        .unwrap();

        let op_amp = crate::catalog::create_component(
            &src_db,
            crate::catalog::CreateComponent {
                name: "TL072IP Dual JFET Op-Amp".into(),
                category: "Integrated Circuits (ICs)".into(),
                subtype: "Op-Amp".into(),
                package_or_mounting: "DIP-8".into(),
                raw_spec_text: "TL072IP DIP-8 Dual Op-Amp".into(),
                canonical_value_display: None,
                resistance_ohms: None,
                capacitance_pf: None,
                inductance_uh: None,
                voltage_rating_volts: None,
                low_stock_threshold: 4,
                datasheet_url: "https://ti.com/tl072.pdf".into(),
                notes: "Low noise audio op-amp".into(),
                specs_json: None,
                initial_lot: Some(crate::catalog::CreateInitialStockLot {
                    location_id: Some(drawer_loc.id.clone()),
                    quantity: 12,
                    supplier: "Mouser".into(),
                    supplier_part_no: "595-TL072IP".into(),
                    lot_notes: "Tube A".into(),
                }),
            },
        )
        .await
        .unwrap();

        let build = crate::builds::create_kit_build(
            &src_db,
            crate::builds::CreateKitBuild {
                name: "Aion FX Refractor (Klon)".into(),
                project_type: "Overdrive Pedal".into(),
                status: Some("On the Bench (In Progress)".into()),
                location_id: Some(crate_loc.id.clone()),
                description: "Centaur clone".into(),
            },
        )
        .await
        .unwrap();

        let bom_line = crate::builds::add_bom_line(
            &src_db,
            crate::builds::CreateBomLine {
                kit_build_id: build.id.clone(),
                raw_spec_text: "TL072IP DIP-8".into(),
                category: Some("Integrated Circuits (ICs)".into()),
                package_or_mounting: Some("DIP-8".into()),
                quantity_required: 2,
                reference_designators: "IC1, IC2".into(),
                included_in_kit: false,
                component_id: Some(op_amp.id.clone()),
                substitution_note: "".into(),
                notes: "Dual op-amp stages".into(),
            },
        )
        .await
        .unwrap();

        crate::builds::transition_bom_line_status(
            &src_db,
            &bom_line.id,
            "Picked",
            Some(&op_amp.lots[0].id),
            Some(2),
        )
        .await
        .unwrap();
        crate::builds::toggle_bom_line_designator(&src_db, &bom_line.id, "IC1")
            .await
            .unwrap();

        let log_entry = crate::builds::create_bench_log_entry(
            &src_db,
            crate::builds::CreateBenchLogEntry {
                kit_build_id: build.id.clone(),
                entry_type: "Measurement / Voltage Reading".into(),
                body: "IC1 Pin 8 = +9.08V, IC2 Pin 8 = +17.6V".into(),
            },
        )
        .await
        .unwrap();

        let pdf_payload = b"%PDF-1.4\n%Refractor Build Guide & Schematic Binary\x00\x99\xFE\n";
        let att = crate::builds::add_build_attachment_file(
            &src_db,
            &build.id,
            Some(&log_entry.id),
            "Schematic PDF",
            "refractor_build_doc.pdf",
            "application/pdf",
            pdf_payload,
        )
        .await
        .unwrap();

        // 1. GET /settings renders backup & restore UI
        let settings_res = src_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/settings")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(settings_res.status(), StatusCode::OK);
        let settings_html = body_string(settings_res.into_body()).await;
        assert!(settings_html.contains("/settings/backup.zip"));
        assert!(settings_html.contains("/settings/restore"));

        // 2. GET /settings/backup.zip streams a valid .zip archive
        let zip_res = src_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/settings/backup.zip")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(zip_res.status(), StatusCode::OK);
        assert_eq!(
            zip_res
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok()),
            Some("application/zip")
        );
        assert!(
            zip_res
                .headers()
                .get(header::CONTENT_DISPOSITION)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .contains("audio-inventory-backup.zip")
        );
        let zip_bytes = zip_res
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec();
        assert!(!zip_bytes.is_empty());

        // 3. Restore into a completely fresh data directory via POST /settings/restore
        let dst_tmp = tempfile::tempdir().expect("dst tempdir");
        let dst_dir = dst_tmp.path().join("workshop_restored");
        let dst_db = crate::db::init_data_dir(&dst_dir).await.expect("dst db");
        let dst_app = app_router(dst_db.clone());

        let boundary = "----BackupRestoreBoundary987654";
        let mut body_bytes = Vec::new();
        body_bytes.extend_from_slice(
            format!(
                "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"audio-inventory-backup.zip\"\r\nContent-Type: application/zip\r\n\r\n"
            )
            .as_bytes(),
        );
        body_bytes.extend_from_slice(&zip_bytes);
        body_bytes.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

        let restore_res = dst_app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/settings/restore")
                    .header(
                        "content-type",
                        format!("multipart/form-data; boundary={boundary}"),
                    )
                    .header("HX-Request", "true")
                    .body(Body::from(body_bytes))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(restore_res.status(), StatusCode::OK);
        let restore_html = body_string(restore_res.into_body()).await;
        assert!(restore_html.contains("Backup Restored"));

        // Verify all records and attachment files in dst_db / dst_app
        let locs = crate::catalog::list_storage_locations(&dst_db)
            .await
            .unwrap();
        assert_eq!(locs.len(), 2);
        assert!(
            locs.iter()
                .any(|l| l.breadcrumb_path == "Bench Crate > IC Drawer")
        );

        let comps = crate::catalog::list_components(&dst_db).await.unwrap();
        assert_eq!(comps.len(), 1);
        assert_eq!(comps[0].id, op_amp.id);
        assert_eq!(comps[0].total_on_hand, 10); // 12 initial - 2 picked

        let restored_build = crate::builds::get_kit_build(&dst_db, &build.id)
            .await
            .unwrap()
            .expect("restored build");
        assert_eq!(restored_build.name, "Aion FX Refractor (Klon)");
        assert_eq!(restored_build.bom_lines.len(), 1);
        assert_eq!(restored_build.bom_lines[0].fulfillment_status, "Picked");
        assert_eq!(
            restored_build.bom_lines[0].installed_designators,
            vec!["IC1".to_string()]
        );

        let restored_logs = crate::builds::list_bench_log_entries(&dst_db, &build.id)
            .await
            .unwrap();
        assert_eq!(restored_logs.len(), 1);
        assert_eq!(
            restored_logs[0].body,
            "IC1 Pin 8 = +9.08V, IC2 Pin 8 = +17.6V"
        );

        // Verify attachment download via HTTP from restored app matches byte-for-byte
        let dl_res = dst_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/attachments/{}/download", att.id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(dl_res.status(), StatusCode::OK);
        let dl_bytes = dl_res
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec();
        assert_eq!(dl_bytes, pdf_payload);

        // 4. Reject invalid zip upload cleanly via POST /settings/restore without clobbering restored data
        let bad_body = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"corrupt.zip\"\r\nContent-Type: application/zip\r\n\r\nnot-a-zip-archive\r\n--{boundary}--\r\n"
        );
        let bad_res = dst_app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/settings/restore")
                    .header(
                        "content-type",
                        format!("multipart/form-data; boundary={boundary}"),
                    )
                    .header("HX-Request", "true")
                    .body(Body::from(bad_body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(bad_res.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            crate::catalog::list_components(&dst_db)
                .await
                .unwrap()
                .len(),
            1
        );
    }
}
