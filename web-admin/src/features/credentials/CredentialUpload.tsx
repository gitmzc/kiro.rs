import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { UploadCloud, Copy, Check, Upload } from "lucide-react";
import { useState } from "react";
import { cn } from "@/lib/utils";

const credentialTemplates = {
  social: {
    refreshToken: "your-refresh-token-here",
    profileArn: "arn:aws:codecatalyst:region::user/user-id",
    authMethod: "social",
    priority: 0
  },
  builderID: {
    accessToken: "your-access-token-here",
    refreshToken: "your-refresh-token-here",
    profileArn: "arn:aws:codecatalyst:region::user/user-id",
    expiresAt: "2026-01-03T15:00:00Z",
    authMethod: "builder-id",
    priority: 0
  },
  idc: {
    refreshToken: "your-refresh-token-here",
    authMethod: "idc",
    clientId: "your-client-id",
    clientSecret: "your-client-secret",
    priority: 0
  }
};

export function CredentialUpload() {
  const [isDragging, setIsDragging] = useState(false);
  const [copied, setCopied] = useState<string | null>(null);
  const [jsonInput, setJsonInput] = useState("");

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(true);
  };

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
    const files = e.dataTransfer.files;
    if (files.length > 0) {
      const file = files[0];
      const reader = new FileReader();
      reader.onload = (event) => {
        const content = event.target?.result as string;
        setJsonInput(content);
      };
      reader.readAsText(file);
    }
  };

  const copyTemplate = (type: keyof typeof credentialTemplates) => {
    const template = JSON.stringify(credentialTemplates[type], null, 2);
    navigator.clipboard.writeText(template);
    setCopied(type);
    setTimeout(() => setCopied(null), 2000);
  };

  const handleImport = async () => {
    try {
      // 验证 JSON 格式
      const credential = JSON.parse(jsonInput);

      // 基本验证 - refreshToken 是必需的
      if (!credential.refreshToken && !credential.refresh_token) {
        alert("验证失败: 凭据必须包含 refreshToken 或 refresh_token 字段");
        return;
      }

      // 调用后端 API 导入凭据
      const formData = new FormData();
      const blob = new Blob([jsonInput], { type: 'application/json' });
      formData.append('file', blob, 'credential.json');

      const response = await fetch('/api/admin/credentials/upload', {
        method: 'POST',
        headers: {
          'x-api-key': 'sk-kiro-rs-qazWSXedcRFV123456',
        },
        body: formData,
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.message || '上传失败');
      }

      const result = await response.json();
      alert(`导入成功: ${result.message || '凭据已成功导入'}`);
      setJsonInput("");

      // 刷新页面以显示新凭据
      window.location.reload();
    } catch (error) {
      alert(`导入失败: ${error instanceof Error ? error.message : "JSON 格式错误"}`);
    }
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle>导入凭据</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        {/* JSON 输入区域 */}
        <div className="space-y-2">
          <label className="text-sm font-medium">粘贴 JSON 到这里：</label>
          <Textarea
            placeholder='{"refreshToken": "...", "clientId": "...", "clientSecret": "...", "authMethod": "idc"}'
            value={jsonInput}
            onChange={(e) => setJsonInput(e.target.value)}
            className="font-mono text-xs min-h-[200px]"
          />
          <Button
            onClick={handleImport}
            disabled={!jsonInput.trim()}
            className="w-full"
          >
            <Upload className="h-4 w-4 mr-2" />
            导入凭据
          </Button>
        </div>

        {/* 分隔线 */}
        <div className="relative">
          <div className="absolute inset-0 flex items-center">
            <span className="w-full border-t" />
          </div>
          <div className="relative flex justify-center text-xs uppercase">
            <span className="bg-background px-2 text-muted-foreground">
              或使用模板
            </span>
          </div>
        </div>

        {/* 模板按钮 */}
        <div className="space-y-2">
          <p className="text-sm font-medium">复制 JSON 模板：</p>
          <div className="grid gap-2">
            <Button
              variant="outline"
              size="sm"
              className="justify-start"
              onClick={() => copyTemplate('social')}
            >
              {copied === 'social' ? (
                <Check className="h-4 w-4 mr-2" />
              ) : (
                <Copy className="h-4 w-4 mr-2" />
              )}
              Social Auth
            </Button>
            <Button
              variant="outline"
              size="sm"
              className="justify-start"
              onClick={() => copyTemplate('builderID')}
            >
              {copied === 'builderID' ? (
                <Check className="h-4 w-4 mr-2" />
              ) : (
                <Copy className="h-4 w-4 mr-2" />
              )}
              AWS Builder ID
            </Button>
            <Button
              variant="outline"
              size="sm"
              className="justify-start"
              onClick={() => copyTemplate('idc')}
            >
              {copied === 'idc' ? (
                <Check className="h-4 w-4 mr-2" />
              ) : (
                <Copy className="h-4 w-4 mr-2" />
              )}
              IdC (Identity Center)
            </Button>
          </div>
        </div>

        {/* 文件拖放区域 */}
        <div className="space-y-2">
          <p className="text-sm font-medium">或拖放文件：</p>
          <div
            className={cn(
              "border-2 border-dashed rounded-lg p-8 flex flex-col items-center justify-center text-center cursor-pointer transition-colors",
              isDragging ? "border-primary bg-primary/10" : "border-muted-foreground/25 hover:border-primary/50"
            )}
            onDragOver={handleDragOver}
            onDragLeave={handleDragLeave}
            onDrop={handleDrop}
          >
            <UploadCloud className="h-8 w-8 text-muted-foreground mb-2" />
            <p className="text-sm text-muted-foreground">拖拽 JSON 文件到这里</p>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
