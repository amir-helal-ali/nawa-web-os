//! Internationalization (i18n) — multi-language support.
//!
//! Provides:
//! - Language detection (header, query, cookie)
//! - Translation strings (key → value per language)
//! - RTL/LTR direction support
//! - Built-in Arabic + English translations

#![allow(dead_code)]

use std::collections::HashMap;

/// Supported languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Ar,
    En,
}

impl Language {
    /// Get the language code.
    pub fn code(&self) -> &'static str {
        match self {
            Language::Ar => "ar",
            Language::En => "en",
        }
    }

    /// Get the display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Language::Ar => "العربية",
            Language::En => "English",
        }
    }

    /// Get text direction.
    pub fn direction(&self) -> &'static str {
        match self {
            Language::Ar => "rtl",
            Language::En => "ltr",
        }
    }

    /// Parse from a string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "ar" | "arabic" | "ar-sa" | "ar-eg" => Some(Language::Ar),
            "en" | "english" | "en-us" | "en-gb" => Some(Language::En),
            _ => None,
        }
    }

    /// List all supported languages.
    pub fn all() -> Vec<Language> {
        vec![Language::Ar, Language::En]
    }
}

/// The i18n translator.
pub struct I18n {
    /// Translation keys per language.
    translations: HashMap<Language, HashMap<String, String>>,
    /// Default language (fallback).
    default_language: Language,
}

impl I18n {
    /// Create a new i18n instance with built-in translations.
    pub fn new() -> Self {
        let mut translations = HashMap::new();
        translations.insert(Language::Ar, Self::arabic_translations());
        translations.insert(Language::En, Self::english_translations());
        Self {
            translations,
            default_language: Language::Ar,
        }
    }

    /// Translate a key for a specific language.
    pub fn t(&self, key: &str, lang: Language) -> String {
        if let Some(lang_map) = self.translations.get(&lang) {
            if let Some(val) = lang_map.get(key) {
                return val.clone();
            }
        }
        if let Some(default_map) = self.translations.get(&self.default_language) {
            if let Some(val) = default_map.get(key) {
                return val.clone();
            }
        }
        key.to_string()
    }

    /// Detect language from request headers.
    pub fn detect(accept_language: Option<&str>, cookie_lang: Option<&str>, query_lang: Option<&str>) -> Language {
        // Priority: query param > cookie > Accept-Language header > default.
        if let Some(q) = query_lang {
            if let Some(lang) = Language::from_str(q) {
                return lang;
            }
        }
        if let Some(c) = cookie_lang {
            if let Some(lang) = Language::from_str(c) {
                return lang;
            }
        }
        if let Some(al) = accept_language {
            // Parse Accept-Language header (e.g., "ar,en-US;q=0.9")
            for part in al.split(',') {
                let lang_code = part.split(';').next().unwrap_or("").trim();
                if let Some(lang) = Language::from_str(lang_code) {
                    return lang;
                }
            }
        }
        Language::Ar // Default
    }

    /// Get all translations for a language.
    pub fn all_translations(&self, lang: Language) -> &HashMap<String, String> {
        self.translations.get(&lang).unwrap_or_else(|| {
            self.translations.get(&self.default_language).unwrap()
        })
    }

    /// Get i18n info as JSON.
    pub fn info(&self, lang: Language) -> serde_json::Value {
        serde_json::json!({
            "current_language": lang.code(),
            "display_name": lang.display_name(),
            "direction": lang.direction(),
            "supported_languages": Language::all().iter().map(|l| {
                serde_json::json!({
                    "code": l.code(),
                    "name": l.display_name(),
                    "direction": l.direction()
                })
            }).collect::<Vec<_>>(),
            "translation_count": self.all_translations(lang).len()
        })
    }

