type Props = {
  label: string;
  value: string;
  values: number[];
  color?: string;
  height?: number;
  width?: number;
};

/** Lightweight SVG sparkline — no chart library. */
export function MetricSparkline({
  label,
  value,
  values,
  color = "#1370f0",
  height = 48,
  width = 220,
}: Props) {
  const pad = 2;
  const max = Math.max(...values, 0.0001);
  const min = Math.min(...values, 0);
  const span = Math.max(max - min, 0.0001);
  const pts = values.length
    ? values
        .map((v, i) => {
          const x = pad + (i / Math.max(values.length - 1, 1)) * (width - pad * 2);
          const y = height - pad - ((v - min) / span) * (height - pad * 2);
          return `${x.toFixed(1)},${y.toFixed(1)}`;
        })
        .join(" ")
    : "";

  const area =
    values.length > 1
      ? `${pad},${height - pad} ${pts} ${width - pad},${height - pad}`
      : "";

  return (
    <div style={{ minWidth: width, flex: "1 1 200px" }}>
      <div style={{ display: "flex", justifyContent: "space-between", marginBottom: 4, gap: 8 }}>
        <span style={{ fontSize: 13, opacity: 0.75 }}>{label}</span>
        <span style={{ fontSize: 13, fontWeight: 600 }}>{value}</span>
      </div>
      <svg
        width="100%"
        height={height}
        viewBox={`0 0 ${width} ${height}`}
        preserveAspectRatio="none"
        style={{ display: "block", background: "var(--astryx-color-bg-secondary, #f3f5f8)", borderRadius: 6 }}
      >
        {area && (
          <polygon points={area} fill={color} opacity={0.15} />
        )}
        {pts && (
          <polyline
            points={pts}
            fill="none"
            stroke={color}
            strokeWidth={2}
            strokeLinejoin="round"
            strokeLinecap="round"
          />
        )}
      </svg>
    </div>
  );
}
