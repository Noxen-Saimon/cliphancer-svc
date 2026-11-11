# Cliphancer-svc ✂️🔗

> **Ultra-Lightweight Windows Background Clipboard Processor**  
> Dibuat dengan **Rust** dan native Win32 API. Berjalan 100% senyap di latar belakang (*headless*) tanpa jendela konsol dan tanpa tray icon, dengan konsumsi memori hanya **~5–6 MB RAM** dan **0.0% CPU** saat idle.

---

## ⚡ Fitur Utama (Fokus Single-Link Ultra-Fast)

1. **Fast Single-Link Privacy Stripper:**
   Hanya aktif jika Anda menyalin **satu link spesifik** (dimulai dengan `http://` atau `https://` tanpa spasi di tengah).
   * Menghilangkan parameter pelacak secara instan:
     * **YouTube:** Menghapus `?si=...`, `&feature=...`, `&pp=...`
     * **Google Analytics & Ads:** Menghapus `utm_source`, `utm_medium`, `utm_campaign`, `utm_term`, `utm_content`, `gclid`, `gbraid`, `wbraid`, `gad_source`, dll.
     * **Meta (Facebook / Instagram):** Menghapus `fbclid`, `igshid`, `igsh`
     * **Twitter / X:** Menghapus `ref_src`, `ref_url`, `twclid`, `s`, `t`
     * **TikTok:** Menghapus `_r`, `_t`, `_s`
     * **Marketplace / E-Commerce:** Menghapus `spm`, `scm`, `msclkid`, `yclid`, dll.
     * Tetap menjaga parameter penting lainnya (seperti ID video, query pencarian `?q=`, dsb.).

2. **Zero-Overhead / Instant Bypass:**
   * Jika teks yang Anda salin adalah paragraf, artikel, dokumen panjang, kode program, atau kalimat yang memuat banyak link, **cliphancer-svc langsung mengabaikannya dalam hitungan nanodetik tanpa memproses atau mengalokasikan memori**.
   * Ini memastikan service tidak pernah berat dan tidak memakan resource komputer Anda.

3. **Auto-Trim Whitespace:**
   * Menghapus spasi dan baris kosong tak sengaja di awal atau akhir link yang disalin.

4. **Loop & Single-Instance Protection:**
   * Mencegah pemanggilan berulang (infinite loop) saat clipboard ditulis ulang.
   * Single-instance enforcement via Windows Named Mutex (`Global\CliphancerSvc_SingleInstanceMutex`).

---

## 📊 Spesifikasi Kinerja

| Metrik | Nilai |
|---|---|
| **Ukuran Executable (.exe)** | **~302 KB** (Release binary teroptimasi LTO & stripped) |
| **Konsumsi RAM (Working Set)** | **~5–6 MB** |
| **Beban CPU saat Diam** | **0.0%** (Event-driven via Win32 `WM_CLIPBOARDUPDATE`) |
| **Jendela Konsol (CMD)** | **Tidak Ada** (`windows_subsystem = "windows"`) |

---

## 🚀 Cara Penggunaan

### 1. Menjalankan Aplikasi
Cukup klik ganda (double-click) file:
```text
C:\Users\Administrator\Documents\Github Project\cliphancer-svc\cliphancer-svc.exe
```
Aplikasi akan langsung berjalan senyap di background tanpa pop-up jendela.

### 2. Mematikan Aplikasi
Karena aplikasi berjalan *headless*, Anda dapat mematikannya melalui **Task Manager**:
1. Buka Task Manager (`Ctrl + Shift + Esc`).
2. Cari `cliphancer-svc.exe` di tab **Processes** atau **Details**.
3. Klik kanan lalu pilih **End task**.

Atau melalui terminal PowerShell:
```powershell
Stop-Process -Name "cliphancer-svc" -Force
```

---

## 🔄 Menjalankan Otomatis saat Windows Menyala (Opsional)

Jika ingin aplikasi otomatis aktif saat komputer dinyalakan:
1. Tekan `Win + R`, ketik `shell:startup` lalu tekan Enter.
2. Buat shortcut (*shortcut link*) dari file `cliphancer-svc.exe` dan letakkan di dalam folder Startup tersebut.

---

## 🛠️ Kompilasi dari Source Code

Jika ingin memodifikasi atau meng-compile ulang:
```powershell
cd "C:\Users\Administrator\Documents\Github Project\cliphancer-svc"
cargo build --release
```
Output file akan berada di: `target\release\cliphancer-svc.exe`.
