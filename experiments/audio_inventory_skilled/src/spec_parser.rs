use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Category {
    #[serde(rename = "Resistors")]
    Resistors,
    #[serde(rename = "Capacitors")]
    Capacitors,
    #[serde(rename = "Semiconductors — Discrete")]
    SemiconductorsDiscrete,
    #[serde(rename = "Integrated Circuits (ICs)")]
    IntegratedCircuits,
    #[serde(rename = "Potentiometers & Encoders")]
    PotentiometersAndEncoders,
    #[serde(rename = "Switches, Jacks & Connectors")]
    SwitchesJacksAndConnectors,
    #[serde(rename = "Inductors, Transformers & Magnetics")]
    InductorsTransformersAndMagnetics,
    #[serde(rename = "Enclosures, Knobs & Mechanical Hardware")]
    EnclosuresKnobsAndHardware,
    #[serde(rename = "Modules & Sub-Assemblies")]
    ModulesAndSubAssemblies,
    #[serde(rename = "Other / Custom")]
    OtherCustom,
}

impl Category {
    pub const ALL: &'static [Category] = &[
        Category::Resistors,
        Category::Capacitors,
        Category::SemiconductorsDiscrete,
        Category::IntegratedCircuits,
        Category::PotentiometersAndEncoders,
        Category::SwitchesJacksAndConnectors,
        Category::InductorsTransformersAndMagnetics,
        Category::EnclosuresKnobsAndHardware,
        Category::ModulesAndSubAssemblies,
        Category::OtherCustom,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Category::Resistors => "Resistors",
            Category::Capacitors => "Capacitors",
            Category::SemiconductorsDiscrete => "Semiconductors — Discrete",
            Category::IntegratedCircuits => "Integrated Circuits (ICs)",
            Category::PotentiometersAndEncoders => "Potentiometers & Encoders",
            Category::SwitchesJacksAndConnectors => "Switches, Jacks & Connectors",
            Category::InductorsTransformersAndMagnetics => "Inductors, Transformers & Magnetics",
            Category::EnclosuresKnobsAndHardware => "Enclosures, Knobs & Mechanical Hardware",
            Category::ModulesAndSubAssemblies => "Modules & Sub-Assemblies",
            Category::OtherCustom => "Other / Custom",
        }
    }

    pub fn from_label(s: &str) -> Option<Self> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return None;
        }
        let lower = trimmed.to_lowercase();
        match lower.as_str() {
            "resistors" | "resistor" => Some(Category::Resistors),
            "capacitors" | "capacitor" | "caps" => Some(Category::Capacitors),
            "semiconductors — discrete"
            | "semiconductors - discrete"
            | "semiconductors"
            | "discrete"
            | "diodes"
            | "transistors" => Some(Category::SemiconductorsDiscrete),
            "integrated circuits (ics)" | "integrated circuits" | "ics" | "ic" | "op-amps" => {
                Some(Category::IntegratedCircuits)
            }
            "potentiometers & encoders"
            | "potentiometers"
            | "pots"
            | "encoders" => Some(Category::PotentiometersAndEncoders),
            "switches, jacks & connectors"
            | "switches & jacks"
            | "switches"
            | "jacks"
            | "connectors" => Some(Category::SwitchesJacksAndConnectors),
            "inductors, transformers & magnetics"
            | "inductors & transformers"
            | "inductors"
            | "transformers"
            | "magnetics" => Some(Category::InductorsTransformersAndMagnetics),
            "enclosures, knobs & mechanical hardware"
            | "enclosures & hardware"
            | "enclosures"
            | "hardware"
            | "knobs" => Some(Category::EnclosuresKnobsAndHardware),
            "modules & sub-assemblies" | "modules" | "sub-assemblies" => {
                Some(Category::ModulesAndSubAssemblies)
            }
            "other / custom" | "other" | "custom" => Some(Category::OtherCustom),
            _ => None,
        }
    }
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Category {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Category::from_label(s).ok_or_else(|| format!("Unknown category: {s}"))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParsedSpec {
    pub raw_spec_text: String,
    pub canonical_value_display: String,
    pub resistance_ohms: Option<f64>,
    pub capacitance_pf: Option<f64>,
    pub inductance_uh: Option<f64>,
    pub voltage_rating_volts: Option<f64>,
    pub tolerance: Option<String>,
    pub power_rating_watts: Option<f64>,
    pub taper: Option<String>,
    pub package_or_mounting: Option<String>,
    pub composition: Option<String>,
    pub dielectric: Option<String>,
    pub part_code: Option<String>,
    pub inferred_category: Option<Category>,
    pub specs_json: Value,
}

impl ParsedSpec {
    pub fn summary_badge(&self) -> Option<String> {
        let mut parts = Vec::new();
        if !self.canonical_value_display.is_empty() {
            parts.push(self.canonical_value_display.clone());
        }
        if let Some(ref taper) = self.taper {
            parts.push(format!("Taper {taper}"));
        }
        if let Some(ref tol) = self.tolerance {
            parts.push(tol.clone());
        }
        if let Some(w) = self.power_rating_watts {
            parts.push(format!("{}W", format_trimmed_float(w)));
        }
        if let Some(v) = self.voltage_rating_volts {
            parts.push(format!("{}V", format_trimmed_float(v)));
        }
        if let Some(ref pkg) = self.package_or_mounting {
            parts.push(pkg.clone());
        }
        if parts.is_empty() {
            None
        } else {
            Some(parts.join(" · "))
        }
    }
}

pub(crate) fn json_str_attr<'a>(specs: &'a Value, key: &str) -> Option<&'a str> {
    specs
        .get(key)
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

