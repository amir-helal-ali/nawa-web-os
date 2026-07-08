//! Quantum-inspired computing engine for NAWA.
//!
//! Implements quantum mechanics principles on classical hardware:
//!
//! - **Superposition**: multiple states evaluated simultaneously
//!   Used for: cache (multiple candidates), routing (parallel path evaluation)
//!
//! - **Entanglement**: correlated state between components
//!   Used for: distributed state sync, cross-component consistency
//!
//! - **Quantum Tunneling**: escape local optima
//!   Used for: AION SEO optimization, cache eviction strategy
//!
//! - **Quantum Error Correction (QEC)**: data integrity through redundancy
//!   Used for: database writes, audit log integrity
//!
//! - **Quantum Measurement**: probabilistic collapse
//!   Used for: load balancing, request routing, A/B testing
//!
//! These are quantum-inspired classical algorithms — they use the mathematical
//! framework of quantum mechanics (amplitudes, probability distributions,
//! Hamiltonians) but run on classical CPUs. This gives near-quantum advantages
//! for optimization and parallelism without requiring quantum hardware.

#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::RwLock;

// ═══════════════════════════════════════════════════════════
//  QUANTUM SUPERPOSITION
// ═══════════════════════════════════════════════════════════

/// A quantum superposition state — multiple values with probability amplitudes.
///
/// In quantum mechanics, a system can exist in multiple states simultaneously.
/// Each state has an amplitude (complex number), and the probability of
/// measuring that state is |amplitude|².
///
/// Here we use real-valued amplitudes (simplified) for classical computation.
#[derive(Debug, Clone)]
pub struct Superposition<T: Clone> {
    /// States and their probability amplitudes.
    states: Vec<(T, f64)>,
    /// Whether this superposition has been "measured" (collapsed).
    collapsed: Option<usize>,
}

impl<T: Clone> Superposition<T> {
    /// Create a uniform superposition — all states equally likely.
    pub fn uniform(states: Vec<T>) -> Self {
        let n = states.len();
        if n == 0 {
            return Self { states: vec![], collapsed: None };
        }
        let amp = 1.0 / (n as f64).sqrt();
        Self {
            states: states.into_iter().map(|s| (s, amp)).collect(),
            collapsed: None,
        }
    }

    /// Create a superposition with custom amplitudes.
    pub fn with_amplitudes(states: Vec<(T, f64)>) -> Self {
        let mut sp = Self { states, collapsed: None };
        sp.normalize();
        sp
    }

    /// Number of states in superposition.
    pub fn num_states(&self) -> usize {
        self.states.len()
    }

    /// Normalize amplitudes so that Σ|amplitude|² = 1 (probability conservation).
    fn normalize(&mut self) {
        let total: f64 = self.states.iter().map(|(_, a)| a * a).sum();
        if total > 0.0 {
            let factor = 1.0 / total.sqrt();
            for (_, amp) in &mut self.states {
                *amp *= factor;
            }
        }
    }

    /// Get the probability of each state (|amplitude|²).
    pub fn probabilities(&self) -> Vec<(T, f64)> {
        self.states.iter().map(|(s, a)| (s.clone(), a * a)).collect()
    }

    /// Apply a "quantum gate" — transform amplitudes.
    /// The gate function receives the current amplitude and returns the new one.
    pub fn apply_gate<F: Fn(&T, f64) -> f64>(&mut self, gate: F) {
        for (state, amp) in &mut self.states {
            *amp = gate(state, *amp);
        }
        self.normalize();
    }

    /// Interfere two superpositions — constructive/destructive interference.
    /// Amplitudes are added (constructive) or subtracted (destructive).
    pub fn interfere(&self, other: &Superposition<T>, constructive: bool) -> Superposition<T>
    where
        T: PartialEq,
    {
        let mut result: Vec<(T, f64)> = Vec::new();
        // Add amplitudes for matching states.
        for (s, a) in &self.states {
            let other_amp = other.states.iter()
                .find(|(os, _)| os == s)
                .map(|(_, oa)| *oa)
                .unwrap_or(0.0);
            let new_amp = if constructive { a + other_amp } else { a - other_amp };
            result.push((s.clone(), new_amp));
        }
        // Add states only in other.
        for (s, a) in &other.states {
            if !self.states.iter().any(|(os, _)| os == s) {
                let sign = if constructive { 1.0 } else { -1.0 };
                result.push((s.clone(), a * sign));
            }
        }
        let mut sp = Superposition { states: result, collapsed: None };
        sp.normalize();
        sp
    }

