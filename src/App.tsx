import { useState } from "react";
import LiveTranscript from "./components/LiveTranscript";
import StatusIndicator from "./components/StatusIndicator";
import SettingsPanel from "./components/SettingsPanel";
import CorrectionPreview from "./components/CorrectionPreview";

function App() {
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [correctionStyle, setCorrectionStyle] = useState<
    "inline" | "highlighted" | "draft-final"
  >("inline");

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
          Press Ctrl+Shift+Space to toggle recording
        </p>
      </div>

      <SettingsPanel isOpen={settingsOpen} onClose={() => setSettingsOpen(false)} />
    </div>
  );
}

export default App;
