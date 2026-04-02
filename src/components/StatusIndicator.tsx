import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

export default function StatusIndicator() {
  const [isRecording, setIsRecording] = useState(false);

  useEffect(() => {
    let unmounted = false;

    const checkRecording = async () => {
      try {
        const recording = await invoke<boolean>("get_recording_state");
        if (!unmounted) setIsRecording(recording);
      } catch {
        // Tauri not ready
      }
    };

    checkRecording();
    const interval = setInterval(checkRecording, 1000);

    return () => { unmounted = true; clearInterval(interval); };
  }, []);

  return (
    <div className="flex items-center gap-3 text-sm">
      <div className="flex items-center gap-1">
        <div className="w-2 h-2 rounded-full bg-green-500" />
        <span className="text-[var(--text-secondary)]">Ready</span>
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
