// sentiric-sip-uac/src/main.rs

use clap::Parser;
use std::process;
use tokio::sync::mpsc;
use tracing::{info, warn, error, Level};
use sentiric_telecom_client_sdk::{TelecomClient, UacEvent, CallState};

/// Sentiric SIP UAC - Field Testing Tool
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Target IP Address (e.g., 34.122.40.122)
    #[arg(index = 1)]
    target_ip: String,

    /// SIP Port
    #[arg(short, long, default_value_t = 5060)]
    port: u16,

    /// Destination User (Callee)
    #[arg(short, long, default_value = "service")]
    to: String,

    /// Source User (Caller)
    #[arg(short, long, default_value = "cli-uac")]
    from: String,

    /// Enable Headless Mode (Virtual DSP for Docker/CI)
    #[arg(long, default_value_t = false)]
    headless: bool,

    /// Enable Debug Logs (Show RMS levels and internal states)
    #[arg(long, default_value_t = false)]
    debug: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Logger Yapılandırması
    let log_level = if args.debug { Level::DEBUG } else { Level::INFO };
    
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .without_time() // Temiz çıktı için
        .init();

    info!("==================================================");
    info!("🚀 SENTIRIC SIP UAC (CLI) v2.4");
    info!("   Powered by SDK v0.3.13 (Self-Healing Audio)");
    info!("==================================================");
    info!("🎯 Target   : {}:{}", args.target_ip, args.port);
    info!("📞 Call     : {} -> {}", args.from, args.to);
    
    if args.headless {
        info!("👻 Mode     : HEADLESS (Virtual DSP / Ping-Pong)");
        info!("ℹ️  Hint     : Use --debug to see RMS signal levels.");
    } else {
        info!("🎤 Mode     : HARDWARE (Physical Sound Card)");
    }
    info!("--------------------------------------------------");

    let (tx, mut rx) = mpsc::channel::<UacEvent>(100);

    info!("⚙️  Initializing Telecom Engine...");
    let client = TelecomClient::new(tx, args.headless);

    // Event Loop
    let event_handler = tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                UacEvent::Log(msg) => {
                    // SIP paket loglarını sadece Debug modda veya özel durumlarda bas
                    // Ancak SDK'dan gelen önemli logları her zaman göster
                    if !msg.contains("[SIP_PACKET") {
                        info!("🔹 {}", msg); 
                    } else {
                        // Paketleri debug seviyesinde tut
                        tracing::debug!("{}", msg);
                    }
                }
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
                    info!("🎙️  MEDIA ACTIVE: 2-Way Audio Flow Established!");
                }
                UacEvent::RtpStats { rx_cnt, tx_cnt } => {
                     // Her ~2 saniyede bir istatistik bas
                     if rx_cnt % 100 == 0 || tx_cnt % 100 == 0 {
                         info!("📊 RTP Stats: RX={} | TX={}", rx_cnt, tx_cnt);
                     }
                }
            }
        }
    });

    info!("🚀 Dialing...");
    if let Err(e) = client.start_call(args.target_ip, args.port, args.to, args.from).await {
        error!("🔥 Failed to start call: {}", e);
        process::exit(1);
    }

    // Graceful Shutdown (Ctrl+C)
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            warn!("🛑 User interrupted. Sending BYE...");
            let _ = client.end_call().await;
            // BYE gitmesi için kısa bir süre bekle
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
        _ = event_handler => {}
    }

    Ok(())
}