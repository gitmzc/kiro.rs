import { StatsCards } from "@/features/dashboard/StatsCards";
import { TrendChart } from "@/features/dashboard/TrendChart";
import { RequestStream } from "@/features/dashboard/RequestStream";

export default function Dashboard() {
  return (
    <div className="flex-1 space-y-4 p-8 pt-6">
      <div className="flex items-center justify-between space-y-2">
        <h2 className="text-3xl font-bold tracking-tight">仪表盘</h2>
      </div>
      
      <div className="space-y-4">
        <StatsCards />
        
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-7">
          <TrendChart />
          <RequestStream />
        </div>
      </div>
    </div>
  );
}