pub(crate) fn json_f64_attr(specs: &Value, key: &str) -> Option<f64> {
    specs.get(key).and_then(|v| {
        v.as_f64()
            .or_else(|| v.as_str().and_then(|s| s.trim().parse::<f64>().ok()))
    })
}

pub fn format_trimmed_float(val: f64) -> String {
    let rounded = (val * 1_000_000.0).round() / 1_000_000.0;
    if (rounded - rounded.round()).abs() < 1e-7 {
        format!("{}", rounded.round() as i64)
    } else {
        let s = format!("{:.6}", rounded);
        let s = s.trim_end_matches('0').trim_end_matches('.');
        s.to_string()
    }
}

pub fn format_resistance_canonical(ohms: f64) -> String {
    if ohms >= 1_000_000.0 {
        format!("{}MΩ", format_trimmed_float(ohms / 1_000_000.0))
    } else if ohms >= 1_000.0 {
        format!("{}kΩ", format_trimmed_float(ohms / 1_000.0))
    } else {
        format!("{}Ω", format_trimmed_float(ohms))
    }
}

pub fn format_capacitance_canonical(pf: f64) -> String {
    if pf >= 1_000_000.0 {
        format!("{}µF", format_trimmed_float(pf / 1_000_000.0))
    } else if pf >= 1_000.0 {
        format!("{}nF", format_trimmed_float(pf / 1_000.0))
    } else {
        format!("{}pF", format_trimmed_float(pf))
    }
}

pub fn format_inductance_canonical(uh: f64) -> String {
    if uh >= 1_000_000.0 {
        format!("{}H", format_trimmed_float(uh / 1_000_000.0))
    } else if uh >= 1_000.0 {
        format!("{}mH", format_trimmed_float(uh / 1_000.0))
    } else {
        format!("{}µH", format_trimmed_float(uh))
    }
}

