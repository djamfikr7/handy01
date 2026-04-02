import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import LiveTranscript from "./components/LiveTranscript";
import StatusIndicator from "./components/StatusIndicator";
import SettingsPanel from "./components/SettingsPanel";
import CorrectionPreview from "./components/CorrectionPreview";

function App() {
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [correctionStyle, setCorrectionStyle] = useState<
    "inline" | "highlighted" | "draft-final"
  >("inline");
  const [isRecording, setIsRecording] = useState(false);

  useEffect(() => {
    const checkRecording = async () => {
      try {
        const recording = await invoke<boolean>("get_recording_state");
        setIsRecording(recording);
      } catch {
        // ignore
      }
    };
    checkRecording();
    const interval = setInterval(checkRecording, 500);
    return () => clearInterval(interval);
  }, []);

  const handleToggleRecording = async () => {
    try {
      const recording = await invoke<boolean>("toggle_recording");
      setIsRecording(recording);
    } catch (e) {
      console.error("Failed to toggle recording:", e);
    }
  };

  return (
    <div className="min-h-screen bg-[var(--bg-primary)] p-4">
      <div className="max-w-lg mx-auto space-y-4">
        <header className="flex items-center justify-between">
          <h1 className="text-xl font-bold">Handy01</h1>
          <div className="flex items-center gap-3">
            <StatusIndicator />
            <button
              className="text-sm text-[var(--text-secondary)] hover:text-[var(--text-primary)]"
              onClick={() => setSettingsOpen(true)}
            >
              ⚙ Settings
            </button>
          </div>
        </header>

        <button
          className={`w-full py-3 rounded-lg text-lg font-semibold transition-colors ${
            isRecording
              ? "bg-red-600 hover:bg-red-700 text-white"
              : "bg-green-600 hover:bg-green-700 text-white"
          }`}
          onClick={handleToggleRecording}
        >
          {isRecording ? "⏹ Stop Recording" : "🎤 Start Recording"}
        </button>

        <LiveTranscript correctionStyle={correctionStyle} />

        <CorrectionPreview style={correctionStyle} />

        <div className="flex gap-2">
          {(["inline", "highlighted", "draft-final"] as const).map((style) => (
            <button
              key={style}
              className={`px-3 py-1 rounded text-sm ${
                correctionStyle === style
                  ? "bg-[var(--accent-bright)] text-white"
                  : "bg-[var(--bg-secondary)] text-[var(--text-secondary)]"
              }`}
              onClick={() => setCorrectionStyle(style)}
            >
              {style}
            </button>
          ))}
        </div>

        <p className="text-xs text-[var(--text-secondary)] text-center">
          Or press Ctrl+Shift+Space to toggle recording
        </p>
      </div>

      <SettingsPanel isOpen={settingsOpen} onClose={() => setSettingsOpen(false)} />
    </div>
  );
}

export default App;
