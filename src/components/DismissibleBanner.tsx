import { Banner } from "@astryxdesign/core/Banner";

/** Dismissible error/info banner — clears when the user closes it. */
export function DismissibleBanner({
  status,
  title,
  onDismiss,
}: {
  status: "error" | "info" | "success" | "warning";
  title: string;
  onDismiss: () => void;
}) {
  return (
    <div style={{ position: "relative" }}>
      <Banner status={status} title={title} />
      <button
        type="button"
        aria-label="Dismiss"
        onClick={onDismiss}
        style={{
          position: "absolute",
          top: 8,
          right: 10,
          border: "none",
          background: "transparent",
          cursor: "pointer",
          fontSize: 16,
          lineHeight: 1,
          color: "inherit",
          opacity: 0.7,
        }}
      >
        ×
      </button>
    </div>
  );
}