pub fn parse_spec(raw_text: &str, category_hint: Option<Category>) -> ParsedSpec {
    let raw_spec_text = raw_text.to_string();
    let trimmed = raw_text.trim();

    let mut resistance_ohms: Option<f64> = None;
    let mut capacitance_pf: Option<f64> = None;
    let mut inductance_uh: Option<f64> = None;
    let mut voltage_rating_volts: Option<f64> = None;
    let mut tolerance: Option<String> = None;
    let mut power_rating_watts: Option<f64> = None;
    let mut taper: Option<String> = None;
    let mut lead_spacing_mm: Option<f64> = None;

    let raw_tokens: Vec<&str> = trimmed
        .split(|c: char| c.is_whitespace() || c == ',' || c == ';' || c == '(' || c == ')')
        .filter(|s| !s.is_empty())
        .collect();

    for &tok in &raw_tokens {
        let clean = tok.trim_matches(|c: char| c == '.' || c == ':');
        if clean.is_empty() {
            continue;
        }

        // 1. Tolerance: e.g. 1%, 5%, 0.1%, ±5%
        if tolerance.is_none() {
            let t_clean = clean.trim_start_matches('±');
            if let Some(num_str) = t_clean.strip_suffix('%') {
                if num_str.parse::<f64>().is_ok() {
                    tolerance = Some(format!("{num_str}%"));
                    continue;
                }
            }
        }

        // 2. Power rating: e.g. 1/4W, 1/2W, 1/8W, 0.25W, 0.5W, 1W, 2W
        if power_rating_watts.is_none() {
            if let Some(w) = try_parse_wattage_token(clean) {
                power_rating_watts = Some(w);
                continue;
            }
        }

        // 3. Voltage rating: e.g. 63V, 16V, 25V, 50V, 100V, 9VDC
        if voltage_rating_volts.is_none() {
            if let Some(v) = try_parse_voltage_token(clean) {
                voltage_rating_volts = Some(v);
                continue;
            }
        }

        // 4. Lead spacing / body size mm: e.g. 5mm, 2.54mm, 7.5mm
        if let Some(mm_str) = clean.strip_suffix("mm").or_else(|| clean.strip_suffix("MM")) {
            if let Ok(mm_val) = mm_str.parse::<f64>() {
                // 9mm, 16mm, 24mm are commonly pot body sizes/packages; 2.5mm, 5mm, 7.5mm are lead spacings
                if (mm_val - 5.0).abs() < 1e-6
                    || (mm_val - 2.5).abs() < 1e-6
                    || (mm_val - 2.54).abs() < 1e-6
                    || (mm_val - 7.5).abs() < 1e-6
                {
                    lead_spacing_mm = Some(mm_val);
                }
            }
        }

        // 5. Capacitance token: e.g. 0.047uF, 47n, 47000pF, 4n7, 2p2, 100uF
        if capacitance_pf.is_none()
            && !matches!(
                category_hint,
                Some(
                    Category::Resistors
                        | Category::SemiconductorsDiscrete
                        | Category::IntegratedCircuits
                        | Category::PotentiometersAndEncoders
                )
            )
        {
            if let Some(pf) = try_parse_capacitance_token(clean) {
                capacitance_pf = Some(pf);
                continue;
            }
        }

        // 6. Inductance token: e.g. 10mH, 100uH, 4u7H, 500mH
        if inductance_uh.is_none() {
            if let Some(uh) = try_parse_inductance_token(clean) {
                inductance_uh = Some(uh);
                continue;
            }
        }

        // 7. Resistance + Optional Taper token: e.g. 4k7, 4.7k, 4700R, 100kA, B50k, 4.7kΩ
        if resistance_ohms.is_none() && capacitance_pf.is_none() {
            if let Some((ohms, extracted_taper)) =
                try_parse_resistance_or_pot_token(clean, category_hint)
            {
                resistance_ohms = Some(ohms);
                if extracted_taper.is_some() {
                    taper = extracted_taper;
                }
                continue;
            }
        }
    }

    // Check multi-word phrases for package_or_mounting, composition, dielectric, taper
    let package_or_mounting = extract_package_or_mounting(trimmed, &raw_tokens);
    let composition = extract_composition(trimmed);
    let dielectric = extract_dielectric(trimmed);

    if taper.is_none() {
        taper = extract_standalone_taper(trimmed, &raw_tokens, category_hint, package_or_mounting.as_deref());
    }

    // Part code extraction for semiconductors / ICs / general identifiers (e.g., TL072IP, TL072, 1N34A, 2N5457, J201, PT2399)
    let part_code = extract_part_code(&raw_tokens);

    // Infer category if not hinted
    let inferred_category = category_hint.or_else(|| {
        if taper.is_some()
            || package_or_mounting
                .as_deref()
                .is_some_and(|p| p.contains("16mm") || p.contains("9mm") || p.contains("24mm"))
        {
            Some(Category::PotentiometersAndEncoders)
        } else if capacitance_pf.is_some() {
            Some(Category::Capacitors)
        } else if inductance_uh.is_some() {
            Some(Category::InductorsTransformersAndMagnetics)
        } else if resistance_ohms.is_some() {
            Some(Category::Resistors)
        } else if let Some(ref code) = part_code {
            infer_category_from_part_code(code)
        } else {
            None
        }
    });

    // Determine canonical_value_display
    let canonical_value_display = if let Some(ohms) = resistance_ohms {
        format_resistance_canonical(ohms)
    } else if let Some(pf) = capacitance_pf {
        format_capacitance_canonical(pf)
    } else if let Some(uh) = inductance_uh {
        format_inductance_canonical(uh)
    } else if let Some(ref code) = part_code {
        code.clone()
    } else {
        trimmed.to_string()
    };

    // Populate specs_json with extracted parametric attributes
    let mut specs_map = Map::new();
    if let Some(ref tol) = tolerance {
        specs_map.insert("tolerance".to_string(), Value::String(tol.clone()));
    }
    if let Some(w) = power_rating_watts {
        specs_map.insert("power_rating_watts".to_string(), Value::from(w));
    }
    if let Some(ref comp) = composition {
        specs_map.insert("composition".to_string(), Value::String(comp.clone()));
    }
    if let Some(ref diel) = dielectric {
        specs_map.insert("dielectric".to_string(), Value::String(diel.clone()));
    }
    if let Some(mm) = lead_spacing_mm {
        specs_map.insert("lead_spacing_mm".to_string(), Value::from(mm));
    }
    if let Some(ref tap) = taper {
        specs_map.insert("taper".to_string(), Value::String(tap.clone()));
    }
    if let Some(ref code) = part_code {
        specs_map.insert("part_code".to_string(), Value::String(code.clone()));
    }

    ParsedSpec {
        raw_spec_text,
        canonical_value_display,
        resistance_ohms,
        capacitance_pf,
        inductance_uh,
        voltage_rating_volts,
        tolerance,
        power_rating_watts,
        taper,
        package_or_mounting,
        composition,
        dielectric,
        part_code,
        inferred_category,
        specs_json: Value::Object(specs_map),
    }
}

fn try_parse_wattage_token(tok: &str) -> Option<f64> {
    let num_part = tok.strip_suffix('W').or_else(|| tok.strip_suffix('w'))?;
    if num_part.is_empty() {
        return None;
    }
    if let Some((num, den)) = num_part.split_once('/') {
        let n = num.parse::<f64>().ok()?;
        let d = den.parse::<f64>().ok()?;
        if d > 0.0 {
            return Some(n / d);
        }
        return None;
    }
    num_part.parse::<f64>().ok()
}

