pub mod dummy;
pub mod env;
pub mod nvidia;
pub mod rocm;

use anyhow::Result;
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Debug, Clone, serde::Serialize)]
pub struct GpuMetrics {
    pub temp: f32,
    pub load: u32,
    pub power_consumption: f32,
    pub power_limit: u32,
    pub vram_used: u64,
    pub vram_total: u64,
    pub sclk_mhz: u32,
    pub mclk_mhz: u32,
}

pub trait GpuBackend: Send + Sync + 'static {
    fn read_metrics(&self) -> Result<BTreeMap<String, GpuMetrics>>;
    #[allow(dead_code)]
    fn name(&self) -> &str;
}

/// Polls several backends and merges their metrics into one map, so machines
/// with a mix of vendors (e.g. AMD + NVIDIA cards) report every GPU. A failure
/// in one backend is logged and skipped without hiding the others.
pub struct MultiBackend {
    backends: Vec<Arc<dyn GpuBackend>>,
}

impl GpuBackend for MultiBackend {
    fn read_metrics(&self) -> Result<BTreeMap<String, GpuMetrics>> {
        let mut all = BTreeMap::new();
        for backend in &self.backends {
            match backend.read_metrics() {
                Ok(metrics) => all.extend(metrics),
                Err(e) => eprintln!("[error] GPU metrics ({}): {e}", backend.name()),
            }
        }
        Ok(all)
    }

    fn name(&self) -> &str {
        "multi"
    }
}

pub fn detect_backend(force: &str) -> Arc<dyn GpuBackend> {
    match force {
        "rocm" => Arc::new(rocm::RocmBackend),
        "nvidia" => Arc::new(nvidia::NvidiaBackend),
        "none" => Arc::new(dummy::DummyBackend),
        // "auto" / "all" / anything else: monitor every vendor whose tool is present
        _ => {
            let mut backends: Vec<Arc<dyn GpuBackend>> = Vec::new();
            if command_exists("rocm-smi") {
                backends.push(Arc::new(rocm::RocmBackend));
            }
            if command_exists("nvidia-smi") {
                backends.push(Arc::new(nvidia::NvidiaBackend));
            }
            match backends.len() {
                0 => {
                    eprintln!("[warn] No GPU monitoring tool found (rocm-smi / nvidia-smi)");
                    Arc::new(dummy::DummyBackend)
                }
                1 => backends.into_iter().next().unwrap(),
                _ => Arc::new(MultiBackend { backends }),
            }
        }
    }
}

fn command_exists(cmd: &str) -> bool {
    std::process::Command::new("which")
        .arg(cmd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StubBackend {
        name: &'static str,
        cards: Vec<&'static str>,
        fail: bool,
    }

    impl GpuBackend for StubBackend {
        fn read_metrics(&self) -> Result<BTreeMap<String, GpuMetrics>> {
            if self.fail {
                anyhow::bail!("stub failure");
            }
            Ok(self
                .cards
                .iter()
                .map(|c| {
                    (
                        c.to_string(),
                        GpuMetrics {
                            temp: 0.0,
                            load: 0,
                            power_consumption: 0.0,
                            power_limit: 0,
                            vram_used: 0,
                            vram_total: 0,
                            sclk_mhz: 0,
                            mclk_mhz: 0,
                        },
                    )
                })
                .collect())
        }

        fn name(&self) -> &str {
            self.name
        }
    }

    #[test]
    fn multi_backend_merges_all_vendors() {
        let multi = MultiBackend {
            backends: vec![
                Arc::new(StubBackend {
                    name: "rocm",
                    cards: vec!["card0", "card1"],
                    fail: false,
                }),
                Arc::new(StubBackend {
                    name: "nvidia",
                    cards: vec!["GPU0 NVIDIA"],
                    fail: false,
                }),
            ],
        };
        let metrics = multi.read_metrics().unwrap();
        assert_eq!(metrics.len(), 3);
        assert!(metrics.contains_key("card0"));
        assert!(metrics.contains_key("card1"));
        assert!(metrics.contains_key("GPU0 NVIDIA"));
    }

    #[test]
    fn multi_backend_skips_failing_backend() {
        let multi = MultiBackend {
            backends: vec![
                Arc::new(StubBackend {
                    name: "rocm",
                    cards: vec!["card0"],
                    fail: true,
                }),
                Arc::new(StubBackend {
                    name: "nvidia",
                    cards: vec!["GPU0 NVIDIA"],
                    fail: false,
                }),
            ],
        };
        let metrics = multi.read_metrics().unwrap();
        assert_eq!(metrics.len(), 1);
        assert!(metrics.contains_key("GPU0 NVIDIA"));
    }
}
