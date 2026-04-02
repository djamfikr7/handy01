interface CorrectionPreviewProps {
  style: "inline" | "highlighted" | "draft-final";
}

const SAMPLE_RAW = "i went to the store to buy some bred";
const SAMPLE_CORRECTED = "I went to the store to buy some bread";

export default function CorrectionPreview({ style }: CorrectionPreviewProps) {
  const renderPreview = () => {
    switch (style) {
      case "inline":
        return <span>{SAMPLE_CORRECTED}</span>;
      case "highlighted":
        return (
          <span>
            I went to the store to buy some{" "}
            <span className="highlighted-correction">bread</span>
          </span>
        );
      case "draft-final":
        return (
          <span>
            <span className="draft-text">{SAMPLE_RAW}</span>
            <span className="final-text">→ {SAMPLE_CORRECTED}</span>
          </span>
        );
    }
  };

  return (
    <div className="p-3 bg-[var(--bg-primary)] rounded border border-[var(--accent)]">
      <p className="text-xs text-[var(--text-secondary)] mb-1">Preview:</p>
      <p className="text-sm">{renderPreview()}</p>
    </div>
  );
}
