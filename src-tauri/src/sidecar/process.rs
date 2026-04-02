use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

use super::client::SidecarClient;

pub struct SidecarProcess {
    process: Option<Child>,
    running: Arc<AtomicBool>,
    port: u16,
}

impl SidecarProcess {
    pub fn new(port: u16) -> Self {
        Self {
            process: None,
            running: Arc::new(AtomicBool::new(false)),
            port,
        }
    }

    pub fn start(&mut self, sidecar_dir: PathBuf) -> Result<(), String> {
        if self.is_running() {
            return Ok(());
        }

        let python_cmd = Self::find_python()?;
        let server_path = sidecar_dir.join("server.py");

        if !server_path.exists() {
            return Err(format!("Sidecar server.py not found at {:?}", server_path));
        }

        let mut cmd = Command::new(&python_cmd);
        cmd.arg("-m")
            .arg("uvicorn")
            .arg("server:app")
            .arg("--host")
            .arg("127.0.0.1")
            .arg("--port")
            .arg(self.port.to_string())
            .current_dir(&sidecar_dir);

        if let Ok(ld_path) = std::env::var("LD_LIBRARY_PATH") {
            cmd.env("LD_LIBRARY_PATH", ld_path);
        }

        let child = cmd
            .spawn()
            .map_err(|e| format!("Failed to start sidecar: {}", e))?;

        self.process = Some(child);
        self.running.store(true, Ordering::SeqCst);

        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(mut child) = self.process.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.running.store(false, Ordering::SeqCst);
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst) && self.process.is_some()
    }

    pub async fn wait_for_health(&self, client: &SidecarClient, max_retries: u32) -> Result<(), String> {
        for _ in 0..max_retries {
            if client.health_check().await.is_ok() {
                return Ok(());
            }
            sleep(Duration::from_millis(500)).await;
        }
        Err("Sidecar failed to become healthy".to_string())
    }

    fn find_python() -> Result<String, String> {
        which::which("python3")
            .or_else(|_| which::which("python"))
            .map(|p| p.to_string_lossy().to_string())
            .map_err(|_| "Python not found".to_string())
    }
}

impl Drop for SidecarProcess {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_not_running_initially() {
        let process = SidecarProcess::new(8765);
        assert!(!process.is_running());
    }

    #[test]
    fn test_process_start_fails_without_server() {
        let mut process = SidecarProcess::new(8765);
        let result = process.start(PathBuf::from("/nonexistent"));
        assert!(result.is_err());
    }

    #[test]
    fn test_find_python() {
        let result = SidecarProcess::find_python();
        match result {
            Ok(path) => assert!(!path.is_empty()),
            Err(_) => assert!(true),
        }
    }
}
