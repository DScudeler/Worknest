import { BrowserRouter, Navigate, Route, Routes } from "react-router-dom";
import { QueryClientProvider } from "@tanstack/react-query";
import { Toaster } from "react-hot-toast";
import { queryClient } from "./lib/queryClient";
import { AuthProvider } from "./state/auth";
import { ThemeProvider } from "./state/theme";
import { LoginScreen } from "./screens/LoginScreen";
import { RegisterScreen } from "./screens/RegisterScreen";
import { ProtectedRoute } from "./components/ProtectedRoute";
import { AppLayout } from "./components/AppLayout";
import { DashboardWrapper } from "./screens/DashboardWrapper";
import { ProjectScreen } from "./screens/ProjectScreen";
import { PlaceholderScreen } from "./screens/PlaceholderScreen";
import { AgentsScreen } from "./screens/AgentsScreen";

export function App() {
  return (
    <ThemeProvider>
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <AuthProvider>
            <Routes>
              <Route path="/login" element={<LoginScreen />} />
              <Route path="/register" element={<RegisterScreen />} />
              <Route element={<ProtectedRoute />}>
                <Route element={<AppLayout />}>
                  <Route index element={<DashboardWrapper />} />
                  <Route path="projects/:projectId" element={<ProjectScreen />} />
                  <Route path="inbox" element={<PlaceholderScreen title="Inbox" />} />
                  <Route path="my-tickets" element={<PlaceholderScreen title="My tickets" />} />
                  <Route path="agents" element={<AgentsScreen />} />
                  <Route path="settings" element={<PlaceholderScreen title="Settings" />} />
                </Route>
              </Route>
              <Route path="*" element={<Navigate to="/" replace />} />
            </Routes>
            <Toaster
              position="bottom-center"
              toastOptions={{
                style: {
                  background: "var(--text)",
                  color: "var(--bg)",
                  borderRadius: 999,
                  fontSize: 13,
                  padding: "10px 18px",
                  fontWeight: 500,
                },
              }}
            />
          </AuthProvider>
        </BrowserRouter>
      </QueryClientProvider>
    </ThemeProvider>
  );
}
