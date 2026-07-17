// Byte size formatting.
// 翻译自 packages/normalization-core/src/format.ts

use crate::expect::expect_defined;
use crate::error_coercion::CrError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteSizeUnit {
    Byte,
    Kilo,
    Mega,
    Giga,
    Tera,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteSizeStyle {
    Iec,
    LegacyBinary,
}

#[derive(Debug)]
pub struct ByteSizeFormatOptions {
    pub style: ByteSizeStyle,
    pub max_unit: ByteSizeUnit,
    pub separator: &'static str, // "" | " "
    pub fraction_digits: FractionDigits,
    pub floor_units: Option<Vec<ByteSizeUnit>>,
}

pub enum FractionDigits {
    Fixed(i32),
    Dynamic(Box<dyn Fn(f64, ByteSizeUnit) -> Option<i32> + Send + Sync>),
}

impl std::fmt::Debug for FractionDigits {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FractionDigits::Fixed(n) => write!(f, "Fixed({})", n),
            FractionDigits::Dynamic(_) => write!(f, "Dynamic(<closure>)"),
        }
    }
}

const BYTE_SIZE_UNITS: &[ByteSizeUnit] = &[
    ByteSizeUnit::Byte,
    ByteSizeUnit::Kilo,
    ByteSizeUnit::Mega,
    ByteSizeUnit::Giga,
    ByteSizeUnit::Tera,
];

fn style_base(style: ByteSizeStyle) -> f64 {
    match style {
        ByteSizeStyle::Iec => 1024.0,
        ByteSizeStyle::LegacyBinary => 1024.0,
    }
}

fn style_labels(style: ByteSizeStyle) -> &'static [&'static str; 5] {
    match style {
        ByteSizeStyle::Iec => &["B", "KiB", "MiB", "GiB", "TiB"],
        ByteSizeStyle::LegacyBinary => &["B", "KB", "MB", "GB", "TB"],
    }
}

fn unit_index(unit: ByteSizeUnit) -> usize {
    match unit {
        ByteSizeUnit::Byte => 0,
        ByteSizeUnit::Kilo => 1,
        ByteSizeUnit::Mega => 2,
        ByteSizeUnit::Giga => 3,
        ByteSizeUnit::Tera => 4,
    }
}

/// Formats a byte count with caller-explicit scale, labels, precision, and unit cap.
pub fn format_byte_size(bytes: f64, options: &ByteSizeFormatOptions) -> Result<String, CrError> {
    let base = style_base(options.style);
    let labels = style_labels(options.style);
    let max_unit_index = unit_index(options.max_unit);

    let mut unit_index_val = 0usize;
    let mut value = bytes;
    while value >= base && unit_index_val < max_unit_index {
        value /= base;
        unit_index_val += 1;
    }

    let unit = expect_defined(Some(BYTE_SIZE_UNITS[unit_index_val]), "byte-size unit")?;
    let label = labels[unit_index_val];

    let fraction_digits = match &options.fraction_digits {
        FractionDigits::Fixed(n) => Some(*n),
        FractionDigits::Dynamic(f) => f(value, unit),
    };

    let floor_units = options.floor_units.as_ref();
    let value = if let Some(fd) = fraction_digits {
        if fd < 0 {
            return Ok(format!("{}{}{}", value, options.separator, label));
        }
        if let Some(floors) = floor_units {
            if floors.contains(&unit) {
                let pow = 10f64.powi(fd);
                (value * pow).floor() / pow
            } else {
                value
            }
        } else {
            value
        }
    } else {
        return Ok(format!("{}{}{}", value, options.separator, label));
    };

    let fd = fraction_digits.unwrap_or(0);
    Ok(format!("{:.fd$}{}{}", value, options.separator, label, fd = fd as usize))
}
