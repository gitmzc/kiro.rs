import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Line, LineChart, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import { useQuery } from "@tanstack/react-query";
import { apiClient, type StatsTimeseriesResponse } from "@/lib/api-client";

export function TrendChart() {
  const { data, isLoading } = useQuery({
    queryKey: ['stats', 'timeseries'],
    queryFn: () => apiClient.get<StatsTimeseriesResponse>("/stats/timeseries", {
      hours: 24,
      intervalMinutes: 60,
    }),
    refetchInterval: 60000, // Refresh every 60s
    select: (response) => response.points.map((point) => ({
      time: new Date(point.ts).toLocaleTimeString("zh-CN", { hour: "2-digit", minute: "2-digit" }),
      requests: point.requests,
      tokens: point.totalTokens,
    })),
  });

  if (isLoading) {
    return (
      <Card className="col-span-4">
        <CardHeader>
          <CardTitle>24小时趋势</CardTitle>
        </CardHeader>
        <CardContent className="pl-2">
          <div className="h-[300px] flex items-center justify-center text-muted-foreground">
            加载中...
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className="col-span-4">
      <CardHeader>
        <CardTitle>24小时趋势</CardTitle>
      </CardHeader>
      <CardContent className="pl-2">
        <div className="h-[300px]">
          <ResponsiveContainer width="100%" height="100%">
            <LineChart data={data}>
              <XAxis
                dataKey="time"
                stroke="#888888"
                fontSize={12}
                tickLine={false}
                axisLine={false}
              />
              <YAxis
                stroke="#888888"
                fontSize={12}
                tickLine={false}
                axisLine={false}
                tickFormatter={(value) => `${value}`}
              />
              <Tooltip
                contentStyle={{ backgroundColor: 'var(--card)', borderColor: 'var(--border)' }}
                itemStyle={{ color: 'var(--foreground)' }}
              />
              <Line
                type="monotone"
                dataKey="requests"
                stroke="#adfa1d"
                strokeWidth={2}
                activeDot={{ r: 8 }}
              />
              <Line
                type="monotone"
                dataKey="tokens"
                stroke="#2563eb"
                strokeWidth={2}
              />
            </LineChart>
          </ResponsiveContainer>
        </div>
      </CardContent>
    </Card>
  );
}