    fn arabic_translations() -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert("app.name".into(), "NAWA".into());
        m.insert("app.tagline".into(), "نظام تشغيل الويب الثوري".into());
        m.insert("nav.dashboard".into(), "الرئيسية".into());
        m.insert("nav.register".into(), "تسجيل".into());
        m.insert("nav.login".into(), "دخول".into());
        m.insert("nav.logout".into(), "خروج".into());
        m.insert("nav.settings".into(), "الإعدادات".into());
        m.insert("nav.system".into(), "النظام".into());
        m.insert("nav.docs".into(), "التوثيق".into());
        m.insert("nav.api".into(), "API".into());
        m.insert("auth.username".into(), "اسم المستخدم".into());
        m.insert("auth.email".into(), "البريد الإلكتروني".into());
        m.insert("auth.password".into(), "كلمة المرور".into());
        m.insert("auth.register_button".into(), "إنشاء حساب".into());
        m.insert("auth.login_button".into(), "دخول".into());
        m.insert("auth.first_user_admin".into(), "أول مستخدم يصبح أدمن تلقائياً".into());
        m.insert("dashboard.welcome".into(), "مرحباً بك في NAWA".into());
        m.insert("dashboard.users".into(), "المستخدمون".into());
        m.insert("dashboard.db_keys".into(), "مفاتيح قاعدة البيانات".into());
        m.insert("dashboard.ws_connections".into(), "اتصالات WebSocket".into());
        m.insert("dashboard.notifications".into(), "الإشعارات".into());
        m.insert("dashboard.endpoints".into(), "المسارات".into());
        m.insert("system.io_uring".into(), "io_uring نشط".into());
        m.insert("system.wasm_plugins".into(), "إضافات WASM".into());
        m.insert("system.quantum_engine".into(), "المحرك الكمي".into());
        m.insert("system.aion_seo".into(), "AION SEO".into());
        m.insert("button.save".into(), "حفظ".into());
        m.insert("button.cancel".into(), "إلغاء".into());
        m.insert("button.delete".into(), "حذف".into());
        m.insert("button.edit".into(), "تعديل".into());
        m.insert("button.back".into(), "رجوع".into());
        m.insert("error.not_found".into(), "الصفحة غير موجودة".into());
        m.insert("error.internal".into(), "خطأ داخلي في الخادم".into());
        m.insert("error.unauthorized".into(), "غير مصرح".into());
        m.insert("error.forbidden".into(), "ممنوع — صلاحيات الأدمن مطلوبة".into());
        m.insert("status.healthy".into(), "سليم".into());
        m.insert("status.unhealthy".into(), "غير سليم".into());
        m.insert("status.active".into(), "نشط".into());
        m.insert("status.inactive".into(), "غير نشط".into());
        m.insert("quantum.superposition".into(), "التراكب الكمي".into());
        m.insert("quantum.entanglement".into(), "التشابك الكمي".into());
        m.insert("quantum.tunneling".into(), "النفق الكمي".into());
        m.insert("quantum.qec".into(), "تصحيح الأخطاء الكمي".into());
        m.insert("quantum.measure".into(), "قياس".into());
        m.insert("quantum.collapse".into(), "انهيار".into());
        m
    }

    fn english_translations() -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert("app.name".into(), "NAWA".into());
        m.insert("app.tagline".into(), "Revolutionary Web Operating System".into());
        m.insert("nav.dashboard".into(), "Dashboard".into());
        m.insert("nav.register".into(), "Register".into());
        m.insert("nav.login".into(), "Login".into());
        m.insert("nav.logout".into(), "Logout".into());
        m.insert("nav.settings".into(), "Settings".into());
        m.insert("nav.system".into(), "System".into());
        m.insert("nav.docs".into(), "Docs".into());
        m.insert("nav.api".into(), "API".into());
        m.insert("auth.username".into(), "Username".into());
        m.insert("auth.email".into(), "Email".into());
        m.insert("auth.password".into(), "Password".into());
        m.insert("auth.register_button".into(), "Create Account".into());
        m.insert("auth.login_button".into(), "Login".into());
        m.insert("auth.first_user_admin".into(), "First user becomes admin automatically".into());
        m.insert("dashboard.welcome".into(), "Welcome to NAWA".into());
        m.insert("dashboard.users".into(), "Users".into());
        m.insert("dashboard.db_keys".into(), "Database Keys".into());
        m.insert("dashboard.ws_connections".into(), "WebSocket Connections".into());
        m.insert("dashboard.notifications".into(), "Notifications".into());
        m.insert("dashboard.endpoints".into(), "Endpoints".into());
        m.insert("system.io_uring".into(), "io_uring active".into());
        m.insert("system.wasm_plugins".into(), "WASM Plugins".into());
        m.insert("system.quantum_engine".into(), "Quantum Engine".into());
        m.insert("system.aion_seo".into(), "AION SEO".into());
        m.insert("button.save".into(), "Save".into());
        m.insert("button.cancel".into(), "Cancel".into());
        m.insert("button.delete".into(), "Delete".into());
        m.insert("button.edit".into(), "Edit".into());
        m.insert("button.back".into(), "Back".into());
        m.insert("error.not_found".into(), "Page not found".into());
        m.insert("error.internal".into(), "Internal server error".into());
        m.insert("error.unauthorized".into(), "Unauthorized".into());
        m.insert("error.forbidden".into(), "Forbidden — admin access required".into());
        m.insert("status.healthy".into(), "Healthy".into());
        m.insert("status.unhealthy".into(), "Unhealthy".into());
        m.insert("status.active".into(), "Active".into());
        m.insert("status.inactive".into(), "Inactive".into());
        m.insert("quantum.superposition".into(), "Quantum Superposition".into());
        m.insert("quantum.entanglement".into(), "Quantum Entanglement".into());
        m.insert("quantum.tunneling".into(), "Quantum Tunneling".into());
        m.insert("quantum.qec".into(), "Quantum Error Correction".into());
        m.insert("quantum.measure".into(), "Measure".into());
        m.insert("quantum.collapse".into(), "Collapse".into());
        m
    }
}

