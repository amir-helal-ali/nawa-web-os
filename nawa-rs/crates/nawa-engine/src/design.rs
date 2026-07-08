//! NAWA Design System — modern, professional, built-in UI framework.
//!
//! A complete design system integrated into the engine kernel.
//! No external CSS frameworks needed. Includes:
//! - Color tokens (dark amber theme, RTL-first)
//! - Typography scale
//! - Spacing system
//! - Component styles (buttons, cards, tables, forms, nav, etc.)
//! - Animations and transitions
//! - Responsive breakpoints

/// The design system — generates CSS for the entire UI.
pub struct DesignSystem {
    theme: Theme,
}

/// UI theme.
#[derive(Debug, Clone, Copy)]
pub enum Theme {
    /// NAWA dark amber (default).
    Dark,
    /// Light mode.
    Light,
}

impl DesignSystem {
    /// Create with default dark theme.
    pub fn new() -> Self {
        Self { theme: Theme::Dark }
    }

    /// Set theme.
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Generate the complete CSS as a string.
    pub fn css(&self) -> &'static str {
        match self.theme {
            Theme::Dark => DARK_CSS,
            Theme::Light => LIGHT_CSS,
        }
    }

    /// Generate minimal CSS (for tiny pages).
    pub fn minimal_css(&self) -> &'static str {
        MINIMAL_CSS
    }
}

