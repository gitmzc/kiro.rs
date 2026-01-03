import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { cn } from "@/lib/utils";
import { useQuery } from "@tanstack/react-query";
import { apiClient, type StatsRequestsResponse } from "@/lib/api-client";

export function RequestStream() {
  const { data: requests = [], isLoading } = useQuery({
    queryKey: ['stats', 'requests'],
    queryFn: () => apiClient.get<StatsRequestsResponse>("/stats/requests", { limit: 10 }),
    refetchInterval: 5000, // Refresh every 5s
    select: (response) => response.items,
  });

  if (isLoading) {
    return (
      <Card className="col-span-4 lg:col-span-3">
        <CardHeader>
          <CardTitle>最近请求</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="text-center text-muted-foreground py-8">加载中...</div>
        </CardContent>
      </Card>
    );
  }

  if (requests.length === 0) {
    return (
      <Card className="col-span-4 lg:col-span-3">
        <CardHeader>
          <CardTitle>最近请求</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="text-center text-muted-foreground py-8">暂无请求记录</div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className="col-span-4 lg:col-span-3">
      <CardHeader>
        <CardTitle>最近请求</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="space-y-4">
          {requests.map((req, idx) => (
            <div
              key={`${req.ts}-${idx}`}
              className="flex items-center justify-between border-b border-border pb-2 last:border-0 last:pb-0"
            >
              <div className="flex items-center gap-4">
                <div className={cn(
                    "w-2 h-2 rounded-full",
                    req.status === 200 ? "bg-green-500" : req.status >= 500 ? "bg-red-500" : "bg-yellow-500"
                )} />
                <div className="space-y-1">
                    <p className="text-sm font-medium leading-none">{req.model}</p>
                    <p className="text-xs text-muted-foreground">
                      {new Date(req.ts).toLocaleTimeString("zh-CN")} • {req.method}
                    </p>
                </div>
              </div>
              <div className="flex items-center gap-4 text-sm">
                <div className="text-right">
                    <div className="font-medium">{(req.durationMs / 1000).toFixed(1)}s</div>
                    <div className="text-xs text-muted-foreground">{req.totalTokens} Tokens</div>
                </div>
                <div className={cn(
                    "font-bold w-12 text-center rounded px-1",
                     req.status === 200 ? "text-green-500 bg-green-500/10" : "text-red-500 bg-red-500/10"
                )}>
                    {req.status}
                </div>
              </div>
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  );
}