import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Settings {
  correction_style: string;
  whisper_model: string;
  sidecar_port: number;
  openai_api_key: string;
  anthropic_api_key: string;
  hotkey: {
    modifiers: string[];
    key: string;
  };
}

interface SettingsPanelProps {
  isOpen: boolean;
  onClose: () => void;
}

export default function SettingsPanel({ isOpen, onClose }: SettingsPanelProps) {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    if (isOpen) {
      loadSettings();
    }
  }, [isOpen]);

  const loadSettings = async () => {
    try {
      const loaded = await invoke<Settings>("get_settings");
      setSettings(loaded);
    } catch (e) {
      console.error("Failed to load settings:", e);
    }
  };

  const saveSettings = async () => {
    if (!settings) return;
    setSaving(true);
    try {
      await invoke("update_settings", { settings });
      onClose();
    } catch (e) {
      console.error("Failed to save settings:", e);
    } finally {
      setSaving(false);
    }
  };

  if (!isOpen || !settings) return null;

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-[var(--bg-secondary)] rounded-lg p-6 w-[400px] max-h-[80vh] overflow-y-auto">
        <h2 className="text-xl font-bold mb-4">Settings</h2>

        <div className="space-y-4">
          <div>
            <label className="block text-sm text-[var(--text-secondary)] mb-1">
              Correction Style
            </label>
            <select
              className="w-full bg-[var(--bg-primary)] text-[var(--text-primary)] rounded p-2"
              value={settings.correction_style}
              onChange={(e) =>
                setSettings({ ...settings, correction_style: e.target.value })
              }
            >
              <option value="inline">Inline (silent correction)</option>
              <option value="highlighted">Highlighted (show corrections)</option>
              <option value="draft-final">Draft → Final (show both)</option>
            </select>
          </div>

          <div>
            <label className="block text-sm text-[var(--text-secondary)] mb-1">
              Whisper Model
            </label>
            <select
              className="w-full bg-[var(--bg-primary)] text-[var(--text-primary)] rounded p-2"
              value={settings.whisper_model}
              onChange={(e) =>
                setSettings({ ...settings, whisper_model: e.target.value })
              }
            >
              <option value="large-v3">large-v3 (best quality)</option>
              <option value="medium">medium (balanced)</option>
              <option value="small">small (fastest)</option>
            </select>
          </div>

          <div>
            <label className="block text-sm text-[var(--text-secondary)] mb-1">
              OpenAI API Key
            </label>
            <input
              type="password"
              className="w-full bg-[var(--bg-primary)] text-[var(--text-primary)] rounded p-2"
              value={settings.openai_api_key}
              onChange={(e) =>
                setSettings({ ...settings, openai_api_key: e.target.value })
              }
              placeholder="sk-..."
            />
          </div>

          <div>
            <label className="block text-sm text-[var(--text-secondary)] mb-1">
              Anthropic API Key
            </label>
            <input
              type="password"
              className="w-full bg-[var(--bg-primary)] text-[var(--text-primary)] rounded p-2"
              value={settings.anthropic_api_key}
              onChange={(e) =>
                setSettings({ ...settings, anthropic_api_key: e.target.value })
              }
              placeholder="sk-ant-..."
            />
          </div>

          <div>
            <label className="block text-sm text-[var(--text-secondary)] mb-1">
              Sidecar Port
            </label>
            <input
              type="number"
              className="w-full bg-[var(--bg-primary)] text-[var(--text-primary)] rounded p-2"
              value={settings.sidecar_port}
              onChange={(e) =>
                setSettings({
                  ...settings,
                  sidecar_port: parseInt(e.target.value) || 8765,
                })
              }
            />
          </div>
        </div>

        <div className="flex gap-2 mt-6">
          <button
            className="flex-1 bg-[var(--accent-bright)] text-white rounded p-2 hover:opacity-90"
            onClick={saveSettings}
            disabled={saving}
          >
            {saving ? "Saving..." : "Save"}
          </button>
          <button
            className="flex-1 bg-[var(--accent)] text-white rounded p-2 hover:opacity-90"
            onClick={onClose}
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  );
}