fn try_parse_voltage_token(tok: &str) -> Option<f64> {
    let lower = tok.to_lowercase();
    let num_part = lower
        .strip_suffix("vdc")
        .or_else(|| lower.strip_suffix("vac"))
        .or_else(|| lower.strip_suffix('v'))?;
    if num_part.is_empty() || !num_part.chars().next()?.is_ascii_digit() {
        return None;
    }
    num_part.parse::<f64>().ok()
}

fn try_parse_capacitance_token(tok: &str) -> Option<f64> {
    // Reject JEDEC discrete semiconductor part numbers like 1N4148, 2N5457, 2N3904
    let upper = tok.to_uppercase();
    if (upper.starts_with("1N") || upper.starts_with("2N"))
        && upper.len() >= 5
        && upper[2..].chars().take(3).all(|c| c.is_ascii_digit())
    {
        return None;
    }

    // Strip trailing F or f if present, except we must know if there was a unit prefix
    // Supported forms:
    // 1) Radix-replaced: 4n7, 4n7F, 2p2, 2p2F, 1u5, 4u7, 4µ7
    // 2) Standard suffix: 0.047uF, 0.047µF, 0.047u, 47n, 47nF, 47000pF, 47000p, 100uF
    let s = tok.replace('μ', "µ");

    // Check radix-replaced: <digits><p|n|u|µ><digits>[F|f]
    let without_f = s
        .strip_suffix('F')
        .or_else(|| s.strip_suffix('f'))
        .unwrap_or(&s);

    for (sep, mult_pf) in [('p', 1.0), ('P', 1.0), ('n', 1_000.0), ('N', 1_000.0), ('u', 1_000_000.0), ('U', 1_000_000.0), ('µ', 1_000_000.0)] {
        if let Some((int_part, frac_part)) = without_f.split_once(sep) {
            if !int_part.is_empty()
                && !frac_part.is_empty()
                && int_part.chars().all(|c| c.is_ascii_digit())
                && frac_part.chars().all(|c| c.is_ascii_digit())
            {
                let combined = format!("{int_part}.{frac_part}");
                if let Ok(val) = combined.parse::<f64>() {
                    return Some((val * mult_pf * 1_000.0).round() / 1_000.0);
                }
            }
        }
    }

    // Check suffix: pF, nF, uF, µF, p, n, u, µ
    let suffixes: &[(&str, f64)] = &[
        ("pF", 1.0),
        ("pf", 1.0),
        ("PF", 1.0),
        ("nF", 1_000.0),
        ("nf", 1_000.0),
        ("NF", 1_000.0),
        ("uF", 1_000_000.0),
        ("uf", 1_000_000.0),
        ("UF", 1_000_000.0),
        ("µF", 1_000_000.0),
        ("µf", 1_000_000.0),
        ("p", 1.0),
        ("n", 1_000.0),
        ("u", 1_000_000.0),
        ("µ", 1_000_000.0),
    ];

    for &(suf, mult_pf) in suffixes {
        if let Some(num_str) = s.strip_suffix(suf) {
            if !num_str.is_empty() && num_str.chars().all(|c| c.is_ascii_digit() || c == '.') {
                if let Ok(val) = num_str.parse::<f64>() {
                    return Some((val * mult_pf * 1_000.0).round() / 1_000.0);
                }
            }
        }
    }

    None
}

fn try_parse_inductance_token(tok: &str) -> Option<f64> {
    let s = tok.replace('μ', "µ");
    // Only match tokens ending with H or h (like 10mH, 100uH, 4u7H, 1H)
    let without_h = s.strip_suffix('H').or_else(|| s.strip_suffix('h'))?;
    if without_h.is_empty() {
        return None;
    }

    // Radix-replaced e.g. 4u7H, 2m2H
    for (sep, mult_uh) in [('u', 1.0), ('U', 1.0), ('µ', 1.0), ('m', 1_000.0), ('M', 1_000.0)] {
        if let Some((int_part, frac_part)) = without_h.split_once(sep) {
            if !int_part.is_empty()
                && !frac_part.is_empty()
                && int_part.chars().all(|c| c.is_ascii_digit())
                && frac_part.chars().all(|c| c.is_ascii_digit())
            {
                let combined = format!("{int_part}.{frac_part}");
                if let Ok(val) = combined.parse::<f64>() {
                    return Some(val * mult_uh);
                }
            }
        }
    }

    // Standard suffix before H: n, u, µ, m, or plain number (Henries)
    if let Some(num_str) = without_h.strip_suffix('n').or_else(|| without_h.strip_suffix('N')) {
        let val = num_str.parse::<f64>().ok()?;
        return Some(val * 0.001);
    }
    if let Some(num_str) = without_h
        .strip_suffix('u')
        .or_else(|| without_h.strip_suffix('U'))
        .or_else(|| without_h.strip_suffix('µ'))
    {
        let val = num_str.parse::<f64>().ok()?;
        return Some(val);
    }
    if let Some(num_str) = without_h.strip_suffix('m').or_else(|| without_h.strip_suffix('M')) {
        let val = num_str.parse::<f64>().ok()?;
        return Some(val * 1_000.0);
    }
    if without_h.chars().all(|c| c.is_ascii_digit() || c == '.') {
        let val = without_h.parse::<f64>().ok()?;
        return Some(val * 1_000_000.0);
    }

    None
}

