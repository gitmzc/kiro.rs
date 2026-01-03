import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Plus, Trash2, Eye, EyeOff, Copy, Check } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { apiClient, type ApiKeysResponse, type CreateApiKeyRequest, type UpdateApiKeyRequest, type SuccessResponse, type CreateApiKeyResponse } from "@/lib/api-client";

export function ApiKeyManager() {
  const queryClient = useQueryClient();
  const [showCreateDialog, setShowCreateDialog] = useState(false);
  const [newKeyName, setNewKeyName] = useState("");
  const [createdKey, setCreatedKey] = useState<string | null>(null);
  const [copiedId, setCopiedId] = useState<string | null>(null);

  // 获取 API Keys 列表
  const { data: apiKeys, isLoading } = useQuery({
    queryKey: ["api-keys"],
    queryFn: () => apiClient.get<ApiKeysResponse>("/api-keys"),
  });

  // 创建 API Key
  const createMutation = useMutation({
    mutationFn: (request: CreateApiKeyRequest) =>
      apiClient.post<CreateApiKeyResponse>("/api-keys", request),
    onSuccess: (data) => {
      setCreatedKey(data.key);
      setNewKeyName("");
      queryClient.invalidateQueries({ queryKey: ["api-keys"] });
      toast.success("API Key 创建成功");
    },
    onError: (error: Error) => {
      toast.error(`创建失败: ${error.message}`);
    },
  });

  // 更新 API Key
  const updateMutation = useMutation({
    mutationFn: ({ id, request }: { id: string; request: UpdateApiKeyRequest }) =>
      apiClient.put<SuccessResponse>(`/api-keys/${id}`, request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["api-keys"] });
      toast.success("API Key 已更新");
    },
    onError: (error: Error) => {
      toast.error(`更新失败: ${error.message}`);
    },
  });

  // 删除 API Key
  const deleteMutation = useMutation({
    mutationFn: (id: string) => apiClient.delete<SuccessResponse>(`/api-keys/${id}`),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["api-keys"] });
      toast.success("API Key 已删除");
    },
    onError: (error: Error) => {
      toast.error(`删除失败: ${error.message}`);
    },
  });

  const handleCreate = () => {
    if (!newKeyName.trim()) {
      toast.error("请输入 API Key 名称");
      return;
    }
    createMutation.mutate({ name: newKeyName });
  };

  const handleToggleEnabled = (id: string, currentEnabled: boolean) => {
    updateMutation.mutate({
      id,
      request: { enabled: !currentEnabled },
    });
  };

  const handleDelete = (id: string, name: string) => {
    if (confirm(`确定要删除 API Key "${name}" 吗？`)) {
      deleteMutation.mutate(id);
    }
  };

  const copyToClipboard = (text: string, id: string) => {
    navigator.clipboard.writeText(text);
    setCopiedId(id);
    toast.success("已复制到剪贴板");
    setTimeout(() => setCopiedId(null), 2000);
  };

  const formatDate = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleString("zh-CN");
  };

  if (isLoading) {
    return <div className="text-center py-8">加载中...</div>;
  }

  return (
    <div className="space-y-4">
      {/* 创建新 Key */}
      <Card>
        <CardHeader>
          <CardTitle>创建新的 API Key</CardTitle>
          <CardDescription>
            创建新的 API Key 用于客户端访问
          </CardDescription>
        </CardHeader>
        <CardContent>
          {!showCreateDialog && !createdKey && (
            <Button onClick={() => setShowCreateDialog(true)}>
              <Plus className="h-4 w-4 mr-2" />
              创建 API Key
            </Button>
          )}

          {showCreateDialog && !createdKey && (
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium mb-2">名称</label>
                <input
                  type="text"
                  value={newKeyName}
                  onChange={(e) => setNewKeyName(e.target.value)}
                  placeholder="例如：生产环境、测试环境"
                  className="w-full px-3 py-2 border rounded-md"
                />
              </div>
              <div className="flex gap-2">
                <Button onClick={handleCreate} disabled={createMutation.isPending}>
                  {createMutation.isPending ? "创建中..." : "确认创建"}
                </Button>
                <Button
                  variant="outline"
                  onClick={() => {
                    setShowCreateDialog(false);
                    setNewKeyName("");
                  }}
                >
                  取消
                </Button>
              </div>
            </div>
          )}

          {createdKey && (
            <div className="space-y-4 p-4 bg-yellow-50 dark:bg-yellow-900/20 rounded-md border border-yellow-200 dark:border-yellow-800">
              <div>
                <p className="text-sm font-medium text-yellow-800 dark:text-yellow-200 mb-2">
                  ⚠️ 请立即保存此 API Key，它只会显示一次！
                </p>
                <div className="flex items-center gap-2">
                  <code className="flex-1 px-3 py-2 bg-white dark:bg-gray-800 rounded border font-mono text-sm">
                    {createdKey}
                  </code>
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={() => copyToClipboard(createdKey, "new-key")}
                  >
                    {copiedId === "new-key" ? (
                      <Check className="h-4 w-4" />
                    ) : (
                      <Copy className="h-4 w-4" />
                    )}
                  </Button>
                </div>
              </div>
              <Button
                onClick={() => {
                  setCreatedKey(null);
                  setShowCreateDialog(false);
                }}
              >
                我已保存
              </Button>
            </div>
          )}
        </CardContent>
      </Card>

      {/* API Keys 列表 */}
      <Card>
        <CardHeader>
          <CardTitle>API Keys 列表</CardTitle>
          <CardDescription>
            管理现有的 API Keys
          </CardDescription>
        </CardHeader>
        <CardContent>
          {!apiKeys?.apiKeys || apiKeys.apiKeys.length === 0 ? (
            <p className="text-muted-foreground text-center py-8">
              暂无 API Keys
            </p>
          ) : (
            <div className="space-y-3">
              {apiKeys.apiKeys.map((key) => (
                <div
                  key={key.id}
                  className="flex items-center justify-between p-4 border rounded-lg"
                >
                  <div className="flex-1">
                    <div className="flex items-center gap-3">
                      <h3 className="font-medium">{key.name}</h3>
                      <span
                        className={`px-2 py-1 text-xs rounded ${
                          key.enabled
                            ? "bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400"
                            : "bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-400"
                        }`}
                      >
                        {key.enabled ? "启用" : "禁用"}
                      </span>
                    </div>
                    <div className="mt-1 flex items-center gap-2">
                      <code className="text-sm text-muted-foreground font-mono">
                        {key.keyPreview}
                      </code>
                      <Button
                        size="sm"
                        variant="ghost"
                        onClick={() => copyToClipboard(key.keyPreview, key.id)}
                      >
                        {copiedId === key.id ? (
                          <Check className="h-3 w-3" />
                        ) : (
                          <Copy className="h-3 w-3" />
                        )}
                      </Button>
                    </div>
                    <p className="text-xs text-muted-foreground mt-1">
                      创建于: {formatDate(key.createdAt)}
                    </p>
                  </div>
                  <div className="flex items-center gap-2">
                    <Button
                      size="sm"
                      variant="outline"
                      onClick={() => handleToggleEnabled(key.id, key.enabled)}
                      disabled={updateMutation.isPending}
                    >
                      {key.enabled ? (
                        <>
                          <EyeOff className="h-4 w-4 mr-1" />
                          禁用
                        </>
                      ) : (
                        <>
                          <Eye className="h-4 w-4 mr-1" />
                          启用
                        </>
                      )}
                    </Button>
                    <Button
                      size="sm"
                      variant="outline"
                      onClick={() => handleDelete(key.id, key.name)}
                      disabled={deleteMutation.isPending}
                    >
                      <Trash2 className="h-4 w-4" />
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
