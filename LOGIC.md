# 🧬 Autonomous Testing Logic & Test Vectors

Bu belge, Sentiric altyapısının (SBC, Proxy, B2BUA, Media) sınırlarını zorlamak ve çökme durumlarını (Edge Cases) tespit etmek için SIP UAC botunun kullandığı otonom test mantığını tanımlar.

## 1. Otonom Dayanıklılık Testi (Resilience Suite)
Sistemi manuel test etmek yerine, JSON tabanlı senaryo motorunu kullanan otomatik test seti çalıştırılır. Motor, JSON dosyalarından `wait`, `dtmf`, `hangup` gibi adımları okuyarak state machine'i işletir.

## 2. Test Vektörleri ve Beklenen Sonuçlar

### Vektör 1: Immediate Hangup (Race Condition Test)
* **Mantık:** Arama başlatır (INVITE atar) ve 50ms içinde, sunucu daha işlemi bağlamadan CANCEL/BYE atar.
* **Başarı Kriteri:** Orchestrator loglarında asılı/unutulmuş bir RTP portu kalmamalı, `Panic` olmamalıdır.

### Vektör 2: Rapid DTMF (State Machine Stress)
* **Mantık:** 100ms aralıklarla çok hızlı şekilde in-band DTMF paketleri (Payload 101) fırlatır.
* **Başarı Kriteri:** Ses akışı (RTP) kopmamalı, decoder kilitlenmemelidir.

### Vektör 3: Ghost Call (Inactivity Timeout)
* **Mantık:** Arama açılır ve bot 45 saniye boyunca hiçbir RTP paketi göndermeden sessiz kalır.
* **Başarı Kriteri:** Media Service yaklaşık 30. saniyede trafiğin durduğunu anlayıp `RTP_TIMEOUT` hatası fırlatmalı ve çağrıyı kendi kendine sonlandırmalıdır.

### Vektör 4: Long Call (Memory Leak & Jitter Test)
* **Mantık:** 60 saniye boyunca sürekli paket gönderir.
* **Başarı Kriteri:** RAM tüketiminde devasa artış olmamalı ve Jitter Buffer kaymaları onarabilmelidir.