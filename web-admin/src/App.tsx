import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import { useState, useEffect } from "react";
import { Toaster } from "sonner";
import AppLayout from "./components/layout/AppLayout";
import Dashboard from "./pages/Dashboard";
import Credentials from "./pages/Credentials";
// import Logs from "./pages/Logs"; // 暂时禁用以修复构建
import Config from "./pages/Config";
import Chat from "./pages/Chat";
import ApiKeys from "./pages/ApiKeys";
import { ApiKeySetup } from "./components/ApiKeySetup";
import { apiClient } from "./lib/api-client";

function App() {
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [isChecking, setIsChecking] = useState(true);

  useEffect(() => {
    // Check if admin API key is already set
    const savedKey = localStorage.getItem("adminApiKey");
    if (savedKey) {
      apiClient.setAdminApiKey(savedKey);
      setIsAuthenticated(true);
    }
    setIsChecking(false);
  }, []);

  const handleAuthComplete = () => {
    setIsAuthenticated(true);
  };

  if (isChecking) {
    return null; // or a loading spinner
  }

  if (!isAuthenticated) {
    return <ApiKeySetup onComplete={handleAuthComplete} />;
  }

  return (
    <>
      <Toaster position="top-right" richColors />
      <BrowserRouter>
        <Routes>
          <Route path="/" element={<AppLayout />}>
            <Route index element={<Navigate to="/dashboard" replace />} />
            <Route path="dashboard" element={<Dashboard />} />
            <Route path="credentials" element={<Credentials />} />
            {/* <Route path="logs" element={<Logs />} /> */}
            <Route path="config" element={<Config />} />
            <Route path="api-keys" element={<ApiKeys />} />
            <Route path="chat" element={<Chat />} />
          </Route>
        </Routes>
      </BrowserRouter>
    </>
  );
}

export default App;
