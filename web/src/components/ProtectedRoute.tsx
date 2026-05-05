import { Navigate, Outlet, useLocation } from "react-router-dom";
import { useAuth } from "../state/auth";
import { CenterSpinner } from "./Spinner";

export function ProtectedRoute() {
  const { user, token, loading } = useAuth();
  const location = useLocation();

  if (loading) return <CenterSpinner />;
  if (!token || !user) {
    return <Navigate to="/login" replace state={{ from: location }} />;
  }
  return <Outlet />;
}