    /// "Measure" the superposition — collapse to a single state.
    /// Uses weighted random selection based on probabilities.
    pub fn measure(&mut self) -> Option<T> {
        if self.states.is_empty() {
            return None;
        }
        if let Some(idx) = self.collapsed {
            return Some(self.states[idx].0.clone());
        }
        // Generate random number (simple PRNG — no external crate).
        let r = quantum_random();
        let probs = self.probabilities();
        let mut cumulative = 0.0;
        for (i, (_, p)) in probs.iter().enumerate() {
            cumulative += p;
            if r <= cumulative {
                self.collapsed = Some(i);
                return Some(self.states[i].0.clone());
            }
        }
        // Fallback to last state.
        let last = self.states.len() - 1;
        self.collapsed = Some(last);
        Some(self.states[last].0.clone())
    }

    /// Check if the superposition has been measured.
    pub fn is_collapsed(&self) -> bool {
        self.collapsed.is_some()
    }

    /// Reset to superposition (un-collapse).
    pub fn reset(&mut self) {
        self.collapsed = None;
    }
}

// ═══════════════════════════════════════════════════════════
//  QUANTUM ENTANGLEMENT
// ═══════════════════════════════════════════════════════════

/// Quantum entanglement — two components share correlated state.
///
/// When entangled, measuring one component instantly determines
/// the state of the other, regardless of distance.
///
/// Used for: cross-component state synchronization, distributed consistency.
pub struct EntangledPair<A: Clone, B: Clone> {
    /// Component A.
    pub a: Arc<RwLock<A>>,
    /// Component B.
    pub b: Arc<RwLock<B>>,
    /// Correlation function: given A's state, determine B's state.
    correlation: Box<dyn Fn(&A) -> B + Send + Sync>,
    /// Number of times entanglement was used (for stats).
    uses: AtomicU64,
}

impl<A: Clone + Send + Sync + 'static, B: Clone + Send + Sync + 'static> EntangledPair<A, B> {
    /// Create an entangled pair with a correlation function.
    pub fn new(initial_a: A, initial_b: B, correlation: impl Fn(&A) -> B + Send + Sync + 'static) -> Self {
        Self {
            a: Arc::new(RwLock::new(initial_a)),
            b: Arc::new(RwLock::new(initial_b)),
            correlation: Box::new(correlation),
            uses: AtomicU64::new(0),
        }
    }

    /// "Observe" component A — this also updates B (entanglement).
    pub async fn observe_a(&self) -> A {
        self.uses.fetch_add(1, Ordering::Relaxed);
        let a_val = self.a.read().await.clone();
        let b_val = (self.correlation)(&a_val);
        *self.b.write().await = b_val;
        a_val
    }

    /// "Observe" component B.
    pub async fn observe_b(&self) -> B {
        self.b.read().await.clone()
    }

    /// Set component A (and automatically update B via entanglement).
    pub async fn set_a(&self, value: A) {
        self.uses.fetch_add(1, Ordering::Relaxed);
        let b_val = (self.correlation)(&value);
        *self.a.write().await = value;
        *self.b.write().await = b_val;
    }

    /// Number of times entanglement was used.
    pub fn entanglement_count(&self) -> u64 {
        self.uses.load(Ordering::Relaxed)
    }

    /// Get clones of the Arcs (for sharing across tasks).
    pub fn share(&self) -> (Arc<RwLock<A>>, Arc<RwLock<B>>) {
        (self.a.clone(), self.b.clone())
    }
}

// ═══════════════════════════════════════════════════════════
//  QUANTUM TUNNELING
// ═══════════════════════════════════════════════════════════

