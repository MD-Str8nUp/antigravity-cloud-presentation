use crate::catalog::{
    Component, CreateComponent, CreateInitialStockLot, create_component, get_component,
    list_components,
};
use crate::db::Db;
use crate::spec_parser::{
    Category, format_capacitance_canonical, format_inductance_canonical,
    format_resistance_canonical, infer_category_from_designators, parse_spec,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FulfillmentStatus {
    Unallocated,
    Reserved,
    Picked,
    Installed,
    IncludedInKit,
}

impl FulfillmentStatus {
    pub const ALL: &'static [FulfillmentStatus] = &[
        FulfillmentStatus::Unallocated,
        FulfillmentStatus::Reserved,
        FulfillmentStatus::Picked,
        FulfillmentStatus::Installed,
        FulfillmentStatus::IncludedInKit,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            FulfillmentStatus::Unallocated => "Unallocated",
            FulfillmentStatus::Reserved => "Reserved",
            FulfillmentStatus::Picked => "Picked",
            FulfillmentStatus::Installed => "Installed",
            FulfillmentStatus::IncludedInKit => "Included in Kit",
        }
    }

    pub fn from_label(s: &str) -> Option<Self> {
        match s.trim() {
            "Unallocated" => Some(FulfillmentStatus::Unallocated),
            "Reserved" => Some(FulfillmentStatus::Reserved),
            "Picked" => Some(FulfillmentStatus::Picked),
            "Installed" => Some(FulfillmentStatus::Installed),
            "Included in Kit" => Some(FulfillmentStatus::IncludedInKit),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KitBuildStatus {
    WishlistPlanned,
    SourcingWaitingOnParts,
    ReadyToBuildKitted,
    OnTheBenchInProgress,
    BoxingTroubleshooting,
    Completed,
    ShelvedAbandoned,
}

impl KitBuildStatus {
    pub const ALL: &'static [KitBuildStatus] = &[
        KitBuildStatus::WishlistPlanned,
        KitBuildStatus::SourcingWaitingOnParts,
        KitBuildStatus::ReadyToBuildKitted,
        KitBuildStatus::OnTheBenchInProgress,
        KitBuildStatus::BoxingTroubleshooting,
        KitBuildStatus::Completed,
        KitBuildStatus::ShelvedAbandoned,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            KitBuildStatus::WishlistPlanned => "Wishlist / Planned",
            KitBuildStatus::SourcingWaitingOnParts => "Sourcing / Waiting on Parts",
            KitBuildStatus::ReadyToBuildKitted => "Ready to Build (Kitted)",
            KitBuildStatus::OnTheBenchInProgress => "On the Bench (In Progress)",
            KitBuildStatus::BoxingTroubleshooting => "Boxing / Troubleshooting",
            KitBuildStatus::Completed => "Completed",
            KitBuildStatus::ShelvedAbandoned => "Shelved / Abandoned",
        }
    }

    pub fn from_label(s: &str) -> Option<Self> {
        match s.trim() {
            "Wishlist / Planned" => Some(KitBuildStatus::WishlistPlanned),
            "Sourcing / Waiting on Parts" => Some(KitBuildStatus::SourcingWaitingOnParts),
            "Ready to Build (Kitted)" => Some(KitBuildStatus::ReadyToBuildKitted),
            "On the Bench (In Progress)" => Some(KitBuildStatus::OnTheBenchInProgress),
            "Boxing / Troubleshooting" => Some(KitBuildStatus::BoxingTroubleshooting),
            "Completed" => Some(KitBuildStatus::Completed),
            "Shelved / Abandoned" => Some(KitBuildStatus::ShelvedAbandoned),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildReadinessSummary {
    pub total_lines: usize,
    pub ready_lines: usize,
    pub shortfall_lines: usize,
    pub total_shortfall_quantity: i64,
}

impl BuildReadinessSummary {
    pub fn display_label(&self) -> String {
        format!(
            "{}/{} Lines Ready · {} Shortfalls",
            self.ready_lines, self.total_lines, self.shortfall_lines
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AssemblyStage {
    Stage1LowProfile,
    Stage2SocketsAndFilmCaps,
    Stage3ElectrolyticsPotsAndHardware,
}

impl AssemblyStage {
    pub const ALL: &'static [AssemblyStage] = &[
        AssemblyStage::Stage1LowProfile,
        AssemblyStage::Stage2SocketsAndFilmCaps,
        AssemblyStage::Stage3ElectrolyticsPotsAndHardware,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            AssemblyStage::Stage1LowProfile => "Stage 1: Low-Profile Resistors & Diodes",
            AssemblyStage::Stage2SocketsAndFilmCaps => "Stage 2: IC Sockets & Film Capacitors",
            AssemblyStage::Stage3ElectrolyticsPotsAndHardware => {
                "Stage 3: Electrolytics, Pots & Off-Board Hardware"
            }
        }
    }

    pub fn slug(&self) -> &'static str {
        match self {
            AssemblyStage::Stage1LowProfile => "stage-1",
            AssemblyStage::Stage2SocketsAndFilmCaps => "stage-2",
            AssemblyStage::Stage3ElectrolyticsPotsAndHardware => "stage-3",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BenchPlacementProgress {
    pub soldered_placements: usize,
    pub total_placements: usize,
}

impl BenchPlacementProgress {
    pub fn display_label(&self) -> String {
        format!(
            "{} / {} Placements Soldered",
            self.soldered_placements, self.total_placements
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BenchStageGroup {
    pub stage: AssemblyStage,
    pub soldered_placements: usize,
    pub total_placements: usize,
    pub lines: Vec<BomLine>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttachmentKind {
    SchematicPdf,
    BuildDocDrillTemplate,
    Photos,
    ExternalUrl,
}

impl AttachmentKind {
    pub const ALL_FILE_KINDS: &'static [AttachmentKind] = &[
        AttachmentKind::SchematicPdf,
        AttachmentKind::BuildDocDrillTemplate,
        AttachmentKind::Photos,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            AttachmentKind::SchematicPdf => "Schematic PDF",
            AttachmentKind::BuildDocDrillTemplate => "Build Doc / Drill Template",
            AttachmentKind::Photos => "Photos",
            AttachmentKind::ExternalUrl => "External URL",
        }
    }

    pub fn normalize_label(s: &str) -> &'static str {
        match s.trim() {
            "Schematic PDF" | "Schematic" => "Schematic PDF",
            "Build Doc / Drill Template" | "Build Doc" | "Drill Template" => {
                "Build Doc / Drill Template"
            }
            "Photos" | "Photo" => "Photos",
            "External URL" | "Link" | "URL" => "External URL",
            _ => "Schematic PDF",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildAttachment {
    pub id: String,
    pub kit_build_id: String,
    pub bench_log_entry_id: Option<String>,
    pub kind: String,
    pub original_filename: String,
    pub mime_type: String,
    pub relative_path: String,
    pub size_bytes: i64,
    pub external_url: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BenchLogEntryType {
    MeasurementVoltageReading,
    Troubleshooting,
    Modification,
    Note,
}

impl BenchLogEntryType {
    pub const ALL: &'static [BenchLogEntryType] = &[
        BenchLogEntryType::MeasurementVoltageReading,
        BenchLogEntryType::Troubleshooting,
        BenchLogEntryType::Modification,
        BenchLogEntryType::Note,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            BenchLogEntryType::MeasurementVoltageReading => "Measurement / Voltage Reading",
            BenchLogEntryType::Troubleshooting => "Troubleshooting",
            BenchLogEntryType::Modification => "Modification",
            BenchLogEntryType::Note => "Note",
        }
    }

    pub fn from_label(s: &str) -> Option<Self> {
        match s.trim() {
            "Measurement / Voltage Reading" | "Measurement" | "Voltage Reading" => {
                Some(BenchLogEntryType::MeasurementVoltageReading)
            }
            "Troubleshooting" => Some(BenchLogEntryType::Troubleshooting),
            "Modification" => Some(BenchLogEntryType::Modification),
            "Note" => Some(BenchLogEntryType::Note),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateBenchLogEntry {
    pub kit_build_id: String,
    pub entry_type: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BenchLogEntry {
    pub id: String,
    pub kit_build_id: String,
    pub entry_type: String,
    pub body: String,
    pub created_at: String,
    pub attachments: Vec<BuildAttachment>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateKitBuild {
    pub name: String,
    pub project_type: String,
    pub status: Option<String>,
    pub location_id: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateKitBuild {
    pub name: String,
    pub project_type: String,
    pub status: String,
    pub location_id: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateBomLine {
    pub kit_build_id: String,
    pub raw_spec_text: String,
    pub category: Option<String>,
    pub package_or_mounting: Option<String>,
    pub quantity_required: i64,
    pub reference_designators: String,
    pub included_in_kit: bool,
    pub component_id: Option<String>,
    pub substitution_note: String,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BomLine {
    pub id: String,
    pub kit_build_id: String,
    pub component_id: Option<String>,
    pub linked_component: Option<Component>,
    pub source_lot_id: Option<String>,
    pub source_lot_location_breadcrumb: Option<String>,
    pub source_lot_quantity: Option<i64>,
    pub raw_spec_text: String,
    pub canonical_value_display: String,
    pub category: String,
    pub package_or_mounting: String,
    pub resistance_ohms: Option<f64>,
    pub capacitance_pf: Option<f64>,
    pub inductance_uh: Option<f64>,
    pub voltage_rating_volts: Option<f64>,
    pub specs_json: JsonValue,
    pub quantity_required: i64,
    pub quantity_allocated: i64,
    pub shortfall_quantity: i64,
    pub fulfillment_status: String,
    pub included_in_kit: bool,
    pub reference_designators: String,
    pub installed_designators: Vec<String>,
    pub substitution_note: String,
    pub notes: String,
    pub sort_order: i64,
}

impl BomLine {
    pub fn shortfall_quantity(&self) -> i64 {
        self.shortfall_quantity
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BomCandidateMatch {
    pub component: Component,
    pub exact_match: bool,
    pub score: i64,
    pub match_reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KitBuild {
    pub id: String,
    pub name: String,
    pub project_type: String,
    pub status: String,
    pub location_id: Option<String>,
    pub location_breadcrumb: String,
    pub description: String,
    pub readiness: BuildReadinessSummary,
    pub bom_lines: Vec<BomLine>,
}

const SELECT_KIT_BUILD_SQL: &str = "
WITH RECURSIVE location_tree(id, breadcrumb_path) AS (
    SELECT id, name AS breadcrumb_path FROM storage_locations WHERE parent_id IS NULL
    UNION ALL
    SELECT c.id, p.breadcrumb_path || ' > ' || c.name
    FROM storage_locations c
    JOIN location_tree p ON c.parent_id = p.id
)
SELECT
    kb.id,
    kb.name,
    kb.project_type,
    kb.status,
    kb.location_id,
    COALESCE(lt.breadcrumb_path, 'Unassigned Location') AS location_breadcrumb,
    kb.description
FROM kit_builds kb
LEFT JOIN location_tree lt ON kb.location_id = lt.id
";

pub fn compute_readiness_summary(bom_lines: &[BomLine]) -> BuildReadinessSummary {
    let total_lines = bom_lines.len();
    let ready_lines = bom_lines
        .iter()
        .filter(|b| {
            b.fulfillment_status == "Installed"
                || b.fulfillment_status == "Included in Kit"
                || ((b.fulfillment_status == "Picked" || b.fulfillment_status == "Reserved")
                    && b.quantity_allocated >= b.quantity_required)
        })
        .count();
    let shortfall_lines = bom_lines
        .iter()
        .filter(|b| {
            b.fulfillment_status != "Included in Kit" && b.quantity_allocated < b.quantity_required
        })
        .count();
    let total_shortfall_quantity = bom_lines.iter().map(|b| b.shortfall_quantity).sum();

    BuildReadinessSummary {
        total_lines,
        ready_lines,
        shortfall_lines,
        total_shortfall_quantity,
    }
}

async fn map_kit_build_row(db: &Db, row: &turso::Row) -> anyhow::Result<KitBuild> {
    let id = row.get::<String>(0)?;
    let name = row.get::<String>(1)?;
    let project_type = row.get::<String>(2)?;
    let status = row.get::<String>(3)?;
    let location_id = row.get::<Option<String>>(4)?;
    let location_breadcrumb = row.get::<String>(5)?;
    let description = row.get::<String>(6)?;

    let bom_lines = list_bom_lines_for_build(db, &id).await?;
    let readiness = compute_readiness_summary(&bom_lines);

    Ok(KitBuild {
        id,
        name,
        project_type,
        status,
        location_id,
        location_breadcrumb,
        description,
        readiness,
        bom_lines,
    })
}

const SELECT_BOM_LINE_SQL: &str = "
WITH RECURSIVE location_tree(id, breadcrumb_path) AS (
    SELECT id, name AS breadcrumb_path FROM storage_locations WHERE parent_id IS NULL
    UNION ALL
    SELECT c.id, p.breadcrumb_path || ' > ' || c.name
    FROM storage_locations c
    JOIN location_tree p ON c.parent_id = p.id
)
SELECT
    bl.id,
    bl.kit_build_id,
    bl.component_id,
    bl.source_lot_id,
    CASE
        WHEN bl.source_lot_id IS NOT NULL THEN COALESCE(lt.breadcrumb_path, 'Unassigned Location')
        ELSE NULL
    END AS source_lot_location_breadcrumb,
    sl.quantity AS source_lot_quantity,
    bl.raw_spec_text,
    bl.canonical_value_display,
    bl.category,
    bl.package_or_mounting,
    bl.resistance_ohms,
    bl.capacitance_pf,
    bl.inductance_uh,
    bl.voltage_rating_volts,
    bl.specs_json,
    bl.quantity_required,
    bl.quantity_allocated,
    bl.fulfillment_status,
    bl.included_in_kit,
    bl.reference_designators,
    bl.installed_designators_json,
    bl.substitution_note,
    bl.notes,
    bl.sort_order
FROM bom_lines bl
LEFT JOIN stock_lots sl ON bl.source_lot_id = sl.id
LEFT JOIN location_tree lt ON sl.location_id = lt.id
";

async fn map_bom_line_row(db: &Db, row: &turso::Row) -> anyhow::Result<BomLine> {
    let id = row.get::<String>(0)?;
    let kit_build_id = row.get::<String>(1)?;
    let component_id = row.get::<Option<String>>(2)?;
    let source_lot_id = row.get::<Option<String>>(3)?;
    let source_lot_location_breadcrumb = row.get::<Option<String>>(4)?;
    let source_lot_quantity = row.get::<Option<i64>>(5)?;
    let raw_spec_text = row.get::<String>(6)?;
    let canonical_value_display = row.get::<String>(7)?;
    let category = row.get::<String>(8)?;
    let package_or_mounting = row.get::<String>(9)?;
    let resistance_ohms = row.get::<Option<f64>>(10)?;
    let capacitance_pf = row.get::<Option<f64>>(11)?;
    let inductance_uh = row.get::<Option<f64>>(12)?;
    let voltage_rating_volts = row.get::<Option<f64>>(13)?;
    let specs_json_str = row.get::<String>(14)?;
    let quantity_required = row.get::<i64>(15)?;
    let quantity_allocated = row.get::<i64>(16)?;
    let fulfillment_status = row.get::<String>(17)?;
    let included_in_kit_int = row.get::<i64>(18)?;
    let reference_designators = row.get::<String>(19)?;
    let installed_designators_str = row.get::<String>(20)?;
    let substitution_note = row.get::<String>(21)?;
    let notes = row.get::<String>(22)?;
    let sort_order = row.get::<i64>(23)?;

    let included_in_kit = included_in_kit_int != 0 || fulfillment_status == "Included in Kit";
    let shortfall_quantity = if included_in_kit {
        0
    } else {
        (quantity_required - quantity_allocated).max(0)
    };

    let specs_json: JsonValue = serde_json::from_str(&specs_json_str)
        .unwrap_or_else(|_| JsonValue::Object(JsonMap::new()));
    let installed_designators: Vec<String> =
        serde_json::from_str(&installed_designators_str).unwrap_or_default();

    let linked_component = if let Some(ref cid) = component_id {
        get_component(db, cid).await?
    } else {
        None
    };

    Ok(BomLine {
        id,
        kit_build_id,
        component_id,
        linked_component,
        source_lot_id,
        source_lot_location_breadcrumb,
        source_lot_quantity,
        raw_spec_text,
        canonical_value_display,
        category,
        package_or_mounting,
        resistance_ohms,
        capacitance_pf,
        inductance_uh,
        voltage_rating_volts,
        specs_json,
        quantity_required,
        quantity_allocated,
        shortfall_quantity,
        fulfillment_status,
        included_in_kit,
        reference_designators,
        installed_designators,
        substitution_note,
        notes,
        sort_order,
    })
}

use crate::spec_parser::{json_f64_attr, json_str_attr};

#[allow(clippy::too_many_arguments)]
fn rank_candidates_against_catalog(
    all_components: &[Component],
    category: &str,
    raw_spec_text: &str,
    canonical_value_display: &str,
    package_or_mounting: &str,
    resistance_ohms: Option<f64>,
    capacitance_pf: Option<f64>,
    inductance_uh: Option<f64>,
    voltage_rating_volts: Option<f64>,
    specs_json: &JsonValue,
) -> Vec<BomCandidateMatch> {
    let mut matches = Vec::new();
    let has_numeric =
        resistance_ohms.is_some() || capacitance_pf.is_some() || inductance_uh.is_some();
    let bom_part_code = json_str_attr(specs_json, "part_code");
    let bom_taper = json_str_attr(specs_json, "taper");
    let bom_tolerance = json_str_attr(specs_json, "tolerance");
    let bom_wattage = json_f64_attr(specs_json, "power_rating_watts");
    let bom_dielectric = json_str_attr(specs_json, "dielectric");
    let bom_composition = json_str_attr(specs_json, "composition");

    for comp in all_components {
        if category != "Other / Custom" && comp.category != category {
            continue;
        }

        let mut compatible = false;
        let mut reasons = Vec::new();

        if has_numeric {
            if let Some(ohms) = resistance_ohms {
                if comp
                    .resistance_ohms
                    .is_some_and(|co| (co - ohms).abs() < 1e-3)
                {
                    if let Some(tap) = bom_taper {
                        let comp_tap = json_str_attr(&comp.specs_json, "taper");
                        if comp_tap.is_some_and(|ct| ct.eq_ignore_ascii_case(tap)) {
                            compatible = true;
                            reasons.push(format!("{} Taper {}", comp.canonical_value_display, tap));
                        }
                    } else {
                        compatible = true;
                        reasons.push(comp.canonical_value_display.clone());
                    }
                }
            } else if let Some(pf) = capacitance_pf {
                if comp
                    .capacitance_pf
                    .is_some_and(|cp| (cp - pf).abs() < 1e-3)
                {
                    compatible = true;
                    reasons.push(comp.canonical_value_display.clone());
                }
            } else if let Some(uh) = inductance_uh
                && comp
                    .inductance_uh
                    .is_some_and(|cu| (cu - uh).abs() < 1e-3)
            {
                compatible = true;
                reasons.push(comp.canonical_value_display.clone());
            }
        } else {
            let comp_code = json_str_attr(&comp.specs_json, "part_code");
            if let (Some(b_code), Some(c_code)) = (bom_part_code, comp_code) {
                let bu = b_code.to_uppercase();
                let cu = c_code.to_uppercase();
                if bu == cu || cu.starts_with(&bu) || bu.starts_with(&cu) {
                    compatible = true;
                    reasons.push(format!("Part code {c_code}"));
                }
            }
            if !compatible {
                let needle = if !canonical_value_display.trim().is_empty() {
                    canonical_value_display.trim().to_lowercase()
                } else {
                    raw_spec_text.trim().to_lowercase()
                };
                if !needle.is_empty() {
                    let hay = format!(
                        "{} {} {}",
                        comp.canonical_value_display, comp.name, comp.raw_spec_text
                    )
                    .to_lowercase();
                    if hay.contains(&needle) {
                        compatible = true;
                        reasons.push(comp.canonical_value_display.clone());
                    }
                }
            }
        }

        if !compatible {
            continue;
        }

        let mut score = 100i64;
        let mut all_explicit_constraints_met = true;

        if !package_or_mounting.trim().is_empty() {
            if comp
                .package_or_mounting
                .to_lowercase()
                .contains(&package_or_mounting.trim().to_lowercase())
                || package_or_mounting
                    .trim()
                    .to_lowercase()
                    .contains(&comp.package_or_mounting.to_lowercase())
            {
                score += 15;
                reasons.push(comp.package_or_mounting.clone());
            } else {
                all_explicit_constraints_met = false;
            }
        }

        if let Some(tol) = bom_tolerance {
            if json_str_attr(&comp.specs_json, "tolerance")
                .is_some_and(|ct| ct.eq_ignore_ascii_case(tol))
            {
                score += 15;
                reasons.push(tol.to_string());
            } else {
                all_explicit_constraints_met = false;
            }
        }

        if let Some(w) = bom_wattage {
            if json_f64_attr(&comp.specs_json, "power_rating_watts")
                .is_some_and(|cw| (cw - w).abs() < 1e-4)
            {
                score += 15;
                reasons.push(format!("{w}W"));
            } else {
                all_explicit_constraints_met = false;
            }
        }

        if let Some(v) = voltage_rating_volts {
            if comp
                .voltage_rating_volts
                .is_some_and(|cv| (cv - v).abs() < 1e-3)
            {
                score += 15;
                reasons.push(format!("{v}V"));
            } else {
                all_explicit_constraints_met = false;
            }
        }

        if let Some(diel) = bom_dielectric {
            if json_str_attr(&comp.specs_json, "dielectric").is_some_and(|cd| {
                cd.to_lowercase().contains(&diel.to_lowercase())
                    || diel.to_lowercase().contains(&cd.to_lowercase())
            }) {
                score += 10;
                reasons.push(diel.to_string());
            } else {
                all_explicit_constraints_met = false;
            }
        }

        if let Some(comp_mat) = bom_composition {
            if json_str_attr(&comp.specs_json, "composition").is_some_and(|cc| {
                cc.to_lowercase().contains(&comp_mat.to_lowercase())
                    || comp_mat.to_lowercase().contains(&cc.to_lowercase())
            }) {
                score += 10;
                reasons.push(comp_mat.to_string());
            } else {
                all_explicit_constraints_met = false;
            }
        }

        let match_reason = if all_explicit_constraints_met {
            format!("Exact parametric match ({})", reasons.join(" · "))
        } else {
            format!("Compatible candidate ({})", reasons.join(" · "))
        };

        matches.push(BomCandidateMatch {
            component: comp.clone(),
            exact_match: all_explicit_constraints_met,
            score,
            match_reason,
        });
    }

    matches.sort_by(|a, b| {
        b.exact_match
            .cmp(&a.exact_match)
            .then_with(|| b.score.cmp(&a.score))
            .then_with(|| b.component.available_stock.cmp(&a.component.available_stock))
            .then_with(|| a.component.name.cmp(&b.component.name))
    });

    matches
}

pub async fn add_bom_line(db: &Db, input: CreateBomLine) -> anyhow::Result<BomLine> {
    let raw_spec_text = input.raw_spec_text.trim();
    if raw_spec_text.is_empty() {
        anyhow::bail!("BOM Line raw_spec_text cannot be empty");
    }
    let quantity_required = input.quantity_required.max(1);

    // Resolve category: explicit -> designator inference -> spec_parser inference -> Other / Custom
    let explicit_cat = input
        .category
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .and_then(Category::from_label);
    let designator_cat = infer_category_from_designators(&input.reference_designators);
    let category_hint = explicit_cat.or(designator_cat);

    let parsed = parse_spec(raw_spec_text, category_hint);
    let final_category = category_hint
        .or(parsed.inferred_category)
        .unwrap_or(Category::OtherCustom);

    let package_or_mounting = input
        .package_or_mounting
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .or_else(|| parsed.package_or_mounting.clone())
        .unwrap_or_default();

    let canonical_value_display = if !parsed.canonical_value_display.is_empty() {
        parsed.canonical_value_display.clone()
    } else if let Some(ohms) = parsed.resistance_ohms {
        format_resistance_canonical(ohms)
    } else if let Some(pf) = parsed.capacitance_pf {
        format_capacitance_canonical(pf)
    } else if let Some(uh) = parsed.inductance_uh {
        format_inductance_canonical(uh)
    } else {
        raw_spec_text.to_string()
    };

    let (component_id, fulfillment_status, quantity_allocated, included_in_kit_int) =
        if input.included_in_kit {
            (
                None,
                FulfillmentStatus::IncludedInKit.as_str().to_string(),
                quantity_required,
                1i64,
            )
        } else if let Some(explicit_cid) = input
            .component_id
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            (
                Some(explicit_cid.to_string()),
                FulfillmentStatus::Unallocated.as_str().to_string(),
                0i64,
                0i64,
            )
        } else {
            // Auto-link if and only if a single unambiguous match exists in the catalog
            let all_components = list_components(db).await?;
            let candidates = rank_candidates_against_catalog(
                &all_components,
                final_category.as_str(),
                raw_spec_text,
                &canonical_value_display,
                &package_or_mounting,
                parsed.resistance_ohms,
                parsed.capacitance_pf,
                parsed.inductance_uh,
                parsed.voltage_rating_volts,
                &parsed.specs_json,
            );
            let exact_matches: Vec<&BomCandidateMatch> =
                candidates.iter().filter(|c| c.exact_match).collect();
            let auto_cid = if exact_matches.len() == 1 {
                Some(exact_matches[0].component.id.clone())
            } else if exact_matches.is_empty() && candidates.len() == 1 {
                Some(candidates[0].component.id.clone())
            } else {
                None
            };
            (
                auto_cid,
                FulfillmentStatus::Unallocated.as_str().to_string(),
                0i64,
                0i64,
            )
        };

    let conn = db.connect().await?;
    let mut sort_rows = conn
        .query(
            "SELECT COALESCE(MAX(sort_order), 0) + 1 FROM bom_lines WHERE kit_build_id = ?1",
            (input.kit_build_id.as_str(),),
        )
        .await?;
    let next_sort_order = if let Some(r) = sort_rows.next().await? {
        r.get::<i64>(0)?
    } else {
        1
    };

    let id = format!("bom-{}", Uuid::new_v4());
    let specs_json_str = serde_json::to_string(&parsed.specs_json)?;

    conn.execute(
        "INSERT INTO bom_lines (
            id, kit_build_id, component_id, source_lot_id, raw_spec_text,
            canonical_value_display, category, package_or_mounting,
            resistance_ohms, capacitance_pf, inductance_uh, voltage_rating_volts,
            specs_json, quantity_required, quantity_allocated, fulfillment_status,
            included_in_kit, reference_designators, substitution_note, notes, sort_order
        ) VALUES (?1, ?2, ?3, NULL, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20)",
        turso::params![
            id.as_str(),
            input.kit_build_id.as_str(),
            component_id,
            raw_spec_text,
            canonical_value_display.as_str(),
            final_category.as_str(),
            package_or_mounting.as_str(),
            parsed.resistance_ohms,
            parsed.capacitance_pf,
            parsed.inductance_uh,
            parsed.voltage_rating_volts,
            specs_json_str.as_str(),
            quantity_required,
            quantity_allocated,
            fulfillment_status.as_str(),
            included_in_kit_int,
            input.reference_designators.trim(),
            input.substitution_note.trim(),
            input.notes.trim(),
            next_sort_order,
        ],
    )
    .await?;

    get_bom_line(db, &id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Failed to load created BOM Line '{id}'"))
}

pub async fn get_bom_line(db: &Db, bom_line_id: &str) -> anyhow::Result<Option<BomLine>> {
    let conn = db.connect().await?;
    let sql = format!("{SELECT_BOM_LINE_SQL} WHERE bl.id = ?1");
    let mut rows = conn.query(&sql, (bom_line_id,)).await?;
    if let Some(row) = rows.next().await? {
        Ok(Some(map_bom_line_row(db, &row).await?))
    } else {
        Ok(None)
    }
}

pub async fn list_bom_lines_for_build(
    db: &Db,
    kit_build_id: &str,
) -> anyhow::Result<Vec<BomLine>> {
    let conn = db.connect().await?;
    let sql = format!(
        "{SELECT_BOM_LINE_SQL} WHERE bl.kit_build_id = ?1 ORDER BY bl.sort_order ASC, bl.created_at ASC, bl.id ASC"
    );
    let mut rows = conn.query(&sql, (kit_build_id,)).await?;
    let mut out = Vec::new();
    while let Some(row) = rows.next().await? {
        out.push(map_bom_line_row(db, &row).await?);
    }
    Ok(out)
}

pub async fn find_bom_line_candidates(
    db: &Db,
    bom_line_id: &str,
) -> anyhow::Result<Vec<BomCandidateMatch>> {
    let line = get_bom_line(db, bom_line_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("BOM Line '{bom_line_id}' not found"))?;
    let all_components = list_components(db).await?;
    Ok(rank_candidates_against_catalog(
        &all_components,
        &line.category,
        &line.raw_spec_text,
        &line.canonical_value_display,
        &line.package_or_mounting,
        line.resistance_ohms,
        line.capacitance_pf,
        line.inductance_uh,
        line.voltage_rating_volts,
        &line.specs_json,
    ))
}

pub async fn link_bom_line_component(
    db: &Db,
    bom_line_id: &str,
    component_id: Option<&str>,
    substitution_note: Option<&str>,
) -> anyhow::Result<BomLine> {
    let existing = get_bom_line(db, bom_line_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("BOM Line '{bom_line_id}' not found"))?;

    let normalized_cid = component_id
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    if let Some(ref cid) = normalized_cid {
        if get_component(db, cid).await?.is_none() {
            anyhow::bail!("Catalog Component '{cid}' does not exist");
        }
    }

    let sub_note = substitution_note
        .map(str::trim)
        .unwrap_or(&existing.substitution_note);

    let component_changed = existing.component_id.as_deref() != normalized_cid.as_deref();
    let was_picked =
        existing.fulfillment_status == "Picked" && existing.quantity_allocated > 0;

    let mut conn = db.connect().await?;
    let tx = conn.transaction().await?;

    if component_changed && was_picked {
        restore_deducted_stock_tx(
            &tx,
            existing.component_id.as_deref(),
            existing.source_lot_id.as_deref(),
            existing.quantity_allocated,
        )
        .await?;
    }

    if component_changed
        && (was_picked
            || existing.fulfillment_status == "Installed"
            || existing.fulfillment_status == "Reserved"
            || existing.included_in_kit)
    {
        tx.execute(
            "UPDATE bom_lines
             SET component_id = ?1,
                 substitution_note = ?2,
                 fulfillment_status = 'Unallocated',
                 included_in_kit = 0,
                 quantity_allocated = 0,
                 source_lot_id = NULL,
                 installed_designators_json = '[]',
                 updated_at = datetime('now')
             WHERE id = ?3",
            (normalized_cid, sub_note, bom_line_id),
        )
        .await?;
    } else {
        tx.execute(
            "UPDATE bom_lines
             SET component_id = ?1, substitution_note = ?2, updated_at = datetime('now')
             WHERE id = ?3",
            (normalized_cid, sub_note, bom_line_id),
        )
        .await?;
    }

    tx.commit().await?;

    get_bom_line(db, bom_line_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("BOM Line '{bom_line_id}' not found after linking"))
}

pub async fn substitute_bom_line_component(
    db: &Db,
    bom_line_id: &str,
    component_id: &str,
    substitution_note: &str,
) -> anyhow::Result<BomLine> {
    let trimmed_cid = component_id.trim();
    if trimmed_cid.is_empty() {
        anyhow::bail!("Substitute Component ID cannot be empty");
    }
    link_bom_line_component(
        db,
        bom_line_id,
        Some(trimmed_cid),
        Some(substitution_note.trim()),
    )
    .await
}

pub async fn create_component_from_bom_line(
    db: &Db,
    bom_line_id: &str,
    initial_location_id: Option<String>,
    initial_quantity: i64,
) -> anyhow::Result<(Component, BomLine)> {
    let line = get_bom_line(db, bom_line_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("BOM Line '{bom_line_id}' not found"))?;

    let initial_lot = if initial_quantity > 0 || initial_location_id.is_some() {
        Some(CreateInitialStockLot {
            location_id: initial_location_id,
            quantity: initial_quantity.max(0),
            supplier: String::new(),
            supplier_part_no: String::new(),
            lot_notes: format!("Created from BOM Line {}", line.reference_designators),
        })
    } else {
        None
    };

    let created_comp = create_component(
        db,
        CreateComponent {
            name: String::new(), // auto-derived from canonical_value_display + category
            category: line.category.clone(),
            subtype: String::new(),
            package_or_mounting: line.package_or_mounting.clone(),
            raw_spec_text: line.raw_spec_text.clone(),
            canonical_value_display: Some(line.canonical_value_display.clone()),
            resistance_ohms: line.resistance_ohms,
            capacitance_pf: line.capacitance_pf,
            inductance_uh: line.inductance_uh,
            voltage_rating_volts: line.voltage_rating_volts,
            low_stock_threshold: 0,
            datasheet_url: String::new(),
            notes: line.notes.clone(),
            specs_json: Some(line.specs_json.clone()),
            initial_lot,
        },
    )
    .await?;

    let updated_line =
        link_bom_line_component(db, bom_line_id, Some(&created_comp.id), None).await?;
    Ok((created_comp, updated_line))
}

pub fn split_reference_designators(designators: &str) -> Vec<String> {
    designators
        .split(|c: char| c == ',' || c == ';')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

pub fn bom_line_placements(line: &BomLine) -> Vec<String> {
    let parsed = split_reference_designators(&line.reference_designators);
    if !parsed.is_empty() {
        parsed
    } else {
        let count = line.quantity_required.max(1);
        (1..=count).map(|i| format!("#{i}")).collect()
    }
}

fn is_electrolytic_capacitor(line: &BomLine) -> bool {
    let hay = format!(
        "{} {} {} {}",
        line.raw_spec_text,
        line.package_or_mounting,
        line.notes,
        line.linked_component
            .as_ref()
            .map(|c| format!("{} {} {}", c.subtype, c.raw_spec_text, c.name))
            .unwrap_or_default()
    )
    .to_lowercase();

    if hay.contains("electrolytic") || hay.contains("elec") || hay.contains("aluminum") {
        return true;
    }
    if line
        .specs_json
        .get("polarized")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        return true;
    }
    if json_str_attr(&line.specs_json, "dielectric")
        .is_some_and(|d| d.to_lowercase().contains("electrolytic"))
    {
        return true;
    }
    if let Some(pf) = line.capacitance_pf
        && pf >= 1_000_000.0
        && !hay.contains("film")
        && !hay.contains("ceramic")
        && !hay.contains("mlcc")
    {
        return true;
    }
    false
}

fn is_diode_discrete(line: &BomLine) -> bool {
    let designators = split_reference_designators(&line.reference_designators);
    if designators.iter().any(|d| {
        let up = d.to_uppercase();
        up.starts_with('D') && up.chars().nth(1).is_some_and(|c| c.is_ascii_digit())
    }) {
        return true;
    }

    let hay = format!(
        "{} {} {}",
        line.raw_spec_text,
        line.package_or_mounting,
        line.linked_component
            .as_ref()
            .map(|c| format!("{} {}", c.subtype, c.name))
            .unwrap_or_default()
    )
    .to_lowercase();

    hay.contains("diode")
        || hay.contains("schottky")
        || hay.contains("zener")
        || hay.contains("germanium")
        || hay.contains("1n4")
        || hay.contains("1n3")
        || hay.contains("1n5")
        || hay.contains("bat4")
        || hay.contains("do-35")
        || hay.contains("do-41")
        || hay.contains("do-7")
}

pub fn classify_bom_line_stage(line: &BomLine) -> AssemblyStage {
    match line.category.as_str() {
        "Resistors" => AssemblyStage::Stage1LowProfile,
        "Semiconductors — Discrete" => {
            if is_diode_discrete(line) {
                AssemblyStage::Stage1LowProfile
            } else {
                AssemblyStage::Stage2SocketsAndFilmCaps
            }
        }
        "Integrated Circuits (ICs)" | "Inductors, Transformers & Magnetics" => {
            AssemblyStage::Stage2SocketsAndFilmCaps
        }
        "Capacitors" => {
            if is_electrolytic_capacitor(line) {
                AssemblyStage::Stage3ElectrolyticsPotsAndHardware
            } else {
                AssemblyStage::Stage2SocketsAndFilmCaps
            }
        }
        _ => AssemblyStage::Stage3ElectrolyticsPotsAndHardware,
    }
}

pub fn compute_bench_placement_progress(bom_lines: &[BomLine]) -> BenchPlacementProgress {
    let mut soldered_placements = 0usize;
    let mut total_placements = 0usize;
    for line in bom_lines {
        let placements = bom_line_placements(line);
        total_placements += placements.len();
        soldered_placements += placements
            .iter()
            .filter(|des| {
                line.fulfillment_status == "Installed" || line.installed_designators.contains(*des)
            })
            .count();
    }
    BenchPlacementProgress {
        soldered_placements,
        total_placements,
    }
}

pub fn group_bom_lines_by_stage(bom_lines: &[BomLine]) -> Vec<BenchStageGroup> {
    AssemblyStage::ALL
        .iter()
        .map(|&stage| {
            let lines: Vec<BomLine> = bom_lines
                .iter()
                .filter(|l| classify_bom_line_stage(l) == stage)
                .cloned()
                .collect();
            let progress = compute_bench_placement_progress(&lines);
            BenchStageGroup {
                stage,
                soldered_placements: progress.soldered_placements,
                total_placements: progress.total_placements,
                lines,
            }
        })
        .collect()
}

async fn restore_deducted_stock_tx(
    tx: &turso::transaction::Transaction<'_>,
    component_id: Option<&str>,
    source_lot_id: Option<&str>,
    qty: i64,
) -> anyhow::Result<()> {
    if qty <= 0 {
        return Ok(());
    }

    if let Some(lot_id) = source_lot_id {
        let mut rows = tx
            .query("SELECT id FROM stock_lots WHERE id = ?1", (lot_id,))
            .await?;
        if rows.next().await?.is_some() {
            tx.execute(
                "UPDATE stock_lots SET quantity = quantity + ?1, updated_at = datetime('now') WHERE id = ?2",
                (qty, lot_id),
            )
            .await?;
            return Ok(());
        }
    }

    if let Some(cid) = component_id {
        let mut rows = tx
            .query(
                "SELECT id FROM stock_lots WHERE component_id = ?1 ORDER BY created_at ASC LIMIT 1",
                (cid,),
            )
            .await?;
        if let Some(row) = rows.next().await? {
            let fallback_lot_id = row.get::<String>(0)?;
            tx.execute(
                "UPDATE stock_lots SET quantity = quantity + ?1, updated_at = datetime('now') WHERE id = ?2",
                (qty, fallback_lot_id.as_str()),
            )
            .await?;
        } else {
            let new_lot_id = format!("lot-{}", Uuid::new_v4());
            tx.execute(
                "INSERT INTO stock_lots (id, component_id, location_id, quantity, lot_notes)
                 VALUES (?1, ?2, NULL, ?3, 'Restored from BOM Line')",
                (new_lot_id.as_str(), cid, qty),
            )
            .await?;
        }
    }

    Ok(())
}

async fn select_lot_for_deduction_tx(
    tx: &turso::transaction::Transaction<'_>,
    component_id: &str,
    preferred_lot_id: Option<&str>,
) -> anyhow::Result<Option<(String, i64)>> {
    let mut rows = tx
        .query(
            "SELECT id, quantity FROM stock_lots WHERE component_id = ?1 ORDER BY quantity DESC, created_at ASC",
            (component_id,),
        )
        .await?;
    let mut lots = Vec::new();
    while let Some(row) = rows.next().await? {
        lots.push((row.get::<String>(0)?, row.get::<i64>(1)?));
    }

    if let Some(pref_id) = preferred_lot_id.map(str::trim).filter(|s| !s.is_empty()) {
        if let Some(found) = lots.iter().find(|(id, _)| id == pref_id) {
            return Ok(Some(found.clone()));
        }
        anyhow::bail!("Specified Stock Lot '{pref_id}' not found for component '{component_id}'");
    }

    Ok(lots.into_iter().next())
}

pub async fn transition_bom_line_status(
    db: &Db,
    bom_line_id: &str,
    target_status: &str,
    source_lot_id_override: Option<&str>,
    quantity_allocated_override: Option<i64>,
) -> anyhow::Result<BomLine> {
    let target = FulfillmentStatus::from_label(target_status)
        .ok_or_else(|| anyhow::anyhow!("Invalid Fulfillment Status '{target_status}'"))?;

    let line = get_bom_line(db, bom_line_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("BOM Line '{bom_line_id}' not found"))?;

    let was_picked =
        line.fulfillment_status == "Picked" && line.quantity_allocated > 0;

    let mut conn = db.connect().await?;
    let tx = conn.transaction().await?;

    match target {
        FulfillmentStatus::IncludedInKit => {
            if was_picked {
                restore_deducted_stock_tx(
                    &tx,
                    line.component_id.as_deref(),
                    line.source_lot_id.as_deref(),
                    line.quantity_allocated,
                )
                .await?;
            }
            tx.execute(
                "UPDATE bom_lines
                 SET fulfillment_status = 'Included in Kit',
                     included_in_kit = 1,
                     quantity_allocated = ?1,
                     source_lot_id = NULL,
                     updated_at = datetime('now')
                 WHERE id = ?2",
                (line.quantity_required, bom_line_id),
            )
            .await?;
        }
        FulfillmentStatus::Unallocated => {
            if was_picked {
                restore_deducted_stock_tx(
                    &tx,
                    line.component_id.as_deref(),
                    line.source_lot_id.as_deref(),
                    line.quantity_allocated,
                )
                .await?;
            }
            tx.execute(
                "UPDATE bom_lines
                 SET fulfillment_status = 'Unallocated',
                     included_in_kit = 0,
                     quantity_allocated = 0,
                     source_lot_id = NULL,
                     installed_designators_json = '[]',
                     updated_at = datetime('now')
                 WHERE id = ?1",
                (bom_line_id,),
            )
            .await?;
        }
        FulfillmentStatus::Reserved => {
            let cid = line.component_id.as_deref().ok_or_else(|| {
                anyhow::anyhow!(
                    "Cannot transition unlinked BOM Line '{}' to Reserved; link a catalog Component first",
                    line.reference_designators
                )
            })?;

            if was_picked {
                restore_deducted_stock_tx(
                    &tx,
                    Some(cid),
                    line.source_lot_id.as_deref(),
                    line.quantity_allocated,
                )
                .await?;
            }

            let mut on_hand_rows = tx
                .query(
                    "SELECT COALESCE(SUM(quantity), 0) FROM stock_lots WHERE component_id = ?1",
                    (cid,),
                )
                .await?;
            let total_on_hand = match on_hand_rows.next().await? {
                Some(r) => r.get::<i64>(0)?,
                None => 0,
            };

            let mut other_res_rows = tx
                .query(
                    "SELECT COALESCE(SUM(quantity_allocated), 0)
                     FROM bom_lines
                     WHERE component_id = ?1 AND fulfillment_status = 'Reserved' AND id != ?2",
                    (cid, bom_line_id),
                )
                .await?;
            let other_reserved = match other_res_rows.next().await? {
                Some(r) => r.get::<i64>(0)?,
                None => 0,
            };

            let avail_for_line = (total_on_hand - other_reserved).max(0);
            let desired_qty = quantity_allocated_override.unwrap_or(line.quantity_required);
            let new_qty_allocated = desired_qty.clamp(0, avail_for_line.min(line.quantity_required));
            let pinned_lot_id = source_lot_id_override
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string);

            tx.execute(
                "UPDATE bom_lines
                 SET fulfillment_status = 'Reserved',
                     included_in_kit = 0,
                     quantity_allocated = ?1,
                     source_lot_id = ?2,
                     updated_at = datetime('now')
                 WHERE id = ?3",
                (new_qty_allocated, pinned_lot_id, bom_line_id),
            )
            .await?;
        }
        FulfillmentStatus::Picked => {
            let cid = line.component_id.as_deref().ok_or_else(|| {
                anyhow::anyhow!(
                    "Cannot transition unlinked BOM Line '{}' to Picked; link a catalog Component first",
                    line.reference_designators
                )
            })?;

            if was_picked {
                restore_deducted_stock_tx(
                    &tx,
                    Some(cid),
                    line.source_lot_id.as_deref(),
                    line.quantity_allocated,
                )
                .await?;
            }

            let preferred_lot = source_lot_id_override.or({
                if !was_picked {
                    line.source_lot_id.as_deref()
                } else {
                    None
                }
            });
            let (chosen_lot_id, lot_qty) =
                select_lot_for_deduction_tx(&tx, cid, preferred_lot)
                    .await?
                    .ok_or_else(|| {
                        anyhow::anyhow!("Component '{cid}' has no Stock Lots to pick from")
                    })?;

            if lot_qty <= 0 {
                anyhow::bail!(
                    "Insufficient stock in Stock Lot '{chosen_lot_id}' (0 on hand) to pick"
                );
            }

            let default_desired =
                if line.fulfillment_status == "Reserved" && line.quantity_allocated > 0 {
                    line.quantity_allocated
                } else {
                    line.quantity_required
                };
            let desired_qty = quantity_allocated_override.unwrap_or(default_desired);
            let pull_qty = desired_qty.clamp(1, lot_qty.min(line.quantity_required));

            tx.execute(
                "UPDATE stock_lots SET quantity = quantity - ?1, updated_at = datetime('now') WHERE id = ?2",
                (pull_qty, chosen_lot_id.as_str()),
            )
            .await?;

            tx.execute(
                "UPDATE bom_lines
                 SET fulfillment_status = 'Picked',
                     included_in_kit = 0,
                     quantity_allocated = ?1,
                     source_lot_id = ?2,
                     updated_at = datetime('now')
                 WHERE id = ?3",
                (pull_qty, chosen_lot_id.as_str(), bom_line_id),
            )
            .await?;
        }
        FulfillmentStatus::Installed => {
            let all_des = bom_line_placements(&line);
            let all_des_json = serde_json::to_string(&all_des)?;

            if matches!(line.fulfillment_status.as_str(), "Picked" | "Installed")
                && source_lot_id_override.is_none()
                && quantity_allocated_override.is_none()
            {
                let final_alloc = if line.quantity_allocated > 0 {
                    line.quantity_allocated
                } else {
                    line.quantity_required
                };
                tx.execute(
                    "UPDATE bom_lines
                     SET fulfillment_status = 'Installed',
                         included_in_kit = 0,
                         quantity_allocated = ?1,
                         installed_designators_json = ?2,
                         updated_at = datetime('now')
                     WHERE id = ?3",
                    (
                        final_alloc,
                        all_des_json.as_str(),
                        bom_line_id,
                    ),
                )
                .await?;
            } else if let Some(cid) = line.component_id.as_deref() {
                if was_picked {
                    restore_deducted_stock_tx(
                        &tx,
                        Some(cid),
                        line.source_lot_id.as_deref(),
                        line.quantity_allocated,
                    )
                    .await?;
                }

                let preferred_lot = source_lot_id_override.or(line.source_lot_id.as_deref());
                if let Some((chosen_lot_id, lot_qty)) =
                    select_lot_for_deduction_tx(&tx, cid, preferred_lot).await?
                    && lot_qty > 0
                {
                    let default_desired =
                        if line.fulfillment_status == "Reserved" && line.quantity_allocated > 0 {
                            line.quantity_allocated
                        } else {
                            line.quantity_required
                        };
                    let desired_qty = quantity_allocated_override.unwrap_or(default_desired);
                    let pull_qty = desired_qty.clamp(1, lot_qty.min(line.quantity_required));

                    tx.execute(
                        "UPDATE stock_lots SET quantity = quantity - ?1, updated_at = datetime('now') WHERE id = ?2",
                        (pull_qty, chosen_lot_id.as_str()),
                    )
                    .await?;

                    tx.execute(
                        "UPDATE bom_lines
                         SET fulfillment_status = 'Installed',
                             included_in_kit = 0,
                             quantity_allocated = ?1,
                             source_lot_id = ?2,
                             installed_designators_json = ?3,
                             updated_at = datetime('now')
                         WHERE id = ?4",
                        (
                            pull_qty,
                            chosen_lot_id.as_str(),
                            all_des_json.as_str(),
                            bom_line_id,
                        ),
                    )
                    .await?;
                } else {
                    tx.execute(
                        "UPDATE bom_lines
                         SET fulfillment_status = 'Installed',
                             included_in_kit = 0,
                             quantity_allocated = ?1,
                             installed_designators_json = ?2,
                             updated_at = datetime('now')
                         WHERE id = ?3",
                        (
                            line.quantity_required,
                            all_des_json.as_str(),
                            bom_line_id,
                        ),
                    )
                    .await?;
                }
            } else {
                tx.execute(
                    "UPDATE bom_lines
                     SET fulfillment_status = 'Installed',
                         included_in_kit = 0,
                         quantity_allocated = ?1,
                         installed_designators_json = ?2,
                         updated_at = datetime('now')
                     WHERE id = ?3",
                    (
                        line.quantity_required,
                        all_des_json.as_str(),
                        bom_line_id,
                    ),
                )
                .await?;
            }
        }
    }

    tx.commit().await?;

    get_bom_line(db, bom_line_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("BOM Line '{bom_line_id}' not found after transition"))
}

pub async fn allocate_bom_line_partial(
    db: &Db,
    bom_line_id: &str,
    target_status: &str,
    quantity_allocated: i64,
    source_lot_id_override: Option<&str>,
) -> anyhow::Result<BomLine> {
    let target = FulfillmentStatus::from_label(target_status)
        .ok_or_else(|| anyhow::anyhow!("Invalid Fulfillment Status '{target_status}'"))?;
    if !matches!(target, FulfillmentStatus::Reserved | FulfillmentStatus::Picked) {
        anyhow::bail!(
            "Partial allocation is only supported for Reserved or Picked status (got '{}')",
            target.as_str()
        );
    }
    let line = get_bom_line(db, bom_line_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("BOM Line '{bom_line_id}' not found"))?;
    if quantity_allocated <= 0 || quantity_allocated > line.quantity_required {
        anyhow::bail!(
            "Allocated quantity must be between 1 and {} (got {quantity_allocated})",
            line.quantity_required
        );
    }
    transition_bom_line_status(
        db,
        bom_line_id,
        target.as_str(),
        source_lot_id_override,
        Some(quantity_allocated),
    )
    .await
}

pub async fn split_bom_line(
    db: &Db,
    bom_line_id: &str,
    split_quantity: i64,
    split_designators_override: Option<&str>,
) -> anyhow::Result<(BomLine, BomLine)> {
    let original = get_bom_line(db, bom_line_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("BOM Line '{bom_line_id}' not found"))?;

    if original.quantity_required <= 1 {
        anyhow::bail!(
            "Cannot split BOM Line '{}' with quantity_required = {}",
            original.reference_designators,
            original.quantity_required
        );
    }
    if split_quantity <= 0 || split_quantity >= original.quantity_required {
        anyhow::bail!(
            "split_quantity must be between 1 and {} (got {split_quantity})",
            original.quantity_required - 1
        );
    }

    let remaining_qty = original.quantity_required - split_quantity;
    let sibling_qty = split_quantity;

    let orig_tokens = split_reference_designators(&original.reference_designators);
    let (new_orig_designators, sibling_designators) = if let Some(override_str) =
        split_designators_override
            .map(str::trim)
            .filter(|s| !s.is_empty())
    {
        let sibling_tokens = split_reference_designators(override_str);
        let remaining_tokens: Vec<String> = orig_tokens
            .iter()
            .filter(|tok| {
                !sibling_tokens
                    .iter()
                    .any(|st| st.eq_ignore_ascii_case(tok.as_str()))
            })
            .cloned()
            .collect();
        (remaining_tokens.join(", "), sibling_tokens.join(", "))
    } else if orig_tokens.len() as i64 > remaining_qty {
        let split_idx = remaining_qty as usize;
        (
            orig_tokens[..split_idx].join(", "),
            orig_tokens[split_idx..].join(", "),
        )
    } else if orig_tokens.len() >= 2 {
        let split_idx = orig_tokens
            .len()
            .saturating_sub((sibling_qty as usize).min(orig_tokens.len() - 1));
        (
            orig_tokens[..split_idx].join(", "),
            orig_tokens[split_idx..].join(", "),
        )
    } else {
        (
            original.reference_designators.clone(),
            original.reference_designators.clone(),
        )
    };

    let new_orig_allocated = if original.included_in_kit {
        remaining_qty
    } else {
        original.quantity_allocated.min(remaining_qty)
    };
    let excess_allocated = if original.included_in_kit {
        0
    } else {
        (original.quantity_allocated - new_orig_allocated).max(0)
    };
    let was_picked = original.fulfillment_status == "Picked";

    let new_orig_des_tokens = split_reference_designators(&new_orig_designators);
    let new_orig_installed: Vec<String> = original
        .installed_designators
        .iter()
        .filter(|d| {
            new_orig_des_tokens
                .iter()
                .any(|t| t.eq_ignore_ascii_case(d.as_str()))
        })
        .cloned()
        .collect();
    let new_orig_installed_json = serde_json::to_string(&new_orig_installed)?;

    let (sibling_status, sibling_allocated, sibling_included_int) = if original.included_in_kit {
        (
            FulfillmentStatus::IncludedInKit.as_str().to_string(),
            sibling_qty,
            1i64,
        )
    } else {
        (
            FulfillmentStatus::Unallocated.as_str().to_string(),
            0i64,
            0i64,
        )
    };

    let sibling_id = format!("bom-{}", Uuid::new_v4());
    let specs_json_str = serde_json::to_string(&original.specs_json)?;
    let sibling_sort_order = original.sort_order + 1;

    let mut conn = db.connect().await?;
    let tx = conn.transaction().await?;

    if excess_allocated > 0 && was_picked {
        restore_deducted_stock_tx(
            &tx,
            original.component_id.as_deref(),
            original.source_lot_id.as_deref(),
            excess_allocated,
        )
        .await?;
    }

    tx.execute(
        "UPDATE bom_lines
         SET sort_order = sort_order + 1
         WHERE kit_build_id = ?1 AND sort_order > ?2",
        (original.kit_build_id.as_str(), original.sort_order),
    )
    .await?;

    tx.execute(
        "UPDATE bom_lines
         SET quantity_required = ?1,
             quantity_allocated = ?2,
             reference_designators = ?3,
             installed_designators_json = ?4,
             updated_at = datetime('now')
         WHERE id = ?5",
        (
            remaining_qty,
            new_orig_allocated,
            new_orig_designators.as_str(),
            new_orig_installed_json.as_str(),
            bom_line_id,
        ),
    )
    .await?;

    tx.execute(
        "INSERT INTO bom_lines (
            id, kit_build_id, component_id, source_lot_id, raw_spec_text,
            canonical_value_display, category, package_or_mounting,
            resistance_ohms, capacitance_pf, inductance_uh, voltage_rating_volts,
            specs_json, quantity_required, quantity_allocated, fulfillment_status,
            included_in_kit, reference_designators, installed_designators_json,
            substitution_note, notes, sort_order
        ) VALUES (?1, ?2, ?3, NULL, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, '[]', ?18, ?19, ?20)",
        turso::params![
            sibling_id.as_str(),
            original.kit_build_id.as_str(),
            original.component_id.clone(),
            original.raw_spec_text.as_str(),
            original.canonical_value_display.as_str(),
            original.category.as_str(),
            original.package_or_mounting.as_str(),
            original.resistance_ohms,
            original.capacitance_pf,
            original.inductance_uh,
            original.voltage_rating_volts,
            specs_json_str.as_str(),
            sibling_qty,
            sibling_allocated,
            sibling_status.as_str(),
            sibling_included_int,
            sibling_designators.as_str(),
            original.substitution_note.as_str(),
            original.notes.as_str(),
            sibling_sort_order,
        ],
    )
    .await?;

    tx.commit().await?;

    let updated_orig = get_bom_line(db, bom_line_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Original BOM Line '{bom_line_id}' not found after split"))?;
    let created_sibling = get_bom_line(db, &sibling_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Sibling BOM Line '{sibling_id}' not found after split"))?;

    Ok((updated_orig, created_sibling))
}

pub async fn reserve_all_available(db: &Db, kit_build_id: &str) -> anyhow::Result<KitBuild> {
    let build = get_kit_build(db, kit_build_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Kit Build '{kit_build_id}' not found"))?;

    for line in &build.bom_lines {
        if line.fulfillment_status == "Unallocated"
            && let Some(ref cid) = line.component_id
            && let Some(comp) = get_component(db, cid).await?
            && comp.available_stock > 0
        {
            let _ = transition_bom_line_status(db, &line.id, "Reserved", None, None).await?;
        }
    }

    get_kit_build(db, kit_build_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Kit Build '{kit_build_id}' not found after reserve_all"))
}

pub async fn pick_all_reserved(db: &Db, kit_build_id: &str) -> anyhow::Result<KitBuild> {
    let build = get_kit_build(db, kit_build_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Kit Build '{kit_build_id}' not found"))?;

    for line in &build.bom_lines {
        if line.fulfillment_status == "Reserved"
            && line.quantity_allocated > 0
            && line.component_id.is_some()
        {
            let _ = transition_bom_line_status(
                db,
                &line.id,
                "Picked",
                line.source_lot_id.as_deref(),
                Some(line.quantity_allocated),
            )
            .await;
        }
    }

    get_kit_build(db, kit_build_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Kit Build '{kit_build_id}' not found after pick_all"))
}

pub async fn update_kit_build_status(
    db: &Db,
    kit_build_id: &str,
    new_status: &str,
    return_parts: bool,
) -> anyhow::Result<KitBuild> {
    let status_enum = KitBuildStatus::from_label(new_status)
        .ok_or_else(|| anyhow::anyhow!("Invalid Kit Build status '{new_status}'"))?;

    let build = get_kit_build(db, kit_build_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Kit Build '{kit_build_id}' not found"))?;

    let conn = db.connect().await?;
    conn.execute(
        "UPDATE kit_builds SET status = ?1, updated_at = datetime('now') WHERE id = ?2",
        (status_enum.as_str(), kit_build_id),
    )
    .await?;

    if return_parts || status_enum == KitBuildStatus::ShelvedAbandoned && return_parts {
        for line in &build.bom_lines {
            if matches!(line.fulfillment_status.as_str(), "Picked" | "Reserved") {
                let _ = transition_bom_line_status(db, &line.id, "Unallocated", None, None).await?;
            }
        }
    }

    get_kit_build(db, kit_build_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Kit Build '{kit_build_id}' not found after status update"))
}

pub async fn shelve_kit_build_and_return_parts(
    db: &Db,
    kit_build_id: &str,
) -> anyhow::Result<KitBuild> {
    update_kit_build_status(
        db,
        kit_build_id,
        KitBuildStatus::ShelvedAbandoned.as_str(),
        true,
    )
    .await
}

pub async fn create_kit_build(db: &Db, input: CreateKitBuild) -> anyhow::Result<KitBuild> {
    let name = input.name.trim();
    if name.is_empty() {
        anyhow::bail!("Kit Build name cannot be empty");
    }
    let status = match input.status.as_deref() {
        Some(s) if !s.trim().is_empty() => KitBuildStatus::from_label(s)
            .ok_or_else(|| anyhow::anyhow!("Invalid Kit Build status '{s}'"))?
            .as_str(),
        _ => KitBuildStatus::WishlistPlanned.as_str(),
    };
    let location_id = input
        .location_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    let conn = db.connect().await?;
    if let Some(ref loc_id) = location_id {
        let mut loc_rows = conn
            .query(
                "SELECT id FROM storage_locations WHERE id = ?1",
                (loc_id.as_str(),),
            )
            .await?;
        if loc_rows.next().await?.is_none() {
            anyhow::bail!("Storage Location '{loc_id}' does not exist");
        }
    }

    let id = format!("kit-{}", Uuid::new_v4());
    let project_type = input.project_type.trim();
    conn.execute(
        "INSERT INTO kit_builds (id, name, project_type, status, location_id, description)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (
            id.as_str(),
            name,
            project_type,
            status,
            location_id,
            input.description.trim(),
        ),
    )
    .await?;

    get_kit_build(db, &id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Failed to load created Kit Build '{id}'"))
}

pub async fn get_kit_build(db: &Db, id: &str) -> anyhow::Result<Option<KitBuild>> {
    let conn = db.connect().await?;
    let sql = format!("{SELECT_KIT_BUILD_SQL} WHERE kb.id = ?1");
    let mut rows = conn.query(&sql, (id,)).await?;
    if let Some(row) = rows.next().await? {
        Ok(Some(map_kit_build_row(db, &row).await?))
    } else {
        Ok(None)
    }
}

pub async fn list_kit_builds(db: &Db) -> anyhow::Result<Vec<KitBuild>> {
    let conn = db.connect().await?;
    let sql = format!("{SELECT_KIT_BUILD_SQL} ORDER BY kb.created_at ASC, kb.name ASC");
    let mut rows = conn.query(&sql, ()).await?;
    let mut out = Vec::new();
    while let Some(row) = rows.next().await? {
        out.push(map_kit_build_row(db, &row).await?);
    }
    Ok(out)
}

pub async fn update_kit_build(
    db: &Db,
    id: &str,
    input: UpdateKitBuild,
) -> anyhow::Result<KitBuild> {
    let existing = get_kit_build(db, id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Kit Build '{id}' not found"))?;

    let name = if input.name.trim().is_empty() {
        existing.name.as_str()
    } else {
        input.name.trim()
    };
    let status = KitBuildStatus::from_label(&input.status)
        .ok_or_else(|| anyhow::anyhow!("Invalid Kit Build status '{}'", input.status))?
        .as_str();
    let location_id = input
        .location_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    let conn = db.connect().await?;
    if let Some(ref loc_id) = location_id {
        let mut loc_rows = conn
            .query(
                "SELECT id FROM storage_locations WHERE id = ?1",
                (loc_id.as_str(),),
            )
            .await?;
        if loc_rows.next().await?.is_none() {
            anyhow::bail!("Storage Location '{loc_id}' does not exist");
        }
    }

    let project_type = input.project_type.trim();
    conn.execute(
        "UPDATE kit_builds
         SET name = ?1, project_type = ?2, status = ?3, location_id = ?4, description = ?5, updated_at = datetime('now')
         WHERE id = ?6",
        (
            name,
            project_type,
            status,
            location_id,
            input.description.trim(),
            id,
        ),
    )
    .await?;

    get_kit_build(db, id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Kit Build '{id}' not found after update"))
}

pub async fn clone_kit_build(
    db: &Db,
    source_kit_build_id: &str,
    new_name_override: Option<&str>,
) -> anyhow::Result<KitBuild> {
    let source = get_kit_build(db, source_kit_build_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Kit Build '{source_kit_build_id}' not found"))?;

    let cloned_name = match new_name_override.map(str::trim).filter(|s| !s.is_empty()) {
        Some(custom) => custom.to_string(),
        None => format!("{} (Copy)", source.name),
    };

    let mut conn = db.connect().await?;
    let mut att_rows = conn
        .query(
            "SELECT kind, original_filename, mime_type, relative_path, size_bytes, external_url
             FROM attachments
             WHERE kit_build_id = ?1 AND bench_log_entry_id IS NULL
             ORDER BY created_at ASC, id ASC",
            (source_kit_build_id,),
        )
        .await?;
    let mut shared_attachments = Vec::new();
    while let Some(row) = att_rows.next().await? {
        shared_attachments.push((
            row.get::<String>(0)?,
            row.get::<String>(1)?,
            row.get::<String>(2)?,
            row.get::<String>(3)?,
            row.get::<i64>(4)?,
            row.get::<String>(5)?,
        ));
    }

    let cloned_id = format!("kit-{}", Uuid::new_v4());
    let tx = conn.transaction().await?;

    tx.execute(
        "INSERT INTO kit_builds (
            id, name, project_type, status, location_id, description
        ) VALUES (?1, ?2, ?3, ?4, NULL, ?5)",
        (
            cloned_id.as_str(),
            cloned_name.as_str(),
            source.project_type.as_str(),
            KitBuildStatus::WishlistPlanned.as_str(),
            source.description.as_str(),
        ),
    )
    .await?;

    for (kind, orig_name, mime, rel_path, size_bytes, ext_url) in shared_attachments {
        let new_att_id = format!("att-{}", Uuid::new_v4());
        tx.execute(
            "INSERT INTO attachments (
                id, kit_build_id, bench_log_entry_id, kind, original_filename, mime_type, relative_path, size_bytes, external_url
            ) VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?6, ?7, ?8)",
            (
                new_att_id.as_str(),
                cloned_id.as_str(),
                kind.as_str(),
                orig_name.as_str(),
                mime.as_str(),
                rel_path.as_str(),
                size_bytes,
                ext_url.as_str(),
            ),
        )
        .await?;
    }

    for line in &source.bom_lines {
        let new_line_id = format!("bom-{}", Uuid::new_v4());
        let specs_json_str = serde_json::to_string(&line.specs_json)?;
        let (status_str, qty_alloc, inc_int) = if line.included_in_kit {
            (
                FulfillmentStatus::IncludedInKit.as_str(),
                line.quantity_required,
                1i64,
            )
        } else {
            (FulfillmentStatus::Unallocated.as_str(), 0i64, 0i64)
        };

        tx.execute(
            "INSERT INTO bom_lines (
                id, kit_build_id, component_id, source_lot_id, raw_spec_text,
                canonical_value_display, category, package_or_mounting,
                resistance_ohms, capacitance_pf, inductance_uh, voltage_rating_volts,
                specs_json, quantity_required, quantity_allocated, fulfillment_status,
                included_in_kit, reference_designators, installed_designators_json,
                substitution_note, notes, sort_order
            ) VALUES (?1, ?2, ?3, NULL, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, '[]', ?18, ?19, ?20)",
            turso::params![
                new_line_id.as_str(),
                cloned_id.as_str(),
                line.component_id.clone(),
                line.raw_spec_text.as_str(),
                line.canonical_value_display.as_str(),
                line.category.as_str(),
                line.package_or_mounting.as_str(),
                line.resistance_ohms,
                line.capacitance_pf,
                line.inductance_uh,
                line.voltage_rating_volts,
                specs_json_str.as_str(),
                line.quantity_required,
                qty_alloc,
                status_str,
                inc_int,
                line.reference_designators.as_str(),
                line.substitution_note.as_str(),
                line.notes.as_str(),
                line.sort_order,
            ],
        )
        .await?;
    }

    tx.commit().await?;

    get_kit_build(db, &cloned_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Cloned Kit Build '{cloned_id}' not found"))
}

pub async fn mark_all_bom_line_designators_installed(
    db: &Db,
    bom_line_id: &str,
) -> anyhow::Result<BomLine> {
    transition_bom_line_status(db, bom_line_id, "Installed", None, None).await
}

pub async fn toggle_bom_line_designator(
    db: &Db,
    bom_line_id: &str,
    designator: &str,
) -> anyhow::Result<BomLine> {
    let target_des = designator.trim();
    if target_des.is_empty() {
        anyhow::bail!("Designator cannot be empty");
    }

    let line = get_bom_line(db, bom_line_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("BOM Line '{bom_line_id}' not found"))?;

    let placements = bom_line_placements(&line);
    let mut current: Vec<String> =
        if line.fulfillment_status == "Installed" && line.installed_designators.is_empty() {
            placements.clone()
        } else {
            line.installed_designators.clone()
        };

    if let Some(pos) = current.iter().position(|d| d == target_des) {
        current.remove(pos);
    } else {
        current.push(target_des.to_string());
    }

    let mut ordered_installed: Vec<String> = placements
        .iter()
        .filter(|p| current.contains(*p))
        .cloned()
        .collect();
    for extra in &current {
        if !ordered_installed.contains(extra) {
            ordered_installed.push(extra.clone());
        }
    }

    let all_checked =
        !placements.is_empty() && placements.iter().all(|p| ordered_installed.contains(p));

    if all_checked {
        return transition_bom_line_status(db, bom_line_id, "Installed", None, None).await;
    }

    let json_str = serde_json::to_string(&ordered_installed)?;
    let conn = db.connect().await?;

    conn.execute(
        "UPDATE bom_lines
         SET installed_designators_json = ?1,
             updated_at = datetime('now')
         WHERE id = ?2",
        (json_str.as_str(), bom_line_id),
    )
    .await?;

    get_bom_line(db, bom_line_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("BOM Line '{bom_line_id}' not found after designator toggle"))
}

pub use crate::attachments::sanitize_attachment_filename;
use crate::attachments::resolve_effective_data_dir_path as resolve_attachment_base_dir;

fn map_attachment_row(row: &turso::Row) -> anyhow::Result<BuildAttachment> {
    Ok(BuildAttachment {
        id: row.get::<String>(0)?,
        kit_build_id: row.get::<String>(1)?,
        bench_log_entry_id: row.get::<Option<String>>(2)?,
        kind: row.get::<String>(3)?,
        original_filename: row.get::<String>(4)?,
        mime_type: row.get::<String>(5)?,
        relative_path: row.get::<String>(6)?,
        size_bytes: row.get::<i64>(7)?,
        external_url: row.get::<String>(8)?,
        created_at: row.get::<String>(9)?,
    })
}

const SELECT_ATTACHMENT_SQL: &str = "
SELECT
    id,
    kit_build_id,
    bench_log_entry_id,
    kind,
    original_filename,
    mime_type,
    relative_path,
    size_bytes,
    external_url,
    created_at
FROM attachments
";

#[allow(clippy::too_many_arguments)]
pub async fn add_build_attachment_file_in_dir(
    db: &Db,
    data_dir: &Path,
    kit_build_id: &str,
    bench_log_entry_id: Option<&str>,
    kind: &str,
    original_filename: &str,
    mime_type: &str,
    bytes: &[u8],
) -> anyhow::Result<BuildAttachment> {
    if get_kit_build(db, kit_build_id).await?.is_none() {
        anyhow::bail!("Kit Build '{kit_build_id}' does not exist");
    }

    let normalized_kind = AttachmentKind::normalize_label(kind);
    let orig_name = if original_filename.trim().is_empty() {
        "attachment.bin"
    } else {
        original_filename.trim()
    };
    let sanitized = sanitize_attachment_filename(orig_name);
    let file_uuid = Uuid::new_v4();
    let relative_path = format!("attachments/{kit_build_id}/{file_uuid}-{sanitized}");

    let base_dir = resolve_attachment_base_dir(data_dir);
    let full_path = base_dir.join(&relative_path);
    if let Some(parent) = full_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&full_path, bytes)?;

    let id = format!("att-{}", Uuid::new_v4());
    let log_id = bench_log_entry_id
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let mime = if mime_type.trim().is_empty() {
        "application/octet-stream"
    } else {
        mime_type.trim()
    };

    let conn = db.connect().await?;
    conn.execute(
        "INSERT INTO attachments (
            id, kit_build_id, bench_log_entry_id, kind, original_filename,
            mime_type, relative_path, size_bytes, external_url
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, '')",
        (
            id.as_str(),
            kit_build_id,
            log_id,
            normalized_kind,
            orig_name,
            mime,
            relative_path.as_str(),
            bytes.len() as i64,
        ),
    )
    .await?;

    get_build_attachment(db, &id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Failed to load created attachment '{id}'"))
}

pub async fn add_build_attachment_file(
    db: &Db,
    kit_build_id: &str,
    bench_log_entry_id: Option<&str>,
    kind: &str,
    original_filename: &str,
    mime_type: &str,
    bytes: &[u8],
) -> anyhow::Result<BuildAttachment> {
    add_build_attachment_file_in_dir(
        db,
        db.data_dir(),
        kit_build_id,
        bench_log_entry_id,
        kind,
        original_filename,
        mime_type,
        bytes,
    )
    .await
}

pub fn read_build_attachment_bytes_in_dir(
    data_dir: &Path,
    attachment: &BuildAttachment,
) -> anyhow::Result<Vec<u8>> {
    if attachment.relative_path.trim().is_empty() {
        anyhow::bail!(
            "Attachment '{}' is an external link with no local file",
            attachment.id
        );
    }
    let base_dir = resolve_attachment_base_dir(data_dir);
    let full_path = base_dir.join(&attachment.relative_path);
    Ok(std::fs::read(full_path)?)
}

pub fn read_build_attachment_bytes(
    db: &Db,
    attachment: &BuildAttachment,
) -> anyhow::Result<Vec<u8>> {
    read_build_attachment_bytes_in_dir(db.data_dir(), attachment)
}

pub async fn add_build_external_link(
    db: &Db,
    kit_build_id: &str,
    kind: Option<&str>,
    label: &str,
    external_url: &str,
) -> anyhow::Result<BuildAttachment> {
    if get_kit_build(db, kit_build_id).await?.is_none() {
        anyhow::bail!("Kit Build '{kit_build_id}' does not exist");
    }
    let url = external_url.trim();
    if url.is_empty() {
        anyhow::bail!("External URL cannot be empty");
    }

    let display_label = if label.trim().is_empty() {
        url
    } else {
        label.trim()
    };
    let normalized_kind = kind
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(AttachmentKind::normalize_label)
        .unwrap_or("External URL");

    let id = format!("att-{}", Uuid::new_v4());
    let conn = db.connect().await?;
    conn.execute(
        "INSERT INTO attachments (
            id, kit_build_id, bench_log_entry_id, kind, original_filename,
            mime_type, relative_path, size_bytes, external_url
        ) VALUES (?1, ?2, NULL, ?3, ?4, 'text/uri-list', '', 0, ?5)",
        (id.as_str(), kit_build_id, normalized_kind, display_label, url),
    )
    .await?;

    get_build_attachment(db, &id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Failed to load created external link '{id}'"))
}

pub async fn get_build_attachment(
    db: &Db,
    attachment_id: &str,
) -> anyhow::Result<Option<BuildAttachment>> {
    let conn = db.connect().await?;
    let sql = format!("{SELECT_ATTACHMENT_SQL} WHERE id = ?1");
    let mut rows = conn.query(&sql, (attachment_id,)).await?;
    if let Some(row) = rows.next().await? {
        Ok(Some(map_attachment_row(&row)?))
    } else {
        Ok(None)
    }
}

pub async fn list_build_attachments(
    db: &Db,
    kit_build_id: &str,
) -> anyhow::Result<Vec<BuildAttachment>> {
    let conn = db.connect().await?;
    let sql = format!(
        "{SELECT_ATTACHMENT_SQL} WHERE kit_build_id = ?1 ORDER BY created_at ASC, rowid ASC"
    );
    let mut rows = conn.query(&sql, (kit_build_id,)).await?;
    let mut out = Vec::new();
    while let Some(row) = rows.next().await? {
        out.push(map_attachment_row(&row)?);
    }
    Ok(out)
}

pub async fn create_bench_log_entry(
    db: &Db,
    input: CreateBenchLogEntry,
) -> anyhow::Result<BenchLogEntry> {
    if get_kit_build(db, &input.kit_build_id).await?.is_none() {
        anyhow::bail!("Kit Build '{}' does not exist", input.kit_build_id);
    }
    let body = input.body.trim();
    if body.is_empty() {
        anyhow::bail!("Bench Log Entry body cannot be empty");
    }
    let entry_type = if input.entry_type.trim().is_empty() {
        BenchLogEntryType::Note
    } else {
        BenchLogEntryType::from_label(&input.entry_type).ok_or_else(|| {
            anyhow::anyhow!("Invalid Bench Log Entry type '{}'", input.entry_type)
        })?
    };

    let id = format!("log-{}", Uuid::new_v4());
    let conn = db.connect().await?;
    conn.execute(
        "INSERT INTO bench_log_entries (id, kit_build_id, entry_type, body)
         VALUES (?1, ?2, ?3, ?4)",
        (
            id.as_str(),
            input.kit_build_id.as_str(),
            entry_type.as_str(),
            body,
        ),
    )
    .await?;

    get_bench_log_entry(db, &id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Failed to load created Bench Log Entry '{id}'"))
}

pub async fn get_bench_log_entry(db: &Db, id: &str) -> anyhow::Result<Option<BenchLogEntry>> {
    let conn = db.connect().await?;
    let mut rows = conn
        .query(
            "SELECT id, kit_build_id, entry_type, body, created_at FROM bench_log_entries WHERE id = ?1",
            (id,),
        )
        .await?;
    if let Some(row) = rows.next().await? {
        let entry_id = row.get::<String>(0)?;
        let kit_build_id = row.get::<String>(1)?;
        let entry_type = row.get::<String>(2)?;
        let body = row.get::<String>(3)?;
        let created_at = row.get::<String>(4)?;

        let all_atts = list_build_attachments(db, &kit_build_id).await?;
        let attachments = all_atts
            .into_iter()
            .filter(|a| a.bench_log_entry_id.as_deref() == Some(entry_id.as_str()))
            .collect();

        Ok(Some(BenchLogEntry {
            id: entry_id,
            kit_build_id,
            entry_type,
            body,
            created_at,
            attachments,
        }))
    } else {
        Ok(None)
    }
}

pub async fn list_bench_log_entries(
    db: &Db,
    kit_build_id: &str,
) -> anyhow::Result<Vec<BenchLogEntry>> {
    let all_atts = list_build_attachments(db, kit_build_id).await?;
    let conn = db.connect().await?;
    let mut rows = conn
        .query(
            "SELECT id, kit_build_id, entry_type, body, created_at
             FROM bench_log_entries
             WHERE kit_build_id = ?1
             ORDER BY created_at ASC, rowid ASC",
            (kit_build_id,),
        )
        .await?;
    let mut out = Vec::new();
    while let Some(row) = rows.next().await? {
        let entry_id = row.get::<String>(0)?;
        let kb_id = row.get::<String>(1)?;
        let entry_type = row.get::<String>(2)?;
        let body = row.get::<String>(3)?;
        let created_at = row.get::<String>(4)?;
        let attachments = all_atts
            .iter()
            .filter(|a| a.bench_log_entry_id.as_deref() == Some(entry_id.as_str()))
            .cloned()
            .collect();
        out.push(BenchLogEntry {
            id: entry_id,
            kit_build_id: kb_id,
            entry_type,
            body,
            created_at,
            attachments,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{CreateStorageLocation, create_storage_location, move_storage_location};
    use crate::db::init_memory_db;

    #[tokio::test]
    async fn creates_and_updates_kit_builds_across_all_lifecycle_statuses_and_tracks_storage_location_breadcrumbs()
     {
        let db = init_memory_db().await.expect("memory db");

        let shelf = create_storage_location(
            &db,
            CreateStorageLocation {
                name: "Workbench Shelf".into(),
                kind: "Crate".into(),
                parent_id: None,
                notes: "".into(),
            },
        )
        .await
        .unwrap();
        let project_box = create_storage_location(
            &db,
            CreateStorageLocation {
                name: "Project Box #4".into(),
                kind: "Box".into(),
                parent_id: Some(shelf.id.clone()),
                notes: "Klon kit box".into(),
            },
        )
        .await
        .unwrap();

        let build = create_kit_build(
            &db,
            CreateKitBuild {
                name: "Klon Centaur Overdrive".into(),
                project_type: "Guitar Pedal".into(),
                status: None,
                location_id: Some(project_box.id.clone()),
                description: "Transparent overdrive build".into(),
            },
        )
        .await
        .expect("create kit build");

        assert_eq!(build.name, "Klon Centaur Overdrive");
        assert_eq!(build.project_type, "Guitar Pedal");
        assert_eq!(build.status, "Wishlist / Planned");
        assert_eq!(build.location_id.as_deref(), Some(project_box.id.as_str()));
        assert_eq!(
            build.location_breadcrumb,
            "Workbench Shelf > Project Box #4"
        );
        assert_eq!(build.readiness.total_lines, 0);
        assert_eq!(build.readiness.ready_lines, 0);

        // Transition across all 6 lifecycle statuses + Shelved / Abandoned
        for status in KitBuildStatus::ALL {
            let updated = update_kit_build(
                &db,
                &build.id,
                UpdateKitBuild {
                    name: build.name.clone(),
                    project_type: build.project_type.clone(),
                    status: status.as_str().to_string(),
                    location_id: build.location_id.clone(),
                    description: build.description.clone(),
                },
            )
            .await
            .expect("update status");
            assert_eq!(updated.status, status.as_str());
        }

        // Moving the parent container updates the Kit Build's location_breadcrumb automatically
        let closet = create_storage_location(
            &db,
            CreateStorageLocation {
                name: "Closet Rack".into(),
                kind: "Crate".into(),
                parent_id: None,
                notes: "".into(),
            },
        )
        .await
        .unwrap();
        move_storage_location(&db, &project_box.id, Some(&closet.id))
            .await
            .unwrap();

        let fetched = get_kit_build(&db, &build.id)
            .await
            .expect("get build")
            .expect("exists");
        assert_eq!(fetched.location_breadcrumb, "Closet Rack > Project Box #4");

        let all_builds = list_kit_builds(&db).await.expect("list builds");
        assert_eq!(all_builds.len(), 1);
        assert_eq!(all_builds[0].id, build.id);
    }

    #[tokio::test]
    async fn bom_line_creation_auto_links_single_exact_match_and_leaves_ambiguous_or_missing_specs_unlinked_with_candidates_and_one_click_create()
     {
        use crate::catalog::{CreateComponent, CreateInitialStockLot, create_component};

        let db = init_memory_db().await.expect("memory db");
        let drawer = create_storage_location(
            &db,
            CreateStorageLocation {
                name: "Drawer 1".into(),
                kind: "Drawer".into(),
                parent_id: None,
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        // 1. Single exact parametric match in catalog: 4.7kΩ 1% 1/4W Metal Film Resistor
        let r_4k7 = create_component(
            &db,
            CreateComponent {
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
                initial_lot: Some(CreateInitialStockLot {
                    location_id: Some(drawer.id.clone()),
                    quantity: 50,
                    supplier: "Tayda".into(),
                    supplier_part_no: "A-2214".into(),
                    lot_notes: "".into(),
                }),
            },
        )
        .await
        .unwrap();

        // 2. Two competing 47nF film capacitors in catalog (WIMA 63V and Panasonic 100V)
        let cap_wima = create_component(
            &db,
            CreateComponent {
                name: "47nF 63V WIMA MKS2 Box Film".into(),
                category: "Capacitors".into(),
                subtype: "Film".into(),
                package_or_mounting: "Through-Hole 5mm".into(),
                raw_spec_text: "0.047uF 63V 5% Box Film 5mm".into(),
                canonical_value_display: None,
                resistance_ohms: None,
                capacitance_pf: None,
                inductance_uh: None,
                voltage_rating_volts: None,
                low_stock_threshold: 5,
                datasheet_url: "".into(),
                notes: "".into(),
                specs_json: None,
                initial_lot: Some(CreateInitialStockLot {
                    location_id: Some(drawer.id.clone()),
                    quantity: 12,
                    supplier: "Mouser".into(),
                    supplier_part_no: "WIMA-47N".into(),
                    lot_notes: "".into(),
                }),
            },
        )
        .await
        .unwrap();

        let cap_pana = create_component(
            &db,
            CreateComponent {
                name: "47nF 100V Panasonic ECQ Film".into(),
                category: "Capacitors".into(),
                subtype: "Film".into(),
                package_or_mounting: "Through-Hole 5mm".into(),
                raw_spec_text: "47n 100V 5% Polyester Film 5mm".into(),
                canonical_value_display: None,
                resistance_ohms: None,
                capacitance_pf: None,
                inductance_uh: None,
                voltage_rating_volts: None,
                low_stock_threshold: 5,
                datasheet_url: "".into(),
                notes: "".into(),
                specs_json: None,
                initial_lot: Some(CreateInitialStockLot {
                    location_id: Some(drawer.id.clone()),
                    quantity: 4,
                    supplier: "DigiKey".into(),
                    supplier_part_no: "PANA-47N".into(),
                    lot_notes: "".into(),
                }),
            },
        )
        .await
        .unwrap();

        let build = create_kit_build(
            &db,
            CreateKitBuild {
                name: "Tube Screamer TS808".into(),
                project_type: "Overdrive Pedal".into(),
                status: None,
                location_id: None,
                description: "".into(),
            },
        )
        .await
        .unwrap();

        // Case A: Add BOM Line with shorthand "4.7k" and designators "R1, R2" (category inferred as Resistors)
        // Because only one 4.7kΩ resistor exists in the catalog, it auto-links to r_4k7!
        let line_r = add_bom_line(
            &db,
            CreateBomLine {
                kit_build_id: build.id.clone(),
                raw_spec_text: "4.7k".into(),
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
        .expect("add R1, R2 line");

        assert_eq!(line_r.category, "Resistors");
        assert_eq!(line_r.canonical_value_display, "4.7kΩ");
        assert_eq!(line_r.resistance_ohms, Some(4700.0));
        assert_eq!(line_r.component_id.as_deref(), Some(r_4k7.id.as_str()));
        assert_eq!(line_r.fulfillment_status, "Unallocated");
        assert_eq!(line_r.quantity_allocated, 0);

        // Case B: Add BOM Line with shorthand "0.047uF Film" and designators "C3, C4"
        // Because two 47nF capacitors exist in the catalog, component_id remains None (Unlinked)
        // and find_bom_line_candidates returns both ranked candidates!
        let line_c = add_bom_line(
            &db,
            CreateBomLine {
                kit_build_id: build.id.clone(),
                raw_spec_text: "0.047uF Film".into(),
                category: None,
                package_or_mounting: None,
                quantity_required: 2,
                reference_designators: "C3, C4".into(),
                included_in_kit: false,
                component_id: None,
                substitution_note: "".into(),
                notes: "".into(),
            },
        )
        .await
        .expect("add C3, C4 line");

        assert_eq!(line_c.category, "Capacitors");
        assert_eq!(line_c.canonical_value_display, "47nF");
        assert_eq!(line_c.capacitance_pf, Some(47_000.0));
        assert!(
            line_c.component_id.is_none(),
            "ambiguous match must remain Unlinked"
        );

        let candidates = find_bom_line_candidates(&db, &line_c.id)
            .await
            .expect("find candidates");
        assert_eq!(candidates.len(), 2);
        let candidate_ids: Vec<&str> = candidates
            .iter()
            .map(|c| c.component.id.as_str())
            .collect();
        assert!(candidate_ids.contains(&cap_wima.id.as_str()));
        assert!(candidate_ids.contains(&cap_pana.id.as_str()));

        // Builder can link a candidate in 1 click
        let linked_c = link_bom_line_component(&db, &line_c.id, Some(&cap_wima.id), None)
            .await
            .expect("link candidate");
        assert_eq!(linked_c.component_id.as_deref(), Some(cap_wima.id.as_str()));

        // Case C: Add BOM Line with zero matches in catalog ("500kA 16mm PCB", designator "VR1")
        let line_pot = add_bom_line(
            &db,
            CreateBomLine {
                kit_build_id: build.id.clone(),
                raw_spec_text: "500kA 16mm PCB".into(),
                category: None,
                package_or_mounting: None,
                quantity_required: 1,
                reference_designators: "VR1".into(),
                included_in_kit: false,
                component_id: None,
                substitution_note: "".into(),
                notes: "Drive pot".into(),
            },
        )
        .await
        .expect("add VR1 line");

        assert_eq!(line_pot.category, "Potentiometers & Encoders");
        assert_eq!(line_pot.canonical_value_display, "500kΩ");
        assert!(line_pot.component_id.is_none());
        let pot_candidates = find_bom_line_candidates(&db, &line_pot.id).await.unwrap();
        assert!(pot_candidates.is_empty());

        // 1-click "Create Component from BOM Line" creates the catalog component and links the BOM line
        let (created_comp, updated_pot_line) =
            create_component_from_bom_line(&db, &line_pot.id, Some(drawer.id.clone()), 3)
                .await
                .expect("create component from bom line");
        assert_eq!(created_comp.category, "Potentiometers & Encoders");
        assert_eq!(created_comp.canonical_value_display, "500kΩ");
        assert_eq!(created_comp.resistance_ohms, Some(500_000.0));
        assert_eq!(created_comp.specs_json["taper"], "A");
        assert_eq!(created_comp.total_on_hand, 3);
        assert_eq!(
            updated_pot_line.component_id.as_deref(),
            Some(created_comp.id.as_str())
        );

        // Case D: Included in Kit line never links to catalog or counts as a shortfall
        let line_kit = add_bom_line(
            &db,
            CreateBomLine {
                kit_build_id: build.id.clone(),
                raw_spec_text: "TS808 PCB + Pre-Drilled Enclosure".into(),
                category: Some("Enclosures, Knobs & Mechanical Hardware".into()),
                package_or_mounting: None,
                quantity_required: 1,
                reference_designators: "PCB1, ENC1".into(),
                included_in_kit: true,
                component_id: None,
                substitution_note: "".into(),
                notes: "Supplied in kit box".into(),
            },
        )
        .await
        .expect("add kit line");
        assert_eq!(line_kit.fulfillment_status, "Included in Kit");
        assert!(line_kit.included_in_kit);
        assert_eq!(line_kit.quantity_allocated, 1);
        assert_eq!(line_kit.shortfall_quantity, 0);
        assert!(line_kit.component_id.is_none());
    }

    #[tokio::test]
    async fn transitions_bom_line_across_five_fulfillment_statuses_enforcing_available_stock_and_source_lot_invariants()
     {
        use crate::catalog::{
            CreateComponent, CreateInitialStockLot, CreateStockLot, create_component,
            get_component, upsert_or_merge_stock_lot,
        };

        let db = init_memory_db().await.expect("memory db");
        let binder = create_storage_location(
            &db,
            CreateStorageLocation {
                name: "Binder A".into(),
                kind: "Binder".into(),
                parent_id: None,
                notes: "".into(),
            },
        )
        .await
        .unwrap();
        let crate_b = create_storage_location(
            &db,
            CreateStorageLocation {
                name: "Crate B".into(),
                kind: "Crate".into(),
                parent_id: None,
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        // Create TL072IP with two Stock Lots: 6 in Binder A, 4 in Crate B (Total = 10)
        let ic = create_component(
            &db,
            CreateComponent {
                name: "TL072IP Dual Op-Amp".into(),
                category: "Integrated Circuits (ICs)".into(),
                subtype: "Op-Amp".into(),
                package_or_mounting: "DIP-8".into(),
                raw_spec_text: "TL072IP Dual Op-Amp DIP-8".into(),
                canonical_value_display: None,
                resistance_ohms: None,
                capacitance_pf: None,
                inductance_uh: None,
                voltage_rating_volts: None,
                low_stock_threshold: 2,
                datasheet_url: "".into(),
                notes: "".into(),
                specs_json: None,
                initial_lot: Some(CreateInitialStockLot {
                    location_id: Some(binder.id.clone()),
                    quantity: 6,
                    supplier: "Mouser".into(),
                    supplier_part_no: "TL072-M".into(),
                    lot_notes: "".into(),
                }),
            },
        )
        .await
        .unwrap();
        let lot_binder_id = ic.lots[0].id.clone();

        let lot_crate = upsert_or_merge_stock_lot(
            &db,
            CreateStockLot {
                component_id: ic.id.clone(),
                location_id: Some(crate_b.id.clone()),
                quantity: 4,
                supplier: "Tayda".into(),
                supplier_part_no: "TL072-T".into(),
                lot_notes: "".into(),
            },
        )
        .await
        .unwrap();
        let lot_crate_id = lot_crate.id;

        let build = create_kit_build(
            &db,
            CreateKitBuild {
                name: "Klon Centaur".into(),
                project_type: "Overdrive".into(),
                status: Some("Sourcing / Waiting on Parts".into()),
                location_id: None,
                description: "".into(),
            },
        )
        .await
        .unwrap();

        let bom_ic = add_bom_line(
            &db,
            CreateBomLine {
                kit_build_id: build.id.clone(),
                raw_spec_text: "TL072IP DIP-8".into(),
                category: Some("Integrated Circuits (ICs)".into()),
                package_or_mounting: Some("DIP-8".into()),
                quantity_required: 2,
                reference_designators: "IC1, IC2".into(),
                included_in_kit: false,
                component_id: None,
                substitution_note: "".into(),
                notes: "".into(),
            },
        )
        .await
        .unwrap();
        assert_eq!(bom_ic.component_id.as_deref(), Some(ic.id.as_str()));
        assert_eq!(bom_ic.fulfillment_status, "Unallocated");
        assert_eq!(bom_ic.quantity_allocated, 0);

        // 1. Unallocated -> Reserved:
        // Soft-reserves 2 in place: Available Stock drops from 10 to 8, while physical Stock Lot quantities stay 6 + 4 = 10!
        let reserved = transition_bom_line_status(&db, &bom_ic.id, "Reserved", None, None)
            .await
            .expect("transition to Reserved");
        assert_eq!(reserved.fulfillment_status, "Reserved");
        assert_eq!(reserved.quantity_allocated, 2);
        assert_eq!(reserved.shortfall_quantity, 0);

        let ic_after_reserve = get_component(&db, &ic.id).await.unwrap().unwrap();
        assert_eq!(ic_after_reserve.total_on_hand, 10);
        assert_eq!(ic_after_reserve.reserved_quantity, 2);
        assert_eq!(ic_after_reserve.available_stock, 8);

        // 2. Reserved -> Picked (explicitly picking from lot_crate_id):
        // Deducts 2 from lot_crate_id (4 -> 2), records source_lot_id = lot_crate_id.
        // Total on hand is now 6 + 2 = 8, Reserved is 0, Available Stock remains 8!
        let picked =
            transition_bom_line_status(&db, &bom_ic.id, "Picked", Some(&lot_crate_id), None)
                .await
                .expect("transition to Picked");
        assert_eq!(picked.fulfillment_status, "Picked");
        assert_eq!(picked.quantity_allocated, 2);
        assert_eq!(picked.source_lot_id.as_deref(), Some(lot_crate_id.as_str()));
        assert_eq!(
            picked.source_lot_location_breadcrumb.as_deref(),
            Some("Crate B")
        );

        let ic_after_pick = get_component(&db, &ic.id).await.unwrap().unwrap();
        assert_eq!(ic_after_pick.total_on_hand, 8);
        assert_eq!(ic_after_pick.reserved_quantity, 0);
        assert_eq!(ic_after_pick.available_stock, 8);
        let crate_lot_after_pick = ic_after_pick
            .lots
            .iter()
            .find(|l| l.id == lot_crate_id)
            .unwrap();
        assert_eq!(crate_lot_after_pick.quantity, 2);

        // 3. Picked -> Reserved (un-picking back to soft reservation):
        // Restores 2 back onto lot_crate_id (2 -> 4), total_on_hand becomes 10, reserved_quantity becomes 2, available_stock is 8!
        let unpicked_to_reserved =
            transition_bom_line_status(&db, &bom_ic.id, "Reserved", None, None)
                .await
                .expect("transition Picked -> Reserved");
        assert_eq!(unpicked_to_reserved.fulfillment_status, "Reserved");
        assert_eq!(unpicked_to_reserved.quantity_allocated, 2);

        let ic_after_unpick_res = get_component(&db, &ic.id).await.unwrap().unwrap();
        assert_eq!(ic_after_unpick_res.total_on_hand, 10);
        assert_eq!(ic_after_unpick_res.reserved_quantity, 2);
        assert_eq!(ic_after_unpick_res.available_stock, 8);
        let crate_lot_restored = ic_after_unpick_res
            .lots
            .iter()
            .find(|l| l.id == lot_crate_id)
            .unwrap();
        assert_eq!(crate_lot_restored.quantity, 4);

        // 4. Reserved -> Picked (default lot selection picks from primary/largest lot: Binder A with 6 -> 4)
        // and then Picked -> Unallocated (restores 2 back onto Binder A: 4 -> 6, releases reservation)
        let picked_default = transition_bom_line_status(&db, &bom_ic.id, "Picked", None, None)
            .await
            .expect("transition Reserved -> Picked default lot");
        assert_eq!(picked_default.fulfillment_status, "Picked");
        assert_eq!(
            picked_default.source_lot_id.as_deref(),
            Some(lot_binder_id.as_str())
        );

        let unallocated = transition_bom_line_status(&db, &bom_ic.id, "Unallocated", None, None)
            .await
            .expect("transition Picked -> Unallocated");
        assert_eq!(unallocated.fulfillment_status, "Unallocated");
        assert_eq!(unallocated.quantity_allocated, 0);
        assert!(unallocated.source_lot_id.is_none());

        let ic_after_unalloc = get_component(&db, &ic.id).await.unwrap().unwrap();
        assert_eq!(ic_after_unalloc.total_on_hand, 10);
        assert_eq!(ic_after_unalloc.reserved_quantity, 0);
        assert_eq!(ic_after_unalloc.available_stock, 10);

        // 5. Picked -> Included in Kit: restores Picked stock back to source_lot_id and marks Included in Kit
        let picked_again =
            transition_bom_line_status(&db, &bom_ic.id, "Picked", Some(&lot_binder_id), None)
                .await
                .unwrap();
        assert_eq!(picked_again.quantity_allocated, 2);
        assert_eq!(
            get_component(&db, &ic.id).await.unwrap().unwrap().total_on_hand,
            8
        );

        let as_kit = transition_bom_line_status(&db, &bom_ic.id, "Included in Kit", None, None)
            .await
            .expect("transition Picked -> Included in Kit");
        assert_eq!(as_kit.fulfillment_status, "Included in Kit");
        assert!(as_kit.included_in_kit);
        assert_eq!(as_kit.quantity_allocated, 2);
        assert!(as_kit.source_lot_id.is_none());
        let ic_after_kit = get_component(&db, &ic.id).await.unwrap().unwrap();
        assert_eq!(ic_after_kit.total_on_hand, 10);
        assert_eq!(ic_after_kit.available_stock, 10);

        // 6. Picked -> Installed: does NOT double-deduct from stock_lots, marks designators installed
        let _ = transition_bom_line_status(&db, &bom_ic.id, "Picked", Some(&lot_binder_id), None)
            .await
            .unwrap();
        let installed = transition_bom_line_status(&db, &bom_ic.id, "Installed", None, None)
            .await
            .expect("transition Picked -> Installed");
        assert_eq!(installed.fulfillment_status, "Installed");
        assert_eq!(installed.quantity_allocated, 2);
        assert_eq!(
            installed.source_lot_id.as_deref(),
            Some(lot_binder_id.as_str())
        );
        assert_eq!(installed.installed_designators, vec!["IC1", "IC2"]);

        let ic_after_install = get_component(&db, &ic.id).await.unwrap().unwrap();
        assert_eq!(ic_after_install.total_on_hand, 8);
        assert_eq!(ic_after_install.available_stock, 8);

        // 7. Installed parts are permanently consumed: transitioning Installed -> Unallocated/Reserved/Included in Kit
        //    must NOT refund stock_lots.quantity
        let reset_unalloc = transition_bom_line_status(&db, &bom_ic.id, "Unallocated", None, None)
            .await
            .unwrap();
        assert_eq!(reset_unalloc.quantity_allocated, 0);
        let ic_after_installed_unalloc = get_component(&db, &ic.id).await.unwrap().unwrap();
        assert_eq!(
            ic_after_installed_unalloc.total_on_hand, 8,
            "Installed (soldered) parts must remain permanently consumed and never refund stock"
        );

        // 8. Direct Unallocated -> Installed deducts from source_lot_id once (8 -> 6), and remains consumed on status change
        let direct_install = transition_bom_line_status(
            &db,
            &bom_ic.id,
            "Installed",
            Some(&lot_crate_id),
            None,
        )
        .await
        .expect("direct Unallocated -> Installed");
        assert_eq!(direct_install.fulfillment_status, "Installed");
        assert_eq!(direct_install.quantity_allocated, 2);
        assert_eq!(
            direct_install.source_lot_id.as_deref(),
            Some(lot_crate_id.as_str())
        );
        let ic_after_direct_install = get_component(&db, &ic.id).await.unwrap().unwrap();
        assert_eq!(ic_after_direct_install.total_on_hand, 6);

        // 9. An Installed line with source_lot_id = None (e.g. linked to a 0-stock component) never fabricates inventory
        let zero_stock_ic = create_component(
            &db,
            CreateComponent {
                name: "NE5532P Dual Low-Noise Op-Amp".into(),
                category: "Integrated Circuits (ICs)".into(),
                subtype: "Op-Amp".into(),
                package_or_mounting: "DIP-8".into(),
                raw_spec_text: "NE5532P DIP-8".into(),
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
        let bom_ne5532 = add_bom_line(
            &db,
            CreateBomLine {
                kit_build_id: build.id.clone(),
                raw_spec_text: "NE5532P DIP-8".into(),
                category: Some("Integrated Circuits (ICs)".into()),
                package_or_mounting: Some("DIP-8".into()),
                quantity_required: 2,
                reference_designators: "IC3, IC4".into(),
                included_in_kit: false,
                component_id: Some(zero_stock_ic.id.clone()),
                substitution_note: "".into(),
                notes: "".into(),
            },
        )
        .await
        .unwrap();
        let ne_installed =
            transition_bom_line_status(&db, &bom_ne5532.id, "Installed", None, None)
                .await
                .unwrap();
        assert_eq!(ne_installed.fulfillment_status, "Installed");
        assert!(ne_installed.source_lot_id.is_none());
        let _ = transition_bom_line_status(&db, &bom_ne5532.id, "Unallocated", None, None)
            .await
            .unwrap();
        let ne_after = get_component(&db, &zero_stock_ic.id).await.unwrap().unwrap();
        assert_eq!(ne_after.total_on_hand, 0);
        assert!(
            ne_after.lots.is_empty(),
            "Installed line with source_lot_id = None must never fabricate a stock lot"
        );
    }

    #[tokio::test]
    async fn batch_kitting_allocates_and_picks_eligible_lines_and_shelving_returns_picked_and_reserved_stock_while_keeping_installed_consumed()
     {
        use crate::catalog::{
            CreateComponent, CreateInitialStockLot, create_component, get_component,
        };

        let db = init_memory_db().await.expect("memory db");
        let loc = create_storage_location(
            &db,
            CreateStorageLocation {
                name: "Drawer A".into(),
                kind: "Drawer".into(),
                parent_id: None,
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        // 1. Resistor with plenty of stock (20 on hand, need 4)
        let r_10k = create_component(
            &db,
            CreateComponent {
                name: "10kΩ 1% Metal Film Resistor".into(),
                category: "Resistors".into(),
                subtype: "Fixed".into(),
                package_or_mounting: "Through-Hole".into(),
                raw_spec_text: "10k 1% 1/4W Metal Film".into(),
                canonical_value_display: None,
                resistance_ohms: None,
                capacitance_pf: None,
                inductance_uh: None,
                voltage_rating_volts: None,
                low_stock_threshold: 5,
                datasheet_url: "".into(),
                notes: "".into(),
                specs_json: None,
                initial_lot: Some(CreateInitialStockLot {
                    location_id: Some(loc.id.clone()),
                    quantity: 20,
                    supplier: "Tayda".into(),
                    supplier_part_no: "R-10K".into(),
                    lot_notes: "".into(),
                }),
            },
        )
        .await
        .unwrap();

        // 2. Capacitor with full stock (5 on hand, need 2)
        let c_100n = create_component(
            &db,
            CreateComponent {
                name: "100nF Box Film Capacitor".into(),
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
                initial_lot: Some(CreateInitialStockLot {
                    location_id: Some(loc.id.clone()),
                    quantity: 5,
                    supplier: "Mouser".into(),
                    supplier_part_no: "C-100N".into(),
                    lot_notes: "".into(),
                }),
            },
        )
        .await
        .unwrap();

        // 3. Diode with partial stock (only 1 on hand, need 2)
        let d_1n34a = create_component(
            &db,
            CreateComponent {
                name: "1N34A Germanium Diode".into(),
                category: "Semiconductors — Discrete".into(),
                subtype: "Diode".into(),
                package_or_mounting: "DO-7".into(),
                raw_spec_text: "1N34A Ge Diode".into(),
                canonical_value_display: None,
                resistance_ohms: None,
                capacitance_pf: None,
                inductance_uh: None,
                voltage_rating_volts: None,
                low_stock_threshold: 2,
                datasheet_url: "".into(),
                notes: "".into(),
                specs_json: None,
                initial_lot: Some(CreateInitialStockLot {
                    location_id: Some(loc.id.clone()),
                    quantity: 1,
                    supplier: "Small Bear".into(),
                    supplier_part_no: "1N34A".into(),
                    lot_notes: "".into(),
                }),
            },
        )
        .await
        .unwrap();

        let build = create_kit_build(
            &db,
            CreateKitBuild {
                name: "Rat Distortion".into(),
                project_type: "Distortion Pedal".into(),
                status: Some("Wishlist / Planned".into()),
                location_id: Some(loc.id.clone()),
                description: "".into(),
            },
        )
        .await
        .unwrap();

        let line_r = add_bom_line(
            &db,
            CreateBomLine {
                kit_build_id: build.id.clone(),
                raw_spec_text: "10k 1%".into(),
                category: None,
                package_or_mounting: None,
                quantity_required: 4,
                reference_designators: "R1, R2, R3, R4".into(),
                included_in_kit: false,
                component_id: None,
                substitution_note: "".into(),
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        let line_c = add_bom_line(
            &db,
            CreateBomLine {
                kit_build_id: build.id.clone(),
                raw_spec_text: "100nF".into(),
                category: None,
                package_or_mounting: None,
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

        let line_d = add_bom_line(
            &db,
            CreateBomLine {
                kit_build_id: build.id.clone(),
                raw_spec_text: "1N34A".into(),
                category: None,
                package_or_mounting: None,
                quantity_required: 2,
                reference_designators: "D1, D2".into(),
                included_in_kit: false,
                component_id: None,
                substitution_note: "".into(),
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        let _line_kit = add_bom_line(
            &db,
            CreateBomLine {
                kit_build_id: build.id.clone(),
                raw_spec_text: "Rat PCB".into(),
                category: Some("Enclosures, Knobs & Mechanical Hardware".into()),
                package_or_mounting: None,
                quantity_required: 1,
                reference_designators: "PCB1".into(),
                included_in_kit: true,
                component_id: None,
                substitution_note: "".into(),
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        // Initially: 4 lines total, only 1 ready (Included in Kit), 3 shortfalls
        let initial_build = get_kit_build(&db, &build.id).await.unwrap().unwrap();
        assert_eq!(initial_build.readiness.total_lines, 4);
        assert_eq!(initial_build.readiness.ready_lines, 1);
        assert_eq!(initial_build.readiness.shortfall_lines, 3);
        assert_eq!(
            initial_build.readiness.display_label(),
            "1/4 Lines Ready · 3 Shortfalls"
        );

        // Batch "Reserve All Available":
        // - R1-R4 reserves 4/4 (Ready)
        // - C1-C2 reserves 2/2 (Ready)
        // - D1-D2 reserves 1/2 (Partial reservation -> still 1 Shortfall!)
        let after_reserve_all = reserve_all_available(&db, &build.id)
            .await
            .expect("reserve all available");
        assert_eq!(after_reserve_all.readiness.total_lines, 4);
        assert_eq!(after_reserve_all.readiness.ready_lines, 3);
        assert_eq!(after_reserve_all.readiness.shortfall_lines, 1);
        assert_eq!(after_reserve_all.readiness.total_shortfall_quantity, 1);
        assert_eq!(
            after_reserve_all.readiness.display_label(),
            "3/4 Lines Ready · 1 Shortfalls"
        );

        // Verify physical stock lots are unchanged after Reserve All Available
        assert_eq!(
            get_component(&db, &r_10k.id).await.unwrap().unwrap().total_on_hand,
            20
        );
        assert_eq!(
            get_component(&db, &r_10k.id).await.unwrap().unwrap().available_stock,
            16
        );

        // Batch "Pick All Reserved":
        // Picks all Reserved lines into the project box, deducting from their Stock Lots
        let after_pick_all = pick_all_reserved(&db, &build.id)
            .await
            .expect("pick all reserved");
        let r_line_picked = after_pick_all
            .bom_lines
            .iter()
            .find(|l| l.id == line_r.id)
            .unwrap();
        assert_eq!(r_line_picked.fulfillment_status, "Picked");
        assert_eq!(r_line_picked.quantity_allocated, 4);
        assert!(r_line_picked.source_lot_id.is_some());

        assert_eq!(
            get_component(&db, &r_10k.id).await.unwrap().unwrap().total_on_hand,
            16
        );
        assert_eq!(
            get_component(&db, &c_100n.id).await.unwrap().unwrap().total_on_hand,
            3
        );
        assert_eq!(
            get_component(&db, &d_1n34a.id).await.unwrap().unwrap().total_on_hand,
            0
        );

        // Now mark R1-R4 as Installed (soldered onto the PCB), keep C1-C2 as Picked, and transition D1-D2 back to Reserved
        transition_bom_line_status(&db, &line_r.id, "Installed", None, None)
            .await
            .unwrap();
        transition_bom_line_status(&db, &line_d.id, "Reserved", None, None)
            .await
            .unwrap();
        assert_eq!(
            get_component(&db, &d_1n34a.id).await.unwrap().unwrap().total_on_hand,
            1
        );
        assert_eq!(
            get_component(&db, &d_1n34a.id).await.unwrap().unwrap().reserved_quantity,
            1
        );

        // Shelve the Kit Build and confirm returning parts:
        // - C1-C2 (Picked, 2 pcs) is returned to its source_lot_id (3 + 2 = 5 on hand) and reset to Unallocated
        // - D1-D2 (Reserved, 1 pc) is released (reserved_quantity = 0, available_stock = 1) and reset to Unallocated
        // - R1-R4 (Installed, 4 pcs) remains Installed and consumed (total_on_hand stays 16)!
        // - PCB1 (Included in Kit) remains Included in Kit
        let shelved = shelve_kit_build_and_return_parts(&db, &build.id)
            .await
            .expect("shelve and return parts");
        assert_eq!(shelved.status, "Shelved / Abandoned");

        let r_after_shelve = shelved.bom_lines.iter().find(|l| l.id == line_r.id).unwrap();
        let c_after_shelve = shelved.bom_lines.iter().find(|l| l.id == line_c.id).unwrap();
        let d_after_shelve = shelved.bom_lines.iter().find(|l| l.id == line_d.id).unwrap();

        assert_eq!(r_after_shelve.fulfillment_status, "Installed");
        assert_eq!(r_after_shelve.quantity_allocated, 4);

        assert_eq!(c_after_shelve.fulfillment_status, "Unallocated");
        assert_eq!(c_after_shelve.quantity_allocated, 0);
        assert!(c_after_shelve.source_lot_id.is_none());

        assert_eq!(d_after_shelve.fulfillment_status, "Unallocated");
        assert_eq!(d_after_shelve.quantity_allocated, 0);

        assert_eq!(
            get_component(&db, &r_10k.id).await.unwrap().unwrap().total_on_hand,
            16,
            "Installed parts must remain consumed"
        );
        assert_eq!(
            get_component(&db, &c_100n.id).await.unwrap().unwrap().total_on_hand,
            5,
            "Picked parts must be restored to source_lot_id"
        );
        let d_comp_after = get_component(&db, &d_1n34a.id).await.unwrap().unwrap();
        assert_eq!(d_comp_after.total_on_hand, 1);
        assert_eq!(
            d_comp_after.reserved_quantity, 0,
            "Reserved parts must be released"
        );
        assert_eq!(d_comp_after.available_stock, 1);
    }

    #[tokio::test]
    async fn substitutes_bom_line_component_and_records_note_while_preserving_original_raw_spec_text()
     {
        use crate::catalog::{
            CreateComponent, CreateInitialStockLot, create_component, get_component,
        };

        let db = init_memory_db().await.expect("memory db");
        let drawer = create_storage_location(
            &db,
            CreateStorageLocation {
                name: "Drawer JFET".into(),
                kind: "Drawer".into(),
                parent_id: None,
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        let j201 = create_component(
            &db,
            CreateComponent {
                name: "J201 N-Channel JFET".into(),
                category: "Semiconductors — Discrete".into(),
                subtype: "JFET".into(),
                package_or_mounting: "TO-92".into(),
                raw_spec_text: "J201 N-Ch JFET TO-92".into(),
                canonical_value_display: None,
                resistance_ohms: None,
                capacitance_pf: None,
                inductance_uh: None,
                voltage_rating_volts: None,
                low_stock_threshold: 2,
                datasheet_url: "".into(),
                notes: "".into(),
                specs_json: None,
                initial_lot: Some(CreateInitialStockLot {
                    location_id: Some(drawer.id.clone()),
                    quantity: 2,
                    supplier: "Mouser".into(),
                    supplier_part_no: "J201".into(),
                    lot_notes: "".into(),
                }),
            },
        )
        .await
        .unwrap();

        let j5457 = create_component(
            &db,
            CreateComponent {
                name: "2N5457 N-Channel JFET".into(),
                category: "Semiconductors — Discrete".into(),
                subtype: "JFET".into(),
                package_or_mounting: "TO-92".into(),
                raw_spec_text: "2N5457 N-Ch JFET TO-92".into(),
                canonical_value_display: None,
                resistance_ohms: None,
                capacitance_pf: None,
                inductance_uh: None,
                voltage_rating_volts: None,
                low_stock_threshold: 2,
                datasheet_url: "".into(),
                notes: "".into(),
                specs_json: None,
                initial_lot: Some(CreateInitialStockLot {
                    location_id: Some(drawer.id.clone()),
                    quantity: 10,
                    supplier: "Tayda".into(),
                    supplier_part_no: "2N5457".into(),
                    lot_notes: "".into(),
                }),
            },
        )
        .await
        .unwrap();

        let build = create_kit_build(
            &db,
            CreateKitBuild {
                name: "BSIAB II".into(),
                project_type: "Amp-in-a-Box".into(),
                status: None,
                location_id: None,
                description: "".into(),
            },
        )
        .await
        .unwrap();

        let line = add_bom_line(
            &db,
            CreateBomLine {
                kit_build_id: build.id.clone(),
                raw_spec_text: "J201".into(),
                category: None,
                package_or_mounting: None,
                quantity_required: 1,
                reference_designators: "Q1".into(),
                included_in_kit: false,
                component_id: None,
                substitution_note: "".into(),
                notes: "Input buffer".into(),
            },
        )
        .await
        .unwrap();
        assert_eq!(line.component_id.as_deref(), Some(j201.id.as_str()));
        assert_eq!(line.raw_spec_text, "J201");
        assert_eq!(line.canonical_value_display, "J201");

        // Substitute 2N5457 onto Q1 with note "pinout rotated"
        let subbed = substitute_bom_line_component(&db, &line.id, &j5457.id, "pinout rotated")
            .await
            .expect("substitute component");

        assert_eq!(subbed.component_id.as_deref(), Some(j5457.id.as_str()));
        assert_eq!(
            subbed.linked_component.as_ref().unwrap().name,
            "2N5457 N-Channel JFET"
        );
        assert_eq!(
            subbed
                .linked_component
                .as_ref()
                .unwrap()
                .canonical_value_display,
            "2N5457"
        );
        assert_eq!(subbed.substitution_note, "pinout rotated");
        // Original schematic specification and canonical value must remain intact!
        assert_eq!(subbed.raw_spec_text, "J201");
        assert_eq!(subbed.canonical_value_display, "J201");
        assert_eq!(subbed.reference_designators, "Q1");

        // Verify fulfillment transitions now operate against the substituted 2N5457 component
        let picked = transition_bom_line_status(&db, &subbed.id, "Picked", None, None)
            .await
            .unwrap();
        assert_eq!(picked.fulfillment_status, "Picked");
        assert_eq!(picked.quantity_allocated, 1);
        assert_eq!(
            get_component(&db, &j5457.id).await.unwrap().unwrap().total_on_hand,
            9
        );
        assert_eq!(
            get_component(&db, &j201.id).await.unwrap().unwrap().total_on_hand,
            2
        );
    }

    #[tokio::test]
    async fn partially_allocates_bom_lines_when_reserved_or_picked_and_tracks_shortfall_quantity_and_readiness()
     {
        use crate::catalog::{
            CreateComponent, CreateInitialStockLot, create_component, get_component,
        };

        let db = init_memory_db().await.expect("memory db");
        let drawer = create_storage_location(
            &db,
            CreateStorageLocation {
                name: "Drawer Passives".into(),
                kind: "Drawer".into(),
                parent_id: None,
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        let cap = create_component(
            &db,
            CreateComponent {
                name: "100nF 63V Box Film Capacitor".into(),
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
                initial_lot: Some(CreateInitialStockLot {
                    location_id: Some(drawer.id.clone()),
                    quantity: 10,
                    supplier: "Mouser".into(),
                    supplier_part_no: "C-100N".into(),
                    lot_notes: "".into(),
                }),
            },
        )
        .await
        .unwrap();

        let build = create_kit_build(
            &db,
            CreateKitBuild {
                name: "Big Muff Pi".into(),
                project_type: "Fuzz Pedal".into(),
                status: None,
                location_id: None,
                description: "".into(),
            },
        )
        .await
        .unwrap();

        let line = add_bom_line(
            &db,
            CreateBomLine {
                kit_build_id: build.id.clone(),
                raw_spec_text: "100nF".into(),
                category: None,
                package_or_mounting: None,
                quantity_required: 4,
                reference_designators: "C1, C2, C3, C4".into(),
                included_in_kit: false,
                component_id: None,
                substitution_note: "".into(),
                notes: "".into(),
            },
        )
        .await
        .unwrap();
        assert_eq!(line.quantity_required, 4);
        assert_eq!(line.quantity_allocated, 0);
        assert_eq!(line.shortfall_quantity(), 4);

        // 1. Partially allocate 2 of 4 as Reserved
        let reserved_partial = allocate_bom_line_partial(&db, &line.id, "Reserved", 2, None)
            .await
            .expect("partial reserve");
        assert_eq!(reserved_partial.fulfillment_status, "Reserved");
        assert_eq!(reserved_partial.quantity_allocated, 2);
        assert_eq!(reserved_partial.shortfall_quantity(), 2);

        let comp_after_res = get_component(&db, &cap.id).await.unwrap().unwrap();
        assert_eq!(comp_after_res.total_on_hand, 10);
        assert_eq!(comp_after_res.reserved_quantity, 2);
        assert_eq!(comp_after_res.available_stock, 8);

        let build_after_res = get_kit_build(&db, &build.id).await.unwrap().unwrap();
        assert_eq!(build_after_res.readiness.total_lines, 1);
        assert_eq!(build_after_res.readiness.ready_lines, 0);
        assert_eq!(build_after_res.readiness.shortfall_lines, 1);
        assert_eq!(build_after_res.readiness.total_shortfall_quantity, 2);

        // 2. Partially allocate 3 of 4 as Picked
        let picked_partial = allocate_bom_line_partial(&db, &line.id, "Picked", 3, None)
            .await
            .expect("partial pick");
        assert_eq!(picked_partial.fulfillment_status, "Picked");
        assert_eq!(picked_partial.quantity_allocated, 3);
        assert_eq!(picked_partial.shortfall_quantity(), 1);

        let comp_after_pick = get_component(&db, &cap.id).await.unwrap().unwrap();
        assert_eq!(comp_after_pick.total_on_hand, 7);
        assert_eq!(comp_after_pick.reserved_quantity, 0);
        assert_eq!(comp_after_pick.available_stock, 7);

        let build_after_pick = get_kit_build(&db, &build.id).await.unwrap().unwrap();
        assert_eq!(build_after_pick.readiness.total_lines, 1);
        assert_eq!(build_after_pick.readiness.ready_lines, 0);
        assert_eq!(build_after_pick.readiness.shortfall_lines, 1);
        assert_eq!(build_after_pick.readiness.total_shortfall_quantity, 1);

        // 3. Allocate full 4 of 4 as Picked -> line is now Ready with 0 shortfall
        let picked_full = allocate_bom_line_partial(&db, &line.id, "Picked", 4, None)
            .await
            .expect("full pick");
        assert_eq!(picked_full.quantity_allocated, 4);
        assert_eq!(picked_full.shortfall_quantity(), 0);
        let build_ready = get_kit_build(&db, &build.id).await.unwrap().unwrap();
        assert_eq!(build_ready.readiness.ready_lines, 1);
        assert_eq!(build_ready.readiness.shortfall_lines, 0);
        assert_eq!(build_ready.readiness.total_shortfall_quantity, 0);
    }

    #[tokio::test]
    async fn splits_bom_line_by_quantity_and_reference_designators_into_two_flat_sibling_bom_lines()
    {
        use crate::catalog::{
            CreateComponent, CreateInitialStockLot, CreateStockLot, create_component,
            get_component, upsert_or_merge_stock_lot,
        };

        let db = init_memory_db().await.expect("memory db");
        let loc_a = create_storage_location(
            &db,
            CreateStorageLocation {
                name: "Drawer A".into(),
                kind: "Drawer".into(),
                parent_id: None,
                notes: "".into(),
            },
        )
        .await
        .unwrap();
        let loc_b = create_storage_location(
            &db,
            CreateStorageLocation {
                name: "Binder B".into(),
                kind: "Binder".into(),
                parent_id: None,
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        let cap_wima = create_component(
            &db,
            CreateComponent {
                name: "100nF 63V WIMA Box Film".into(),
                category: "Capacitors".into(),
                subtype: "Film".into(),
                package_or_mounting: "Through-Hole 5mm".into(),
                raw_spec_text: "100nF 63V WIMA Box Film".into(),
                canonical_value_display: None,
                resistance_ohms: None,
                capacitance_pf: None,
                inductance_uh: None,
                voltage_rating_volts: None,
                low_stock_threshold: 2,
                datasheet_url: "".into(),
                notes: "".into(),
                specs_json: None,
                initial_lot: Some(CreateInitialStockLot {
                    location_id: Some(loc_a.id.clone()),
                    quantity: 2,
                    supplier: "Mouser".into(),
                    supplier_part_no: "WIMA-100N".into(),
                    lot_notes: "".into(),
                }),
            },
        )
        .await
        .unwrap();
        let lot_a_id = cap_wima.lots[0].id.clone();

        let lot_b = upsert_or_merge_stock_lot(
            &db,
            CreateStockLot {
                component_id: cap_wima.id.clone(),
                location_id: Some(loc_b.id.clone()),
                quantity: 2,
                supplier: "Tayda".into(),
                supplier_part_no: "TAY-100N".into(),
                lot_notes: "".into(),
            },
        )
        .await
        .unwrap();

        let cap_pana = create_component(
            &db,
            CreateComponent {
                name: "100nF 100V Panasonic Film".into(),
                category: "Capacitors".into(),
                subtype: "Film".into(),
                package_or_mounting: "Through-Hole 5mm".into(),
                raw_spec_text: "100nF 100V Panasonic Film".into(),
                canonical_value_display: None,
                resistance_ohms: None,
                capacitance_pf: None,
                inductance_uh: None,
                voltage_rating_volts: None,
                low_stock_threshold: 2,
                datasheet_url: "".into(),
                notes: "".into(),
                specs_json: None,
                initial_lot: Some(CreateInitialStockLot {
                    location_id: Some(loc_b.id.clone()),
                    quantity: 5,
                    supplier: "DigiKey".into(),
                    supplier_part_no: "PANA-100N".into(),
                    lot_notes: "".into(),
                }),
            },
        )
        .await
        .unwrap();

        let build = create_kit_build(
            &db,
            CreateKitBuild {
                name: "Chorus Ensemble".into(),
                project_type: "Modulation Pedal".into(),
                status: None,
                location_id: None,
                description: "".into(),
            },
        )
        .await
        .unwrap();

        let line_c = add_bom_line(
            &db,
            CreateBomLine {
                kit_build_id: build.id.clone(),
                raw_spec_text: "100nF Film".into(),
                category: Some("Capacitors".into()),
                package_or_mounting: Some("Through-Hole 5mm".into()),
                quantity_required: 4,
                reference_designators: "C1, C2, C3, C4".into(),
                included_in_kit: false,
                component_id: Some(cap_wima.id.clone()),
                substitution_note: "".into(),
                notes: "Coupling caps".into(),
            },
        )
        .await
        .unwrap();

        // 1. Auto-partition C1, C2, C3, C4 (qty 4) with split_quantity = 2 -> (2x C1, C2) and (2x C3, C4)
        let (orig_line, sibling_line) = split_bom_line(&db, &line_c.id, 2, None)
            .await
            .expect("split 4 into 2 and 2");

        assert_eq!(orig_line.id, line_c.id);
        assert_eq!(orig_line.quantity_required, 2);
        assert_eq!(orig_line.reference_designators, "C1, C2");
        assert_eq!(orig_line.raw_spec_text, "100nF Film");
        assert_eq!(orig_line.component_id.as_deref(), Some(cap_wima.id.as_str()));

        assert_ne!(sibling_line.id, orig_line.id);
        assert_eq!(sibling_line.kit_build_id, build.id);
        assert_eq!(sibling_line.quantity_required, 2);
        assert_eq!(sibling_line.reference_designators, "C3, C4");
        assert_eq!(sibling_line.raw_spec_text, "100nF Film");
        assert_eq!(sibling_line.canonical_value_display, "100nF");
        assert_eq!(
            sibling_line.component_id.as_deref(),
            Some(cap_wima.id.as_str())
        );
        assert_eq!(sibling_line.fulfillment_status, "Unallocated");
        assert_eq!(sibling_line.quantity_allocated, 0);

        // Both flat sibling lines can be fulfilled from different Stock Lots or substitute Components!
        let picked_orig =
            transition_bom_line_status(&db, &orig_line.id, "Picked", Some(&lot_a_id), None)
                .await
                .unwrap();
        assert_eq!(picked_orig.source_lot_id.as_deref(), Some(lot_a_id.as_str()));
        assert_eq!(picked_orig.quantity_allocated, 2);

        let subbed_sibling = substitute_bom_line_component(
            &db,
            &sibling_line.id,
            &cap_pana.id,
            "Used 100V Panasonic for C3, C4",
        )
        .await
        .unwrap();
        let picked_sibling =
            transition_bom_line_status(&db, &subbed_sibling.id, "Picked", None, None)
                .await
                .unwrap();
        assert_eq!(picked_sibling.quantity_allocated, 2);
        assert_eq!(
            get_component(&db, &cap_wima.id)
                .await
                .unwrap()
                .unwrap()
                .total_on_hand,
            2
        );
        assert_eq!(
            get_component(&db, &cap_pana.id)
                .await
                .unwrap()
                .unwrap()
                .total_on_hand,
            3
        );
        let _ = lot_b;

        // 2. Split with explicit split_designators_override
        let line_r = add_bom_line(
            &db,
            CreateBomLine {
                kit_build_id: build.id.clone(),
                raw_spec_text: "10k 1%".into(),
                category: None,
                package_or_mounting: None,
                quantity_required: 3,
                reference_designators: "R10, R11, R12".into(),
                included_in_kit: false,
                component_id: None,
                substitution_note: "".into(),
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        let (r_orig, r_sib) = split_bom_line(&db, &line_r.id, 1, Some("R11"))
            .await
            .expect("split with designator override");
        assert_eq!(r_orig.quantity_required, 2);
        assert_eq!(r_orig.reference_designators, "R10, R12");
        assert_eq!(r_sib.quantity_required, 1);
        assert_eq!(r_sib.reference_designators, "R11");

        // Reject splitting a line with quantity_required == 1
        assert!(split_bom_line(&db, &r_sib.id, 1, None).await.is_err());
    }

    #[tokio::test]
    async fn clones_kit_build_preserving_bom_specs_substitutions_and_shared_attachments_while_resetting_status_allocations_and_logs()
     {
        use crate::catalog::{
            CreateComponent, CreateInitialStockLot, create_component, get_component,
        };

        let db = init_memory_db().await.expect("memory db");
        let box_loc = create_storage_location(
            &db,
            CreateStorageLocation {
                name: "Project Box #1".into(),
                kind: "Box".into(),
                parent_id: None,
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        let j5457 = create_component(
            &db,
            CreateComponent {
                name: "2N5457 N-Channel JFET".into(),
                category: "Semiconductors — Discrete".into(),
                subtype: "JFET".into(),
                package_or_mounting: "TO-92".into(),
                raw_spec_text: "2N5457 N-Ch JFET TO-92".into(),
                canonical_value_display: None,
                resistance_ohms: None,
                capacitance_pf: None,
                inductance_uh: None,
                voltage_rating_volts: None,
                low_stock_threshold: 2,
                datasheet_url: "".into(),
                notes: "".into(),
                specs_json: None,
                initial_lot: Some(CreateInitialStockLot {
                    location_id: Some(box_loc.id.clone()),
                    quantity: 5,
                    supplier: "Tayda".into(),
                    supplier_part_no: "2N5457".into(),
                    lot_notes: "".into(),
                }),
            },
        )
        .await
        .unwrap();

        let source_build = create_kit_build(
            &db,
            CreateKitBuild {
                name: "Fuzz Face #1".into(),
                project_type: "Fuzz Pedal".into(),
                status: Some("On the Bench (In Progress)".into()),
                location_id: Some(box_loc.id.clone()),
                description: "Vintage silicon/JFET hybrid build".into(),
            },
        )
        .await
        .unwrap();

        // Seed a Bench Log Entry, a shared Schematic PDF attachment, and a log-attached photo
        let conn = db.connect().await.unwrap();
        conn.execute(
            "INSERT INTO bench_log_entries (id, kit_build_id, entry_type, body)
             VALUES ('log-1', ?1, 'Measurement / Voltage Reading', 'Q1 drain = 4.52V')",
            (source_build.id.as_str(),),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO attachments (id, kit_build_id, bench_log_entry_id, kind, original_filename, mime_type, relative_path, size_bytes, external_url)
             VALUES ('att-schem-1', ?1, NULL, 'Schematic PDF', 'fuzz_face_schematic.pdf', 'application/pdf', 'attachments/fuzz_face_schematic.pdf', 42000, 'https://example.com/schem.pdf')",
            (source_build.id.as_str(),),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO attachments (id, kit_build_id, bench_log_entry_id, kind, original_filename, mime_type, relative_path, size_bytes, external_url)
             VALUES ('att-photo-1', ?1, 'log-1', 'Photo', 'solder_joint.jpg', 'image/jpeg', 'attachments/solder_joint.jpg', 18000, '')",
            (source_build.id.as_str(),),
        )
        .await
        .unwrap();

        // Add a substituted & Installed BOM Line + an Included in Kit BOM Line
        let line_q = add_bom_line(
            &db,
            CreateBomLine {
                kit_build_id: source_build.id.clone(),
                raw_spec_text: "J201".into(),
                category: Some("Semiconductors — Discrete".into()),
                package_or_mounting: Some("TO-92".into()),
                quantity_required: 2,
                reference_designators: "Q1, Q2".into(),
                included_in_kit: false,
                component_id: Some(j5457.id.clone()),
                substitution_note: "2N5457 — pinout rotated".into(),
                notes: "Matched pair".into(),
            },
        )
        .await
        .unwrap();
        transition_bom_line_status(&db, &line_q.id, "Installed", None, None)
            .await
            .unwrap();

        let _line_kit = add_bom_line(
            &db,
            CreateBomLine {
                kit_build_id: source_build.id.clone(),
                raw_spec_text: "Fuzz Face PCB".into(),
                category: Some("Enclosures, Knobs & Mechanical Hardware".into()),
                package_or_mounting: None,
                quantity_required: 1,
                reference_designators: "PCB1".into(),
                included_in_kit: true,
                component_id: None,
                substitution_note: "".into(),
                notes: "Supplied in kit".into(),
            },
        )
        .await
        .unwrap();

        // Stock after installing 2 on source build is 3
        assert_eq!(
            get_component(&db, &j5457.id).await.unwrap().unwrap().total_on_hand,
            3
        );

        // Clone the Kit Build
        let cloned = clone_kit_build(&db, &source_build.id, None)
            .await
            .expect("clone kit build");

        assert_ne!(cloned.id, source_build.id);
        assert_eq!(cloned.name, "Fuzz Face #1 (Copy)");
        assert_eq!(cloned.project_type, "Fuzz Pedal");
        assert_eq!(cloned.description, "Vintage silicon/JFET hybrid build");
        assert_eq!(cloned.status, "Wishlist / Planned");
        assert!(cloned.location_id.is_none());
        assert_eq!(cloned.location_breadcrumb, "Unassigned Location");

        // Verify BOM Lines on cloned build
        assert_eq!(cloned.bom_lines.len(), 2);
        let cloned_q = cloned
            .bom_lines
            .iter()
            .find(|l| l.reference_designators == "Q1, Q2")
            .unwrap();
        assert_eq!(cloned_q.raw_spec_text, "J201");
        assert_eq!(cloned_q.quantity_required, 2);
        assert_eq!(cloned_q.component_id.as_deref(), Some(j5457.id.as_str()));
        assert_eq!(cloned_q.substitution_note, "2N5457 — pinout rotated");
        assert_eq!(cloned_q.notes, "Matched pair");
        assert_eq!(cloned_q.fulfillment_status, "Unallocated");
        assert_eq!(cloned_q.quantity_allocated, 0);
        assert!(cloned_q.source_lot_id.is_none());
        assert!(cloned_q.installed_designators.is_empty());

        let cloned_kit = cloned
            .bom_lines
            .iter()
            .find(|l| l.reference_designators == "PCB1")
            .unwrap();
        assert_eq!(cloned_kit.fulfillment_status, "Included in Kit");
        assert!(cloned_kit.included_in_kit);
        assert_eq!(cloned_kit.quantity_allocated, 1);

        // Verify shared attachment reference copied, and bench log stream starts empty
        let mut att_rows = conn
            .query(
                "SELECT kind, original_filename, relative_path, external_url FROM attachments WHERE kit_build_id = ?1",
                (cloned.id.as_str(),),
            )
            .await
            .unwrap();
        let att_row = att_rows
            .next()
            .await
            .unwrap()
            .expect("shared schematic attachment copied");
        assert_eq!(att_row.get::<String>(0).unwrap(), "Schematic PDF");
        assert_eq!(
            att_row.get::<String>(1).unwrap(),
            "fuzz_face_schematic.pdf"
        );
        assert_eq!(
            att_row.get::<String>(2).unwrap(),
            "attachments/fuzz_face_schematic.pdf"
        );
        assert_eq!(
            att_row.get::<String>(3).unwrap(),
            "https://example.com/schem.pdf"
        );
        assert!(
            att_rows.next().await.unwrap().is_none(),
            "bench-log-attached photo should not be copied"
        );

        let mut log_rows = conn
            .query(
                "SELECT COUNT(*) FROM bench_log_entries WHERE kit_build_id = ?1",
                (cloned.id.as_str(),),
            )
            .await
            .unwrap();
        let log_count = log_rows.next().await.unwrap().unwrap().get::<i64>(0).unwrap();
        assert_eq!(log_count, 0, "cloned build starts with empty Bench Log Entry stream");

        // Verify catalog stock was not deducted by cloning
        assert_eq!(
            get_component(&db, &j5457.id).await.unwrap().unwrap().total_on_hand,
            3
        );
    }

    #[tokio::test]
    async fn groups_bom_lines_into_three_profile_height_assembly_stages_and_computes_placement_progress()
     {
        let db = init_memory_db().await.expect("memory db");
        let build = create_kit_build(
            &db,
            CreateKitBuild {
                name: "Klon Centaur".into(),
                project_type: "Overdrive".into(),
                status: Some("On the Bench (In Progress)".into()),
                location_id: None,
                description: "".into(),
            },
        )
        .await
        .unwrap();

        let r_line = add_bom_line(
            &db,
            CreateBomLine {
                kit_build_id: build.id.clone(),
                raw_spec_text: "4k7 1% 1/4W Metal Film".into(),
                category: None,
                package_or_mounting: None,
                quantity_required: 3,
                reference_designators: "R1, R2, R5".into(),
                included_in_kit: false,
                component_id: None,
                substitution_note: "".into(),
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        let d_line = add_bom_line(
            &db,
            CreateBomLine {
                kit_build_id: build.id.clone(),
                raw_spec_text: "1N34A Germanium Diode".into(),
                category: None,
                package_or_mounting: None,
                quantity_required: 2,
                reference_designators: "D1, D2".into(),
                included_in_kit: false,
                component_id: None,
                substitution_note: "".into(),
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        let ic_line = add_bom_line(
            &db,
            CreateBomLine {
                kit_build_id: build.id.clone(),
                raw_spec_text: "TL072IP DIP-8".into(),
                category: None,
                package_or_mounting: None,
                quantity_required: 2,
                reference_designators: "IC1, IC2".into(),
                included_in_kit: false,
                component_id: None,
                substitution_note: "".into(),
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        let film_cap_line = add_bom_line(
            &db,
            CreateBomLine {
                kit_build_id: build.id.clone(),
                raw_spec_text: "0.047uF 63V Box Film".into(),
                category: None,
                package_or_mounting: None,
                quantity_required: 2,
                reference_designators: "C3, C6".into(),
                included_in_kit: false,
                component_id: None,
                substitution_note: "".into(),
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        let elec_cap_line = add_bom_line(
            &db,
            CreateBomLine {
                kit_build_id: build.id.clone(),
                raw_spec_text: "47uF 25V Electrolytic Radial".into(),
                category: None,
                package_or_mounting: None,
                quantity_required: 2,
                reference_designators: "C10, C11".into(),
                included_in_kit: false,
                component_id: None,
                substitution_note: "".into(),
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        let pot_line = add_bom_line(
            &db,
            CreateBomLine {
                kit_build_id: build.id.clone(),
                raw_spec_text: "100kB Dual-Gang 16mm PCB".into(),
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

        let lines = list_bom_lines_for_build(&db, &build.id).await.unwrap();
        let stage_groups = group_bom_lines_by_stage(&lines);
        assert_eq!(stage_groups.len(), 3);

        assert_eq!(
            stage_groups[0].stage.as_str(),
            "Stage 1: Low-Profile Resistors & Diodes"
        );
        let stage1_ids: Vec<&str> = stage_groups[0]
            .lines
            .iter()
            .map(|l| l.id.as_str())
            .collect();
        assert_eq!(stage1_ids, vec![r_line.id.as_str(), d_line.id.as_str()]);

        assert_eq!(
            stage_groups[1].stage.as_str(),
            "Stage 2: IC Sockets & Film Capacitors"
        );
        let stage2_ids: Vec<&str> = stage_groups[1]
            .lines
            .iter()
            .map(|l| l.id.as_str())
            .collect();
        assert_eq!(
            stage2_ids,
            vec![ic_line.id.as_str(), film_cap_line.id.as_str()]
        );

        assert_eq!(
            stage_groups[2].stage.as_str(),
            "Stage 3: Electrolytics, Pots & Off-Board Hardware"
        );
        let stage3_ids: Vec<&str> = stage_groups[2]
            .lines
            .iter()
            .map(|l| l.id.as_str())
            .collect();
        assert_eq!(
            stage3_ids,
            vec![elec_cap_line.id.as_str(), pot_line.id.as_str()]
        );

        let progress = compute_bench_placement_progress(&lines);
        assert_eq!(progress.soldered_placements, 0);
        assert_eq!(progress.total_placements, 12);
        assert_eq!(progress.display_label(), "0 / 12 Placements Soldered");
    }

    #[tokio::test]
    async fn toggles_per_designator_soldering_chips_and_auto_transitions_bom_line_to_installed_when_all_placements_checked()
     {
        use crate::catalog::{
            CreateComponent, CreateInitialStockLot, create_component, get_component,
        };

        let db = init_memory_db().await.expect("memory db");
        let drawer = create_storage_location(
            &db,
            CreateStorageLocation {
                name: "Drawer 1".into(),
                kind: "Drawer".into(),
                parent_id: None,
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        let r_4k7 = create_component(
            &db,
            CreateComponent {
                name: "4.7kΩ 1% Resistor".into(),
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
                initial_lot: Some(CreateInitialStockLot {
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

        let c_47n = create_component(
            &db,
            CreateComponent {
                name: "47nF Film Cap".into(),
                category: "Capacitors".into(),
                subtype: "Film".into(),
                package_or_mounting: "Through-Hole".into(),
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
                initial_lot: Some(CreateInitialStockLot {
                    location_id: Some(drawer.id.clone()),
                    quantity: 6,
                    supplier: "Mouser".into(),
                    supplier_part_no: "C-47N".into(),
                    lot_notes: "".into(),
                }),
            },
        )
        .await
        .unwrap();

        let build = create_kit_build(
            &db,
            CreateKitBuild {
                name: "Bench Test Pedal".into(),
                project_type: "Overdrive".into(),
                status: Some("On the Bench (In Progress)".into()),
                location_id: None,
                description: "".into(),
            },
        )
        .await
        .unwrap();

        let line_r = add_bom_line(
            &db,
            CreateBomLine {
                kit_build_id: build.id.clone(),
                raw_spec_text: "4k7 1%".into(),
                category: None,
                package_or_mounting: None,
                quantity_required: 3,
                reference_designators: "R1, R2, R5".into(),
                included_in_kit: false,
                component_id: None,
                substitution_note: "".into(),
                notes: "".into(),
            },
        )
        .await
        .unwrap();
        transition_bom_line_status(&db, &line_r.id, "Reserved", None, None)
            .await
            .unwrap();

        let line_c = add_bom_line(
            &db,
            CreateBomLine {
                kit_build_id: build.id.clone(),
                raw_spec_text: "47nF".into(),
                category: None,
                package_or_mounting: None,
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
        transition_bom_line_status(&db, &line_c.id, "Picked", None, None)
            .await
            .unwrap();
        assert_eq!(
            get_component(&db, &c_47n.id).await.unwrap().unwrap().total_on_hand,
            4
        );

        let after_r1 = toggle_bom_line_designator(&db, &line_r.id, "R1")
            .await
            .expect("toggle R1");
        assert_eq!(after_r1.installed_designators, vec!["R1"]);
        assert_eq!(after_r1.fulfillment_status, "Reserved");
        assert_eq!(
            get_component(&db, &r_4k7.id).await.unwrap().unwrap().total_on_hand,
            10
        );

        let after_r2_on = toggle_bom_line_designator(&db, &line_r.id, "R2")
            .await
            .expect("toggle R2 on");
        assert_eq!(after_r2_on.installed_designators, vec!["R1", "R2"]);
        assert_eq!(after_r2_on.fulfillment_status, "Reserved");

        let after_r2_off = toggle_bom_line_designator(&db, &line_r.id, "R2")
            .await
            .expect("toggle R2 off");
        assert_eq!(after_r2_off.installed_designators, vec!["R1"]);

        let _ = toggle_bom_line_designator(&db, &line_r.id, "R2")
            .await
            .unwrap();

        let after_r5 = toggle_bom_line_designator(&db, &line_r.id, "R5")
            .await
            .expect("toggle final designator R5");
        assert_eq!(after_r5.installed_designators, vec!["R1", "R2", "R5"]);
        assert_eq!(after_r5.fulfillment_status, "Installed");
        assert_eq!(after_r5.quantity_allocated, 3);
        assert!(after_r5.source_lot_id.is_some());
        assert_eq!(
            get_component(&db, &r_4k7.id).await.unwrap().unwrap().total_on_hand,
            7
        );

        let c_installed = mark_all_bom_line_designators_installed(&db, &line_c.id)
            .await
            .expect("mark all installed");
        assert_eq!(c_installed.fulfillment_status, "Installed");
        assert_eq!(c_installed.installed_designators, vec!["C1", "C2"]);
        assert_eq!(
            get_component(&db, &c_47n.id).await.unwrap().unwrap().total_on_hand,
            4
        );

        let all_lines = list_bom_lines_for_build(&db, &build.id).await.unwrap();
        let progress = compute_bench_placement_progress(&all_lines);
        assert_eq!(progress.soldered_placements, 5);
        assert_eq!(progress.total_placements, 5);
        assert_eq!(progress.display_label(), "5 / 5 Placements Soldered");
    }

    #[tokio::test]
    async fn stores_build_attachment_files_on_disk_and_persists_attachment_metadata_and_external_links()
     {
        let db = init_memory_db().await.expect("memory db");
        let tmp = tempfile::tempdir().expect("tempdir");
        let data_dir = tmp.path();

        let build = create_kit_build(
            &db,
            CreateKitBuild {
                name: "Aion Refractor Klon".into(),
                project_type: "Overdrive".into(),
                status: Some("On the Bench (In Progress)".into()),
                location_id: None,
                description: "".into(),
            },
        )
        .await
        .unwrap();

        let pdf_bytes = b"%PDF-1.4 fake schematic content";
        let schematic = add_build_attachment_file_in_dir(
            &db,
            data_dir,
            &build.id,
            None,
            "Schematic PDF",
            "Klon Centaur Schematic (Rev 2).pdf",
            "application/pdf",
            pdf_bytes,
        )
        .await
        .expect("upload schematic pdf");

        assert_eq!(schematic.kit_build_id, build.id);
        assert_eq!(schematic.kind, "Schematic PDF");
        assert_eq!(
            schematic.original_filename,
            "Klon Centaur Schematic (Rev 2).pdf"
        );
        assert_eq!(schematic.mime_type, "application/pdf");
        assert_eq!(schematic.size_bytes, pdf_bytes.len() as i64);
        assert!(
            schematic
                .relative_path
                .starts_with(&format!("attachments/{}/", build.id))
        );

        let full_disk_path = data_dir.join(&schematic.relative_path);
        assert!(full_disk_path.exists());
        let read_back =
            read_build_attachment_bytes_in_dir(data_dir, &schematic).expect("read back bytes");
        assert_eq!(read_back, pdf_bytes);

        let drill_doc = add_build_attachment_file_in_dir(
            &db,
            data_dir,
            &build.id,
            None,
            "Build Doc / Drill Template",
            "125B_drill_template.pdf",
            "application/pdf",
            b"drill-template-bytes",
        )
        .await
        .expect("upload drill template");
        assert_eq!(drill_doc.kind, "Build Doc / Drill Template");

        let forum_link = add_build_external_link(
            &db,
            &build.id,
            Some("External URL"),
            "DIYSB Klon Build Thread",
            "https://www.diystompboxes.com/smfforum/index.php?topic=12345",
        )
        .await
        .expect("add external link");
        assert_eq!(forum_link.original_filename, "DIYSB Klon Build Thread");
        assert_eq!(
            forum_link.external_url,
            "https://www.diystompboxes.com/smfforum/index.php?topic=12345"
        );

        let all_attachments = list_build_attachments(&db, &build.id)
            .await
            .expect("list attachments");
        assert_eq!(all_attachments.len(), 3);
        assert_eq!(all_attachments[0].id, schematic.id);
        assert_eq!(all_attachments[1].id, drill_doc.id);
        assert_eq!(all_attachments[2].id, forum_link.id);
    }

    #[tokio::test]
    async fn creates_and_lists_chronological_timestamped_bench_log_entries_with_optional_attached_photos()
     {
        let db = init_memory_db().await.expect("memory db");
        let tmp = tempfile::tempdir().expect("tempdir");
        let data_dir = tmp.path();

        let build = create_kit_build(
            &db,
            CreateKitBuild {
                name: "Klon Centaur".into(),
                project_type: "Overdrive".into(),
                status: Some("On the Bench (In Progress)".into()),
                location_id: None,
                description: "".into(),
            },
        )
        .await
        .unwrap();

        let entry1 = create_bench_log_entry(
            &db,
            CreateBenchLogEntry {
                kit_build_id: build.id.clone(),
                entry_type: "Measurement / Voltage Reading".into(),
                body: "MAX1044 Pin 8 = +17.84V, Pin 5 = -8.72V before inserting TL072s.".into(),
            },
        )
        .await
        .expect("create voltage reading entry");
        assert_eq!(entry1.entry_type, "Measurement / Voltage Reading");
        assert!(!entry1.created_at.is_empty());

        let entry2 = create_bench_log_entry(
            &db,
            CreateBenchLogEntry {
                kit_build_id: build.id.clone(),
                entry_type: "Troubleshooting".into(),
                body: "Audio probe at IC1B pin 7 shows clean signal; cold solder joint on R11.".into(),
            },
        )
        .await
        .expect("create troubleshooting entry");

        let photo = add_build_attachment_file_in_dir(
            &db,
            data_dir,
            &build.id,
            Some(&entry2.id),
            "Photos",
            "cold_joint_r11.jpg",
            "image/jpeg",
            b"fake-jpeg-bytes",
        )
        .await
        .expect("attach photo to bench log entry");
        assert_eq!(photo.bench_log_entry_id.as_deref(), Some(entry2.id.as_str()));

        let entry3 = create_bench_log_entry(
            &db,
            CreateBenchLogEntry {
                kit_build_id: build.id.clone(),
                entry_type: "Modification".into(),
                body: "Socketed D1/D2 clipping pair to compare 1N34A vs BAT41.".into(),
            },
        )
        .await
        .expect("create modification entry");

        let entry4 = create_bench_log_entry(
            &db,
            CreateBenchLogEntry {
                kit_build_id: build.id.clone(),
                entry_type: "Note".into(),
                body: "Reflowed R11 and boxed in 125B enclosure.".into(),
            },
        )
        .await
        .expect("create note entry");

        let entries = list_bench_log_entries(&db, &build.id)
            .await
            .expect("list bench log entries");
        assert_eq!(entries.len(), 4);
        assert_eq!(entries[0].id, entry1.id);
        assert_eq!(entries[0].entry_type, "Measurement / Voltage Reading");
        assert!(entries[0].attachments.is_empty());

        assert_eq!(entries[1].id, entry2.id);
        assert_eq!(entries[1].entry_type, "Troubleshooting");
        assert_eq!(entries[1].attachments.len(), 1);
        assert_eq!(entries[1].attachments[0].id, photo.id);
        assert_eq!(entries[1].attachments[0].original_filename, "cold_joint_r11.jpg");

        assert_eq!(entries[2].id, entry3.id);
        assert_eq!(entries[2].entry_type, "Modification");

        assert_eq!(entries[3].id, entry4.id);
        assert_eq!(entries[3].entry_type, "Note");
    }
}
