import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

export default function StatusIndicator() {
  const [sidecarHealthy, setSidecarHealthy] = useState<boolean | null>(null);
  const [isRecording, setIsRecording] = useState(false);

  useEffect(() => {
    const checkHealth = async () => {
      try {
        const healthy = await invoke<boolean>("check_sidecar_health");
        setSidecarHealthy(healthy);
      } catch {
        setSidecarHealthy(false);
      }
    };

    const checkRecording = async () => {
      try {
        const recording = await invoke<boolean>("get_recording_state");
        setIsRecording(recording);
      } catch {
        // Ignore
      }
    };

    checkHealth();
    checkRecording();

    const interval = setInterval(() => {
      checkHealth();
      checkRecording();
    }, 2000);

    return () => clearInterval(interval);
  }, []);

  return (
    <div className="flex items-center gap-3 text-sm">
      <div className="flex items-center gap-1">
        <div
          className={`w-2 h-2 rounded-full ${
            sidecarHealthy === true
              ? "bg-green-500"
              : sidecarHealthy === false
                ? "bg-red-500"
                : "bg-yellow-500"
          }`}
        />
        <span className="text-[var(--text-secondary)]">Sidecar</span>
      </div>
      <div className="flex items-center gap-1">
        <div
          className={`w-2 h-2 rounded-full ${isRecording ? "bg-red-500 recording-indicator" : "bg-gray-500"}`}
        />
        <span className="text-[var(--text-secondary)]">
          {isRecording ? "Recording" : "Idle"}
        </span>
      </div>
    </div>
  );
}
