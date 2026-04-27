#![allow(dead_code)]

use clap::ValueEnum;
use fbool::certificate::CertificateComplexity;
use fbool::fvalue::FValue;
use fbool::optimal5::WithMinimalGates;
use fbool::sensitivity::Sensitivity;
use fbool::{Entanglement, Entropy, Fragmentation, StructuralMetrics};
use polars::prelude::*;

pub trait Metric: Sync + Send {
    fn column_names(&self) -> Vec<&str>;

    fn column_types(&self) -> Vec<DataType>;

    fn compute(&self, npn: u32) -> Vec<AnyValue<'static>>;
}

macro_rules! define_metrics {
    (
        $(
            $variant:ident => {
                struct: $name:ident,
                columns: [$($col:literal),+ $(,)?],
                types: [$($dtype:expr),+ $(,)?],
                compute: |$f:ident| $body:expr
            }
        ),+ $(,)?
    ) => {
        #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
        pub enum MetricType {
            $($variant,)+
            All,
        }

        pub const METRIC_ORDER: [MetricType; define_metrics!(@count $($variant),+)] = [
            $(MetricType::$variant,)+
        ];

        pub fn metric_columns(metric: MetricType) -> Vec<&'static str> {
            match metric {
                $(MetricType::$variant => vec![$($col),+],)+
                MetricType::All => METRIC_ORDER
                    .iter()
                    .flat_map(|m| metric_columns(*m))
                    .collect(),
            }
        }

        pub fn metric_impl(metric: MetricType) -> Option<Box<dyn Metric>> {
            match metric {
                $(MetricType::$variant => Some(Box::new($name)),)+
                MetricType::All => None,
            }
        }

        $(
            struct $name;

            impl Metric for $name {
                fn column_names(&self) -> Vec<&str> {
                    vec![$($col),+]
                }

                fn column_types(&self) -> Vec<DataType> {
                    vec![$($dtype),+]
                }

                fn compute(&self, npn: u32) -> Vec<AnyValue<'static>> {
                    let $f = FValue::<bool>::from_usize(npn as usize, 5);
                    $body
                }
            }
        )+
    };

    (@count $head:ident $(, $tail:ident)*) => {
        1usize $(+ { let _ = stringify!($tail); 1usize })*
    };
}

define_metrics!(
    Gates => {
        struct: GatesMetric,
        columns: ["min_gates"],
        types: [DataType::UInt32],
        compute: |f| vec![AnyValue::UInt32(f.minimal_gates().unwrap_or(0))]
    },
    EntanglementEntropy => {
        struct: EntanglementEntropyMetric,
        columns: ["entanglement_entropy"],
        types: [DataType::Float32],
        compute: |f| vec![AnyValue::Float32(f.entropy())]
    },
    Entanglement => {
        struct: EntanglementMetric,
        columns: ["entanglement"],
        types: [DataType::UInt32],
        compute: |f| vec![AnyValue::UInt32(f.entanglement() as u32)]
    },
    Influence => {
        struct: InfluenceMetric,
        columns: ["influence"],
        types: [DataType::Float32],
        compute: |f| vec![AnyValue::Float32(f.total_influence())]
    },
    SpectralEntropy => {
        struct: SpectralEntropyMetric,
        columns: ["spectral_entropy"],
        types: [DataType::Float32],
        compute: |f| vec![AnyValue::Float32(f.spectral_entropy())]
    },
    Nolinearity => {
        struct: NolinearityMetric,
        columns: ["nolinearity"],
        types: [DataType::UInt32],
        compute: |f| vec![AnyValue::UInt32(f.no_linearity())]
    },
    SimpleEntropy => {
        struct: EntropyMetric,
        columns: ["entropy"],
        types: [DataType::Float32],
        compute: |f| vec![AnyValue::Float32(f.fragmentation_k(5))]
    },
    Sensitivity => {
        struct: SensitivityMetric,
        columns: ["sensitivity"],
        types: [DataType::UInt32],
        compute: |f| vec![AnyValue::UInt32(f.max_sensitivity())]
    },
    CertificateComplexity => {
        struct: CertificateComplexityMetric,
        columns: ["certificate_complexity"],
        types: [DataType::UInt32],
        compute: |f| vec![AnyValue::UInt32(f.certificate_complexity())]
    },
    Degree => {
        struct: DegreeMetric,
        columns: ["degree"],
        types: [DataType::UInt32],
        compute: |f| vec![AnyValue::UInt32(f.degree() as u32)]
    },
    Counting => {
        struct: CountingMetric,
        columns: ["counting"],
        types: [DataType::UInt32],
        compute: |f| vec![AnyValue::UInt32(f.counting())]
    },
    Repetitiveness => {
        struct: RepetitivenessMetric,
        columns: ["repetitiveness"],
        types: [DataType::UInt32],
        compute: |f| vec![AnyValue::UInt32(f.repetitiveness())]
    },
    FragmentationProfile => {
        struct: FragmentationProfileMetric,
        columns: ["fp_k0", "fp_k1", "fp_k2", "fp_k3", "fp_k4", "fp_k5"],
        types: [
            DataType::Float32,
            DataType::Float32,
            DataType::Float32,
            DataType::Float32,
            DataType::Float32,
            DataType::Float32,
        ],
        compute: |f| {
            let profile = f.fragmentation_profile();
            vec![
                AnyValue::Float32(profile[0]),
                AnyValue::Float32(profile[1]),
                AnyValue::Float32(profile[2]),
                AnyValue::Float32(profile[3]),
                AnyValue::Float32(profile[4]),
                AnyValue::Float32(profile[5]),
            ]
        }
    },
    FragmentationPeak => {
        struct: FragmentationPeakMetric,
        columns: ["k_star", "s_max"],
        types: [DataType::UInt32, DataType::Float32],
        compute: |f| {
            let peak = f.fragmentation_peak();
            vec![
                AnyValue::UInt32(peak.k_star as u32),
                AnyValue::Float32(peak.s_max),
            ]
        }
    },

);
