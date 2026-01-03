import { Activity, Server, Zap, Key } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { useQuery } from "@tanstack/react-query";
import { apiClient, type StatsSummaryResponse, type HealthResponse, type CredentialsStatusResponse } from "@/lib/api-client";

export function StatsCards() {
  const { data: stats, isLoading: statsLoading } = useQuery({
    queryKey: ['stats', 'summary'],
    queryFn: () => apiClient.get<StatsSummaryResponse>("/stats/summary", { hours: 24 }),
    refetchInterval: 30000, // Refresh every 30s
  });

  const { data: health, isLoading: healthLoading } = useQuery({
    queryKey: ['health'],
    queryFn: () => apiClient.get<HealthResponse>("/health"),
    refetchInterval: 30000,
  });

  const { data: credentials, isLoading: credentialsLoading } = useQuery({
    queryKey: ['credentials'],
    queryFn: () => apiClient.get<CredentialsStatusResponse>("/credentials"),
    refetchInterval: 30000,
  });

  const formatUptime = (seconds: number) => {
    const days = Math.floor(seconds / 86400);
    const hours = Math.floor((seconds % 86400) / 3600);
    if (days > 0) return `${days}d ${hours}h`;
    return `${hours}h ${Math.floor((seconds % 3600) / 60)}m`;
  };

  const formatTokens = (tokens: number) => {
    if (tokens >= 1000000) return `${(tokens / 1000000).toFixed(1)}M`;
    if (tokens >= 1000) return `${(tokens / 1000).toFixed(1)}K`;
    return tokens.toString();
  };

  if (statsLoading || healthLoading || credentialsLoading) {
    return (
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        {[...Array(4)].map((_, i) => (
          <Card key={i}>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">加载中...</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">--</div>
            </CardContent>
          </Card>
        ))}
      </div>
    );
  }

  return (
    <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">服务状态</CardTitle>
          <Activity className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">{health?.status === "ok" ? "在线" : "异常"}</div>
          <p className="text-xs text-muted-foreground">
            运行时间: {health ? formatUptime(health.uptimeSeconds) : "--"}
          </p>
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">凭据状态</CardTitle>
          <Key className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">
            {credentials?.available ?? "--"} / {credentials?.total ?? "--"}
          </div>
          <p className="text-xs text-muted-foreground">
            活跃 / 总数
          </p>
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">今日请求</CardTitle>
          <Server className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">{stats?.requests.total ?? "--"}</div>
          <p className="text-xs text-muted-foreground">
            错误率: {stats ? `${(stats.requests.errorRate * 100).toFixed(1)}%` : "--"}
          </p>
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">Token 消耗</CardTitle>
          <Zap className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">{stats ? formatTokens(stats.tokens.total) : "--"}</div>
          <p className="text-xs text-muted-foreground">
            累计消耗 Token
          </p>
        </CardContent>
      </Card>
    </div>
  );
}