const PROMPTS = [
  "Kann ich mir ein Leasing Auto leisten?",
  "Welche Abos sind teurer geworden?",
  "Warum bin ich diesen Monat über Budget?",
  "Wie viel spare ich wenn ich Netflix kündige?",
  "Top Ausgabenkategorien diesen Monat",
];

export function SuggestedPrompts({ onSelect }: { onSelect: (text: string) => void }) {
  return (
    <div style={{ display: "flex", flexWrap: "wrap", gap: "0.5rem", marginBottom: "1rem" }}>
      {PROMPTS.map((p) => (
        <button
          key={p}
          type="button"
          className="btn"
          style={{ fontSize: "0.85rem" }}
          onClick={() => onSelect(p)}
        >
          {p}
        </button>
      ))}
    </div>
  );
}