fn try_parse_resistance_or_pot_token(
    tok: &str,
    category_hint: Option<Category>,
) -> Option<(f64, Option<String>)> {
    // Reject obvious non-resistance tokens (e.g. packages like 0805, 0603, 1206, 0402, 16mm, 5mm, 125B, or part numbers like 1N34A, 2N5457)
    let upper = tok.to_uppercase();
    if matches!(
        upper.as_str(),
        "0402" | "0603" | "0805" | "1206" | "1210" | "2512" | "125B" | "1590B" | "1590BB"
    ) {
        return None;
    }
    if upper.ends_with("MM")
        || upper.starts_with("1N")
        || upper.starts_with("2N")
        || upper.starts_with("2S")
        || upper.starts_with("BC")
        || upper.starts_with("BF")
        || upper.starts_with("TL")
        || upper.starts_with("LM")
        || upper.starts_with("NE")
        || upper.starts_with("J2")
        || upper.starts_with("PT")
        || upper.starts_with("CD")
        || upper.starts_with("MN")
    {
        return None;
    }

    let mut working = tok;
    let mut extracted_taper: Option<String> = None;

    // Check leading taper prefix e.g. B50k, A100k, C10k, W20k
    if working.len() >= 3 {
        let first = working.chars().next().unwrap();
        let rest = &working[first.len_utf8()..];
        if matches!(first, 'A' | 'B' | 'C' | 'W')
            && rest.chars().next().is_some_and(|c| c.is_ascii_digit())
            && (rest.contains('k')
                || rest.contains('K')
                || rest.contains('M')
                || rest.contains('R')
                || rest.contains('Ω'))
        {
            extracted_taper = Some(first.to_string());
            working = rest;
        }
    }

    // Strip trailing ohm symbols: Ω, ohm, ohms
    let working_no_ohm = working
        .strip_suffix("ohms")
        .or_else(|| working.strip_suffix("OHMS"))
        .or_else(|| working.strip_suffix("ohm"))
        .or_else(|| working.strip_suffix("OHM"))
        .or_else(|| working.strip_suffix('Ω'))
        .or_else(|| working.strip_suffix('ω'))
        .unwrap_or(working);

    // Check trailing taper suffix e.g. 100kA, 100kB, 50kC, 20kW, 4k7B
    let working_core = if extracted_taper.is_none() && working_no_ohm.len() >= 3 {
        let last = working_no_ohm.chars().last().unwrap();
        let without_last = &working_no_ohm[..working_no_ohm.len() - last.len_utf8()];
        if matches!(last, 'A' | 'B' | 'C' | 'W')
            && (without_last.ends_with('k')
                || without_last.ends_with('K')
                || without_last.ends_with('M')
                || without_last.ends_with('R')
                || without_last.chars().any(|c| matches!(c, 'k' | 'K' | 'M' | 'R' | 'r')))
        {
            extracted_taper = Some(last.to_string());
            without_last
        } else {
            working_no_ohm
        }
    } else {
        working_no_ohm
    };

    // 1) Radix-replaced resistance: 4k7, 4K7, 4R7, 4r7, 2M2, 2m2
    for (sep, mult) in [
        ('r', 1.0),
        ('R', 1.0),
        ('k', 1_000.0),
        ('K', 1_000.0),
        ('m', 1_000_000.0),
        ('M', 1_000_000.0),
    ] {
        if let Some((int_part, frac_part)) = working_core.split_once(sep) {
            if !int_part.is_empty()
                && !frac_part.is_empty()
                && int_part.chars().all(|c| c.is_ascii_digit())
                && frac_part.chars().all(|c| c.is_ascii_digit())
            {
                let combined = format!("{int_part}.{frac_part}");
                if let Ok(val) = combined.parse::<f64>() {
                    return Some(((val * mult * 1_000.0).round() / 1_000.0, extracted_taper));
                }
            }
        }
    }

    // 2) Standard suffix: k, K, M, R, r
    for (suf, mult) in [
        ("k", 1_000.0),
        ("K", 1_000.0),
        ("M", 1_000_000.0),
        ("R", 1.0),
        ("r", 1.0),
    ] {
        if let Some(num_str) = working_core.strip_suffix(suf) {
            if !num_str.is_empty() && num_str.chars().all(|c| c.is_ascii_digit() || c == '.') {
                if let Ok(val) = num_str.parse::<f64>() {
                    return Some(((val * mult * 1_000.0).round() / 1_000.0, extracted_taper));
                }
            }
        }
    }

    // 3) If explicit ohm suffix was stripped or category_hint is Resistors, allow bare number e.g. "4700Ω" or "4700"
    let had_ohm_symbol = working_no_ohm.len() < working.len();
    if (had_ohm_symbol || category_hint == Some(Category::Resistors))
        && !working_core.is_empty()
        && working_core.chars().all(|c| c.is_ascii_digit() || c == '.')
    {
        if let Ok(val) = working_core.parse::<f64>() {
            return Some((val, extracted_taper));
        }
    }

    None
}

