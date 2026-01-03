import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";
import { Pause, Play, Search, XCircle } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { fetchEventSource } from "@microsoft/fetch-event-source";

type LogLevel = "INFO" | "WARN" | "ERROR" | "DEBUG" | "ALL";

export function LogViewer() {
  const [logs, setLogs] = useState<string[]>([]);
  const [isPaused, setIsPaused] = useState(false);
  const [filterLevel, setFilterLevel] = useState<LogLevel>("ALL");
  const [searchQuery, setSearchQuery] = useState("");
  const [isConnected, setIsConnected] = useState(false);
  const scrollRef = useRef<HTMLDivElement>(null);
  const abortControllerRef = useRef<AbortController | null>(null);
  const maxLogs = 1000;

  // SSE connection using fetch-event-source (supports custom headers)
  useEffect(() => {
    const connect = async () => {
      // Abort previous connection
      if (abortControllerRef.current) {
        abortControllerRef.current.abort();
      }

      const abortController = new AbortController();
      abortControllerRef.current = abortController;

      const url = `/api/admin/logs/stream${filterLevel !== "ALL" ? `?level=${filterLevel.toLowerCase()}` : ""}`;
      const adminApiKey = localStorage.getItem("adminApiKey");

      try {
        await fetchEventSource(url, {
          signal: abortController.signal,
          headers: {
            "x-api-key": adminApiKey || "",
          },
          onopen: async (response) => {
            if (response.ok) {
              setIsConnected(true);
            } else {
              throw new Error(`Failed to connect: ${response.status}`);
            }
          },
          onmessage: (event) => {
            if (!isPaused && event.data) {
              setLogs((prev) => {
                const newLogs = [...prev, event.data];
                return newLogs.slice(-maxLogs);
              });
            }
          },
          onerror: (error) => {
            setIsConnected(false);
            throw error; // Will trigger reconnect
          },
          openWhenHidden: true,
        });
      } catch (error) {
        if (abortController.signal.aborted) {
          return; // Normal abort, don't reconnect
        }
        setIsConnected(false);
        // Auto-reconnect after 5 seconds
        setTimeout(() => {
          if (!isPaused) {
            connect();
          }
        }, 5000);
      }
    };

    connect();

    return () => {
      if (abortControllerRef.current) {
        abortControllerRef.current.abort();
      }
    };
  }, [filterLevel, isPaused]);

  // Auto-scroll
  useEffect(() => {
    if (!isPaused && scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [logs, isPaused]);

  const filteredLogs = logs.filter((log) => {
    if (searchQuery && !log.toLowerCase().includes(searchQuery.toLowerCase())) return false;
    return true;
  });

  const getLogColor = (log: string) => {
    if (log.includes("ERROR")) return "text-red-400";
    if (log.includes("WARN")) return "text-yellow-400";
    if (log.includes("INFO")) return "text-blue-400";
    if (log.includes("DEBUG")) return "text-gray-400";
    return "text-gray-300";
  };

  return (
    <Card className="h-[calc(100vh-10rem)] flex flex-col">
      <CardHeader className="py-4 border-b">
        <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <CardTitle>系统日志</CardTitle>
              <div className="flex items-center gap-2">
                <div className={cn("w-2 h-2 rounded-full", isConnected ? "bg-green-500" : "bg-red-500")} />
                <span className="text-sm text-muted-foreground">
                  {isConnected ? "已连接" : "未连接"}
                </span>
              </div>
            </div>
            <div className="flex items-center gap-2">
                <Button variant={filterLevel === 'ALL' ? 'default' : 'outline'} size="sm" onClick={() => setFilterLevel("ALL")}>全部</Button>
                <Button variant={filterLevel === 'INFO' ? 'default' : 'outline'} size="sm" onClick={() => setFilterLevel("INFO")}>INFO</Button>
                <Button variant={filterLevel === 'WARN' ? 'default' : 'outline'} size="sm" onClick={() => setFilterLevel("WARN")}>WARN</Button>
                <Button variant={filterLevel === 'ERROR' ? 'default' : 'outline'} size="sm" onClick={() => setFilterLevel("ERROR")}>ERROR</Button>
            </div>
        </div>
        <div className="flex items-center gap-2 mt-2">
            <div className="relative flex-1">
                <Search className="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground" />
                <Input
                    placeholder="搜索日志..."
                    className="pl-8"
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                />
            </div>
            <Button variant="outline" size="icon" onClick={() => setIsPaused(!isPaused)} title={isPaused ? "继续" : "暂停"}>
                {isPaused ? <Play className="h-4 w-4" /> : <Pause className="h-4 w-4" />}
            </Button>
            <Button variant="outline" size="icon" onClick={() => setLogs([])} title="清空">
                <XCircle className="h-4 w-4" />
            </Button>
        </div>
      </CardHeader>
      <CardContent className="flex-1 p-0 overflow-hidden">
        <div ref={scrollRef} className="h-full overflow-auto p-4 font-mono text-xs space-y-0.5 bg-black/90">
            {filteredLogs.map((log, idx) => (
                <div key={idx} className={cn("hover:bg-white/5 px-2 rounded whitespace-pre-wrap break-all", getLogColor(log))}>
                    {log}
                </div>
            ))}
            {filteredLogs.length === 0 && <div className="text-center text-gray-500 mt-10">暂无日志</div>}
        </div>
      </CardContent>
    </Card>
  );
}