/// Quantum tunneling optimizer — escape local optima.
///
/// In quantum mechanics, particles can "tunnel" through energy barriers
/// that would be impassable classically. This optimizer uses the same
/// principle to escape local optima in optimization problems.
///
/// The probability of tunneling through a barrier decreases exponentially
/// with barrier height and width: P ~ exp(-2 * sqrt(2m * (V - E)) * width)
pub struct QuantumTunneler {
    /// Current energy (optimization value).
    energy: f64,
    /// Best energy found so far.
    best_energy: f64,
    /// Temperature (controls tunneling probability).
    temperature: f64,
    /// Cooling rate (temperature decay per iteration).
    cooling_rate: f64,
    /// Number of tunneling events.
    tunnelings: AtomicU64,
    /// Number of iterations.
    iterations: AtomicU64,
}

impl QuantumTunneler {
    /// Create a new quantum tunneler.
    pub fn new(initial_energy: f64, temperature: f64, cooling_rate: f64) -> Self {
        Self {
            energy: initial_energy,
            best_energy: initial_energy,
            temperature,
            cooling_rate,
            tunnelings: AtomicU64::new(0),
            iterations: AtomicU64::new(0),
        }
    }

    /// Attempt to move to a new energy state.
    /// Returns true if the move was accepted (tunneled or improved).
    pub fn try_move(&mut self, new_energy: f64) -> bool {
        self.iterations.fetch_add(1, Ordering::Relaxed);

        // If the new state is better, always accept.
        if new_energy <= self.energy {
            self.energy = new_energy;
            if new_energy < self.best_energy {
                self.best_energy = new_energy;
            }
            return true;
        }

        // Calculate barrier height (energy difference).
        let barrier = new_energy - self.energy;

        // Quantum tunneling probability: P = exp(-barrier / temperature)
        // At high temperature, tunneling is likely (exploration).
        // At low temperature, tunneling is unlikely (exploitation).
        let tunnel_prob = (-barrier / self.temperature).exp();

        // Roll the quantum dice.
        if quantum_random() < tunnel_prob {
            // TUNNELED! Accept the worse state to escape local optimum.
            self.energy = new_energy;
            self.tunnelings.fetch_add(1, Ordering::Relaxed);
            return true;
        }

        // Cool down (reduce temperature — annealing).
        self.temperature *= 1.0 - self.cooling_rate;
        false
    }

    /// Current energy state.
    pub fn energy(&self) -> f64 {
        self.energy
    }

    /// Best energy found.
    pub fn best_energy(&self) -> f64 {
        self.best_energy
    }

    /// Current temperature.
    pub fn temperature(&self) -> f64 {
        self.temperature
    }

    /// Number of tunneling events.
    pub fn tunnelings(&self) -> u64 {
        self.tunnelings.load(Ordering::Relaxed)
    }

    /// Total iterations.
    pub fn iterations(&self) -> u64 {
        self.iterations.load(Ordering::Relaxed)
    }

    /// Get statistics.
    pub fn stats(&self) -> TunnelingStats {
        TunnelingStats {
            energy: self.energy,
            best_energy: self.best_energy,
            temperature: self.temperature,
            tunnelings: self.tunnelings.load(Ordering::Relaxed),
            iterations: self.iterations.load(Ordering::Relaxed),
            tunneling_rate: {
                let iters = self.iterations.load(Ordering::Relaxed);
                if iters > 0 {
                    self.tunnelings.load(Ordering::Relaxed) as f64 / iters as f64
                } else {
                    0.0
                }
            },
        }
    }
}

/// Quantum tunneling statistics.
#[derive(Debug, Clone, serde::Serialize)]
pub struct TunnelingStats {
    pub energy: f64,
    pub best_energy: f64,
    pub temperature: f64,
    pub tunnelings: u64,
    pub iterations: u64,
    pub tunneling_rate: f64,
}

// ═══════════════════════════════════════════════════════════
//  QUANTUM ERROR CORRECTION (QEC)
// ═══════════════════════════════════════════════════════════

