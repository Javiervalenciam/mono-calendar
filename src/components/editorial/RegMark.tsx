// Marca de registro de imprenta — gesto editorial único del widget (T022).
// Lineal, 1 px, color heredado; no transmite estado (solo firma visual).
export function RegMark({ className = "" }: { className?: string }) {
  return (
    <svg
      viewBox="0 0 24 24"
      width="16"
      height="16"
      aria-hidden="true"
      className={className}
      fill="none"
      stroke="currentColor"
      strokeWidth="1"
    >
      <circle cx="12" cy="12" r="7" />
      <line x1="12" y1="1" x2="12" y2="23" />
      <line x1="1" y1="12" x2="23" y2="12" />
    </svg>
  );
}

/** Anotación elíptica terracota (subrayado manuscrito) para estados vacíos. */
export function EditorialEllipse({ className = "" }: { className?: string }) {
  return (
    <svg
      viewBox="0 0 220 44"
      aria-hidden="true"
      className={className}
      fill="none"
      stroke="var(--color-terra)"
      strokeWidth="1.5"
      strokeLinecap="round"
    >
      <path
        d="M12 30 C 40 12, 180 8, 208 20 C 216 24, 200 34, 150 37 C 96 40, 30 38, 12 30 Z"
        strokeDasharray="6 5"
      />
    </svg>
  );
}
