import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface LiveTranscriptProps {
  correctionStyle: "inline" | "highlighted" | "draft-final";
}

export default function LiveTranscript({ correctionStyle }: LiveTranscriptProps) {
  const [text, setText] = useState("");
  const [isRecording, setIsRecording] = useState(false);

  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        const currentText = await invoke<string>("get_current_text");
        setText(currentText);
      } catch {
        // Ignore errors during polling
      }
    }, 200);

    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    const checkRecording = async () => {
      try {
        const recording = await invoke<boolean>("get_recording_state");
        setIsRecording(recording);
      } catch {
        // Ignore errors
      }
    };

    const interval = setInterval(checkRecording, 500);
    return () => clearInterval(interval);
  }, []);

  const renderText = () => {
    if (!text) {
      return (
        <span className="text-gray-500 italic">
          {isRecording ? "Listening..." : "Press Ctrl+Shift+Space to start"}
        </span>
      );
    }

    switch (correctionStyle) {
      case "highlighted":
        return <span className="highlighted-correction">{text}</span>;
      case "draft-final":
        return <span className="final-text">{text}</span>;
      default:
        return <span>{text}</span>;
    }
  };

  return (
    <div className="p-4 min-h-[200px] bg-[var(--bg-secondary)] rounded-lg">
      <div className="flex items-center gap-2 mb-2">
        {isRecording && (
          <div className="w-3 h-3 rounded-full bg-red-500 recording-indicator" />
        )}
        <span className="text-sm text-[var(--text-secondary)]">
          {isRecording ? "Recording" : "Idle"}
        </span>
      </div>
      <div className="text-lg leading-relaxed">{renderText()}</div>
    </div>
  );
}