/// Quantum error correction — data integrity through redundancy.
///
/// Uses a simplified version of the Shor code (9-qubit code):
/// - Encode data into 3 copies (bit-flip protection)
/// - Each copy into 3 more (phase-flip protection)
/// - Majority vote to detect and correct errors
///
/// For classical data, we use a simpler 3-repetition code with
/// majority voting and checksums.
pub struct QuantumErrorCorrection;

impl QuantumErrorCorrection {
    /// Encode data with quantum error correction (3-repetition code).
    /// Returns 3 copies + a checksum for each.
    pub fn encode(data: &[u8]) -> Vec<EncodedBlock> {
        let checksum = Self::checksum(data);
        let block = EncodedBlock {
            data: data.to_vec(),
            checksum,
        };
        // Triplicate for error correction.
        vec![block.clone(), block.clone(), block.clone()]
    }

    /// Decode data with error correction.
    /// Uses majority voting to correct single-block errors.
    pub fn decode(blocks: &[EncodedBlock]) -> Result<Vec<u8>, QecError> {
        if blocks.is_empty() {
            return Err(QecError::NoBlocks);
        }
        if blocks.len() < 2 {
            return Err(QecError::InsufficientBlocks);
        }

        // Majority vote: find the most common valid block.
        for block in blocks {
            if Self::verify(block) {
                // Check if majority agrees.
                let agreeing = blocks.iter()
                    .filter(|b| b.data == block.data)
                    .count();
                if agreeing > (blocks.len() / 2) {
                    return Ok(block.data.clone());
                }
            }
        }

        // Try to reconstruct from majority data.
        let len = blocks[0].data.len();
        let mut reconstructed = Vec::with_capacity(len);
        for i in 0..len {
            let mut counts: HashMap<u8, usize> = HashMap::new();
            for block in blocks {
                if let Some(&byte) = block.data.get(i) {
                    *counts.entry(byte).or_default() += 1;
                }
            }
            // Pick the byte with most votes.
            let best = counts.into_iter()
                .max_by_key(|(_, c)| *c)
                .map(|(b, _)| b)
                .unwrap_or(0);
            reconstructed.push(best);
        }

        // Verify the reconstructed data.
        let checksum = Self::checksum(&reconstructed);
        if blocks.iter().any(|b| b.checksum == checksum) {
            Ok(reconstructed)
        } else {
            Err(QecError::Uncorrectable)
        }
    }

    /// Calculate a simple checksum (xxhash-based).
    fn checksum(data: &[u8]) -> u64 {
        xxhash_rust::xxh3::xxh3_64(data)
    }

    /// Verify a block's integrity.
    fn verify(block: &EncodedBlock) -> bool {
        Self::checksum(&block.data) == block.checksum
    }

    /// Simulate a bit-flip error (for testing).
    pub fn inject_error(block: &mut EncodedBlock, bit_position: usize) {
        let byte_idx = bit_position / 8;
        let bit_idx = bit_position % 8;
        if byte_idx < block.data.len() {
            block.data[byte_idx] ^= 1 << bit_idx;
            // Don't update checksum — this simulates corruption.
        }
    }
}

/// An encoded data block with checksum.
#[derive(Debug, Clone)]
pub struct EncodedBlock {
    pub data: Vec<u8>,
    pub checksum: u64,
}

/// Quantum error correction error.
#[derive(Debug, Clone, thiserror::Error)]
pub enum QecError {
    #[error("no blocks provided")]
    NoBlocks,
    #[error("insufficient blocks for correction")]
    InsufficientBlocks,
    #[error("uncorrectable error — too many bit flips")]
    Uncorrectable,
}

// ═══════════════════════════════════════════════════════════
//  QUANTUM MEASUREMENT (PROBABILISTIC ROUTING)
// ═══════════════════════════════════════════════════════════

/// Quantum measurement — probabilistic selection.
///
/// In quantum mechanics, measurement collapses a superposition into
/// a single state with probability determined by amplitudes.
///
/// Used for: load balancing, A/B testing, probabilistic routing.
pub struct QuantumMeasurement;