impl Default for DesignSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Dark amber theme — the NAWA signature look.
const DARK_CSS: &str = r#"
/* ═══ NAWA Design System v1.0 — Dark Amber ═══ */
:root {
  --nawa-bg: #0d0c0a;
  --nawa-surface: #1a1a1a;
  --nawa-surface-hover: #222;
  --nawa-border: #2a2a2a;
  --nawa-text: #e0e0e0;
  --nawa-text-muted: #888;
  --nawa-primary: #f59e0b;
  --nawa-primary-hover: #d97706;
  --nawa-accent: #10b981;
  --nawa-danger: #dc2626;
  --nawa-radius: 12px;
  --nawa-radius-sm: 8px;
  --nawa-transition: 0.2s ease;
}
* { margin:0; padding:0; box-sizing:border-box; }
body {
  font-family: 'Noto Sans Arabic', system-ui, -apple-system, sans-serif;
  background: var(--nawa-bg);
  color: var(--nawa-text);
  line-height: 1.8;
  min-height: 100vh;
  -webkit-font-smoothing: antialiased;
}
/* ─── Layout ─── */
.nawa-container { max-width: 900px; margin: 0 auto; padding: 2rem; }
.nawa-nav {
  display: flex; justify-content: space-between; align-items: center;
  padding: 1rem 2rem; background: var(--nawa-surface);
  border-bottom: 1px solid var(--nawa-border);
  position: sticky; top: 0; z-index: 100; backdrop-filter: blur(12px);
}
.nawa-nav-brand { color: var(--nawa-primary); font-weight: 700; font-size: 1.25rem; text-decoration: none; }
.nawa-nav-links { display: flex; gap: 1rem; }
.nawa-nav-links a { color: var(--nawa-primary); text-decoration: none; transition: var(--nawa-transition); }
.nawa-nav-links a:hover { opacity: 0.7; }
/* ─── Cards ─── */
.nawa-card {
  background: var(--nawa-surface); border: 1px solid var(--nawa-border);
  border-radius: var(--nawa-radius); padding: 1.5rem; margin: 1rem 0;
  transition: border-color var(--nawa-transition);
}
.nawa-card:hover { border-color: var(--nawa-primary); }
.nawa-card h2 { color: var(--nawa-primary); margin-bottom: 0.75rem; }
/* ─── Buttons ─── */
.nawa-btn {
  display: inline-flex; align-items: center; gap: 0.5rem;
  padding: 0.75rem 1.5rem; border: none; border-radius: var(--nawa-radius-sm);
  cursor: pointer; font-size: 1rem; font-family: inherit;
  transition: all var(--nawa-transition); text-decoration: none;
}
.nawa-btn-primary { background: var(--nawa-primary); color: var(--nawa-bg); font-weight: 700; }
.nawa-btn-primary:hover { background: var(--nawa-primary-hover); transform: translateY(-1px); }
.nawa-btn-secondary { background: var(--nawa-surface-hover); color: var(--nawa-text); border: 1px solid var(--nawa-border); }
.nawa-btn-secondary:hover { background: var(--nawa-border); }
.nawa-btn-danger { background: var(--nawa-danger); color: #fff; }
.nawa-btn-sm { padding: 0.4rem 0.9rem; font-size: 0.85rem; }
/* ─── Forms ─── */
.nawa-input {
  width: 100%; padding: 0.8rem; margin: 0.5rem 0;
  background: var(--nawa-bg); border: 1px solid var(--nawa-border);
  border-radius: var(--nawa-radius-sm); color: var(--nawa-text);
  font-size: 1rem; font-family: inherit; transition: var(--nawa-transition);
}
.nawa-input:focus { border-color: var(--nawa-primary); outline: none; box-shadow: 0 0 0 3px rgba(245,158,11,0.15); }
.nawa-label { display: block; margin: 0.8rem 0 0.3rem; color: var(--nawa-text-muted); font-size: 0.9rem; }
.nawa-label input[type="checkbox"] { display: inline-block; width: auto; margin-left: 0.5rem; }
/* ─── Tables ─── */
.nawa-table { width: 100%; border-collapse: collapse; margin: 1rem 0; }
.nawa-table th, .nawa-table td { padding: 0.7rem; text-align: right; border-bottom: 1px solid var(--nawa-border); }
.nawa-table th { color: var(--nawa-primary); font-size: 0.85rem; text-transform: uppercase; }
.nawa-table tr:hover { background: var(--nawa-surface-hover); }
/* ─── Badges ─── */
.nawa-badge { display: inline-block; padding: 0.2rem 0.6rem; border-radius: 4px; font-size: 0.75rem; }
.nawa-badge-ok { background: rgba(16,185,129,0.15); color: var(--nawa-accent); }
.nawa-badge-warn { background: rgba(245,158,11,0.15); color: var(--nawa-primary); }
.nawa-badge-danger { background: rgba(220,38,38,0.15); color: var(--nawa-danger); }
.nawa-badge-info { background: rgba(59,130,246,0.15); color: #3b82f6; }
/* ─── Stats ─── */
.nawa-stats { display: flex; gap: 2rem; margin: 1rem 0; }
.nawa-stat { text-align: center; }
.nawa-stat-val { color: var(--nawa-primary); font-size: 1.5rem; font-weight: 700; }
.nawa-stat-label { color: var(--nawa-text-muted); font-size: 0.8rem; }
/* ─── Islands ─── */
[data-island] { margin: 1rem 0; padding: 1.5rem; background: var(--nawa-surface); border-radius: var(--nawa-radius); border: 1px solid var(--nawa-border); }
[data-island]:hover { border-color: var(--nawa-primary); }
[data-suspense] { padding: 1rem; background: var(--nawa-surface); border-radius: var(--nawa-radius-sm); border: 1px dashed var(--nawa-border); color: var(--nawa-text-muted); text-align: center; }
/* ─── Animations ─── */
@keyframes nawa-fade-in { from { opacity: 0; transform: translateY(10px); } to { opacity: 1; transform: translateY(0); } }
.nawa-fade-in { animation: nawa-fade-in 0.4s ease; }
@keyframes nawa-pulse { 0%,100% { opacity: 1; } 50% { opacity: 0.5; } }
.nawa-pulse { animation: nawa-pulse 2s ease-in-out infinite; }
/* ─── Responsive ─── */
@media (max-width: 768px) {
  .nawa-nav { flex-direction: column; gap: 0.5rem; }
  .nawa-container { padding: 1rem; }
  .nawa-stats { flex-direction: column; gap: 1rem; }
}
"#;

/// Light theme.
const LIGHT_CSS: &str = r#"
:root {
  --nawa-bg: #fafafa; --nawa-surface: #fff; --nawa-surface-hover: #f0f0f0;
  --nawa-border: #e0e0e0; --nawa-text: #1a1a1a; --nawa-text-muted: #666;
  --nawa-primary: #d97706; --nawa-primary-hover: #b45309;
  --nawa-accent: #059669; --nawa-danger: #dc2626;
  --nawa-radius: 12px; --nawa-radius-sm: 8px; --nawa-transition: 0.2s ease;
}
* { margin:0; padding:0; box-sizing:border-box; }
body { font-family: system-ui, sans-serif; background: var(--nawa-bg); color: var(--nawa-text); line-height: 1.8; }
.nawa-card { background: var(--nawa-surface); border: 1px solid var(--nawa-border); border-radius: var(--nawa-radius); padding: 1.5rem; margin: 1rem 0; }
.nawa-btn-primary { background: var(--nawa-primary); color: #fff; padding: 0.75rem 1.5rem; border: none; border-radius: var(--nawa-radius-sm); cursor: pointer; }
.nawa-input { width: 100%; padding: 0.8rem; border: 1px solid var(--nawa-border); border-radius: var(--nawa-radius-sm); }
"#;

/// Minimal CSS for tiny pages.
const MINIMAL_CSS: &str = r#"
*{margin:0;padding:0;box-sizing:border-box}
body{font-family:system-ui,sans-serif;background:#0d0c0a;color:#e0e0e0;line-height:1.8}
a{color:#f59e0b}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dark_css_present() {
        let ds = DesignSystem::new();
        let css = ds.css();
        assert!(css.contains("--nawa-primary"));
        assert!(css.contains("nawa-card"));
        assert!(css.contains("nawa-btn"));
        assert!(css.contains("nawa-table"));
        assert!(css.contains("nawafade-in") || css.contains("nawa-fade-in"));
    }

    #[test]
    fn light_css_present() {
        let ds = DesignSystem::new().theme(Theme::Light);
        let css = ds.css();
        assert!(css.contains("--nawa-bg: #fafafa"));
    }

    #[test]
    fn minimal_css() {
        let ds = DesignSystem::new();
        let css = ds.minimal_css();
        assert!(css.len() < 500);
    }

    #[test]
    fn responsive_present() {
        let ds = DesignSystem::new();
        assert!(ds.css().contains("@media"));
        assert!(ds.css().contains("768px"));
    }
}
