// Imports
use rand::{Rng, SeedableRng};
use rnote_compose::Color;
use rnote_compose::Constraints;
use rnote_compose::Style;
use rnote_compose::builders::ShapeBuilderType;
use rnote_compose::constraints::ConstraintRatio;
use rnote_compose::style::rough::RoughOptions;
use rnote_compose::style::smooth::SmoothOptions;
use serde::{Deserialize, Serialize};

#[derive(
    Copy, Clone, Debug, Serialize, Deserialize, num_derive::FromPrimitive, num_derive::ToPrimitive,
)]
#[serde(rename = "shaper_style")]
pub enum ShaperStyle {
    #[serde(rename = "smooth")]
    Smooth = 0,
    #[serde(rename = "rough")]
    Rough,
}

impl Default for ShaperStyle {
    fn default() -> Self {
        Self::Smooth
    }
}

impl TryFrom<u32> for ShaperStyle {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        num_traits::FromPrimitive::from_u32(value).ok_or_else(|| {
            anyhow::anyhow!("ShaperStyle try_from::<u32>() for value {} failed", value)
        })
    }
}

#[derive(
    Copy, Clone, Debug, Serialize, Deserialize, num_derive::FromPrimitive, num_derive::ToPrimitive,
)]
#[serde(rename = "shaper_brush_style")]
pub enum ShaperBrushType {
    #[serde(rename = "solid")]
    Solid = 0,
    #[serde(rename = "highlighter")]
    Highlighter,
}

impl Default for ShaperBrushType {
    fn default() -> Self {
        Self::Solid
    }
}

impl TryFrom<u32> for ShaperBrushType {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        num_traits::FromPrimitive::from_u32(value).ok_or_else(|| {
            anyhow::anyhow!(
                "ShaperBrushType try_from::<u32>() for value {} failed",
                value
            )
        })
    }
}

impl ShaperBrushType {
    pub(crate) const HIGHLIGHTER_STROKE_COLOR_ALPHA: f64 = 0.45;
    pub(crate) const HIGHLIGHTER_FILL_COLOR_ALPHA: f64 = 0.5;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename = "shaper_config")]
pub struct ShaperConfig {
    #[serde(rename = "builder_type")]
    pub builder_type: ShapeBuilderType,
    #[serde(rename = "style")]
    pub style: ShaperStyle,
    #[serde(rename = "brush_type")]
    pub brush_type: ShaperBrushType,
    #[serde(rename = "smooth_options")]
    pub smooth_options: SmoothOptions,
    #[serde(rename = "rough_options")]
    pub rough_options: RoughOptions,
    #[serde(rename = "constraints")]
    pub constraints: Constraints,
}

impl Default for ShaperConfig {
    fn default() -> Self {
        let mut constraints = Constraints::default();
        constraints.ratios.insert(ConstraintRatio::OneToOne);
        constraints.ratios.insert(ConstraintRatio::Horizontal);
        constraints.ratios.insert(ConstraintRatio::Vertical);

        Self {
            builder_type: ShapeBuilderType::default(),
            style: ShaperStyle::default(),
            brush_type: ShaperBrushType::default(),
            smooth_options: SmoothOptions::default(),
            rough_options: RoughOptions::default(),
            constraints,
        }
    }
}

impl ShaperConfig {
    pub const STROKE_WIDTH_MIN: f64 = 0.1;
    pub const STROKE_WIDTH_MAX: f64 = 500.0;

    /// A new seed for new shapes
    pub(crate) fn new_style_seeds(&mut self) {
        let seed = Some(rand_pcg::Pcg64::from_os_rng().random());
        self.rough_options.seed = seed;
    }

    pub(crate) fn gen_style_for_current_options(&self) -> Style {
        match &self.style {
            ShaperStyle::Smooth => {
                let mut options = self.smooth_options.clone();
                if matches!(&self.brush_type, ShaperBrushType::Highlighter) {
                    if let Some(ref mut color) = options.stroke_color {
                        color.a = ShaperBrushType::HIGHLIGHTER_STROKE_COLOR_ALPHA
                    }
                    if let Some(ref mut color) = options.fill_color {
                        color.a = ShaperBrushType::HIGHLIGHTER_FILL_COLOR_ALPHA
                    }
                }

                Style::Smooth(options)
            }
            ShaperStyle::Rough => {
                let mut options = self.rough_options.clone();
                if matches!(&self.brush_type, ShaperBrushType::Highlighter) {
                    if let Some(ref mut color) = options.stroke_color {
                        color.a = ShaperBrushType::HIGHLIGHTER_STROKE_COLOR_ALPHA
                    }
                    if let Some(ref mut color) = options.fill_color {
                        color.a = ShaperBrushType::HIGHLIGHTER_FILL_COLOR_ALPHA
                    }
                }

                Style::Rough(options)
            }
        }
    }
}
