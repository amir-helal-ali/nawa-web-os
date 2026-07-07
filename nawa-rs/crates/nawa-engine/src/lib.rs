//! # NAWA Unified Engine
//!
//! محرك واحد موحد يدمج: قاعدة البيانات + تصيير الواجهة + الإرسال
//! في pipeline واحد بـ **صفر نسخ**.
//!
//! ## Zero-Copy Chain
//!
//! ```text
//! mmap'd DB page (kernel memory)
//!     ↓ &u8 reference (0 copies)
//! ZeroCopyHtml writer → buffer
//!     ↓ buffer pointer (0 copies)
//! io_uring send → socket (0 copies)
//! ```
//!
//! ## Unified Architecture
//!
//! لا يوجد "محرك خلفية" و"محرك واجهة" منفصلان.
//! هناك محرك واحد يفعل كل شيء:
//!
//! - يقرأ من DB (عبر mmap — لا نسخ)
//! - يُصيّر HTML (مباشرة لـ buffer — لا String alloc)
//! - يرسل للـ socket (عبر io_uring — لا نسخ)
//! - يدير الـ UI components (مدمجة في المحرك)

pub mod components;
pub mod design;
pub mod pipeline;
pub mod zerocopy_html;

pub use components::*;
pub use design::{DesignSystem, Theme};
pub use pipeline::{UnifiedEngine, EngineContext, RenderResult};
pub use zerocopy_html::ZeroCopyHtml;

/// Engine error type.
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("render error: {0}")]
    Render(String),
    #[error("db error: {0}")]
    Db(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, EngineError>;
