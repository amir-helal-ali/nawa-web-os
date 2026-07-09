"""Render a visual preview of NAWA v2.4.0 UI (English labels since no Arabic font available)."""
import matplotlib.pyplot as plt
import matplotlib.patches as patches
from matplotlib.patches import FancyBboxPatch, Rectangle, Circle
import matplotlib.font_manager as fm

# Use available fonts
plt.rcParams['font.sans-serif'] = ['DejaVu Sans']
plt.rcParams['axes.unicode_minus'] = False

# Colors from App.svelte :root
BG = '#0a0a0f'
SURFACE = '#14141e'
SURFACE_LIGHT = '#1a1a26'
BORDER = '#3a3320'  # rgba(245,158,11,0.15) approximated
PRIMARY = '#f59e0b'
PRIMARY_LIGHT = '#fbbf24'
ACCENT = '#10b981'
ACCENT_LIGHT = '#34d399'
TEXT = '#e8e8ef'
MUTED = '#8b8b9a'
DANGER = '#ef4444'

fig, ax = plt.subplots(figsize=(16, 10), facecolor=BG)
ax.set_xlim(0, 16)
ax.set_ylim(0, 10)
ax.set_facecolor(BG)
ax.axis('off')

# === Top Navigation Bar ===
nav = Rectangle((0, 9.3), 16, 0.7, facecolor='#0d0d14', edgecolor=BORDER, linewidth=0.5)
ax.add_patch(nav)
ax.text(0.3, 9.65, '◆', fontsize=20, color=PRIMARY, va='center', ha='left')
ax.text(0.75, 9.65, 'NAWA', fontsize=15, color=PRIMARY, fontweight='bold', va='center')
vbadge = FancyBboxPatch((1.65, 9.45), 0.7, 0.25, boxstyle="round,pad=0.05", 
                         facecolor='#3a2e10', edgecolor=PRIMARY, linewidth=0.5)
ax.add_patch(vbadge)
ax.text(2.0, 9.575, 'v2.4.0', fontsize=8, color=PRIMARY, ha='center', va='center', fontweight='bold')
# Nav links
for i, (label, x, active) in enumerate([('Home', 3.0, True), ('AION', 4.0, False), 
                                          ('Admin', 4.9, False), ('API', 5.7, False), 
                                          ('Docs', 6.4, False)]):
    color = PRIMARY if active else MUTED
    if active:
        bg = FancyBboxPatch((x-0.15, 9.4), 0.9, 0.4, boxstyle="round,pad=0.05",
                             facecolor='#3a2e10', edgecolor='none')
        ax.add_patch(bg)
    ax.text(x+0.3, 9.65, label, fontsize=9, color=color, ha='center', va='center')
# Login/Register buttons
login = FancyBboxPatch((12.5, 9.4), 0.9, 0.4, boxstyle="round,pad=0.05",
                        facecolor=SURFACE, edgecolor=BORDER, linewidth=0.5)
ax.add_patch(login)
ax.text(12.95, 9.6, 'Login', fontsize=8, color=TEXT, ha='center', va='center')
register = FancyBboxPatch((13.6, 9.4), 1.5, 0.4, boxstyle="round,pad=0.05",
                           facecolor=PRIMARY, edgecolor='none')
ax.add_patch(register)
ax.text(14.35, 9.6, 'Get Started →', fontsize=8, color=BG, ha='center', va='center', fontweight='bold')

# === Hero Section ===
badge = FancyBboxPatch((0.7, 8.0), 3.5, 0.35, boxstyle="round,pad=0.1",
                        facecolor='#0f2a1f', edgecolor=ACCENT, linewidth=0.5)
ax.add_patch(badge)
ax.plot(0.95, 8.175, 'o', markersize=5, color=ACCENT)
ax.text(1.15, 8.175, 'Quantum-Powered Web OS · v2.4.0', fontsize=8, color=ACCENT, va='center')

# Hero title (gradient effect via two lines)
ax.text(0.7, 7.3, 'NAWA Web', fontsize=32, color=PRIMARY, fontweight='bold', va='center')
ax.text(0.7, 6.5, 'Operating System', fontsize=32, color=ACCENT, fontweight='bold', va='center')

# Subtitle
ax.text(0.7, 5.85, 'One pure Rust binary · No Node.js · Quantum mechanics · Zero copies',
        fontsize=9, color=MUTED, va='center')
ax.text(0.7, 5.55, '87 endpoints · 26 modules · 530+ tests · 10.5MB binary',
        fontsize=8, color=MUTED, va='center', style='italic')

# Hero buttons
btn1 = FancyBboxPatch((0.7, 4.7), 2.2, 0.5, boxstyle="round,pad=0.1",
                       facecolor=PRIMARY, edgecolor='none')
