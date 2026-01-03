import { Link, Outlet, useLocation } from "react-router-dom";
import { LayoutDashboard, Key, FileText, Settings, Activity, MessageSquare, Menu, X, Shield } from "lucide-react";
import { cn } from "@/lib/utils";
import { useState } from "react";
import { Button } from "@/components/ui/button";

const navItems = [
  { icon: LayoutDashboard, label: "仪表盘", href: "/dashboard" },
  { icon: Key, label: "凭据管理", href: "/credentials" },
  { icon: Shield, label: "API Keys", href: "/api-keys" },
  { icon: FileText, label: "日志监控", href: "/logs" },
  { icon: Settings, label: "系统配置", href: "/config" },
  { icon: MessageSquare, label: "Chat 测试", href: "/chat" },
];

export default function AppLayout() {
  const location = useLocation();
  const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false);

  return (
    <div className="flex h-screen bg-background text-foreground overflow-hidden">
      {/* Mobile Header */}
      <div className="md:hidden fixed top-0 left-0 right-0 h-14 border-b bg-background z-50 flex items-center px-4 justify-between">
        <div className="flex items-center gap-2">
            <Activity className="h-6 w-6 text-primary" />
            <h1 className="font-bold text-lg">Kiro 管理后台</h1>
        </div>
        <Button variant="ghost" size="icon" onClick={() => setIsMobileMenuOpen(!isMobileMenuOpen)}>
            {isMobileMenuOpen ? <X className="h-5 w-5" /> : <Menu className="h-5 w-5" />}
        </Button>
      </div>

      {/* Sidebar Overlay (Mobile) */}
      {isMobileMenuOpen && (
        <div 
            className="fixed inset-0 bg-black/50 z-40 md:hidden"
            onClick={() => setIsMobileMenuOpen(false)}
        />
      )}

      {/* Sidebar */}
      <aside className={cn(
        "fixed inset-y-0 left-0 z-50 w-64 border-r border-border bg-card transition-transform duration-200 ease-in-out md:relative md:translate-x-0",
        isMobileMenuOpen ? "translate-x-0" : "-translate-x-full"
      )}>
        <div className="p-6 flex items-center gap-2 border-b border-border h-14 md:h-auto">
          <Activity className="h-6 w-6 text-primary hidden md:block" />
          <h1 className="font-bold text-xl hidden md:block">Kiro 管理后台</h1>
          {/* Mobile Title inside sidebar as well? No, hidden on mobile sidebar header to save space or keep consistency */}
          <div className="md:hidden flex items-center gap-2">
            <Activity className="h-6 w-6 text-primary" />
            <h1 className="font-bold text-lg">菜单</h1>
          </div>
        </div>
        <nav className="p-4 space-y-2">
          {navItems.map((item) => {
            const Icon = item.icon;
            const isActive = location.pathname.startsWith(item.href);
            return (
              <Link
                key={item.href}
                to={item.href}
                onClick={() => setIsMobileMenuOpen(false)}
                className={cn(
                  "flex items-center gap-3 px-3 py-2 rounded-md transition-colors",
                  isActive
                    ? "bg-primary text-primary-foreground"
                    : "hover:bg-accent hover:text-accent-foreground text-muted-foreground"
                )}
              >
                <Icon className="h-4 w-4" />
                <span>{item.label}</span>
              </Link>
            );
          })}
        </nav>
      </aside>

      {/* Main Content */}
      <main className="flex-1 overflow-auto pt-14 md:pt-0 w-full relative">
        <Outlet />
      </main>
    </div>
  );
}