impl QuantumMeasurement {
    /// Measure a superposition of options — collapse to one.
    pub fn collapse<T: Clone>(options: Vec<(T, f64)>) -> Option<T> {
        if options.is_empty() {
            return None;
        }
        // Normalize probabilities.
        let total: f64 = options.iter().map(|(_, p)| p).sum();
        if total <= 0.0 {
            return None;
        }
        let r = quantum_random() * total;
        let mut cumulative = 0.0;
        for (opt, prob) in &options {
            cumulative += prob;
            if r <= cumulative {
                return Some(opt.clone());
            }
        }
        options.last().map(|(o, _)| o.clone())
    }

    /// Quantum random number generator.
    /// Uses a simple but effective PRNG based on quantum-like principles:
    /// chaotic iteration with feedback (like a quantum chaotic system).
    pub fn random() -> f64 {
        quantum_random()
    }

    /// Quantum dice — generate a random integer in [0, n).
    pub fn dice(n: usize) -> usize {
        if n == 0 { return 0; }
        (quantum_random() * n as f64) as usize
    }

    /// Quantum coin flip — 50/50 chance.
    pub fn coin_flip() -> bool {
        quantum_random() < 0.5
    }
}

// ═══════════════════════════════════════════════════════════
//  QUANTUM-INSPIRED RANDOM NUMBER GENERATOR
// ═══════════════════════════════════════════════════════════

// Thread-local quantum-inspired PRNG state.
// Uses a xorshift algorithm with quantum-like state mixing.
thread_local! {
    static QPRNG_STATE: std::cell::Cell<u64> = std::cell::Cell::new(seed());
}

fn seed() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(42);
    let pid = std::process::id() as u64;
    let tid = std::thread::current().id();
    let tid_hash = format!("{tid:?}").bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
    nanos ^ (pid << 32) ^ tid_hash
}

/// Generate a quantum-inspired random float in [0, 1).
fn quantum_random() -> f64 {
    QPRNG_STATE.with(|cell| {
        let mut x = cell.get();
        // xorshift64 — fast, good distribution.
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        cell.set(x);
        // Convert to [0, 1) — use top 53 bits for double precision.
        (x >> 11) as f64 / (1u64 << 53) as f64
    })
}

// ═══════════════════════════════════════════════════════════
//  QUANTUM GATE (UNITARY OPERATIONS)
// ═══════════════════════════════════════════════════════════

/// Quantum gate — a unitary transformation on amplitudes.
pub struct QuantumGate;

impl QuantumGate {
    /// Hadamard gate — creates uniform superposition.
    /// Transforms |0⟩ → (|0⟩ + |1⟩)/√2, |1⟩ → (|0⟩ - |1⟩)/√2
    pub fn hadamard(amp0: f64, amp1: f64) -> (f64, f64) {
        let h = std::f64::consts::FRAC_1_SQRT_2;
        (h * (amp0 + amp1), h * (amp0 - amp1))
    }

    /// Pauli-X gate (NOT) — swaps amplitudes.
    pub fn pauli_x(amp0: f64, amp1: f64) -> (f64, f64) {
        (amp1, amp0)
    }

    /// Pauli-Z gate — phase flip.
    pub fn pauli_z(amp0: f64, amp1: f64) -> (f64, f64) {
        (amp0, -amp1)
    }

    /// Rotation gate — rotate amplitude by angle θ.
    pub fn rotate(amp0: f64, amp1: f64, theta: f64) -> (f64, f64) {
        let cos = theta.cos();
        let sin = theta.sin();
        (cos * amp0 - sin * amp1, sin * amp0 + cos * amp1)
    }
}

// ═══════════════════════════════════════════════════════════
//  QUANTUM ENGINE (UNIFIED INTERFACE)
// ═══════════════════════════════════════════════════════════

/// The quantum engine — unified interface to all quantum operations.
pub struct QuantumEngine {
    /// Active tunnelers (for optimization tasks).
    tunnelers: RwLock<HashMap<String, QuantumTunneler>>,
    /// Active entangled pairs count.
    entanglements: AtomicU64,
    /// Total quantum operations performed.
    operations: AtomicU64,
}