ax.add_patch(btn1)
ax.text(1.8, 4.95, 'Get Started →', fontsize=10, color=BG, ha='center', va='center', fontweight='bold')

btn2 = FancyBboxPatch((3.1, 4.7), 2.3, 0.5, boxstyle="round,pad=0.1",
                       facecolor=SURFACE, edgecolor=BORDER, linewidth=0.5)
ax.add_patch(btn2)
ax.text(4.25, 4.95, '◎ AION Engine', fontsize=10, color=TEXT, ha='center', va='center')

# Stats card
card = FancyBboxPatch((9, 4.5), 5.5, 4.0, boxstyle="round,pad=0.15",
                       facecolor=SURFACE, edgecolor=BORDER, linewidth=0.5)
ax.add_patch(card)
glow = FancyBboxPatch((8.95, 4.45), 5.6, 4.1, boxstyle="round,pad=0.2",
                       facecolor='none', edgecolor=PRIMARY, linewidth=0.3, alpha=0.2)
ax.add_patch(glow)

stats = [
    ('Endpoints', '87', PRIMARY),
    ('Modules', '26', PRIMARY),
    ('Tests', '530+', PRIMARY),
    ('Binary', '10.5MB', PRIMARY),
    ('Node.js', 'NOT required', DANGER),
    ('Polling', 'zero', DANGER),
]
for i, (label, value, color) in enumerate(stats):
    y = 8.1 - i * 0.55
    ax.text(9.3, y, label, fontsize=9, color=MUTED, va='center')
    ax.text(14.2, y, value, fontsize=10, color=color, va='center', ha='right', fontweight='bold',
            family='monospace')
    if i < 5:
        ax.plot([9.3, 14.2], [y - 0.27, y - 0.27], '-', color='#1f1f2e', linewidth=0.5)

# === Features Grid ===
ax.text(8, 3.7, 'Revolutionary Features', fontsize=18, color=TEXT, ha='center', fontweight='bold')

features = [
    ('Q', 'Quantum Mechanics', 'Superposition · Entanglement · Tunneling'),
    ('A', 'AION SEO Engine', 'Knowledge Graph · Photon · QEC'),
    ('W', 'WASM SSR', 'wasmtime · Zero-copy · Sandbox'),
    ('S', 'WebSocket Pub/Sub', 'Real-time · 6 Channels · Event Bus'),
    ('H', 'High Security', 'CSP · HSTS · RBAC · 11 headers'),
    ('D', 'Pro Design', '3 Themes · Glassmorphism · RTL'),
]

for i, (icon, title, tags) in enumerate(features):
    col = i % 3
    row = i // 3
    x = 0.7 + col * 5.1
    y = 2.7 - row * 1.3
    
    card = FancyBboxPatch((x, y - 0.9), 4.7, 1.1, boxstyle="round,pad=0.1",
                           facecolor=SURFACE, edgecolor=BORDER, linewidth=0.5)
    ax.add_patch(card)
    # Icon as colored circle with letter
    icon_circle = Circle((x + 0.45, y - 0.15), 0.22, facecolor=PRIMARY, edgecolor='none', alpha=0.15)
    ax.add_patch(icon_circle)
    ax.text(x + 0.45, y - 0.15, icon, fontsize=14, color=PRIMARY, va='center', ha='center', fontweight='bold')
    ax.text(x + 0.9, y - 0.05, title, fontsize=11, color=PRIMARY, fontweight='bold', va='center')
    ax.text(x + 0.9, y - 0.4, tags, fontsize=7, color=MUTED, va='center')
    # Tags as small badges
    tag_x = x + 0.9
    for tag in tags.split(' · '):
        tag_w = 0.15 + 0.08 * len(tag)
        tag_bg = FancyBboxPatch((tag_x, y - 0.65), tag_w, 0.18, boxstyle="round,pad=0.03",
                                 facecolor='#3a2e10', edgecolor='none')
        ax.add_patch(tag_bg)
        ax.text(tag_x + tag_w/2, y - 0.56, tag, fontsize=6, color=PRIMARY, ha='center', va='center')
        tag_x += tag_w + 0.1

# Footer
ax.text(8, 0.15, '◆ NAWA Web Operating System v2.4.0 — Revolutionary',
        fontsize=9, color=MUTED, ha='center', va='center', style='italic')
ax.text(8, 0.0, 'GitHub · Docs · OpenAPI · Photon',
        fontsize=7, color=PRIMARY, ha='center', va='center')

plt.tight_layout(pad=0)
plt.savefig('/home/z/my-project/download/nawa-v2.4.0-home.png', 
            dpi=120, facecolor=BG, bbox_inches='tight', pad_inches=0.1)
print("✓ Screenshot saved")
