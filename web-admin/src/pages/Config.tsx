import { ConfigEditor } from "@/features/config/ConfigEditor";

export default function Config() {
  return (
    <div className="flex-1 space-y-4 p-8 pt-6">
      <div className="flex items-center justify-between space-y-2">
        <h2 className="text-3xl font-bold tracking-tight">系统配置</h2>
      </div>
      <ConfigEditor />
    </div>
  );
}
