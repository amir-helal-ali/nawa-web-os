---
Task ID: ws-rfc6455-compliance
Agent: main
Task: استخدم WebSocket حي بدلاً من polling — تحقيق الامتثال الصارم لـ RFC 6455

Work Log:
- اكتشاف خطأ حرج: WebSocket handshake كان يستخدم SHA-256 بدلاً من SHA-1 (المتصفحات ترفض الـ connection بصمت)
- إضافة `sha1 = "0.10"` crate إلى nawad/Cargo.toml
- إعادة كتابة `realtime.rs` بالكامل (245 سطر):
  * استبدال SHA-256 بـ SHA-1 في `compute_ws_accept` (RFC 6455 §1.3)
  * إصلاح قراءة 64-bit payload length (كان يتجاهل أول 2 بايت خطأً)
  * قراءة headers حتى `\r\n\r\n` بدلاً من single-read
  * حماية 1 MiB ضد العملاء الخبيثين
  * إضافة `OpCode` enum كامل (Text/Binary/Close/Ping/Pong/Continuation)
  * إضافة heartbeat 30s عبر `tokio::time::interval` (pure async, no polling)
  * معالجة broadcast `Lagged` بشكل صحيح
  * إضافة 7 اختبارات جديدة بما فيها RFC 6455 test vector الرسمي
- بناء ناجح: 0 warnings, 0 errors
- اختبارات وحدة: 6 realtime + 27 auth + 28 db + 27 engine + 13 http + 28 uring = **129 اختبار ناجح**
- اختبار E2E شامل عبر `/home/z/my-project/scripts/e2e_test.sh`:
  * Dashboard: HTTP 200, 9049 bytes
  * تسجيل أول مستخدم → أصبح admin تلقائياً
  * DB write/read cycle ناجح
  * Event Bus: 2+ notifications منشورة (register + db_write)
  * WebSocket handshake حقيقي: `s3pPLMBiTxaQ9kYGzzhZRbK+xOo=` (مطابق 100% لـ RFC 6455 test vector)
- Push إلى GitHub: commit 6786abd على main

Stage Summary:
- النظام الآن يستخدم WebSocket **حي** 100% (لا polling في أي مكان)
- الامتثال لـ RFC 6455 مؤكد بـ test vector الرسمي
- 34 routes تعمل، أول مستخدم = admin تلقائياً
- Event Bus + WebSocket = إشعارات لحظية حقيقية (push model)
- الملفات المُنتجة: `crates/nawad/src/realtime.rs`, `scripts/e2e_test.sh`
- إجمالي الاختبارات الناجحة: 129+