fn extract_package_or_mounting(raw: &str, tokens: &[&str]) -> Option<String> {
    let lower = raw.to_lowercase();

    // Check multi-word package patterns first
    for phrase in [
        "16mm PCB Right-Angle",
        "16mm PCB",
        "9mm PCB",
        "24mm PCB",
        "Through-Hole 5mm",
        "SMD 0805",
        "SMD 0603",
        "SMD 1206",
        "SMD 0402",
        "DO-35 Axial",
        "DO-7 Axial",
        "Through-Hole",
    ] {
        if lower.contains(&phrase.to_lowercase()) {
            return Some(phrase.to_string());
        }
    }

    // Check single-token package patterns
    for &tok in tokens {
        let clean = tok.trim_matches(|c: char| c == ',' || c == ';');
        let u = clean.to_uppercase();
        if u.starts_with("DIP-")
            || u.starts_with("SOIC-")
            || u.starts_with("TSSOP-")
            || u.starts_with("TO-")
            || u.starts_with("SOT-")
            || u.starts_with("DO-")
            || matches!(
                u.as_str(),
                "0402"
                    | "0603"
                    | "0805"
                    | "1206"
                    | "1210"
                    | "2512"
                    | "DIP8"
                    | "DIP14"
                    | "DIP16"
                    | "16MM"
                    | "9MM"
                    | "24MM"
                    | "125B"
                    | "1590B"
                    | "1590BB"
                    | "THT"
                    | "SMD"
            )
        {
            if u == "DIP8" {
                return Some("DIP-8".to_string());
            }
            return Some(clean.to_string());
        }
    }

    None
}

fn extract_composition(raw: &str) -> Option<String> {
    let lower = raw.to_lowercase();
    for comp in [
        "Metal Film",
        "Carbon Film",
        "Carbon Comp",
        "Carbon Composition",
        "Metal Oxide",
        "Wirewound",
        "Thin Film",
        "Thick Film",
    ] {
        if lower.contains(&comp.to_lowercase()) {
            return Some(comp.to_string());
        }
    }
    None
}

fn extract_dielectric(raw: &str) -> Option<String> {
    let lower = raw.to_lowercase();
    for diel in [
        "Box Film",
        "Polyester Film",
        "Polypropylene Film",
        "Film",
        "Electrolytic",
        "Ceramic",
        "MLCC",
        "Tantalum",
        "Silver Mica",
        "Mica",
        "C0G",
        "NP0",
        "X7R",
    ] {
        if lower.contains(&diel.to_lowercase()) {
            return Some(diel.to_string());
        }
    }
    None
}

fn extract_standalone_taper(
    raw: &str,
    tokens: &[&str],
    category_hint: Option<Category>,
    package: Option<&str>,
) -> Option<String> {
    let lower = raw.to_lowercase();
    if lower.contains("audio taper") || lower.contains("log taper") || lower.contains("taper a") {
        return Some("A".to_string());
    }
    if lower.contains("linear taper") || lower.contains("taper b") {
        return Some("B".to_string());
    }
    if lower.contains("reverse audio") || lower.contains("rev log") || lower.contains("taper c") {
        return Some("C".to_string());
    }
    if lower.contains("taper w") {
        return Some("W".to_string());
    }

    let is_pot_context = category_hint == Some(Category::PotentiometersAndEncoders)
        || lower.contains("pot")
        || lower.contains("dual-gang")
        || package.is_some_and(|p| p.contains("16mm") || p.contains("9mm") || p.contains("24mm"));

    if is_pot_context {
        for &tok in tokens {
            if matches!(tok, "A" | "B" | "C" | "W") {
                return Some(tok.to_string());
            }
        }
    }
    None
}

fn extract_part_code(tokens: &[&str]) -> Option<String> {
    for &tok in tokens {
        let clean = tok.trim_matches(|c: char| !c.is_alphanumeric() && c != '-');
        if clean.len() < 3 {
            continue;
        }
        let u = clean.to_uppercase();
        if u.starts_with("DIP-")
            || u.starts_with("SOIC-")
            || u.starts_with("TO-")
            || u.starts_with("DO-")
            || u.starts_with("SOT-")
            || u.ends_with("MM")
            || u.ends_with("UF")
            || u.ends_with("NF")
            || u.ends_with("PF")
            || u.ends_with("OHM")
            || matches!(u.as_str(), "125B" | "1590B" | "1590BB" | "SMD" | "THT" | "PCB")
        {
            continue;
        }
        // Standard semiconductor / IC prefixes (1N..., 2N..., TL..., LM..., NE..., PT..., J201, BC..., BS..., BAT..., MN..., CD...)
        let has_alpha = u.chars().any(|c| c.is_ascii_alphabetic());
        let has_digit = u.chars().any(|c| c.is_ascii_digit());
        if has_alpha
            && has_digit
            && (u.starts_with("1N")
                || u.starts_with("2N")
                || u.starts_with("2S")
                || u.starts_with("TL")
                || u.starts_with("LM")
                || u.starts_with("NE")
                || u.starts_with("OP")
                || u.starts_with("RC")
                || u.starts_with("PT")
                || u.starts_with("MN")
                || u.starts_with("CD")
                || u.starts_with("BC")
                || u.starts_with("BF")
                || u.starts_with("BS")
                || u.starts_with("BAT")
                || u.starts_with("MPF")
                || u.starts_with("J2")
                || u.starts_with("J1")
                || u.starts_with("LF")
                || u.starts_with("CA")
                || u.starts_with("SSM")
                || u.starts_with("V2")
                || u.starts_with("BTDR"))
        {
            return Some(clean.to_string());
        }
    }
    None
}