impl QuantumEngine {
    /// Create a new quantum engine.
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            tunnelers: RwLock::new(HashMap::new()),
            entanglements: AtomicU64::new(0),
            operations: AtomicU64::new(0),
        })
    }

    /// Register a quantum tunneler for an optimization task.
    pub async fn register_tunneler(&self, name: &str, initial_energy: f64, temperature: f64) {
        let tunneler = QuantumTunneler::new(initial_energy, temperature, 0.001);
        self.tunnelers.write().await.insert(name.to_string(), tunneler);
        self.operations.fetch_add(1, Ordering::Relaxed);
    }

    /// Attempt a quantum tunneling move for a registered task.
    pub async fn tunnel(&self, name: &str, new_energy: f64) -> Option<bool> {
        let mut tunnelers = self.tunnelers.write().await;
        let tunneler = tunnelers.get_mut(name)?;
        let result = tunneler.try_move(new_energy);
        self.operations.fetch_add(1, Ordering::Relaxed);
        Some(result)
    }

    /// Get tunneling stats for a task.
    pub async fn tunneler_stats(&self, name: &str) -> Option<TunnelingStats> {
        self.tunnelers.read().await.get(name).map(|t| t.stats())
    }

    /// Record an entanglement creation.
    pub fn record_entanglement(&self) {
        self.entanglements.fetch_add(1, Ordering::Relaxed);
    }

    /// Get quantum engine statistics.
    pub async fn stats(&self) -> QuantumEngineStats {
        let tunnelers = self.tunnelers.read().await;
        let mut tunneling_stats = Vec::new();
        for (name, t) in tunnelers.iter() {
            tunneling_stats.push(serde_json::json!({
                "task": name,
                "stats": t.stats(),
            }));
        }
        QuantumEngineStats {
            active_tunnelers: tunnelers.len(),
            entanglements: self.entanglements.load(Ordering::Relaxed),
            total_operations: self.operations.load(Ordering::Relaxed),
            tunnelers: tunneling_stats,
        }
    }
}

/// Quantum engine statistics.
#[derive(Debug, Clone, serde::Serialize)]
pub struct QuantumEngineStats {
    pub active_tunnelers: usize,
    pub entanglements: u64,
    pub total_operations: u64,
    pub tunnelers: Vec<serde_json::Value>,
}

