/* ===== src/components/settings/LanguageRow.tsx ===== */
// 职责：语言行——用 shadcn Select（请 npx shadcn add select）。
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { settingsStore, setLang } from "@/store/settings.store";

export function LanguageRow() {
  return (
    <Select value={settingsStore.lang} onValueChange={setLang}>
      <SelectTrigger class="min-w-37.5" variant="outline">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value="English">English</SelectItem>
        <SelectItem value="中文">中文</SelectItem>
      </SelectContent>
    </Select>
  );
}
