/* ===== src/components/settings/LanguageRow.tsx ===== */
// 职责：语言行——用 shadcn Select，直接读写 i18n 的 locale。
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { LANGUAGES, locale, setLocale, type Locale } from "@/i18n";
import { For } from "solid-js";

const OPTIONS = Object.entries(LANGUAGES) as [Locale, string][];

export function LanguageRow() {
  return (
    <Select value={locale()} onValueChange={(v) => setLocale(v as Locale)}>
      <SelectTrigger class="min-w-37.5" variant="outline">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        <For each={OPTIONS}>
          {([value, label]) => <SelectItem value={value}>{label}</SelectItem>}
        </For>
      </SelectContent>
    </Select>
  );
}
