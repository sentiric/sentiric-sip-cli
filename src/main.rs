// sentiric-sip-uac/src/main.rs

use std::env;
use std::process;
use tokio::sync::mpsc;
use tracing::{info, warn, error, Level};
// SDK Importları
use sentiric_telecom_client_sdk::{TelecomClient, UacEvent, CallState};

fn print_usage(program_name: &str) {
    println!("Usage: {} <TARGET_IP> [TARGET_PORT] [TO_USER] [FROM_USER] [--headless]", program_name);
    println!("Flags:");
    println!("  --headless    : Disables audio hardware access (Use for Docker/CI)");
    println!("Example:");
    println!("  {} 34.122.40.122 5060 9999 cli-tester --headless", program_name);
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Logger Kurulumu
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .without_time() // CLI'da daha temiz görünüm için zamanı gizle
        .init();

    // 2. Argüman Ayrıştırma (Flag Parsing)
    let args: Vec<String> = env::args().collect();
    
    // --headless bayrağını kontrol et
    let headless = args.iter().any(|arg| arg == "--headless");

    // Argüman listesinden flagleri temizle (Logic için temiz liste)
    let clean_args: Vec<String> = args.iter().filter(|a| !a.starts_with("--")).cloned().collect();

    if clean_args.len() < 2 {
        error!("❌ Missing arguments.");
        print_usage(&args[0]);
        process::exit(1);
    }

    let target_ip = clean_args[1].clone();
    let target_port: u16 = clean_args.get(2).and_then(|s| s.parse().ok()).unwrap_or(5060);
    let to_user = clean_args.get(3).cloned().unwrap_or_else(|| "service".to_string());
    let from_user = clean_args.get(4).cloned().unwrap_or_else(|| "cli-uac".to_string());

    info!("==========================================");
    info!("🚀 SENTIRIC SIP UAC v2.2 (SDK v0.3.0)");
    info!("==========================================");
    info!("🎯 Target   : {}:{}", target_ip, target_port);
    info!("📞 Call     : {} -> {}", from_user, to_user);
    
    if headless {
        info!("👻 Mode     : HEADLESS (Virtual DSP - Docker Safe)");
    } else {
        info!("🎤 Mode     : HARDWARE (Physical Sound Card)");
    }
    info!("------------------------------------------");

    // 3. Kanal Kurulumu (SDK -> CLI)
    let (tx, mut rx) = mpsc::channel::<UacEvent>(100);

    // 4. SDK Motorunu Başlat (Headless parametresi ile)
    info!("⚙️  Initializing Telecom Engine...");
    
    // [KRİTİK GÜNCELLEME]: new fonksiyonu artık headless bool alıyor
    let client = TelecomClient::new(tx, headless);

    // 5. Olay Dinleyici (Background Task)
    let event_handler = tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                // SDK'dan gelen detaylı loglar (SIP Paketleri dahil)
                UacEvent::Log(msg) => {
                    println!("{}", msg); 
                }
                // Çağrı Durum Değişiklikleri
                UacEvent::CallStateChanged(state) => {
                    info!("🔔 CALL STATE: {:?}", state);
                    if state == CallState::Terminated {
                        info!("🏁 Call Terminated. Exiting...");
                        process::exit(0);
                    }
                }
                UacEvent::Error(err) => {
                    error!("❌ SDK ERROR: {}", err);
                    process::exit(1);
                }
                UacEvent::MediaActive => {
                    info!("🎙️  MEDIA ACTIVE: 2-Way Audio Established!");
                }
                UacEvent::RtpStats { rx_cnt, tx_cnt } => {
                     // İstatistikleri çok sık basmamak için debug seviyesinde tutabiliriz veya 50 pakette bir basarız
                     if rx_cnt % 50 == 0 {
                         info!("📊 RTP Stats: RX={} TX={}", rx_cnt, tx_cnt);
                     }
                }
            }
        }
    });

    // 6. Aramayı Başlat
    info!("🚀 Dialing...");
    if let Err(e) = client.start_call(target_ip, target_port, to_user, from_user).await {
        error!("🔥 Failed to start call: {}", e);
        process::exit(1);
    }

    // 7. Kapanış Sinyali Bekleme
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            warn!("🛑 User interrupted. Sending BYE...");
            let _ = client.end_call().await;
            // Bye gönderimi için kısa bir süre tanı
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
        _ = event_handler => {}
    }

    Ok(())
}