# 📠 Sentiric SIP UAC (CLI)

[![Status](https://img.shields.io/badge/status-active-success.svg)]()
[![Core](https://img.shields.io/badge/sdk-v0.3.13-orange.svg)]()
[![Docker](https://img.shields.io/badge/docker-ready-blue.svg)]()

**Sentiric SIP UAC**, sunucu tarafı SIP/RTP uygulamalarını (SBC, B2BUA, Media Server) test etmek için tasarlanmış, Rust tabanlı, yüksek performanslı bir komut satırı aracıdır.

Mobil sürümün aksine, bu araç **Arayüzsüz (Headless)** ortamlarda çalışmak üzere optimize edilmiştir.

## 🌟 Temel Özellikler

*   **Virtual DSP (Headless Mode):** Ses kartı olmayan sunucularda (CI/CD, Docker) çalışabilir.
    *   **TX:** Yapay bir sinüs dalgası (440Hz) üretir ve gönderir.
    *   **RX:** Gelen ses paketlerini decode eder ve sinyal seviyesini (RMS) ölçerek "Sesin gerçekten geldiğini" doğrular.
*   **RFC 3261 Uyumlu:** Tam stateful SIP yığını (`INVITE`, `ACK`, `BYE`, `Auto-Reply`).
*   **NAT Traversal:** Simetrik RTP ve Latching desteği.
*   **Telemetri:** Gelişmiş RTP paket sayacı ve jitter analizi.

## 🛠️ Kurulum

### Yöntem 1: Kaynaktan Derleme (Rust Gerekir)

```bash
# Bağımlılıkları yükle (Debian/Ubuntu)
sudo apt install libasound2-dev protobuf-compiler

# Release modunda derle
cargo build --release
```

### Yöntem 2: Docker (Önerilen)

```bash
docker build -t sentiric-uac .
```

## 💻 Kullanım

### Parametreler

```text
Usage: sentiric-sip-uac [OPTIONS] <TARGET_IP>

Arguments:
  <TARGET_IP>  Target IP Address (e.g., 34.122.40.122)

Options:
  -p, --port <PORT>      SIP Port [default: 5060]
  -t, --to <TO>          Destination User [default: service]
  -f, --from <FROM>      Source User [default: cli-uac]
      --headless         Enable Headless Mode (Virtual DSP)
      --debug            Enable Debug Logs (RMS Levels)
  -h, --help             Print help
```

### Senaryolar

#### 1. Manuel Echo Testi (Laptop/PC)
Kendi bilgisayarınızdan, donanım ses kartını kullanarak test yapın.
```bash
./target/release/sentiric-sip-uac 34.122.40.122 --port 5060 --to 9999
```

#### 2. Otomasyon Testi (CI/CD & Docker)
Ses kartı olmayan bir sunucuda, sesin gidip geldiğini (RMS seviyeleriyle) doğrulayın.
```bash
# --debug flag'i RMS loglarını açar
./target/release/sentiric-sip-uac 34.122.40.122 --headless --debug
```
*Beklenen Çıktı (Debug Mod):*
> `DEBUG ... [Headless RX] Voice Signal Detected! Level: 1245.32`

---
© 2026 Sentiric Team | GNU AGPL-3.0 License


---