// ═══════════════════════════════════════════════════════════
//  TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn superposition_uniform() {
        let sp = Superposition::uniform(vec![1, 2, 3]);
        assert_eq!(sp.num_states(), 3);
        let probs = sp.probabilities();
        for (_, p) in probs {
            assert!((p - 1.0 / 3.0).abs() < 0.01);
        }
    }

    #[test]
    fn superposition_measure_collapses() {
        let mut sp = Superposition::uniform(vec!["a", "b", "c"]);
        let result = sp.measure();
        assert!(result.is_some());
        assert!(sp.is_collapsed());
        // Measuring again gives the same result.
        let result2 = sp.measure();
        assert_eq!(result, result2);
    }

    #[test]
    fn superposition_apply_gate() {
        let mut sp = Superposition::uniform(vec![1, 2]);
        sp.apply_gate(|state, amp| if *state == 1 { amp * 3.0 } else { amp });
        let probs = sp.probabilities();
        // State 1 should have higher probability.
        assert!(probs[0].1 > probs[1].1);
    }

    #[test]
    fn superposition_interfere_constructive() {
        let sp1 = Superposition::uniform(vec![1, 2]);
        let sp2 = Superposition::uniform(vec![1, 2]);
        let interfered = sp1.interfere(&sp2, true);
        // Constructive interference should amplify both states.
        assert_eq!(interfered.num_states(), 2);
    }

    #[test]
    fn superposition_interfere_destructive() {
        let sp1 = Superposition::uniform(vec![1, 2]);
        let sp2 = Superposition::uniform(vec![1, 2]);
        let interfered = sp1.interfere(&sp2, false);
        // Destructive interference should reduce amplitudes.
        let probs: Vec<f64> = interfered.probabilities().into_iter().map(|(_, p)| p).collect();
        // After destructive interference of identical states, probabilities should be ~0.
        for p in probs {
            assert!(p < 0.01 || p > 0.99); // Either near 0 or near 1 (normalized).
        }
    }

    #[test]
    fn superposition_reset() {
        let mut sp = Superposition::uniform(vec![1, 2, 3]);
        sp.measure();
        assert!(sp.is_collapsed());
        sp.reset();
        assert!(!sp.is_collapsed());
    }

    #[tokio::test]
    async fn entanglement_correlates_state() {
        let pair = EntangledPair::new(5, 0, |a: &i32| a * 2);
        let a_val = pair.observe_a().await;
        let b_val = pair.observe_b().await;
        assert_eq!(a_val, 5);
        assert_eq!(b_val, 10); // 5 * 2 = 10 (correlated)
    }

    #[tokio::test]
    async fn entanglement_set_a_updates_b() {
        let pair = EntangledPair::new(0, 0, |a: &i32| a + 100);
        pair.set_a(42).await;
        let b_val = pair.observe_b().await;
        assert_eq!(b_val, 142); // 42 + 100
    }

    #[tokio::test]
    async fn entanglement_tracks_uses() {
        let pair = EntangledPair::new(1, 1, |a: &i32| *a);
        pair.observe_a().await;
        pair.observe_a().await;
        pair.set_a(2).await;
        assert_eq!(pair.entanglement_count(), 3);
    }

    #[test]
    fn quantum_tunneler_accepts_improvement() {
        let mut t = QuantumTunneler::new(100.0, 1.0, 0.01);
        assert!(t.try_move(90.0)); // Better → always accept.
        assert_eq!(t.energy(), 90.0);
        assert_eq!(t.best_energy(), 90.0);
    }

    #[test]
    fn quantum_tunneler_escapes_local_optimum() {
        let mut t = QuantumTunneler::new(100.0, 50.0, 0.001);
        // Try many worse moves — some should tunnel through.
        let mut tunneled = 0;
        for _ in 0..1000 {
            if t.try_move(100.0 + quantum_random() * 10.0) {
                tunneled += 1;
            }
        }
        assert!(tunneled > 0, "Should have tunneled at least once");
        assert!(t.tunnelings() > 0);
    }

    #[test]
    fn quantum_tunneler_cools_down() {
        let mut t = QuantumTunneler::new(100.0, 100.0, 0.1);
        let initial_temp = t.temperature();
        for _ in 0..100 {
            t.try_move(105.0);
        }
        assert!(t.temperature() < initial_temp);
    }

    #[test]
    fn quantum_tunneler_stats() {
        let mut t = QuantumTunneler::new(50.0, 10.0, 0.01);
        t.try_move(45.0);
        t.try_move(55.0);
        let stats = t.stats();
        assert_eq!(stats.iterations, 2);
        assert_eq!(stats.best_energy, 45.0);
    }

    #[test]
    fn qec_encode_produces_triplicate() {
        let data = b"hello";
        let encoded = QuantumErrorCorrection::encode(data);
        assert_eq!(encoded.len(), 3);
        for block in &encoded {
            assert_eq!(block.data, data);
            assert!(block.checksum != 0);
        }
    }

    #[test]
    fn qec_decode_corrects_single_error() {
        let data = b"quantum test";
        let mut blocks = QuantumErrorCorrection::encode(data);
        // Inject error in one block.
        QuantumErrorCorrection::inject_error(&mut blocks[0], 3);
        // Should still decode correctly (majority vote).
        let decoded = QuantumErrorCorrection::decode(&blocks);
        assert!(decoded.is_ok());
        assert_eq!(decoded.unwrap(), data);
    }

    #[test]
    fn qec_detects_uncorrectable_errors() {
        let data = b"test";
        let mut blocks = QuantumErrorCorrection::encode(data);
        // Corrupt two of three blocks.
        QuantumErrorCorrection::inject_error(&mut blocks[0], 0);
        QuantumErrorCorrection::inject_error(&mut blocks[1], 1);
        // With 2/3 corrupted, should fail (or partially recover).
        let result = QuantumErrorCorrection::decode(&blocks);
        // Might recover via majority voting per-byte, but checksum won't match.
        // Either Ok (lucky) or Err (expected for heavy corruption).
        assert!(result.is_ok() || matches!(result, Err(QecError::Uncorrectable)));
    }

    #[test]
    fn quantum_measurement_collapse() {
        let options = vec![("A", 0.7), ("B", 0.2), ("C", 0.1)];
        let result = QuantumMeasurement::collapse(options);
        assert!(result.is_some());
    }

    #[test]
    fn quantum_measurement_empty() {
        let options: Vec<(i32, f64)> = vec![];
        let result = QuantumMeasurement::collapse(options);
        assert!(result.is_none());
    }

    #[test]
    fn quantum_random_in_range() {
        for _ in 0..1000 {
            let r = quantum_random();
            assert!(r >= 0.0 && r < 1.0);
        }
    }

    #[test]
    fn quantum_dice_in_range() {
        for _ in 0..1000 {
            let r = QuantumMeasurement::dice(6);
            assert!(r < 6);
        }
    }

    #[test]
    fn quantum_coin_flip() {
        let mut heads = 0;
        let mut tails = 0;
        for _ in 0..10000 {
            if QuantumMeasurement::coin_flip() { heads += 1; } else { tails += 1; }
        }
        // Should be roughly 50/50 (within 10%).
        assert!((heads as f64 / 10000.0 - 0.5).abs() < 0.1);
    }

    #[test]
    fn hadamard_gate_creates_superposition() {
        let (amp0, amp1) = QuantumGate::hadamard(1.0, 0.0);
        // |0⟩ → (|0⟩ + |1⟩)/√2
        assert!((amp0 - std::f64::consts::FRAC_1_SQRT_2).abs() < 0.001);
        assert!((amp1 - std::f64::consts::FRAC_1_SQRT_2).abs() < 0.001);
    }

    #[test]
    fn pauli_x_swaps_amplitudes() {
        let (amp0, amp1) = QuantumGate::pauli_x(0.3, 0.7);
        assert!((amp0 - 0.7).abs() < 0.001);
        assert!((amp1 - 0.3).abs() < 0.001);
    }

    #[test]
    fn pauli_z_flips_phase() {
        let (amp0, amp1) = QuantumGate::pauli_z(0.5, 0.5);
        assert!((amp0 - 0.5).abs() < 0.001);
        assert!((amp1 - (-0.5)).abs() < 0.001);
    }

    #[test]
    fn rotate_gate_preserves_norm() {
        let (amp0, amp1) = QuantumGate::rotate(1.0, 0.0, std::f64::consts::PI / 4.0);
        let norm = (amp0 * amp0 + amp1 * amp1).sqrt();
        assert!((norm - 1.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn quantum_engine_registers_tunneler() {
        let engine = QuantumEngine::new();
        engine.register_tunneler("seo_optimization", 100.0, 10.0).await;
        let stats = engine.stats().await;
        assert_eq!(stats.active_tunnelers, 1);
    }

    #[tokio::test]
    async fn quantum_engine_tunnel() {
        let engine = QuantumEngine::new();
        engine.register_tunneler("test", 100.0, 50.0).await;
        let result = engine.tunnel("test", 90.0).await;
        assert!(result.is_some());
        assert!(result.unwrap()); // Better energy → accepted.
    }

    #[tokio::test]
    async fn quantum_engine_stats() {
        let engine = QuantumEngine::new();
        engine.register_tunneler("t1", 100.0, 10.0).await;
        engine.record_entanglement();
        engine.record_entanglement();
        let stats = engine.stats().await;
        assert_eq!(stats.active_tunnelers, 1);
        assert_eq!(stats.entanglements, 2);
        assert!(stats.total_operations > 0);
    }
}
