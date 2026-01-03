import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Save } from "lucide-react";
import { useState, useEffect } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { apiClient, type ConfigView, type ConfigPatch } from "@/lib/api-client";
import { toast } from "sonner";
import { PasswordChanger } from "./PasswordChanger";

export function ConfigEditor() {
  const queryClient = useQueryClient();
  const [thinkingBudget, setThinkingBudget] = useState("");
  const [modelMap, setModelMap] = useState("");

  const { data: config, isLoading } = useQuery({
    queryKey: ['config'],
    queryFn: () => apiClient.get<ConfigView>("/config"),
    staleTime: Infinity, // 配置数据不自动刷新，避免覆盖用户编辑
    refetchOnWindowFocus: false,
  });

  // 使用 useEffect 初始化表单数据（React Query v5 兼容）
  useEffect(() => {
    if (config) {
      setThinkingBudget(config.thinkingBudgetTokens.toString());
      setModelMap(JSON.stringify(config.modelMapping, null, 2));
    }
  }, [config]);

  const saveMutation = useMutation({
    mutationFn: (patch: ConfigPatch) => apiClient.post<ConfigView>("/config", patch),
    onSuccess: (data) => {
      queryClient.setQueryData(['config'], data);
      toast.success("配置已保存");
    },
    onError: () => {
      toast.error("保存失败，请重试");
    },
  });

  const handleSave = () => {
    let parsedModelMap: Record<string, string> = {};
    try {
      parsedModelMap = JSON.parse(modelMap);
    } catch (e) {
      toast.error("模型映射 JSON 格式错误");
      return;
    }

    saveMutation.mutate({
      thinkingBudgetTokens: parseInt(thinkingBudget, 10),
      modelMapping: parsedModelMap,
    });
  };

  if (isLoading) {
    return (
      <div className="space-y-6">
        <Card>
          <CardHeader>
            <CardTitle>运行时配置</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-center text-muted-foreground py-8">加载中...</div>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <PasswordChanger />

      <Card>
        <CardHeader>
          <CardTitle>运行时配置</CardTitle>
          <CardDescription>
            这些设置可以在不重启服务器的情况下更新。
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="thinking-budget">Thinking 预算 (Token)</Label>
            <Input 
                id="thinking-budget" 
                value={thinkingBudget} 
                onChange={(e) => setThinkingBudget(e.target.value)} 
            />
            <p className="text-xs text-muted-foreground">思考过程分配的最大 Token 数。</p>
          </div>
          
          <div className="space-y-2">
            <Label htmlFor="model-map">模型映射 (JSON)</Label>
            <textarea 
                id="model-map" 
                className="flex min-h-[150px] w-full rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 font-mono"
                value={modelMap}
                onChange={(e) => setModelMap(e.target.value)}
            />
            <p className="text-xs text-muted-foreground">将传入的模型名称映射到内部提供商。</p>
          </div>

          <div className="pt-4">
            <Button onClick={handleSave} disabled={saveMutation.isPending}>
                <Save className="mr-2 h-4 w-4" />
                {saveMutation.isPending ? "保存中..." : "保存更改"}
            </Button>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>静态配置 (只读)</CardTitle>
          <CardDescription>
            修改这些值需要重启服务器。
          </CardDescription>
        </CardHeader>
        <CardContent>
            <pre className="bg-muted p-4 rounded-lg overflow-auto text-xs font-mono">
{config ? JSON.stringify({
  host: config.host,
  port: config.port,
  region: config.region,
  kiroVersion: config.kiroVersion,
  systemVersion: config.systemVersion,
  nodeVersion: config.nodeVersion,
  countTokensApiUrl: config.countTokensApiUrl,
  proxyUrl: config.proxyUrl,
  apiKey: config.apiKey,
  adminApiKey: config.adminApiKey,
}, null, 2) : "加载中..."}
            </pre>
        </CardContent>
      </Card>
    </div>
  );
}