import { Star } from "lucide-react";
import { useFavorites } from "../lib/favorites";
import type { FavoriteKind } from "../lib/types";
import { favoriteId } from "../lib/types";
import { useI18n } from "../i18n";

type Props = {
  kind: FavoriteKind;
  itemKey: string;
  label: string;
  subtitle?: string | null;
  iconUrl?: string | null;
  size?: number;
  className?: string;
};

export function FavoriteButton({
  kind,
  itemKey,
  label,
  subtitle,
  iconUrl,
  size = 18,
  className,
}: Props) {
  const { t } = useI18n();
  const { isFavorite, toggle } = useFavorites();
  const id = favoriteId(kind, itemKey);
  const on = isFavorite(id);

  return (
    <button
      type="button"
      className={`euml-fav-btn${on ? " is-on" : ""}${className ? ` ${className}` : ""}`}
      title={on ? t("unfavorite") : t("favorite")}
      aria-label={on ? t("unfavorite") : t("favorite")}
      aria-pressed={on}
      onClick={(e) => {
        e.preventDefault();
        e.stopPropagation();
        void toggle({ kind, key: itemKey, label, subtitle, iconUrl });
      }}
    >
      <Star size={size} fill={on ? "currentColor" : "none"} strokeWidth={2} />
    </button>
  );
}