impl Default for I18n {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_codes() {
        assert_eq!(Language::Ar.code(), "ar");
        assert_eq!(Language::En.code(), "en");
    }

    #[test]
    fn language_directions() {
        assert_eq!(Language::Ar.direction(), "rtl");
        assert_eq!(Language::En.direction(), "ltr");
    }

    #[test]
    fn language_from_str() {
        assert_eq!(Language::from_str("ar"), Some(Language::Ar));
        assert_eq!(Language::from_str("en"), Some(Language::En));
        assert_eq!(Language::from_str("AR"), Some(Language::Ar));
        assert_eq!(Language::from_str("fr"), None);
    }

    #[test]
    fn translate_arabic() {
        let i18n = I18n::new();
        assert_eq!(i18n.t("nav.dashboard", Language::Ar), "الرئيسية".to_string());
        assert_eq!(i18n.t("auth.username", Language::Ar), "اسم المستخدم".to_string());
    }

    #[test]
    fn translate_english() {
        let i18n = I18n::new();
        assert_eq!(i18n.t("nav.dashboard", Language::En), "Dashboard".to_string());
        assert_eq!(i18n.t("auth.username", Language::En), "Username".to_string());
    }

    #[test]
    fn translate_fallback_to_key() {
        let i18n = I18n::new();
        assert_eq!(i18n.t("nonexistent.key", Language::Ar), "nonexistent.key".to_string());
    }

    #[test]
    fn detect_from_query() {
        let lang = I18n::detect(Some("en"), Some("ar"), Some("en"));
        assert_eq!(lang, Language::En); // Query takes priority.
    }

    #[test]
    fn detect_from_cookie() {
        let lang = I18n::detect(Some("en"), Some("ar"), None);
        assert_eq!(lang, Language::Ar); // Cookie takes priority over header.
    }

    #[test]
    fn detect_from_header() {
        let lang = I18n::detect(Some("ar,en-US;q=0.9"), None, None);
        assert_eq!(lang, Language::Ar);
    }

    #[test]
    fn detect_default() {
        let lang = I18n::detect(None, None, None);
        assert_eq!(lang, Language::Ar); // Default is Arabic.
    }

    #[test]
    fn all_translations_have_entries() {
        let i18n = I18n::new();
        let ar = i18n.all_translations(Language::Ar);
        let en = i18n.all_translations(Language::En);
        assert!(ar.len() > 30);
        assert!(en.len() > 30);
        assert_eq!(ar.len(), en.len()); // Same number of keys.
    }

    #[test]
    fn info_returns_json() {
        let i18n = I18n::new();
        let info = i18n.info(Language::Ar);
        assert_eq!(info["current_language"], "ar");
        assert_eq!(info["direction"], "rtl");
        assert!(info["supported_languages"].is_array());
    }

    #[test]
    fn language_all_returns_both() {
        let langs = Language::all();
        assert_eq!(langs.len(), 2);
        assert!(langs.contains(&Language::Ar));
        assert!(langs.contains(&Language::En));
    }
}
