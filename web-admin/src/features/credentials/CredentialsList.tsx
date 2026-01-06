import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Checkbox } from "@/components/ui/checkbox";
import { Play, Pause, RefreshCw, Trash2 } from "lucide-react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { apiClient, type CredentialsStatusResponse, type SuccessResponse, type BalanceResponse, type BatchResponse } from "@/lib/api-client";
import { toast } from "sonner";
import { useState } from "react";

export function CredentialsList() {
  const queryClient = useQueryClient();
  const [balanceInfo, setBalanceInfo] = useState<Record<number, BalanceResponse | null>>({});
  const [loadingBalance, setLoadingBalance] = useState<Record<number, boolean>>({});
  const [selectedIds, setSelectedIds] = useState<Set<number>>(new Set());

  const { data: credentials, isLoading } = useQuery({
    queryKey: ['credentials'],
    queryFn: () => apiClient.get<CredentialsStatusResponse>("/credentials"),
    refetchInterval: 10000,
  });

  const toggleStatusMutation = useMutation({
    mutationFn: ({ id, disabled }: { id: number; disabled: boolean }) =>
      apiClient.post<SuccessResponse>(`/credentials/${id}/disabled`, { disabled }),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ['credentials'] });
      toast.success(variables.disabled ? "凭据已禁用" : "凭据已启用");
    },
    onError: () => {
      toast.error("操作失败，请重试");
    },
  });

  const resetMutation = useMutation({
    mutationFn: (id: number) =>
      apiClient.post<SuccessResponse>(`/credentials/${id}/reset`, {}),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['credentials'] });
      toast.success("失败计数已重置");
    },
    onError: () => {
      toast.error("重置失败，请重试");
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (id: number) =>
      apiClient.delete<SuccessResponse>(`/credentials/${id}`),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['credentials'] });
      toast.success("凭据已删除");
    },
    onError: () => {
      toast.error("删除失败，请重试");
    },
  });

  const batchDisabledMutation = useMutation({
    mutationFn: ({ ids, disabled }: { ids: number[]; disabled: boolean }) =>
      apiClient.post<BatchResponse>("/credentials/batch/disabled", { ids, disabled }),
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: ['credentials'] });
      toast.success(data.message);
      setSelectedIds(new Set());
    },
    onError: () => {
      toast.error("批量操作失败");
    },
  });

  const batchResetMutation = useMutation({
    mutationFn: (ids: number[]) =>
      apiClient.post<BatchResponse>("/credentials/batch/reset", { ids }),
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: ['credentials'] });
      toast.success(data.message);
      setSelectedIds(new Set());
    },
    onError: () => {
      toast.error("批量重置失败");
    },
  });

  const batchDeleteMutation = useMutation({
    mutationFn: (ids: number[]) =>
      apiClient.post<BatchResponse>("/credentials/batch/delete", { ids }),
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: ['credentials'] });
      toast.success(data.message);
      setSelectedIds(new Set());
    },
    onError: () => {
      toast.error("批量删除失败");
    },
  });

  const fetchBalance = async (id: number) => {
    setLoadingBalance(prev => ({ ...prev, [id]: true }));
    try {
      const balance = await apiClient.get<BalanceResponse>(`/credentials/${id}/balance`);
      setBalanceInfo(prev => ({ ...prev, [id]: balance }));
      toast.success(`余额: ${balance.remaining.toLocaleString()} / ${balance.usageLimit.toLocaleString()} (${(100 - balance.usagePercentage).toFixed(1)}%)`);
    } catch {
      toast.error("查询余额失败");
      setBalanceInfo(prev => ({ ...prev, [id]: null }));
    } finally {
      setLoadingBalance(prev => ({ ...prev, [id]: false }));
    }
  };

  const getStatusText = (cred: CredentialsStatusResponse["credentials"][0]) => {
    if (cred.disabled) return "Disabled";
    if (cred.isCurrent) return "Active";
    return "Standby";
  };

  const maskKey = (authMethod: string) => {
    const prefixes: Record<string, string> = {
      "idc": "idc-***",
      "builder-id": "bid-***",
      "social": "soc-***",
    };
    return prefixes[authMethod] || `${authMethod}-***`;
  };

  const toggleSelect = (id: number) => {
    setSelectedIds(prev => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  };

  const toggleSelectAll = () => {
    if (!credentials) return;
    if (selectedIds.size === credentials.credentials.length) {
      setSelectedIds(new Set());
    } else {
      setSelectedIds(new Set(credentials.credentials.map(c => c.id)));
    }
  };

  const selectedArray = Array.from(selectedIds);
  const hasSelection = selectedIds.size > 0;
  const isBatchLoading = batchDisabledMutation.isPending || batchResetMutation.isPending || batchDeleteMutation.isPending;

  if (isLoading) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>凭据列表</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="text-center text-muted-foreground py-8">加载中...</div>
        </CardContent>
      </Card>
    );
  }

  if (!credentials || credentials.credentials.length === 0) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>凭据列表</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="text-center text-muted-foreground py-8">暂无凭据</div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-4">
        <CardTitle>凭据列表 ({credentials.available} / {credentials.total} 可用)</CardTitle>
        {hasSelection && (
          <div className="flex items-center gap-2">
            <span className="text-sm text-muted-foreground">已选 {selectedIds.size} 项</span>
            <Button
              variant="outline"
              size="sm"
              onClick={() => batchDisabledMutation.mutate({ ids: selectedArray, disabled: false })}
              disabled={isBatchLoading}
            >
              <Play className="h-4 w-4 mr-1" />
              批量启用
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={() => batchDisabledMutation.mutate({ ids: selectedArray, disabled: true })}
              disabled={isBatchLoading}
            >
              <Pause className="h-4 w-4 mr-1" />
              批量禁用
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={() => batchResetMutation.mutate(selectedArray)}
              disabled={isBatchLoading}
            >
              <RefreshCw className="h-4 w-4 mr-1" />
              批量重置
            </Button>
            <Button
              variant="outline"
              size="sm"
              className="text-destructive"
              onClick={() => {
                if (confirm(`确定要删除选中的 ${selectedIds.size} 个凭据吗？`)) {
                  batchDeleteMutation.mutate(selectedArray);
                }
              }}
              disabled={isBatchLoading}
            >
              <Trash2 className="h-4 w-4 mr-1" />
              批量删除
            </Button>
          </div>
        )}
      </CardHeader>
      <CardContent>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead className="w-12">
                <Checkbox
                  checked={selectedIds.size === credentials.credentials.length && credentials.credentials.length > 0}
                  onCheckedChange={toggleSelectAll}
                />
              </TableHead>
              <TableHead>ID</TableHead>
              <TableHead>优先级</TableHead>
              <TableHead>类型</TableHead>
              <TableHead>Key</TableHead>
              <TableHead>状态</TableHead>
              <TableHead>失败次数</TableHead>
              <TableHead>余额</TableHead>
              <TableHead>过期时间</TableHead>
              <TableHead className="text-right">操作</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {credentials.credentials.map((cred) => (
              <TableRow key={cred.id}>
                <TableCell>
                  <Checkbox
                    checked={selectedIds.has(cred.id)}
                    onCheckedChange={() => toggleSelect(cred.id)}
                  />
                </TableCell>
                <TableCell>{cred.id}</TableCell>
                <TableCell>{cred.priority}</TableCell>
                <TableCell>{cred.authMethod}</TableCell>
                <TableCell className="font-mono text-xs">{maskKey(cred.authMethod)}</TableCell>
                <TableCell>
                  <StatusBadge status={getStatusText(cred)} />
                </TableCell>
                <TableCell>{cred.failureCount}</TableCell>
                <TableCell className="text-xs">
                  {balanceInfo[cred.id] ? (
                    <span className={balanceInfo[cred.id]!.usagePercentage > 80 ? "text-red-500" : "text-green-500"}>
                      {balanceInfo[cred.id]!.remaining.toLocaleString()} / {balanceInfo[cred.id]!.usageLimit.toLocaleString()}
                    </span>
                  ) : (
                    <Button
                      variant="ghost"
                      size="sm"
                      className="h-6 px-2 text-xs"
                      onClick={() => fetchBalance(cred.id)}
                      disabled={loadingBalance[cred.id]}
                    >
                      {loadingBalance[cred.id] ? "查询中..." : "查询"}
                    </Button>
                  )}
                </TableCell>
                <TableCell className="text-xs">
                  {cred.expiresAt ? new Date(cred.expiresAt).toLocaleString("zh-CN") : "-"}
                </TableCell>
                <TableCell className="text-right space-x-2">
                  <Button
                    variant="ghost"
                    size="icon"
                    title={cred.disabled ? "启用" : "禁用"}
                    onClick={() => toggleStatusMutation.mutate({ id: cred.id, disabled: !cred.disabled })}
                    disabled={toggleStatusMutation.isPending}
                  >
                    {cred.disabled ? <Play className="h-4 w-4" /> : <Pause className="h-4 w-4" />}
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon"
                    title="重置失败计数"
                    onClick={() => resetMutation.mutate(cred.id)}
                    disabled={resetMutation.isPending}
                  >
                    <RefreshCw className="h-4 w-4" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon"
                    className="text-destructive"
                    title="删除"
                    onClick={() => {
                      if (confirm("确定要删除此凭据吗？")) {
                        deleteMutation.mutate(cred.id);
                      }
                    }}
                    disabled={deleteMutation.isPending}
                  >
                    <Trash2 className="h-4 w-4" />
                  </Button>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  );
}

function StatusBadge({ status }: { status: string }) {
    let color = "bg-gray-500";
    let text = status;

    if (status === "Active") {
      color = "bg-green-500";
      text = "活跃";
    }
    if (status === "Cooldown") {
      color = "bg-yellow-500";
      text = "冷却中";
    }
    if (status === "RateLimited") {
      color = "bg-orange-500";
      text = "被限流";
    }
    if (status === "Expired") {
      color = "bg-red-500";
      text = "已过期";
    }
    if (status === "Disabled") {
      color = "bg-gray-500";
      text = "已禁用";
    }
    if (status === "Standby") {
      color = "bg-blue-500";
      text = "待命";
    }

    return (
        <span className={`inline-flex items-center px-2 py-1 rounded-full text-xs font-medium text-white ${color}`}>
            {text}
        </span>
    )
}
