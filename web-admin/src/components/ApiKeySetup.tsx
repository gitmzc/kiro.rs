import { useState, useEffect } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Key } from "lucide-react";
import { apiClient } from "@/lib/api-client";

interface ApiKeySetupProps {
  onComplete: () => void;
}

export function ApiKeySetup({ onComplete }: ApiKeySetupProps) {
  const [adminApiKey, setAdminApiKey] = useState("");
  const [isVerifying, setIsVerifying] = useState(false);
  const [error, setError] = useState("");

  useEffect(() => {
    // Check if API key is already set
    const savedKey = localStorage.getItem("adminApiKey");
    if (savedKey) {
      apiClient.setAdminApiKey(savedKey);
      verifyApiKey(savedKey);
    }
  }, []);

  const verifyApiKey = async (key: string) => {
    setIsVerifying(true);
    setError("");

    try {
      apiClient.setAdminApiKey(key);
      // Test the API key by calling health endpoint
      await apiClient.get("/health");
      localStorage.setItem("adminApiKey", key);
      onComplete();
    } catch (err) {
      setError("API Key 验证失败，请检查是否正确");
      apiClient.setAdminApiKey("");
    } finally {
      setIsVerifying(false);
    }
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (adminApiKey.trim()) {
      verifyApiKey(adminApiKey.trim());
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-background p-4">
      <Card className="w-full max-w-md">
        <CardHeader className="text-center">
          <div className="mx-auto w-12 h-12 bg-primary/10 rounded-full flex items-center justify-center mb-4">
            <Key className="h-6 w-6 text-primary" />
          </div>
          <CardTitle>欢迎使用 Kiro Admin</CardTitle>
          <CardDescription>
            请输入 Admin API Key 以继续使用管理后台
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit} className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="admin-api-key">Admin API Key</Label>
              <Input
                id="admin-api-key"
                type="password"
                placeholder="输入您的 Admin API Key"
                value={adminApiKey}
                onChange={(e) => setAdminApiKey(e.target.value)}
                disabled={isVerifying}
                autoFocus
              />
              {error && (
                <p className="text-sm text-destructive">{error}</p>
              )}
              <p className="text-xs text-muted-foreground">
                Admin API Key 在配置文件的 <code className="bg-muted px-1 rounded">adminApiKey</code> 字段中设置
              </p>
            </div>
            <Button
              type="submit"
              className="w-full"
              disabled={!adminApiKey.trim() || isVerifying}
            >
              {isVerifying ? "验证中..." : "继续"}
            </Button>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}