fn infer_category_from_part_code(code: &str) -> Option<Category> {
    let u = code.to_uppercase();
    if u.starts_with("1N")
        || u.starts_with("2N")
        || u.starts_with("2S")
        || u.starts_with("BC")
        || u.starts_with("BF")
        || u.starts_with("BS")
        || u.starts_with("BAT")
        || u.starts_with("MPF")
        || u.starts_with("J2")
        || u.starts_with("J1")
    {
        Some(Category::SemiconductorsDiscrete)
    } else if u.starts_with("TL")
        || u.starts_with("LM")
        || u.starts_with("NE")
        || u.starts_with("OP")
        || u.starts_with("RC")
        || u.starts_with("PT")
        || u.starts_with("MN")
        || u.starts_with("CD")
        || u.starts_with("LF")
        || u.starts_with("CA")
        || u.starts_with("SSM")
        || u.starts_with("V2")
    {
        Some(Category::IntegratedCircuits)
    } else if u.starts_with("BTDR") {
        Some(Category::ModulesAndSubAssemblies)
    } else {
        None
    }
}

pub fn infer_category_from_designators(designators: &str) -> Option<Category> {
    for raw_tok in designators.split(|c: char| {
        c == ',' || c == ';' || c.is_whitespace() || c == '/' || c == '(' || c == ')'
    }) {
        let tok = raw_tok.trim();
        if tok.is_empty() {
            continue;
        }
        // Handle ranges like R1-R4 by taking first side
        let first_part = tok.split('-').next().unwrap_or(tok);
        let prefix: String = first_part
            .chars()
            .take_while(|c| c.is_ascii_alphabetic() || *c == '_')
            .collect();
        if prefix.is_empty() {
            continue;
        }
        let u = prefix.to_uppercase();
        let cat = match u.as_str() {
            "R" | "RN" | "PR" => Some(Category::Resistors),
            "C" | "CP" => Some(Category::Capacitors),
            "D" | "LED" | "Q" | "TR" | "ZD" | "CR" => Some(Category::SemiconductorsDiscrete),
            "U" | "IC" | "OK" | "OP" => Some(Category::IntegratedCircuits),
            "RV" | "VR" | "POT" | "ENC" | "TM" | "TRIM" => {
                Some(Category::PotentiometersAndEncoders)
            }
            "SW" | "S" | "J" | "JACK" | "DC" | "P" | "CON" | "FSW" | "CN" => {
                Some(Category::SwitchesJacksAndConnectors)
            }
            "L" | "T" | "TR_MAG" | "FB" | "IND" => {
                Some(Category::InductorsTransformersAndMagnetics)
            }
            "PCB" | "ENC_HW" | "HW" | "KNOB" | "CASE" | "BOX" => {
                Some(Category::EnclosuresKnobsAndHardware)
            }
            "MOD" | "MCU" | "DSP" => Some(Category::ModulesAndSubAssemblies),
            _ => None,
        };
        if cat.is_some() {
            return cat;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_resistance_and_potentiometer_shorthand() {
        for input in ["4k7", "4.7k", "4700R", "4.7kΩ"] {
            let parsed = parse_spec(input, None);
            assert_eq!(parsed.raw_spec_text, input);
            assert_eq!(parsed.resistance_ohms, Some(4700.0), "input: {input}");
            assert_eq!(parsed.canonical_value_display, "4.7kΩ", "input: {input}");
        }

        let pot_a = parse_spec("100kA 16mm PCB", None);
        assert_eq!(pot_a.raw_spec_text, "100kA 16mm PCB");
        assert_eq!(pot_a.resistance_ohms, Some(100_000.0));
        assert_eq!(pot_a.canonical_value_display, "100kΩ");
        assert_eq!(pot_a.taper.as_deref(), Some("A"));
        assert_eq!(pot_a.package_or_mounting.as_deref(), Some("16mm PCB"));

        let pot_b = parse_spec("B50k", None);
        assert_eq!(pot_b.raw_spec_text, "B50k");
        assert_eq!(pot_b.resistance_ohms, Some(50_000.0));
        assert_eq!(pot_b.canonical_value_display, "50kΩ");
        assert_eq!(pot_b.taper.as_deref(), Some("B"));
    }

    #[test]
    fn normalizes_capacitance_shorthand_and_formats_1nf_to_999nf_as_nf() {
        for input in ["0.047uF", "47n", "47000pF", "47nF"] {
            let parsed = parse_spec(input, None);
            assert_eq!(parsed.raw_spec_text, input);
            assert_eq!(parsed.capacitance_pf, Some(47_000.0), "input: {input}");
            assert_eq!(parsed.canonical_value_display, "47nF", "input: {input}");
        }

        let c_4n7 = parse_spec("4n7", None);
        assert_eq!(c_4n7.raw_spec_text, "4n7");
        assert_eq!(c_4n7.capacitance_pf, Some(4_700.0));
        assert_eq!(c_4n7.canonical_value_display, "4.7nF");

        let c_1nf = parse_spec("0.001uF", None);
        assert_eq!(c_1nf.capacitance_pf, Some(1_000.0));
        assert_eq!(c_1nf.canonical_value_display, "1nF");

        let c_100pf = parse_spec("100pF", None);
        assert_eq!(c_100pf.capacitance_pf, Some(100.0));
        assert_eq!(c_100pf.canonical_value_display, "100pF");

        let c_100uf = parse_spec("100uF 16V Electrolytic", None);
        assert_eq!(c_100uf.capacitance_pf, Some(100_000_000.0));
        assert_eq!(c_100uf.canonical_value_display, "100µF");
        assert_eq!(c_100uf.voltage_rating_volts, Some(16.0));
    }

    #[test]
    fn extracts_tolerance_power_voltage_and_package_while_preserving_raw_spec_text() {
        let r = parse_spec("4k7 1% 1/4W Metal Film 0805", Some(Category::Resistors));
        assert_eq!(r.raw_spec_text, "4k7 1% 1/4W Metal Film 0805");
        assert_eq!(r.resistance_ohms, Some(4700.0));
        assert_eq!(r.canonical_value_display, "4.7kΩ");
        assert_eq!(r.tolerance.as_deref(), Some("1%"));
        assert_eq!(r.power_rating_watts, Some(0.25));
        assert_eq!(r.composition.as_deref(), Some("Metal Film"));
        assert_eq!(r.package_or_mounting.as_deref(), Some("0805"));

        let r2 = parse_spec("10k 5% 0.25W Carbon Film", None);
        assert_eq!(r2.resistance_ohms, Some(10_000.0));
        assert_eq!(r2.tolerance.as_deref(), Some("5%"));
        assert_eq!(r2.power_rating_watts, Some(0.25));

        let c = parse_spec("0.047uF 63V 5% Box Film 5mm", Some(Category::Capacitors));
        assert_eq!(c.raw_spec_text, "0.047uF 63V 5% Box Film 5mm");
        assert_eq!(c.capacitance_pf, Some(47_000.0));
        assert_eq!(c.canonical_value_display, "47nF");
        assert_eq!(c.voltage_rating_volts, Some(63.0));
        assert_eq!(c.tolerance.as_deref(), Some("5%"));
        assert_eq!(c.dielectric.as_deref(), Some("Box Film"));

        let ic = parse_spec("TL072IP Dual Op-Amp DIP-8", None);
        assert_eq!(ic.raw_spec_text, "TL072IP Dual Op-Amp DIP-8");
        assert_eq!(ic.part_code.as_deref(), Some("TL072IP"));
        assert_eq!(ic.package_or_mounting.as_deref(), Some("DIP-8"));
        assert_eq!(ic.inferred_category, Some(Category::IntegratedCircuits));
    }

    #[test]
    fn infers_category_from_reference_designators() {
        assert_eq!(
            infer_category_from_designators("R1, R2, R5"),
            Some(Category::Resistors)
        );
        assert_eq!(
            infer_category_from_designators("C3, C6"),
            Some(Category::Capacitors)
        );
        assert_eq!(
            infer_category_from_designators("D1, D2"),
            Some(Category::SemiconductorsDiscrete)
        );
        assert_eq!(
            infer_category_from_designators("Q1, Q2"),
            Some(Category::SemiconductorsDiscrete)
        );
        assert_eq!(
            infer_category_from_designators("U1"),
            Some(Category::IntegratedCircuits)
        );
        assert_eq!(
            infer_category_from_designators("IC1, IC2"),
            Some(Category::IntegratedCircuits)
        );
        assert_eq!(
            infer_category_from_designators("RV1"),
            Some(Category::PotentiometersAndEncoders)
        );
        assert_eq!(
            infer_category_from_designators("VR1 (GAIN)"),
            Some(Category::PotentiometersAndEncoders)
        );
        assert_eq!(
            infer_category_from_designators("SW1"),
            Some(Category::SwitchesJacksAndConnectors)
        );
        assert_eq!(
            infer_category_from_designators("J1, J2"),
            Some(Category::SwitchesJacksAndConnectors)
        );
        assert_eq!(
            infer_category_from_designators("L1"),
            Some(Category::InductorsTransformersAndMagnetics)
        );
        assert_eq!(
            infer_category_from_designators("T1"),
            Some(Category::InductorsTransformersAndMagnetics)
        );
    }
}
