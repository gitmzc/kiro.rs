import { LogViewer } from "@/features/logs/LogViewer";

export default function Logs() {
  return (
    <div className="flex-1 space-y-4 p-8 pt-6 h-screen flex flex-col">
      <div className="flex items-center justify-between space-y-2">
        <h2 className="text-3xl font-bold tracking-tight">日志监控</h2>
      </div>
      <div className="flex-1">
        <LogViewer />
      </div>
    </div>
  );
}
