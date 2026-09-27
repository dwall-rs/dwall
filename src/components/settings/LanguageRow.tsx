// Responsibility: Language row — uses the shadcn Select, reading and writing the i18n locale directly.
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
